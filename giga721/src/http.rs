use crate::storage::StableStorage;
use ic_cdk_macros::{query};

use common::*;

use serde_bytes::{ByteBuf};
use std::borrow::Cow;

// use crate::state::get_state;

#[query]
fn http_request(req: HttpRequest) -> HttpResponse {
//Splits request url in to parts before ? and after ?
    let parts: Vec<&str> = req.url.split('?').collect();
    let probably_an_asset = parts[0];

    // let state = get_state();

    // let asset = state.assets.get(probably_an_asset);
    let asset = StableStorage::get().borrow_mut().get_asset(probably_an_asset).ok();

    match asset {
        Some((headers, _value)) => {
            let mut headers = headers.clone();
            //We can enable cache, NFT asset will never change
            headers.push(("Cache-Control".to_string(),"public, max-age=604800, immutable".to_string()));
            headers.push(("Location".to_string(),format!("https://cache.icpunks.com/icats/{}", probably_an_asset)));

            let bytes = ByteBuf::new();

            HttpResponse {
                status_code: 302,
                headers: headers,
                body: Cow::Owned(bytes)
            }

            // HttpResponse {
            //     status_code: 200,
            //     headers: headers,
            //     body: Cow::Borrowed(Bytes::new(value)),
            // }
        }
        None => HttpResponse {
            status_code: 404,
            headers: vec![],
            // headers: vec![certificate_header],
            body: Cow::Owned(ByteBuf::from(format!(
                "Asset {} not found.",
                probably_an_asset
            ))),
        },
    }
}


