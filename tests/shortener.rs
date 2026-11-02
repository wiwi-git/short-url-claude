#![cfg(feature = "server")]

//! 경계 1: Shortener 모듈. 테스트마다 새 Postgres 데이터베이스를 쓴다 (ADR 0002).

mod common;

use common::{shortener, TestDb};
use short_url::shortener::{
    OpenError, Rejection, ShortLink, ShortLinkPage, ShortenError, Shortener, Stats, User,
};

#[tokio::test]
async fn shorten_then_follow_round_trips_to_the_original_url() {
    let db = TestDb::new().await;
    let shortener = shortener(&db).await;
    let user = common::user(&shortener).await;

    let short_link = shortener
        .shorten(&user, "https://example.com/a")
        .await
        .unwrap();

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
    let user = common::user(&shortener).await;

    let mut links = Vec::new();
    for url in [
        "https://example.com/a",
        "https://example.com/b",
        "https://example.com/c",
    ] {
        links.push(shortener.shorten(&user, url).await.unwrap());
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
    let user = common::user(&shortener).await;

    assert_eq!(
        shortener
            .shorten(&user, "https://example.com/a")
            .await
            .unwrap(),
        "https://sho.rt/1"
    );
}

#[tokio::test]
async fn short_links_survive_reopening_the_storage() {
    let db = TestDb::new().await;
    let first = shortener(&db).await;
    let user = common::user(&first).await;
    first.shorten(&user, "https://example.com/a").await.unwrap();

    let reopened = shortener(&db).await;

    assert_eq!(
        reopened.follow("1").await.unwrap().as_deref(),
        Some("https://example.com/a")
    );
    assert_eq!(
        reopened
            .shorten(&user, "https://example.com/b")
            .await
            .unwrap(),
        "http://localhost:8080/2"
    );
}

#[tokio::test]
async fn follow_finds_nothing_for_a_link_number_never_issued() {
    let db = TestDb::new().await;
    let shortener = shortener(&db).await;
    let user = common::user(&shortener).await;
    shortener
        .shorten(&user, "https://example.com/a")
        .await
        .unwrap();

    assert_eq!(shortener.follow("2").await, Ok(None));
}

#[tokio::test]
async fn follow_finds_nothing_for_path_segments_that_are_not_link_numbers() {
    let db = TestDb::new().await;
    let shortener = shortener(&db).await;
    let user = common::user(&shortener).await;
    for i in 0..7 {
        shortener
            .shorten(&user, &format!("https://example.com/{i}"))
            .await
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
        assert_eq!(
            shortener.follow(path).await,
            Ok(None),
            "path segment {path:?}"
        );
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
    let user = common::user(&shortener).await;

    for input in ["", "   ", "\t\n "] {
        assert_eq!(
            shortener.shorten(&user, input).await,
            Err(ShortenError::Rejected(Rejection::Empty)),
            "input {input:?}"
        );
    }
}

#[tokio::test]
async fn shorten_rejects_input_that_is_not_a_url() {
    let db = TestDb::new().await;
    let shortener = shortener(&db).await;
    let user = common::user(&shortener).await;

    for input in [
        "hello world",
        "example.com/a",
        "https://",
        "http://exa mple.com",
    ] {
        assert_eq!(
            shortener.shorten(&user, input).await,
            Err(ShortenError::Rejected(Rejection::NotAUrl)),
            "input {input:?}"
        );
    }
}

#[tokio::test]
async fn shorten_rejects_schemes_other_than_http_and_https() {
    let db = TestDb::new().await;
    let shortener = shortener(&db).await;
    let user = common::user(&shortener).await;

    for input in [
        "ftp://example.com/a",
        "javascript:alert(1)",
        "mailto:someone@example.com",
        "file:///etc/passwd",
    ] {
        assert_eq!(
            shortener.shorten(&user, input).await,
            Err(ShortenError::Rejected(Rejection::UnsupportedScheme)),
            "input {input:?}"
        );
    }
    assert!(shortener
        .shorten(&user, "HTTP://example.com/a")
        .await
        .is_ok());
}

#[tokio::test]
async fn shorten_rejects_input_longer_than_2048_characters_before_trimming() {
    let db = TestDb::new().await;
    let shortener = shortener(&db).await;
    let user = common::user(&shortener).await;
    let url_of_len = |len: usize| {
        let prefix = "https://example.com/";
        format!("{prefix}{}", "a".repeat(len - prefix.len()))
    };

    assert_eq!(
        shortener.shorten(&user, &url_of_len(2049)).await,
        Err(ShortenError::Rejected(Rejection::TooLong))
    );
    assert_eq!(
        shortener
            .shorten(&user, &format!("{} ", url_of_len(2048)))
            .await,
        Err(ShortenError::Rejected(Rejection::TooLong))
    );
    assert!(shortener.shorten(&user, &url_of_len(2048)).await.is_ok());
}

#[tokio::test]
async fn shorten_rejects_urls_on_the_service_address_host_and_port() {
    let db = TestDb::new().await;
    let shortener = shortener(&db).await;
    let user = common::user(&shortener).await;

    for input in [
        "http://localhost:8080/1",
        "http://localhost:8080",
        "https://LocalHost:8080/anything?x=1",
    ] {
        assert_eq!(
            shortener.shorten(&user, input).await,
            Err(ShortenError::Rejected(Rejection::SelfAddress)),
            "input {input:?}"
        );
    }
    for input in [
        "http://localhost:3000/1",
        "http://localhost/1",
        "http://127.0.0.1:8080/1",
    ] {
        assert!(
            shortener.shorten(&user, input).await.is_ok(),
            "input {input:?}"
        );
    }
}

#[tokio::test]
async fn self_address_check_ignores_default_ports() {
    let db = TestDb::new().await;
    let shortener = Shortener::open("https://sho.rt", &db.url).await.unwrap();
    let user = common::user(&shortener).await;

    for input in [
        "https://sho.rt/1",
        "https://SHO.RT:443/1",
        "http://sho.rt/1",
    ] {
        assert_eq!(
            shortener.shorten(&user, input).await,
            Err(ShortenError::Rejected(Rejection::SelfAddress)),
            "input {input:?}"
        );
    }
    assert!(shortener
        .shorten(&user, "https://sho.rt:8443/1")
        .await
        .is_ok());
}

#[tokio::test]
async fn rejected_input_does_not_use_up_a_link_number() {
    let db = TestDb::new().await;
    let shortener = shortener(&db).await;
    let user = common::user(&shortener).await;
    shortener
        .shorten(&user, "https://example.com/a")
        .await
        .unwrap();

    for input in [
        "",
        "not a url",
        "ftp://example.com/a",
        &format!("https://example.com/{}", "a".repeat(2048)),
        "http://localhost:8080/1",
    ] {
        assert!(
            shortener.shorten(&user, input).await.is_err(),
            "input {input:?}"
        );
    }

    assert_eq!(
        shortener
            .shorten(&user, "https://example.com/b")
            .await
            .unwrap(),
        "http://localhost:8080/2"
    );
    assert_eq!(shortener.follow("3").await, Ok(None));
}

#[tokio::test]
async fn shortening_the_same_original_url_again_returns_the_same_short_link() {
    let db = TestDb::new().await;
    let shortener = shortener(&db).await;
    let user = common::user(&shortener).await;

    let first = shortener
        .shorten(&user, "https://example.com/a")
        .await
        .unwrap();
    let again = shortener
        .shorten(&user, "https://example.com/a")
        .await
        .unwrap();

    assert_eq!(first, "http://localhost:8080/1");
    assert_eq!(again, "http://localhost:8080/1");
    assert_eq!(
        shortener
            .shorten(&user, "https://example.com/b")
            .await
            .unwrap(),
        "http://localhost:8080/2",
        "다시 Shorten해도 번호를 소모하지 않는다"
    );
}

#[tokio::test]
async fn notation_differences_in_whitespace_case_and_default_port_give_the_same_short_link() {
    let db = TestDb::new().await;
    let shortener = shortener(&db).await;
    let user = common::user(&shortener).await;

    for input in [
        "https://example.com/a",
        "  https://example.com/a\t",
        "HTTPS://Example.COM/a",
        "https://example.com:443/a",
        "HTTPS://Example.COM:443/a",
    ] {
        assert_eq!(
            shortener.shorten(&user, input).await.unwrap(),
            "http://localhost:8080/1",
            "input {input:?}"
        );
    }
    for input in ["HTTP://Example.com:80/a", "http://example.com/a"] {
        assert_eq!(
            shortener.shorten(&user, input).await.unwrap(),
            "http://localhost:8080/2",
            "input {input:?}"
        );
    }
    assert_eq!(
        shortener
            .shorten(&user, "https://example.com:8443/a")
            .await
            .unwrap(),
        "http://localhost:8080/3"
    );
}

#[tokio::test]
async fn path_query_fragment_and_trailing_slash_are_kept_as_typed() {
    let db = TestDb::new().await;
    let shortener = shortener(&db).await;
    let user = common::user(&shortener).await;
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
            shortener.shorten(&user, input).await.unwrap(),
            format!("http://localhost:8080/{}", i + 1),
            "input {input:?}"
        );
    }
    for (i, input) in inputs.iter().enumerate() {
        assert_eq!(
            shortener
                .follow(&(i + 1).to_string())
                .await
                .unwrap()
                .as_deref(),
            Some(*input)
        );
    }
}

