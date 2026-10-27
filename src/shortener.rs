//! Shortener: Shorten과 Follow 규칙을 화면, HTTP와 떼어 놓은 모듈.
//!
//! 공개 인터페이스는 [`Shortener::shorten`]과 [`Shortener::follow`] 두 개뿐이다.
//! 서비스 주소와 저장소(SQLite 파일)는 생성 시점에 받는다.
//! [`Rejection`]은 화면에서도 쓰므로 server 기능 없이도 컴파일된다.

use serde::{Deserialize, Serialize};

/// Shorten이 받는 입력의 최대 길이(문자 수, 앞뒤 공백 포함).
pub const MAX_INPUT_CHARS: usize = 2048;

/// Shorten이 Original URL을 받아들이지 않은 이유.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rejection {
    /// 비었거나 공백뿐이다.
    Empty,
    /// 절대 URL로 읽을 수 없다.
    NotAUrl,
    /// 스킴이 `http`나 `https`가 아니다.
    UnsupportedScheme,
    /// 앞뒤 공백을 떼기 전 입력이 [`MAX_INPUT_CHARS`]자를 넘는다.
    TooLong,
    /// 이 서비스 자기 주소다 (호스트와 포트가 서비스 주소와 같다).
    SelfAddress,
}

impl std::fmt::Display for Rejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Rejection::Empty => f.write_str("줄일 주소를 입력해 주세요"),
            Rejection::NotAUrl => {
                f.write_str("주소 형식이 아니에요. https://로 시작하는 전체 주소를 넣어 주세요")
            }
            Rejection::UnsupportedScheme => {
                f.write_str("http:// 또는 https://로 시작하는 주소만 줄일 수 있어요")
            }
            Rejection::TooLong => write!(
                f,
                "주소가 너무 길어요. {MAX_INPUT_CHARS}자까지 줄일 수 있어요"
            ),
            Rejection::SelfAddress => f.write_str("이 서비스의 주소는 줄일 수 없어요"),
        }
    }
}

#[cfg(feature = "server")]
pub use server::{OpenError, ShortenError, Shortener, StorageError};

#[cfg(feature = "server")]
mod server {
    use super::{Rejection, MAX_INPUT_CHARS};
    use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
    use std::path::Path;
    use std::sync::Mutex;
    use url::Url;

    pub struct Shortener {
        base_url: String,
        /// 서비스 주소의 호스트와 포트(스킴 기본 포트면 `None`). 자기 주소 판단에 쓴다.
        own_host: HostAndPort,
        conn: Mutex<Connection>,
    }

    type HostAndPort = (String, Option<u16>);

    /// Shorten이 Short Link를 돌려주지 못한 이유.
    #[derive(Debug, PartialEq)]
    pub enum ShortenError {
        /// 입력을 Original URL로 받아들이지 않았다. 사용자가 입력을 고쳐야 한다.
        Rejected(Rejection),
        /// 입력은 받아들였지만 저장소에 쓰거나 읽지 못했다.
        Storage(StorageError),
    }

    /// Shortener를 열지 못한 이유.
    #[derive(Debug)]
    pub enum OpenError {
        /// 서비스 주소가 호스트가 있는 http(s) URL이 아니다.
        InvalidBaseUrl(String),
        /// 저장소 파일을 열거나 준비하지 못했다.
        Storage(StorageError),
    }

