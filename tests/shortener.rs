#![cfg(feature = "server")]

//! 경계 1: Shortener 모듈. 테스트마다 새 Postgres 데이터베이스를 쓴다 (ADR 0002).

mod common;

use common::{shortener, TestDb};
use short_url::shortener::{OpenError, Rejection, ShortenError, Shortener};

#[tokio::test]
async fn shorten_then_follow_round_trips_to_the_original_url() {
    let db = TestDb::new().await;
    let shortener = shortener(&db).await;

    let short_link = shortener.shorten("https://example.com/a").await.unwrap();

    assert_eq!(short_link, "http://localhost:8080/1");
    assert_eq!(
        shortener.follow("1").await.unwrap().as_deref(),
        Some("https://example.com/a")
    );
}

#[tokio::test]
async fn link_numbers_are_issued_in_order_starting_from_one() {
    let db = TestDb::new().await;
    let shortener = shortener(&db).await;

    let mut links = Vec::new();
    for url in [
        "https://example.com/a",
        "https://example.com/b",
        "https://example.com/c",
    ] {
        links.push(shortener.shorten(url).await.unwrap());
    }

    assert_eq!(
        links,
        [
            "http://localhost:8080/1",
            "http://localhost:8080/2",
            "http://localhost:8080/3"
        ]
    );
    assert_eq!(
        shortener.follow("2").await.unwrap().as_deref(),
        Some("https://example.com/b")
    );
}

#[tokio::test]
async fn short_link_starts_with_the_service_address_without_trailing_slash() {
    let db = TestDb::new().await;
    let shortener = Shortener::open("https://sho.rt/", &db.url).await.unwrap();

    assert_eq!(
        shortener.shorten("https://example.com/a").await.unwrap(),
        "https://sho.rt/1"
    );
}

#[tokio::test]
async fn short_links_survive_reopening_the_storage() {
    let db = TestDb::new().await;
    shortener(&db).await.shorten("https://example.com/a").await.unwrap();

    let reopened = shortener(&db).await;

    assert_eq!(
        reopened.follow("1").await.unwrap().as_deref(),
        Some("https://example.com/a")
    );
    assert_eq!(
        reopened.shorten("https://example.com/b").await.unwrap(),
        "http://localhost:8080/2"
    );
}

#[tokio::test]
async fn follow_finds_nothing_for_a_link_number_never_issued() {
    let db = TestDb::new().await;
    let shortener = shortener(&db).await;
    shortener.shorten("https://example.com/a").await.unwrap();

    assert_eq!(shortener.follow("2").await, Ok(None));
}

#[tokio::test]
async fn follow_finds_nothing_for_path_segments_that_are_not_link_numbers() {
    let db = TestDb::new().await;
    let shortener = shortener(&db).await;
    for i in 0..7 {
        shortener
            .shorten(&format!("https://example.com/{i}")).await
            .unwrap();
    }

    for path in [
        "0",
        "007",
        "07",
        "abc",
        "7a",
        "",
        "+7",
        "-7",
        "99999999999999999999999",
    ] {
        assert_eq!(shortener.follow(path).await, Ok(None), "path segment {path:?}");
    }
    assert_eq!(
        shortener.follow("7").await.unwrap().as_deref(),
        Some("https://example.com/6")
    );
}

#[tokio::test]
async fn shorten_rejects_empty_and_whitespace_only_input() {
    let db = TestDb::new().await;
    let shortener = shortener(&db).await;

    for input in ["", "   ", "\t\n "] {
        assert_eq!(
            shortener.shorten(input).await,
            Err(ShortenError::Rejected(Rejection::Empty)),
            "input {input:?}"
        );
    }
}

#[tokio::test]
async fn shorten_rejects_input_that_is_not_a_url() {
    let db = TestDb::new().await;
    let shortener = shortener(&db).await;

    for input in [
        "hello world",
        "example.com/a",
        "https://",
        "http://exa mple.com",
    ] {
        assert_eq!(
            shortener.shorten(input).await,
            Err(ShortenError::Rejected(Rejection::NotAUrl)),
            "input {input:?}"
        );
    }
}

#[tokio::test]
async fn shorten_rejects_schemes_other_than_http_and_https() {
    let db = TestDb::new().await;
    let shortener = shortener(&db).await;

    for input in [
        "ftp://example.com/a",
        "javascript:alert(1)",
        "mailto:someone@example.com",
        "file:///etc/passwd",
    ] {
        assert_eq!(
            shortener.shorten(input).await,
            Err(ShortenError::Rejected(Rejection::UnsupportedScheme)),
            "input {input:?}"
        );
    }
    assert!(shortener.shorten("HTTP://example.com/a").await.is_ok());
}

