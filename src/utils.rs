use std::collections::HashMap;

use crate::errors::client::InvalidUrl;
use cdumay_error::Result;
use reqwest::header::HeaderMap;
use reqwest::Url;

pub fn merge_headers(h1: &HeaderMap, h2: Option<HeaderMap>) -> HeaderMap {
    let mut headers = h1.clone();
    if let Some(additional_headers) = h2 {
        headers.extend(additional_headers);
    }
    headers
}

pub fn build_url(root: &Url, path: String, params: Option<HashMap<String, String>>) -> Result<Url> {
    let mut url = root.clone();
    let spath: Vec<&str> = path.split("/").filter(|part| part.len() != 0).collect();
    url.path_segments_mut()
        .map_err(|_| InvalidUrl::new().set_message("Cannot build url".to_string()))?
        .extend(&spath);

    if let Some(data) = params {
        for (key, value) in data.iter() {
            url.query_pairs_mut().append_pair(key, value);
        }
    }
    Ok(url)
}
