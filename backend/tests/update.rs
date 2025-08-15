use std::fs;

use wiremock::matchers::path;
use wiremock::{Mock, ResponseTemplate};

use etl::{fill_db, Post};

use tests::spawn_api;

// #[tokio::test]
// async fn test_update_fills_db() {
//     let app = spawn_api().await;

//     let pixiv_json = fs::read_to_string("assets/json/pixiv.json").expect("Unable to read the file");
//     let mihoyo_json =
//         fs::read_to_string("assets/json/mihoyo.json").expect("Unable to read the file");
//     let twitter_home_json =
//         fs::read_to_string("assets/json/twitter-home.json").expect("Unable to read the file");
//     let twitter_honkai_json =
//         fs::read_to_string("assets/json/twitter-honkai.json").expect("Unable to read the file");
//     let _pixiv_mock = Mock::given(path("/pixiv"))
//         .respond_with(ResponseTemplate::new(200).set_body_raw(pixiv_json, "application/json"))
//         .expect(1)
//         .named("pixiv")
//         .mount(&app.mock_server)
//         .await;
//     let _mihoyo_mock = Mock::given(path("/mihoyo"))
//         .respond_with(ResponseTemplate::new(200).set_body_raw(mihoyo_json, "application/json"))
//         .expect(1)
//         .named("mihoyo")
//         .mount(&app.mock_server)
//         .await;
//     let _twitter_home_mock = Mock::given(path("/twitter_home"))
//         .respond_with(
//             ResponseTemplate::new(200).set_body_raw(twitter_home_json, "application/json"),
//         )
//         .expect(1)
//         .named("twitter_home")
//         .mount(&app.mock_server)
//         .await;
//     let _twitter_honkai_mock = Mock::given(path("/twitter_honkai"))
//         .respond_with(
//             ResponseTemplate::new(200).set_body_raw(twitter_honkai_json, "application/json"),
//         )
//         .expect(1)
//         .named("twitter_honkai")
//         .mount(&app.mock_server)
//         .await;

//     fill_db(&app.scraper_state).await.unwrap();

//     let posts = app
//         .api_state
//         .api_client
//         .get(format!("{}/api/honkai?page=1", app.addr))
//         .send()
//         .await
//         .unwrap()
//         .json::<Vec<Post>>()
//         .await
//         .unwrap();

//     assert!(!posts.is_empty());
// }
