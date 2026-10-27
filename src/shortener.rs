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
    /// 저장소에 쓰거나 읽지 못했다.
    StorageUnavailable,
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
            Rejection::StorageUnavailable => {
                f.write_str("지금은 줄일 수 없어요. 잠시 뒤 다시 시도해 주세요")
            }
        }
    }
}

#[cfg(feature = "server")]
pub use server::Shortener;

#[cfg(feature = "server")]
mod server {
    use super::{Rejection, MAX_INPUT_CHARS};
    use rusqlite::{params, Connection, OptionalExtension};
    use std::path::Path;
    use std::sync::Mutex;
    use url::Url;

    pub struct Shortener {
        base_url: String,
        /// 서비스 주소의 호스트와 포트(스킴 기본 포트면 `None`). 자기 주소 판단에 쓴다.
        own_host: Option<HostAndPort>,
        conn: Mutex<Connection>,
    }

    type HostAndPort = (String, Option<u16>);

    /// 호스트(소문자)와 기본 포트가 아닌 포트. URL 파서가 호스트를 소문자로, 기본 포트를 `None`으로 만든다.
    fn host_and_port(url: &Url) -> Option<HostAndPort> {
        Some((url.host_str()?.to_string(), url.port()))
    }

    impl Shortener {
        /// `db_path`의 SQLite 파일을 열고, 없으면 만들어 저장소를 준비한다.
        /// `base_url`은 Short Link 앞부분이며 끝의 `/`는 뗀다.
        pub fn open(base_url: &str, db_path: impl AsRef<Path>) -> rusqlite::Result<Self> {
            let conn = Connection::open(db_path)?;
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS short_links (
                     link_number  INTEGER PRIMARY KEY AUTOINCREMENT,
                     original_url TEXT NOT NULL UNIQUE
                 );",
            )?;
            Ok(Self {
                base_url: base_url.trim_end_matches('/').to_string(),
                own_host: Url::parse(base_url).ok().as_ref().and_then(host_and_port),
                conn: Mutex::new(conn),
            })
        }

        /// Original URL에 Link Number를 발급하고 Short Link를 돌려준다.
        pub fn shorten(&self, input: &str) -> Result<String, Rejection> {
            self.check(input)?;
            let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
            conn.execute(
                "INSERT INTO short_links (original_url) VALUES (?1)",
                params![input],
            )
            .map_err(|_| Rejection::StorageUnavailable)?;
            let link_number = conn.last_insert_rowid();
            Ok(format!("{}/{}", self.base_url, link_number))
        }

        /// 입력이 Original URL로 받아들일 만한지 본다. 거절이면 저장소에 손대지 않는다.
        fn check(&self, input: &str) -> Result<(), Rejection> {
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
            if self.own_host.is_some() && host_and_port(&url) == self.own_host {
                return Err(Rejection::SelfAddress);
            }
            Ok(())
        }

        /// 경로 조각을 Link Number로 읽어 Original URL을 찾는다. 없으면 `None`.
        /// 올바른 Link Number가 아닌 경로 조각(`0`, `007`, `abc`, `+1`, 정수 범위를 넘는 수)도 `None`.
        pub fn follow(&self, path: &str) -> Option<String> {
            let link_number = parse_link_number(path)?;
            let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
            conn.query_row(
                "SELECT original_url FROM short_links WHERE link_number = ?1",
                params![link_number],
                |row| row.get(0),
            )
            .optional()
            .ok()
            .flatten()
        }
    }

    /// 1부터 시작하는 10진수만 Link Number로 읽는다 (ADR 0001). 앞자리 0과 부호는 받지 않는다.
    fn parse_link_number(path: &str) -> Option<i64> {
        let first = path.bytes().next()?;
        if !(b'1'..=b'9').contains(&first) || !path.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        path.parse().ok()
    }
}