#[tokio::test]
async fn shorten_rejects_input_longer_than_2048_characters_before_trimming() {
    let db = TestDb::new().await;
    let shortener = shortener(&db).await;
    let url_of_len = |len: usize| {
        let prefix = "https://example.com/";
        format!("{prefix}{}", "a".repeat(len - prefix.len()))
    };

    assert_eq!(
        shortener.shorten(&url_of_len(2049)).await,
        Err(ShortenError::Rejected(Rejection::TooLong))
    );
    assert_eq!(
        shortener.shorten(&format!("{} ", url_of_len(2048))).await,
        Err(ShortenError::Rejected(Rejection::TooLong))
    );
    assert!(shortener.shorten(&url_of_len(2048)).await.is_ok());
}

#[tokio::test]
async fn shorten_rejects_urls_on_the_service_address_host_and_port() {
    let db = TestDb::new().await;
    let shortener = shortener(&db).await;

    for input in [
        "http://localhost:8080/1",
        "http://localhost:8080",
        "https://LocalHost:8080/anything?x=1",
    ] {
        assert_eq!(
            shortener.shorten(input).await,
            Err(ShortenError::Rejected(Rejection::SelfAddress)),
            "input {input:?}"
        );
    }
    for input in [
        "http://localhost:3000/1",
        "http://localhost/1",
        "http://127.0.0.1:8080/1",
    ] {
        assert!(shortener.shorten(input).await.is_ok(), "input {input:?}");
    }
}

#[tokio::test]
async fn self_address_check_ignores_default_ports() {
    let db = TestDb::new().await;
    let shortener = Shortener::open("https://sho.rt", &db.url).await.unwrap();

    for input in [
        "https://sho.rt/1",
        "https://SHO.RT:443/1",
        "http://sho.rt/1",
    ] {
        assert_eq!(
            shortener.shorten(input).await,
            Err(ShortenError::Rejected(Rejection::SelfAddress)),
            "input {input:?}"
        );
    }
    assert!(shortener.shorten("https://sho.rt:8443/1").await.is_ok());
}

#[tokio::test]
async fn rejected_input_does_not_use_up_a_link_number() {
    let db = TestDb::new().await;
    let shortener = shortener(&db).await;
    shortener.shorten("https://example.com/a").await.unwrap();

    for input in [
        "",
        "not a url",
        "ftp://example.com/a",
        &format!("https://example.com/{}", "a".repeat(2048)),
        "http://localhost:8080/1",
    ] {
        assert!(shortener.shorten(input).await.is_err(), "input {input:?}");
    }

    assert_eq!(
        shortener.shorten("https://example.com/b").await.unwrap(),
        "http://localhost:8080/2"
    );
    assert_eq!(shortener.follow("3").await, Ok(None));
}

#[tokio::test]
async fn shortening_the_same_original_url_again_returns_the_same_short_link() {
    let db = TestDb::new().await;
    let shortener = shortener(&db).await;

    let first = shortener.shorten("https://example.com/a").await.unwrap();
    let again = shortener.shorten("https://example.com/a").await.unwrap();

    assert_eq!(first, "http://localhost:8080/1");
    assert_eq!(again, "http://localhost:8080/1");
    assert_eq!(
        shortener.shorten("https://example.com/b").await.unwrap(),
        "http://localhost:8080/2",
        "다시 Shorten해도 번호를 소모하지 않는다"
    );
}

#[tokio::test]
async fn notation_differences_in_whitespace_case_and_default_port_give_the_same_short_link() {
    let db = TestDb::new().await;
    let shortener = shortener(&db).await;

    for input in [
        "https://example.com/a",
        "  https://example.com/a\t",
        "HTTPS://Example.COM/a",
        "https://example.com:443/a",
        "HTTPS://Example.COM:443/a",
    ] {
        assert_eq!(
            shortener.shorten(input).await.unwrap(),
            "http://localhost:8080/1",
            "input {input:?}"
        );
    }
    for input in ["HTTP://Example.com:80/a", "http://example.com/a"] {
        assert_eq!(
            shortener.shorten(input).await.unwrap(),
            "http://localhost:8080/2",
            "input {input:?}"
        );
    }
    assert_eq!(
        shortener.shorten("https://example.com:8443/a").await.unwrap(),
        "http://localhost:8080/3"
    );
}