    impl std::fmt::Display for OpenError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                OpenError::InvalidBaseUrl(base_url) => write!(
                    f,
                    "서비스 주소(BASE_URL) {base_url:?}는 호스트가 있는 http:// 또는 https:// 주소여야 해요"
                ),
                OpenError::Storage(e) => e.fmt(f),
            }
        }
    }

    impl std::error::Error for OpenError {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            match self {
                OpenError::InvalidBaseUrl(_) => None,
                OpenError::Storage(e) => Some(e),
            }
        }
    }

    /// 저장소(SQLite)를 쓰거나 읽지 못했다.
    #[derive(Debug, PartialEq)]
    pub struct StorageError(rusqlite::Error);

    impl std::fmt::Display for StorageError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "저장소 오류: {}", self.0)
        }
    }

    impl std::error::Error for StorageError {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            Some(&self.0)
        }
    }

    impl From<rusqlite::Error> for StorageError {
        fn from(e: rusqlite::Error) -> Self {
            StorageError(e)
        }
    }

    /// 호스트(소문자)와 기본 포트가 아닌 포트. URL 파서가 호스트를 소문자로, 기본 포트를 `None`으로 만든다.
    fn host_and_port(url: &Url) -> Option<HostAndPort> {
        Some((url.host_str()?.to_string(), url.port()))
    }

    impl Shortener {
        /// `db_path`의 SQLite 파일을 열고, 없으면 만들어 저장소를 준비한다.
        /// `base_url`은 Short Link 앞부분이며 끝의 `/`는 뗀다.
        /// `base_url`이 호스트가 있는 http(s) URL이 아니면 저장소를 열지 않고 실패한다.
        pub fn open(base_url: &str, db_path: impl AsRef<Path>) -> Result<Self, OpenError> {
            let own_host = Url::parse(base_url)
                .ok()
                .filter(|url| matches!(url.scheme(), "http" | "https"))
                .as_ref()
                .and_then(host_and_port)
                .ok_or_else(|| OpenError::InvalidBaseUrl(base_url.to_string()))?;
            let open_storage = || -> rusqlite::Result<Connection> {
                let conn = Connection::open(db_path)?;
                conn.execute_batch(
                    "CREATE TABLE IF NOT EXISTS short_links (
                         link_number  INTEGER PRIMARY KEY AUTOINCREMENT,
                         original_url TEXT NOT NULL UNIQUE
                     );",
                )?;
                Ok(conn)
            };
            let conn = open_storage().map_err(|e| OpenError::Storage(e.into()))?;
            Ok(Self {
                base_url: base_url.trim_end_matches('/').to_string(),
                own_host,
                conn: Mutex::new(conn),
            })
        }

        /// Original URL에 Link Number를 발급하고 Short Link를 돌려준다.
        /// 이미 줄인 Original URL이면 기존 Link Number를 돌려주고 새 번호를 쓰지 않는다.
        pub fn shorten(&self, input: &str) -> Result<String, ShortenError> {
            let original_url = self
                .accepted_original_url(input)
                .map_err(ShortenError::Rejected)?;
            let link_number = link_number_of(&mut self.conn(), &original_url)
                .map_err(|e| ShortenError::Storage(e.into()))?;
            Ok(format!("{}/{}", self.base_url, link_number))
        }

        /// 저장소 연결. 다른 스레드가 잡은 채 패닉했어도 연결 자체는 멀쩡하므로 그대로 쓴다.
        fn conn(&self) -> std::sync::MutexGuard<'_, Connection> {
            self.conn.lock().unwrap_or_else(|e| e.into_inner())
        }

        /// 입력이 Original URL로 받아들일 만한지 보고, 받아들이면 정리한 Original URL을 돌려준다.
        /// 거절이면 저장소에 손대지 않는다.
        fn accepted_original_url(&self, input: &str) -> Result<String, Rejection> {
            let trimmed = input.trim();
            if trimmed.is_empty() {
                return Err(Rejection::Empty);
            }
            if input.chars().count() > MAX_INPUT_CHARS {
                return Err(Rejection::TooLong);
            }
            let url = Url::parse(trimmed).map_err(|_| Rejection::NotAUrl)?;
            if !matches!(url.scheme(), "http" | "https") {
                return Err(Rejection::UnsupportedScheme);
            }
            if host_and_port(&url).as_ref() == Some(&self.own_host) {
                return Err(Rejection::SelfAddress);
            }
            Ok(normalize(trimmed, &url))
        }

        /// 경로 조각 하나를 Link Number로 읽어 Original URL을 찾는다. 없으면 `Ok(None)`.
        /// 올바른 Link Number가 아닌 경로 조각(`0`, `007`, `abc`, `+1`, 정수 범위를 넘는 수)도
        /// 저장소를 보지 않고 `Ok(None)`. 저장소를 읽지 못하면 `Err`.
        pub fn follow(&self, link_number_text: &str) -> Result<Option<String>, StorageError> {
            let Some(link_number) = parse_link_number(link_number_text) else {
                return Ok(None);
            };
            let original_url = self
                .conn()
                .query_row(
                    "SELECT original_url FROM short_links WHERE link_number = ?1",
                    params![link_number],
                    |row| row.get(0),
                )
                .optional()?;
            Ok(original_url)
        }
    }

    /// URL 정리 규칙: 스킴과 호스트를 소문자로, 기본 포트를 뺀다 (앞뒤 공백은 이미 뗐다).
    /// 사용자 정보, 경로, 쿼리, 프래그먼트, 끝의 `/`는 입력 그대로 둔다. 그래서
    /// `https://example.com`과 `https://example.com/`는 다른 Original URL이다.
    /// `url` 크레이트의 직렬화는 빈 경로에 `/`를 붙이고 퍼센트 인코딩도 하므로 쓰지 않는다.
    ///
    /// `scheme://` 꼴이 아니거나(`http:example.com` 등) 역슬래시, 제어 문자가 있어
    /// 입력 그대로 둘 수 없는 드문 입력만 파서가 다듬은 형태로 저장한다.
    fn normalize(trimmed: &str, url: &Url) -> String {
        let scheme = url.scheme();
        let after_scheme = trimmed
            .get(scheme.len()..)
            .and_then(|rest| rest.strip_prefix("://"));
        let after_scheme = match after_scheme {
            Some(rest) if !rest.contains('\\') && !rest.chars().any(|c| c.is_control()) => rest,
            _ => return url.as_str().to_string(),
        };
        let authority_end = after_scheme
            .find(['/', '?', '#'])
            .unwrap_or(after_scheme.len());
        let (authority, tail) = after_scheme.split_at(authority_end);
        let (userinfo, host_and_port) = match authority.rfind('@') {
            Some(at) => authority.split_at(at + 1),
            None => ("", authority),
        };
        // IPv6 호스트(`[::1]`) 안의 `:`는 포트 구분자가 아니다.
        let host_end = match host_and_port.rfind(':') {
            Some(colon) if !host_and_port[colon..].contains(']') => colon,
            _ => host_and_port.len(),
        };
        let (host, port) = host_and_port.split_at(host_end);
        // 파서가 기본 포트(와 빈 포트)를 `None`으로 읽는다.
        let port = if url.port().is_some() { port } else { "" };
        format!("{scheme}://{userinfo}{}{port}{tail}", host.to_lowercase())
    }

    /// Original URL의 Link Number. 없으면 새로 발급한다.
    ///
    /// 읽기와 INSERT를 한 IMMEDIATE 트랜잭션으로 묶어, 같은 파일을 여는 다른 연결과도
    /// 한 번에 하나씩만 확인하고 넣는다. 유일 제약에 걸린 INSERT도 AUTOINCREMENT 번호를
    /// 써 버리므로, 충돌을 제약에만 맡기면 번호가 건너뛴다. 그래도 충돌하면 기존 행을 읽는다.
    fn link_number_of(conn: &mut Connection, original_url: &str) -> rusqlite::Result<i64> {
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let existing = |tx: &Connection| {
            tx.query_row(
                "SELECT link_number FROM short_links WHERE original_url = ?1",
                params![original_url],
                |row| row.get(0),
            )
        };
        let link_number = match existing(&tx).optional()? {
            Some(link_number) => link_number,
            None => {
                tx.execute(
                    "INSERT INTO short_links (original_url) VALUES (?1)
                     ON CONFLICT (original_url) DO NOTHING",
                    params![original_url],
                )?;
                existing(&tx)?
            }
        };
        tx.commit()?;
        Ok(link_number)
    }

    /// 1부터 시작하는 10진수만 Link Number로 읽는다 (ADR 0001). 앞자리 0과 부호는 받지 않는다.
    fn parse_link_number(link_number_text: &str) -> Option<i64> {
        let first = link_number_text.bytes().next()?;
        if !(b'1'..=b'9').contains(&first) || !link_number_text.bytes().all(|b| b.is_ascii_digit())
        {
            return None;
        }
        link_number_text.parse().ok()
    }
}
