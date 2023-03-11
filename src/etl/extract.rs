use std::time::Duration;

use axum::extract::State;
use reqwest::{header, Client, Error, Response};
use sqlx::PgPool;

use crate::etl::{save_honkai_posts, Post, TwitterResponse};
use crate::Result;

async fn request_raw_data(client: Client) -> Result<Response, Error> {
    let url = "https://twitter.com/i/api/2/search/adaptive.json?include_profile_interstitial_type=1&include_blocking=1&include_blocked_by=1&include_followed_by=1&include_want_retweets=1&include_mute_edge=1&include_can_dm=1&include_can_media_tag=1&include_ext_has_nft_avatar=1&skip_status=1&cards_platform=Web-12&include_cards=1&include_ext_alt_text=true&include_quote_count=true&include_reply_count=1&tweet_mode=extended&include_entities=true&include_user_entities=true&include_ext_media_color=true&include_ext_media_availability=true&include_ext_sensitive_media_warning=true&include_ext_trusted_friends_metadata=true&send_error_codes=true&simple_quoted_tweet=true&q=%23%E7%AC%A6%E5%8D%8E%20OR%20%23%E5%B4%A9%E5%9D%8F3%20OR%20%23%E3%83%95%E3%82%AB%20OR%20%23%E5%B4%A9%E5%9D%8F3rd%20OR%20%23%E5%B4%A9%E5%A3%9E3rd%20OR%20%23%EB%B6%95%EA%B4%B43rd%20OR%20%23Honkaiimpact3rd%20OR%20%23%E5%B4%A9%E5%A3%8A3rd%20min_faves%3A2&tweet_search_mode=live&count=20&query_source=typed_query&pc=1&spelling_corrections=1&ext=mediaStats%2ChighlightedLabel%2ChasNftAvatar%2CreplyvotingDownvotePerspective%2CvoiceInfo%2Cenrichments%2CsuperFollowMetadata%2CunmentionInfo";
    client.get(url).send().await
}

fn create_request_client() -> Result<Client, Error> {
    let headers = vec![("cookie", "__utmv=235335808.|2=login ever=no=1^9=p_ab_id=3=1^10=p_ab_id_2=8=1^11=lang=en=1; PHPSESSID=37028420_4PABfDM1JsDaGbTR1FVeZpB9abuTEwkq; auth_token=269bcbc47c601743694a83bf4d78306dd6a6f168; ct0=bd586d27427110176ba725b0a89d363e1683254ff3b05992815e58d80f8ee96de172a713b043d21c060a146da4ebcc2c0583c5b0916b9f797bc4bf3c41950873cf31f67d63d962cda77107e537d6be66"), ("x-user-id", "37028420"), ("authorization", "Bearer AAAAAAAAAAAAAAAAAAAAAF7aAAAAAAAASCiRjWvh7R5wxaKkFp7MM%2BhYBqM%3DbQ0JPmjU9F6ZoMhDfI4uTNAaQuTDm2uO9x3WFVr2xBZ2nhjdP0"), ("x-csrf-token", "bd586d27427110176ba725b0a89d363e1683254ff3b05992815e58d80f8ee96de172a713b043d21c060a146da4ebcc2c0583c5b0916b9f797bc4bf3c41950873cf31f67d63d962cda77107e537d6be66"), ("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64)"), ("Referer", "https://www.pixiv.net/")];
    let mut headers_map = header::HeaderMap::new();
    for (key, val) in headers {
        let mut val = header::HeaderValue::from_static(val);
        val.set_sensitive(true);
        headers_map.insert(key, val);
    }
    Ok(Client::builder()
        .timeout(Duration::from_secs(10))
        .default_headers(headers_map)
        .build())?
}

async fn fetch_source() -> Vec<Post> {
    let client = create_request_client().unwrap();
    request_raw_data(client)
        .await
        .unwrap()
        .json::<TwitterResponse>()
        .await
        .unwrap()
        .into()
}

pub async fn fill_db(db_pool: State<PgPool>) -> Result<()> {
    let posts = fetch_source().await;
    save_honkai_posts(&db_pool.0, posts).await.unwrap();
    Ok(())
}