#[tokio::test]
async fn path_query_fragment_and_trailing_slash_are_kept_as_typed() {
    let db = TestDb::new().await;
    let shortener = shortener(&db).await;
    let inputs = [
        "https://example.com",
        "https://example.com/",
        "https://example.com/a",
        "https://example.com/a/",
        "https://example.com/A",
        "https://example.com/a?q=1",
        "https://example.com/a?Q=1",
        "https://example.com/a#x",
        "https://example.com/~a",
        "https://example.com/%7Ea",
    ];

    for (i, input) in inputs.iter().enumerate() {
        assert_eq!(
            shortener.shorten(input).await.unwrap(),
            format!("http://localhost:8080/{}", i + 1),
            "input {input:?}"
        );
    }
    for (i, input) in inputs.iter().enumerate() {
        assert_eq!(
            shortener.follow(&(i + 1).to_string()).await.unwrap().as_deref(),
            Some(*input)
        );
    }
}

#[tokio::test]
async fn follow_goes_to_the_normalized_original_url() {
    let db = TestDb::new().await;
    let shortener = shortener(&db).await;

    shortener
        .shorten("  HTTPS://Example.COM:443/Path?Q=A#Frag ").await
        .unwrap();
    shortener
        .shorten("HTTP://User:Pw@Example.COM:8080/x").await
        .unwrap();

    assert_eq!(
        shortener.follow("1").await.unwrap().as_deref(),
        Some("https://example.com/Path?Q=A#Frag")
    );
    assert_eq!(
        shortener.follow("2").await.unwrap().as_deref(),
        Some("http://User:Pw@example.com:8080/x")
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn concurrent_shortens_make_one_short_link_per_original_url_without_skipping_numbers() {
    let db = TestDb::new().await;
    // 서버 여러 대처럼, 작업마다 같은 데이터베이스에 자기 Shortener(자기 연결 풀)를 동시에 연다.
    const TASKS: usize = 16;
    let barrier = std::sync::Arc::new(tokio::sync::Barrier::new(TASKS));
    let tasks: Vec<_> = (0..TASKS)
        .map(|i| {
            let url = db.url.clone();
            let barrier = barrier.clone();
            tokio::spawn(async move {
                let own = Shortener::open("http://localhost:8080", &url).await.unwrap();
                barrier.wait().await;
                let same = own.shorten("https://example.com/same").await.unwrap();
                let distinct = own
                    .shorten(&format!("https://example.com/{i}"))
                    .await
                    .unwrap();
                (same, distinct)
            })
        })
        .collect();
    let mut same_links = Vec::new();
    let mut link_numbers = Vec::new();
    for task in tasks {
        let (same, distinct) = task.await.unwrap();
        for link in [&same, &distinct] {
            let n: u64 = link.rsplit('/').next().unwrap().parse().unwrap();
            link_numbers.push(n);
        }
        same_links.push(same);
    }

    assert!(
        same_links.iter().all(|link| link == &same_links[0]),
        "같은 Original URL은 Short Link 하나: {same_links:?}"
    );
    link_numbers.sort();
    link_numbers.dedup();
    assert_eq!(
        link_numbers,
        (1..=TASKS as u64 + 1).collect::<Vec<_>>(),
        "Original URL {}개에 1부터 빈틈없는 번호",
        TASKS + 1
    );
    let shortener = shortener(&db).await;
    assert_eq!(
        shortener.shorten("https://example.com/next").await.unwrap(),
        format!("http://localhost:8080/{}", TASKS + 2)
    );
}

#[tokio::test]
async fn shorten_reports_storage_failure_separately_from_rejections() {
    let db = TestDb::new().await;
    let shortener = shortener(&db).await;
    db.break_storage().await;

    assert!(matches!(
        shortener.shorten("https://example.com/a").await,
        Err(ShortenError::Storage(_))
    ));
    assert_eq!(
        shortener.shorten("ftp://example.com/a").await,
        Err(ShortenError::Rejected(Rejection::UnsupportedScheme)),
        "거절은 저장소에 손대지 않으므로 그대로 거절이다"
    );
}

#[tokio::test]
async fn opening_with_a_service_address_that_is_not_an_http_url_fails() {
    let db = TestDb::new().await;

    for base_url in [
        "localhost:8080",
        "not a url",
        "",
        "file:///tmp/x",
        "http://",
    ] {
        assert!(
            matches!(
                Shortener::open(base_url, &db.url).await,
                Err(OpenError::InvalidBaseUrl(_))
            ),
            "base_url {base_url:?}"
        );
    }
}

#[tokio::test]
async fn follow_reports_storage_failure_instead_of_finding_nothing() {
    let db = TestDb::new().await;
    let shortener = shortener(&db).await;
    shortener.shorten("https://example.com/a").await.unwrap();
    db.break_storage().await;

    assert!(shortener.follow("1").await.is_err());
    assert_eq!(
        shortener.follow("abc").await,
        Ok(None),
        "Link Number가 아닌 경로 조각은 저장소를 보지 않고 없음이다"
    );
}
