//! 카카오 로그인 경계: 카카오 로그인 주소를 만들고, 인가 코드를 카카오 회원번호와 닉네임으로 바꾼다.
//!
//! 서버는 [`Kakao`]를 주입받는다. 실제 구현 [`KakaoApi`]는 카카오 서버와 통신하고,
//! 테스트는 가짜를 꽂는다. 카카오 토큰은 사용자 정보를 읽는 데만 쓰고 저장하지 않는다.

use std::future::Future;
use std::pin::Pin;

/// 카카오에서 받은 User 정보. 회원번호와 닉네임만 받는다 (ADR 0003).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KakaoProfile {
    pub kakao_id: i64,
    pub nickname: String,
}

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait Kakao: Send + Sync {
    /// 로그인 버튼이 보내는 카카오 인가 주소. `state`는 콜백에서 그대로 돌아온다.
    fn login_url(&self, state: &str) -> String;

    /// 콜백으로 받은 인가 코드를 카카오 회원번호와 닉네임으로 바꾼다. 실패하면 이유 문구.
    fn profile<'a>(&'a self, code: &'a str) -> BoxFuture<'a, Result<KakaoProfile, String>>;
}

/// 닉네임 동의를 받지 못했을 때 보여 줄 이름.
const FALLBACK_NICKNAME: &str = "카카오 사용자";

/// 카카오 서버와 통신하는 실제 구현. REST API 키와 Client Secret, Redirect URI가 필요하다.
pub struct KakaoApi {
    rest_api_key: String,
    client_secret: String,
    redirect_uri: String,
    http: reqwest::Client,
}

impl KakaoApi {
    pub fn new(rest_api_key: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            rest_api_key,
            client_secret,
            redirect_uri,
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("HTTP 클라이언트를 만들지 못했어요"),
        }
    }

    async fn fetch_profile(&self, code: &str) -> Result<KakaoProfile, String> {
        #[derive(serde::Deserialize)]
        struct Token {
            access_token: String,
        }
        let token: Token = self
            .http
            .post("https://kauth.kakao.com/oauth/token")
            .form(&[
                ("grant_type", "authorization_code"),
                ("client_id", &self.rest_api_key),
                ("client_secret", &self.client_secret),
                ("redirect_uri", &self.redirect_uri),
                ("code", code),
            ])
            .send()
            .await
            .and_then(|res| res.error_for_status())
            .map_err(|e| format!("카카오 토큰 발급 실패: {e}"))?
            .json()
            .await
            .map_err(|e| format!("카카오 토큰 응답을 읽지 못함: {e}"))?;

        let me: serde_json::Value = self
            .http
            .get("https://kapi.kakao.com/v2/user/me")
            .bearer_auth(&token.access_token)
            .send()
            .await
            .and_then(|res| res.error_for_status())
            .map_err(|e| format!("카카오 사용자 정보 조회 실패: {e}"))?
            .json()
            .await
            .map_err(|e| format!("카카오 사용자 정보를 읽지 못함: {e}"))?;
        profile_from_user_me(&me)
            .ok_or_else(|| format!("카카오 사용자 정보에 회원번호가 없음: {me}"))
    }
}

/// `/v2/user/me` 응답에서 회원번호와 닉네임을 꺼낸다.
fn profile_from_user_me(me: &serde_json::Value) -> Option<KakaoProfile> {
    let kakao_id = me.get("id")?.as_i64()?;
    let nickname = me
        .pointer("/kakao_account/profile/nickname")
        .or_else(|| me.pointer("/properties/nickname"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(FALLBACK_NICKNAME)
        .to_string();
    Some(KakaoProfile { kakao_id, nickname })
}

impl Kakao for KakaoApi {
    fn login_url(&self, state: &str) -> String {
        url::Url::parse_with_params(
            "https://kauth.kakao.com/oauth/authorize",
            [
                ("response_type", "code"),
                ("client_id", self.rest_api_key.as_str()),
                ("redirect_uri", self.redirect_uri.as_str()),
                ("state", state),
            ],
        )
        .expect("카카오 인가 주소는 늘 올바른 URL이다")
        .to_string()
    }

    fn profile<'a>(&'a self, code: &'a str) -> BoxFuture<'a, Result<KakaoProfile, String>> {
        Box::pin(self.fetch_profile(code))
    }
}
