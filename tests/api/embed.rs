use std::fs;

use reqwest::redirect::Policy;
use reqwest::Client;
use tokio;
use wiremock::{matchers::path, Mock, ResponseTemplate};

use crate::helpers::{assert_is_redirect_to, spawn_app};

#[tokio::test]
async fn test_embed_user_agent_check_from_discord() {
    // if discord's user-agent send html
    let app = spawn_app().await;
    let response = app
        .state
        .api_client
        .get(format!("{}/en/artworks/123", app.addr))
        .header(
            "User-Agent",
            "Mozilla/5.0 (compatible; Discordbot/2.0; +https://discordapp.com)",
        )
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert_eq!(
        response,
        r#"<meta name="twitter:card" content="summary_large_image"><meta name="twitter:image" content="https://pixiv.sbs/en/artworks/123.jpg">"#
    );

    let response = app
        .state
        .api_client
        .get(format!("{}/en/artworks/123", app.addr))
        .header(
            "User-Agent",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 11.6; rv:92.0) Gecko/20100101 Firefox/92.0",
        )
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert_eq!(
        response,
        r#"<meta name="twitter:card" content="summary_large_image"><meta name="twitter:image" content="https://pixiv.sbs/en/artworks/123.jpg">"#
    );
}

#[tokio::test]
async fn test_embed_user_agent_check_from_browser() {
    // if not a discord's user-agent, then redirect to pixiv
    let app = spawn_app().await;
    let client = Client::builder().redirect(Policy::none()).build().unwrap();
    let response = client
        .get(format!("{}/en/artworks/106595952", app.addr))
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/111.0",
        )
        .send()
        .await
        .unwrap();
    assert_is_redirect_to(&response, "https://www.pixiv.net/en/artworks/106595952");
}

#[tokio::test]
async fn test_embed_jpg_single_image() {
    let app = spawn_app().await;
    let pixiv_details_json =
        fs::read_to_string("tests/assets/json/embed-single.json").expect("Unable to read the file");
    let pixiv_image =
        fs::read("tests/assets/test-single-image-embed.jpg").expect("Unable to read the file");
    let _pixiv_details_mock = Mock::given(path("/pixiv_details106859625"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(pixiv_details_json, "application/json"),
        )
        .expect(1)
        .named("pixiv_details")
        .mount(&app.mock_server)
        .await;
    let _pixiv_image_mock = Mock::given(path(
        "/pixiv_image/img/2023/04/04/17/31/23/106859625_p0_master1200.jpg",
    ))
    .respond_with(ResponseTemplate::new(200).set_body_raw(pixiv_image.clone(), "image/jpeg"))
    .expect(1)
    .named("pixiv_image")
    .mount(&app.mock_server)
    .await;

    let response = app
        .state
        .api_client
        .get(format!("{}/en/artworks/106859625.jpg", app.addr))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 200);
    let received_image = response.bytes().await.unwrap().to_vec();
    assert_eq!(received_image, pixiv_image);
}

#[tokio::test]
async fn test_embed_jpg_p_separator() {
    let app = spawn_app().await;
    let pixiv_details_json = fs::read_to_string("tests/assets/json/embed-multiple.json")
        .expect("Unable to read the file");
    let pixiv_image =
        fs::read("tests/assets/test-multiple-image-embed.jpg").expect("Unable to read the file");
    let _pixiv_details_mock = Mock::given(path("/pixiv_details106856624"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(pixiv_details_json, "application/json"),
        )
        .expect(1)
        .named("pixiv_details")
        .mount(&app.mock_server)
        .await;
    let _pixiv_image_mock = Mock::given(path(
        "/pixiv_image/img/2023/04/04/14/54/34/106856624_p1_master1200.jpg",
    ))
    .respond_with(ResponseTemplate::new(200).set_body_raw(pixiv_image.clone(), "image/jpeg"))
    .expect(1)
    .named("pixiv_image")
    .mount(&app.mock_server)
    .await;

    let response = app
        .state
        .api_client
        .get(format!("{}/en/artworks/106856624_p1.jpg", app.addr))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 200);
    let received_image = response.bytes().await.unwrap().to_vec();
    assert_eq!(received_image, pixiv_image);
}

#[tokio::test]
async fn test_embed_jpg_slash_separator() {
    let app = spawn_app().await;
    let pixiv_details_json = fs::read_to_string("tests/assets/json/embed-multiple.json")
        .expect("Unable to read the file");
    let pixiv_image =
        fs::read("tests/assets/test-multiple-image-embed.jpg").expect("Unable to read the file");
    let _pixiv_details_mock = Mock::given(path("/pixiv_details106856624"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(pixiv_details_json, "application/json"),
        )
        .expect(1)
        .named("pixiv_details")
        .mount(&app.mock_server)
        .await;
    let _pixiv_image_mock = Mock::given(path(
        "/pixiv_image/img/2023/04/04/14/54/34/106856624_p1_master1200.jpg",
    ))
    .respond_with(ResponseTemplate::new(200).set_body_raw(pixiv_image.clone(), "image/jpeg"))
    .expect(1)
    .named("pixiv_image")
    .mount(&app.mock_server)
    .await;

    let response = app
        .state
        .api_client
        .get(format!("{}/en/artworks/106856624/2.jpg", app.addr))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 200);
    let received_image = response.bytes().await.unwrap().to_vec();
    assert_eq!(received_image, pixiv_image);
}