#[tokio::test]
async fn follow_goes_to_the_normalized_original_url() {
    let db = TestDb::new().await;
    let shortener = shortener(&db).await;
    let user = common::user(&shortener).await;

    shortener
        .shorten(&user, "  HTTPS://Example.COM:443/Path?Q=A#Frag ")
        .await
        .unwrap();
    shortener
        .shorten(&user, "HTTP://User:Pw@Example.COM:8080/x")
        .await
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
                let own = Shortener::open("http://localhost:8080", &url)
                    .await
                    .unwrap();
                let user = own.sign_in(i as i64, "동시").await.unwrap();
                barrier.wait().await;
                let same = own
                    .shorten(&user, "https://example.com/same")
                    .await
                    .unwrap();
                let distinct = own
                    .shorten(&user, &format!("https://example.com/{i}"))
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
    let user = common::user(&shortener).await;
    assert_eq!(
        shortener
            .shorten(&user, "https://example.com/next")
            .await
            .unwrap(),
        format!("http://localhost:8080/{}", TASKS + 2)
    );
}

#[tokio::test]
async fn shorten_reports_storage_failure_separately_from_rejections() {
    let db = TestDb::new().await;
    let shortener = shortener(&db).await;
    let user = common::user(&shortener).await;
    db.break_storage().await;

    assert!(matches!(
        shortener.shorten(&user, "https://example.com/a").await,
        Err(ShortenError::Storage(_))
    ));
    assert_eq!(
        shortener.shorten(&user, "ftp://example.com/a").await,
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
    let user = common::user(&shortener).await;
    shortener
        .shorten(&user, "https://example.com/a")
        .await
        .unwrap();
    db.break_storage().await;

    assert!(shortener.follow("1").await.is_err());
    assert_eq!(
        shortener.follow("abc").await,
        Ok(None),
        "Link Number가 아닌 경로 조각은 저장소를 보지 않고 없음이다"
    );
}

/// 페이지의 Short Link 주소만 순서대로.
fn links(page: &ShortLinkPage) -> Vec<&str> {
    page.short_links
        .iter()
        .map(|link| link.short_link.as_str())
        .collect()
}

#[tokio::test]
async fn my_links_are_empty_before_any_shorten() {
    let db = TestDb::new().await;
    let shortener = shortener(&db).await;
    let user = common::user(&shortener).await;

    let page = shortener.my_links(&user, 0).await.unwrap();

    assert_eq!(
        page,
        ShortLinkPage {
            short_links: vec![],
            has_next: false
        }
    );
}

#[tokio::test]
async fn my_links_are_newest_first_and_do_not_repeat_reshortened_urls() {
    let db = TestDb::new().await;
    let shortener = shortener(&db).await;
    let user = common::user(&shortener).await;
    for url in [
        "https://example.com/a",
        "https://example.com/b",
        "https://example.com/a",
        "https://example.com/c",
    ] {
        shortener.shorten(&user, url).await.unwrap();
    }

    let page = shortener.my_links(&user, 0).await.unwrap();

    assert_eq!(
        page,
        ShortLinkPage {
            short_links: [3, 2, 1]
                .map(|n| ShortLink {
                    link_number: n,
                    short_link: format!("http://localhost:8080/{n}"),
                })
                .to_vec(),
            has_next: false,
        }
    );
}

#[tokio::test]
async fn my_links_page_by_50_and_tell_whether_a_next_page_exists() {
    let db = TestDb::new().await;
    let shortener = shortener(&db).await;
    let user = common::user(&shortener).await;
    for i in 1..=50 {
        shortener
            .shorten(&user, &format!("https://example.com/{i}"))
            .await
            .unwrap();
    }

    let exactly_fifty = shortener.my_links(&user, 0).await.unwrap();
    assert_eq!(exactly_fifty.short_links.len(), 50);
    assert_eq!(links(&exactly_fifty)[0], "http://localhost:8080/50");
    assert_eq!(links(&exactly_fifty)[49], "http://localhost:8080/1");
    assert!(!exactly_fifty.has_next, "딱 50개면 다음 페이지가 없다");

    shortener
        .shorten(&user, "https://example.com/51")
        .await
        .unwrap();

    let first = shortener.my_links(&user, 0).await.unwrap();
    assert_eq!(first.short_links.len(), 50);
    assert_eq!(links(&first)[0], "http://localhost:8080/51");
    assert_eq!(links(&first)[49], "http://localhost:8080/2");
    assert!(first.has_next, "51개면 다음 페이지가 있다");

    let second = shortener.my_links(&user, 1).await.unwrap();
    assert_eq!(links(&second), ["http://localhost:8080/1"]);
    assert!(!second.has_next);
    assert_eq!(
        shortener.my_links(&user, 2).await.unwrap(),
        ShortLinkPage {
            short_links: vec![],
            has_next: false
        },
        "범위를 넘는 페이지는 빈 페이지"
    );
}

#[tokio::test]
async fn users_shortening_the_same_original_url_share_one_short_link() {
    let db = TestDb::new().await;
    let shortener = shortener(&db).await;
    let alice = shortener.sign_in(1, "앨리스").await.unwrap();
    let bob = shortener.sign_in(2, "밥").await.unwrap();

    let alices = shortener
        .shorten(&alice, "https://example.com/shared")
        .await
        .unwrap();
    shortener
        .shorten(&alice, "https://example.com/only-alice")
        .await
        .unwrap();
    let bobs = shortener
        .shorten(&bob, "https://example.com/shared")
        .await
        .unwrap();

    assert_eq!(alices, "http://localhost:8080/1");
    assert_eq!(bobs, alices, "같은 Original URL은 같은 Short Link");
    assert_eq!(
        links(&shortener.my_links(&alice, 0).await.unwrap()),
        ["http://localhost:8080/2", "http://localhost:8080/1"]
    );
    assert_eq!(
        links(&shortener.my_links(&bob, 0).await.unwrap()),
        ["http://localhost:8080/1"],
        "다른 User의 Short Link는 보이지 않는다"
    );
}

#[tokio::test]
async fn shared_short_link_goes_to_the_top_of_my_links_when_i_add_it() {
    let db = TestDb::new().await;
    let shortener = shortener(&db).await;
    let alice = shortener.sign_in(1, "앨리스").await.unwrap();
    let bob = shortener.sign_in(2, "밥").await.unwrap();
    shortener
        .shorten(&alice, "https://example.com/old")
        .await
        .unwrap();
    shortener
        .shorten(&bob, "https://example.com/mine")
        .await
        .unwrap();

    shortener
        .shorten(&bob, "https://example.com/old")
        .await
        .unwrap();

    assert_eq!(
        links(&shortener.my_links(&bob, 0).await.unwrap()),
        ["http://localhost:8080/1", "http://localhost:8080/2"],
        "My Links는 Link Number가 아니라 담은 순서의 최신순"
    );
}

#[tokio::test]
async fn signing_in_again_keeps_the_user_and_updates_the_nickname() {
    let db = TestDb::new().await;
    let shortener = shortener(&db).await;

    let first = shortener.sign_in(42, "예전 이름").await.unwrap();
    let again = shortener.sign_in(42, "새 이름").await.unwrap();
    let other = shortener.sign_in(43, "다른 사람").await.unwrap();

    assert_eq!(again.id, first.id);
    assert_eq!(again.nickname, "새 이름");
    assert_ne!(other.id, first.id);
}

#[tokio::test]
async fn session_finds_its_user_until_it_ends() {
    let db = TestDb::new().await;
    let shortener = shortener(&db).await;
    let user = shortener.sign_in(42, "예전 이름").await.unwrap();

    let token = shortener.start_session(&user).await.unwrap();
    shortener.sign_in(42, "새 이름").await.unwrap();

    assert_eq!(
        shortener.user_for_session(&token).await.unwrap(),
        Some(User {
            id: user.id,
            nickname: "새 이름".to_string()
        })
    );
    assert_eq!(shortener.user_for_session("unknown").await.unwrap(), None);

    shortener.end_session(&token).await.unwrap();
    assert_eq!(shortener.user_for_session(&token).await.unwrap(), None);
}

#[tokio::test]
async fn sessions_are_unguessable_and_expire() {
    let db = TestDb::new().await;
    let shortener = shortener(&db).await;
    let user = common::user(&shortener).await;

    let one = shortener.start_session(&user).await.unwrap();
    let two = shortener.start_session(&user).await.unwrap();
    assert_ne!(one, two);
    assert!(one.len() >= 64, "토큰은 256비트 이상: {one}");

    let short = shortener
        .start_session_lasting(&user, std::time::Duration::from_secs(1))
        .await
        .unwrap();
    assert!(shortener.user_for_session(&short).await.unwrap().is_some());
    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
    assert_eq!(
        shortener.user_for_session(&short).await.unwrap(),
        None,
        "만료된 세션은 로그인 안 한 것"
    );
    assert!(shortener.user_for_session(&one).await.unwrap().is_some());
}

#[tokio::test]
async fn removing_a_short_link_others_still_hold_keeps_it_working() {
    let db = TestDb::new().await;
    let shortener = shortener(&db).await;
    let alice = shortener.sign_in(1, "앨리스").await.unwrap();
    let bob = shortener.sign_in(2, "밥").await.unwrap();
    shortener
        .shorten(&alice, "https://example.com/shared")
        .await
        .unwrap();
    shortener
        .shorten(&bob, "https://example.com/shared")
        .await
        .unwrap();

    shortener.remove(&alice, 1).await.unwrap();

    assert!(shortener
        .my_links(&alice, 0)
        .await
        .unwrap()
        .short_links
        .is_empty());
    assert_eq!(
        links(&shortener.my_links(&bob, 0).await.unwrap()),
        ["http://localhost:8080/1"]
    );
    assert_eq!(
        shortener.follow("1").await.unwrap().as_deref(),
        Some("https://example.com/shared")
    );
}

#[tokio::test]
async fn removing_the_last_holder_deletes_the_short_link_and_its_number_is_never_reused() {
    let db = TestDb::new().await;
    let shortener = shortener(&db).await;
    let user = common::user(&shortener).await;
    shortener
        .shorten(&user, "https://example.com/a")
        .await
        .unwrap();
    shortener
        .shorten(&user, "https://example.com/b")
        .await
        .unwrap();

    shortener.remove(&user, 2).await.unwrap();

    assert_eq!(shortener.follow("2").await.unwrap(), None);
    assert_eq!(
        shortener
            .shorten(&user, "https://example.com/c")
            .await
            .unwrap(),
        "http://localhost:8080/3",
        "지운 가장 큰 번호도 다시 쓰지 않는다"
    );
    assert_eq!(
        shortener
            .shorten(&user, "https://example.com/b")
            .await
            .unwrap(),
        "http://localhost:8080/4",
        "지운 Original URL을 다시 줄이면 새 번호"
    );
    assert_eq!(
        links(&shortener.my_links(&user, 0).await.unwrap()),
        [
            "http://localhost:8080/4",
            "http://localhost:8080/3",
            "http://localhost:8080/1"
        ]
    );
}

#[tokio::test]
async fn removing_a_short_link_not_in_my_links_does_nothing() {
    let db = TestDb::new().await;
    let shortener = shortener(&db).await;
    let alice = shortener.sign_in(1, "앨리스").await.unwrap();
    let bob = shortener.sign_in(2, "밥").await.unwrap();
    shortener
        .shorten(&alice, "https://example.com/a")
        .await
        .unwrap();

    shortener.remove(&bob, 1).await.unwrap();
    shortener.remove(&bob, 999).await.unwrap();

    assert_eq!(
        links(&shortener.my_links(&alice, 0).await.unwrap()),
        ["http://localhost:8080/1"]
    );
    assert_eq!(
        shortener.follow("1").await.unwrap().as_deref(),
        Some("https://example.com/a")
    );
}

#[tokio::test]
async fn stats_count_users_and_short_links_that_still_work() {
    let db = TestDb::new().await;
    let shortener = shortener(&db).await;
    assert_eq!(
        shortener.stats().await.unwrap(),
        Stats {
            users: 0,
            short_links: 0
        }
    );

    let alice = shortener.sign_in(1, "앨리스").await.unwrap();
    let bob = shortener.sign_in(2, "밥").await.unwrap();
    shortener.sign_in(1, "앨리스 새 이름").await.unwrap();
    shortener
        .shorten(&alice, "https://example.com/a")
        .await
        .unwrap();
    shortener
        .shorten(&bob, "https://example.com/a")
        .await
        .unwrap();
    shortener
        .shorten(&bob, "https://example.com/b")
        .await
        .unwrap();
    shortener.remove(&bob, 2).await.unwrap();

    assert_eq!(
        shortener.stats().await.unwrap(),
        Stats {
            users: 2,
            short_links: 1
        }
    );
}

#[tokio::test]
async fn opening_storage_from_before_login_deletes_ownerless_short_links_once() {
    let db = TestDb::new().await;
    {
        // 로그인 기능 전 스키마: short_links 표만 있다.
        use sqlx::Connection;
        let mut conn = sqlx::PgConnection::connect(&db.url).await.unwrap();
        sqlx::raw_sql(
            "CREATE TABLE short_links (link_number BIGINT PRIMARY KEY, original_url TEXT NOT NULL UNIQUE);
             INSERT INTO short_links VALUES (1, 'https://example.com/old-1'), (54, 'https://example.com/old-54');",
        )
        .execute(&mut conn)
        .await
        .unwrap();
    }

    let shortener = shortener(&db).await;
    let user = common::user(&shortener).await;

    assert_eq!(shortener.follow("1").await.unwrap(), None);
    assert_eq!(shortener.follow("54").await.unwrap(), None);
    assert_eq!(
        shortener
            .shorten(&user, "https://example.com/new")
            .await
            .unwrap(),
        "http://localhost:8080/1",
        "번호는 1부터 다시 시작한다"
    );

    let reopened = common::shortener(&db).await;
    assert_eq!(
        reopened.follow("1").await.unwrap().as_deref(),
        Some("https://example.com/new"),
        "두 번째로 열 때는 지우지 않는다"
    );
}
