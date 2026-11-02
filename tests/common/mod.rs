//! 테스트가 함께 쓰는 준비 코드: 테스트마다 따로 쓰는 Postgres 데이터베이스와 HTTP 라우터.

#![allow(dead_code)]

use dioxus::server::axum::{body::Body, http::Response, Router};
use http_body_util::BodyExt;
use short_url::kakao::{BoxFuture, Kakao, KakaoProfile};
use short_url::shortener::{Shortener, User};
use sqlx::{Connection, PgConnection};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// `TEST_DATABASE_URL`이 없을 때 붙는 로컬 Postgres (AGENTS.md의 docker 명령으로 띄운다).
const DEFAULT_TEST_DATABASE_URL: &str = "postgres://postgres:postgres@localhost:5432/short_url";

/// 테스트 하나가 혼자 쓰는 빈 데이터베이스. 버릴 때 데이터베이스를 지운다.
/// 테스트마다 데이터베이스가 따로라 병렬로 돌아도 Link Number가 섞이지 않는다.
pub struct TestDb {
    pub url: String,
    name: String,
    admin_url: String,
}

impl TestDb {
    pub async fn new() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let admin_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| DEFAULT_TEST_DATABASE_URL.to_string());
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let name = format!(
            "short_url_test_{}_{}_{}",
            std::process::id(),
            nanos,
            COUNTER.fetch_add(1, Ordering::Relaxed)
        );
        let mut admin = PgConnection::connect(&admin_url).await.unwrap_or_else(|e| {
            panic!("테스트용 Postgres({admin_url})에 붙지 못했어요. AGENTS.md의 명령으로 띄워 주세요: {e}")
        });
        sqlx::raw_sql(sqlx::AssertSqlSafe(format!("CREATE DATABASE {name}")))
            .execute(&mut admin)
            .await
            .unwrap();
        let url = with_database(&admin_url, &name);
        TestDb {
            url,
            name,
            admin_url,
        }
    }

    /// 열린 Shortener 아래의 테이블을 지워 저장소를 못 쓰게 만든다.
    pub async fn break_storage(&self) {
        let mut conn = PgConnection::connect(&self.url).await.unwrap();
        sqlx::raw_sql("DROP TABLE short_links CASCADE")
            .execute(&mut conn)
            .await
            .unwrap();
    }
}

impl Drop for TestDb {
    fn drop(&mut self) {
        let admin_url = self.admin_url.clone();
        let name = self.name.clone();
        // Drop은 async가 아니고 테스트 런타임 안에서 불리므로 다른 스레드의 런타임에서 지운다.
        let _ = std::thread::spawn(move || {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    let mut admin = PgConnection::connect(&admin_url).await?;
                    sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
                        "DROP DATABASE IF EXISTS {name} WITH (FORCE)"
                    )))
                    .execute(&mut admin)
                    .await
                    .map(|_| ())
                })
        })
        .join();
    }
}

/// 접속 주소의 데이터베이스 이름만 바꾼다 (`.../short_url?x=y` → `.../{name}?x=y`).
fn with_database(url: &str, name: &str) -> String {
    let (base, query) = match url.find('?') {
        Some(i) => url.split_at(i),
        None => (url, ""),
    };
    let after_scheme = base.find("://").map_or(0, |i| i + 3);
    let host_end = base[after_scheme..]
        .find('/')
        .map_or(base.len(), |i| after_scheme + i);
    format!("{}/{name}{query}", &base[..host_end])
}

/// 서비스 주소 `http://localhost:8080`으로 새 데이터베이스에 연 Shortener.
pub async fn shortener(db: &TestDb) -> Shortener {
    Shortener::open("http://localhost:8080", &db.url)
        .await
        .unwrap()
}

/// 새 데이터베이스를 붙인 실제 서버 라우터. 돌려준 `TestDb`가 살아 있는 동안 저장소가 남는다.
pub async fn router() -> (Router, TestDb) {
    let db = TestDb::new().await;
    let router = router_with(shortener(&db).await);
    (router, db)
}

/// 새 데이터베이스를 붙인 실제 서버 라우터와, 테스트 User로 로그인한 세션의 `Cookie` 헤더 값.
pub async fn signed_in_router() -> (Router, TestDb, String) {
    let db = TestDb::new().await;
    let shortener = shortener(&db).await;
    let cookie = session_cookie(&shortener, &user(&shortener).await).await;
    (router_with(shortener), db, cookie)
}

/// User의 세션을 열어 `Cookie` 헤더 값(`session=...`)을 돌려준다.
pub async fn session_cookie(shortener: &Shortener, user: &User) -> String {
    format!("session={}", shortener.start_session(user).await.unwrap())
}

/// 주어진 Shortener로 서버 라우터를 만든다.
pub fn router_with(shortener: Shortener) -> Router {
    // dx 없이 실행하면 실행 파일 옆에 public/ 이 없어 dioxus가 패닉한다.
    // 빈 디렉터리를 정적 자산 경로로 지정한다.
    let public = std::env::temp_dir().join("short-url-test-public");
    std::fs::create_dir_all(&public).unwrap();
    std::env::set_var("DIOXUS_PUBLIC_PATH", &public);
    short_url::build_router(shortener, Arc::new(FakeKakao))
}

/// 테스트용 User(카카오 회원번호 1, 닉네임 "테스터").
pub async fn user(shortener: &Shortener) -> User {
    shortener.sign_in(1, "테스터").await.unwrap()
}

/// 카카오 서버 대신 쓰는 가짜. 인가 코드 `"{회원번호}:{닉네임}"`을 그 프로필로 바꾸고,
/// 그 밖의 코드는 실패로 본다. 로그인 주소는 `https://kauth.test/authorize?state=...`.
pub struct FakeKakao;

pub const FAKE_LOGIN_URL: &str = "https://kauth.test/authorize";

impl Kakao for FakeKakao {
    fn login_url(&self, state: &str) -> String {
        format!("{FAKE_LOGIN_URL}?state={state}")
    }

    fn profile<'a>(&'a self, code: &'a str) -> BoxFuture<'a, Result<KakaoProfile, String>> {
        Box::pin(async move {
            let (id, nickname) = code.split_once(':').ok_or("잘못된 코드")?;
            Ok(KakaoProfile {
                kakao_id: id.parse().map_err(|_| "잘못된 회원번호")?,
                nickname: nickname.to_string(),
            })
        })
    }
}

/// 응답 본문을 문자열로 읽는다.
pub async fn body_text(res: Response<Body>) -> String {
    let body = res.into_body().collect().await.unwrap().to_bytes();
    String::from_utf8(body.to_vec()).unwrap()
}
