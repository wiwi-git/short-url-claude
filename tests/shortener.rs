#![cfg(feature = "server")]

//! 경계 1: Shortener 모듈. 테스트마다 새 임시 SQLite를 쓴다.

use short_url::shortener::{Rejection, Shortener};

fn shortener(dir: &tempfile::TempDir) -> Shortener {
    Shortener::open("http://localhost:8080", dir.path().join("links.db")).unwrap()
}

#[test]
fn shorten_then_follow_round_trips_to_the_original_url() {
    let dir = tempfile::tempdir().unwrap();
    let shortener = shortener(&dir);

    let short_link = shortener.shorten("https://example.com/a").unwrap();

    assert_eq!(short_link, "http://localhost:8080/1");
    assert_eq!(shortener.follow("1").as_deref(), Some("https://example.com/a"));
}

#[test]
fn link_numbers_are_issued_in_order_starting_from_one() {
    let dir = tempfile::tempdir().unwrap();
    let shortener = shortener(&dir);

    let links: Vec<String> = [
        "https://example.com/a",
        "https://example.com/b",
        "https://example.com/c",
    ]
    .iter()
    .map(|url| shortener.shorten(url).unwrap())
    .collect();

    assert_eq!(
        links,
        [
            "http://localhost:8080/1",
            "http://localhost:8080/2",
            "http://localhost:8080/3"
        ]
    );
    assert_eq!(shortener.follow("2").as_deref(), Some("https://example.com/b"));
}

#[test]
fn short_link_starts_with_the_service_address_without_trailing_slash() {
    let dir = tempfile::tempdir().unwrap();
    let shortener = Shortener::open("https://sho.rt/", dir.path().join("links.db")).unwrap();

    assert_eq!(
        shortener.shorten("https://example.com/a").unwrap(),
        "https://sho.rt/1"
    );
}

#[test]
fn short_links_survive_reopening_the_storage() {
    let dir = tempfile::tempdir().unwrap();
    shortener(&dir).shorten("https://example.com/a").unwrap();

    let reopened = shortener(&dir);

    assert_eq!(reopened.follow("1").as_deref(), Some("https://example.com/a"));
    assert_eq!(
        reopened.shorten("https://example.com/b").unwrap(),
        "http://localhost:8080/2"
    );
}

#[test]
fn follow_finds_nothing_for_a_link_number_never_issued() {
    let dir = tempfile::tempdir().unwrap();
    let shortener = shortener(&dir);
    shortener.shorten("https://example.com/a").unwrap();

    assert_eq!(shortener.follow("2"), None);
}

#[test]
fn follow_finds_nothing_for_path_segments_that_are_not_link_numbers() {
    let dir = tempfile::tempdir().unwrap();
    let shortener = shortener(&dir);
    for i in 0..7 {
        shortener.shorten(&format!("https://example.com/{i}")).unwrap();
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
        assert_eq!(shortener.follow(path), None, "path segment {path:?}");
    }
    assert_eq!(shortener.follow("7").as_deref(), Some("https://example.com/6"));
}

#[test]
fn shorten_rejects_empty_and_whitespace_only_input() {
    let dir = tempfile::tempdir().unwrap();
    let shortener = shortener(&dir);

    for input in ["", "   ", "\t\n "] {
        assert_eq!(
            shortener.shorten(input),
            Err(Rejection::Empty),
            "input {input:?}"
        );
    }
}

#[test]
fn shorten_rejects_input_that_is_not_a_url() {
    let dir = tempfile::tempdir().unwrap();
    let shortener = shortener(&dir);

    for input in [
        "hello world",
        "example.com/a",
        "https://",
        "http://exa mple.com",
    ] {
        assert_eq!(
            shortener.shorten(input),
            Err(Rejection::NotAUrl),
            "input {input:?}"
        );
    }
}

#[test]
fn shorten_rejects_schemes_other_than_http_and_https() {
    let dir = tempfile::tempdir().unwrap();
    let shortener = shortener(&dir);

    for input in [
        "ftp://example.com/a",
        "javascript:alert(1)",
        "mailto:someone@example.com",
        "file:///etc/passwd",
    ] {
        assert_eq!(
            shortener.shorten(input),
            Err(Rejection::UnsupportedScheme),
            "input {input:?}"
        );
    }
    assert!(shortener.shorten("HTTP://example.com/a").is_ok());
}

#[test]
fn shorten_rejects_input_longer_than_2048_characters_before_trimming() {
    let dir = tempfile::tempdir().unwrap();
    let shortener = shortener(&dir);
    let url_of_len = |len: usize| {
        let prefix = "https://example.com/";
        format!("{prefix}{}", "a".repeat(len - prefix.len()))
    };

    assert_eq!(
        shortener.shorten(&url_of_len(2049)),
        Err(Rejection::TooLong)
    );
    assert_eq!(
        shortener.shorten(&format!("{} ", url_of_len(2048))),
        Err(Rejection::TooLong)
    );
    assert!(shortener.shorten(&url_of_len(2048)).is_ok());
}

