//! Shortener: Shorten과 Follow 규칙을 화면, HTTP와 떼어 놓은 모듈.
//!
//! 공개 인터페이스는 [`Shortener::shorten`]과 [`Shortener::follow`] 두 개뿐이다.
//! 서비스 주소와 저장소(SQLite 파일)는 생성 시점에 받는다.
//! [`Rejection`]은 화면에서도 쓰므로 server 기능 없이도 컴파일된다.

use serde::{Deserialize, Serialize};

/// Shorten이 Original URL을 받아들이지 않은 이유.
///
/// 입력 검증 사유(빈 입력, URL 형식 아님, 허용되지 않는 스킴, 너무 김, 자기 주소)는
/// 티켓 04에서 이 enum에 더한다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rejection {
    /// 저장소에 쓰거나 읽지 못했다.
    StorageUnavailable,
}

impl std::fmt::Display for Rejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
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
    use super::Rejection;
    use rusqlite::{params, Connection, OptionalExtension};
    use std::path::Path;
    use std::sync::Mutex;

    pub struct Shortener {
        base_url: String,
        conn: Mutex<Connection>,
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
                conn: Mutex::new(conn),
            })
        }

        /// Original URL에 Link Number를 발급하고 Short Link를 돌려준다.
        pub fn shorten(&self, input: &str) -> Result<String, Rejection> {
            let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
            conn.execute(
                "INSERT INTO short_links (original_url) VALUES (?1)",
                params![input],
            )
            .map_err(|_| Rejection::StorageUnavailable)?;
            let link_number = conn.last_insert_rowid();
            Ok(format!("{}/{}", self.base_url, link_number))
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
