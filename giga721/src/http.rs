use common::rc_bytes::RcBytes;
use common::*;

use crate::storage::STORAGE;

use ic_cdk_macros::query;
use serde_bytes::ByteBuf;

#[query]
fn http_request(req: HttpRequest) -> HttpResponse {
    //Splits request url in to parts before ? and after ?
    let parts: Vec<&str> = req.url.split('?').collect();
    let probably_an_asset = parts[0];


    let mut headers = Vec::<(String, String)>::default();
    //We can enable cache, NFT asset will never change
    headers.push(("Cache-Control".to_string(),"public, max-age=604800, immutable".to_string()));
    headers.push(("Location".to_string(),format!("https://cache.icpunks.com/metachess/{}", probably_an_asset)));

    let bytes = ByteBuf::new();

    HttpResponse {
        status_code: 302,
        headers: headers,
        body: RcBytes::from(bytes)
    }

    // STORAGE.with(|state| {
    //     // let state = get_state();
    //     // let state = StableStorage::get();
    //     // let mut state = state.borrow_mut();

    //     let mut state = state.borrow_mut();

    //     // let asset = state.assets.get(probably_an_asset);
    //     let asset = state.get_asset(probably_an_asset).ok();

    //     match asset {
    //         Some((headers, value)) => {
    //             let mut headers = headers.clone();
    //             //We can enable cache, NFT asset will never change
    //             headers.push((
    //                 "Cache-Control".to_string(),
    //                 "public, max-age=604800, immutable".to_string(),
    //             ));

    //             HttpResponse {
    //                 status_code: 200,
    //                 headers: headers,
    //                 body: RcBytes::from(value)
    //             }
    //         }
    //         None => HttpResponse {
    //             status_code: 404,
    //             headers: vec![],
    //             // headers: vec![certificate_header],
    //             body: RcBytes::from(ByteBuf::from(format!(
    //                 "Asset {} not found.",
    //                 probably_an_asset))), 
    //             // Cow::Owned(ByteBuf::from(format!(
    //                 // "Asset {} not found.",
    //                 // probably_an_asset
    //             // ))),
    //         },
    //     }
    // })
}