#[test]
fn shorten_rejects_urls_on_the_service_address_host_and_port() {
    let dir = tempfile::tempdir().unwrap();
    let shortener = shortener(&dir);

    for input in [
        "http://localhost:8080/1",
        "http://localhost:8080",
        "https://LocalHost:8080/anything?x=1",
    ] {
        assert_eq!(
            shortener.shorten(input),
            Err(Rejection::SelfAddress),
            "input {input:?}"
        );
    }
    for input in [
        "http://localhost:3000/1",
        "http://localhost/1",
        "http://127.0.0.1:8080/1",
    ] {
        assert!(shortener.shorten(input).is_ok(), "input {input:?}");
    }
}

#[test]
fn self_address_check_ignores_default_ports() {
    let dir = tempfile::tempdir().unwrap();
    let shortener = Shortener::open("https://sho.rt", dir.path().join("links.db")).unwrap();

    for input in [
        "https://sho.rt/1",
        "https://SHO.RT:443/1",
        "http://sho.rt/1",
    ] {
        assert_eq!(
            shortener.shorten(input),
            Err(Rejection::SelfAddress),
            "input {input:?}"
        );
    }
    assert!(shortener.shorten("https://sho.rt:8443/1").is_ok());
}

#[test]
fn rejected_input_does_not_use_up_a_link_number() {
    let dir = tempfile::tempdir().unwrap();
    let shortener = shortener(&dir);
    shortener.shorten("https://example.com/a").unwrap();

    for input in [
        "",
        "not a url",
        "ftp://example.com/a",
        &format!("https://example.com/{}", "a".repeat(2048)),
        "http://localhost:8080/1",
    ] {
        assert!(shortener.shorten(input).is_err(), "input {input:?}");
    }

    assert_eq!(
        shortener.shorten("https://example.com/b").unwrap(),
        "http://localhost:8080/2"
    );
    assert_eq!(shortener.follow("3"), None);
}

#[test]
fn shortening_the_same_original_url_again_returns_the_same_short_link() {
    let dir = tempfile::tempdir().unwrap();
    let shortener = shortener(&dir);

    let first = shortener.shorten("https://example.com/a").unwrap();
    let again = shortener.shorten("https://example.com/a").unwrap();

    assert_eq!(first, "http://localhost:8080/1");
    assert_eq!(again, "http://localhost:8080/1");
    assert_eq!(
        shortener.shorten("https://example.com/b").unwrap(),
        "http://localhost:8080/2",
        "다시 Shorten해도 번호를 소모하지 않는다"
    );
}

#[test]
fn notation_differences_in_whitespace_case_and_default_port_give_the_same_short_link() {
    let dir = tempfile::tempdir().unwrap();
    let shortener = shortener(&dir);

    for input in [
        "https://example.com/a",
        "  https://example.com/a\t",
        "HTTPS://Example.COM/a",
        "https://example.com:443/a",
        "HTTPS://Example.COM:443/a",
    ] {
        assert_eq!(
            shortener.shorten(input).unwrap(),
            "http://localhost:8080/1",
            "input {input:?}"
        );
    }
    for input in ["HTTP://Example.com:80/a", "http://example.com/a"] {
        assert_eq!(
            shortener.shorten(input).unwrap(),
            "http://localhost:8080/2",
            "input {input:?}"
        );
    }
    assert_eq!(
        shortener.shorten("https://example.com:8443/a").unwrap(),
        "http://localhost:8080/3"
    );
}

#[test]
fn path_query_fragment_and_trailing_slash_are_kept_as_typed() {
    let dir = tempfile::tempdir().unwrap();
    let shortener = shortener(&dir);
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
            shortener.shorten(input).unwrap(),
            format!("http://localhost:8080/{}", i + 1),
            "input {input:?}"
        );
    }
    for (i, input) in inputs.iter().enumerate() {
        assert_eq!(
            shortener.follow(&(i + 1).to_string()).as_deref(),
            Some(*input)
        );
    }
}

#[test]
fn follow_goes_to_the_normalized_original_url() {
    let dir = tempfile::tempdir().unwrap();
    let shortener = shortener(&dir);

    shortener
        .shorten("  HTTPS://Example.COM:443/Path?Q=A#Frag ")
        .unwrap();
    shortener
        .shorten("HTTP://User:Pw@Example.COM:8080/x")
        .unwrap();

    assert_eq!(
        shortener.follow("1").as_deref(),
        Some("https://example.com/Path?Q=A#Frag")
    );
    assert_eq!(
        shortener.follow("2").as_deref(),
        Some("http://User:Pw@example.com:8080/x")
    );
}

#[test]
fn concurrent_shortens_of_the_same_original_url_make_one_short_link() {
    let dir = tempfile::tempdir().unwrap();
    // 저장소를 먼저 만들어 두고, 스레드마다 같은 파일에 자기 연결을 연다.
    drop(shortener(&dir));
    let barrier = std::sync::Barrier::new(16);

    let links: Vec<String> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..16)
            .map(|_| {
                scope.spawn(|| {
                    let own = shortener(&dir);
                    barrier.wait();
                    own.shorten("https://example.com/a").unwrap()
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });

    assert!(
        links.iter().all(|link| link == "http://localhost:8080/1"),
        "{links:?}"
    );
    let shortener = shortener(&dir);
    assert_eq!(shortener.follow("2"), None);
    assert_eq!(
        shortener.shorten("https://example.com/b").unwrap(),
        "http://localhost:8080/2"
    );
}
