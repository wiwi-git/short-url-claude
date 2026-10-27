#![cfg(feature = "server")]

//! 경계 1: Shortener 모듈. 테스트마다 새 임시 SQLite를 쓴다.

use short_url::shortener::Shortener;

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
