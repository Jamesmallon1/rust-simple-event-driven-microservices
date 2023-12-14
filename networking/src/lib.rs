use log::{debug, error};
use reqwest::header::HeaderMap;
use reqwest::Client;

use std::collections::HashMap;

/// Executes a Standard RESTful GET request over the network. This method can only be accessed within the networking
/// crate. A developer should create simple network level services that prepare data for these base functions.
///
/// # Arguments
///
/// * `url` - The URL that the request is being made to.
/// * `headers` - A HeaderMap is similar to a standard map.
/// * `params` - A hashmap of URL parameters that will be added to the URL in format: ?param1=foo&param2=bar&param3=...
pub async fn execute_get_request<T: serde::de::DeserializeOwned>(
    url: &str,
    headers: Option<HeaderMap>,
    params: Option<HashMap<String, String>>,
) -> Result<T, NetworkError> {
    execute_request(HttpMethod::Get { params }, url, headers).await
}

/// Executes a Standard RESTful POST request over the network. This method can only be accessed within the networking
/// crate. A developer should create simple network level services that prepare data for these base functions.
///
/// # Arguments
///
/// * `url` - The URL that the request is being made to.
/// * `headers` - A HeaderMap is similar to a standard map.
/// * `body` - The main body of the request that will be transmitted over the network.
pub async fn execute_post_request<T: serde::de::DeserializeOwned>(
    url: &str,
    headers: Option<HeaderMap>,
    body: Option<String>,
) -> Result<T, NetworkError> {
    execute_request(HttpMethod::Post { body }, url, headers).await
}

async fn execute_request<T: serde::de::DeserializeOwned>(
    method: HttpMethod,
    url: &str,
    headers: Option<HeaderMap>,
) -> Result<T, NetworkError> {
    debug!("Making a {:?} request to: {}", method, url);
    let client = Client::builder().build().unwrap();
    let mut request_builder = match &method {
        HttpMethod::Get { params } => {
            let mut full_url = url.to_string();
            if let Some(parameters) = params {
                let query_string = serde_urlencoded::to_string(parameters).unwrap();
                full_url.push_str("?");
                full_url.push_str(&query_string);
            }
            client.get(&full_url)
        }
        HttpMethod::Post { body } => {
            let builder = client.post(url);
            if let Some(b) = body {
                builder.body(b.to_string())
            } else {
                builder
            }
        }
    };

    if let Some(hdrs) = headers {
        request_builder = request_builder.headers(hdrs);
    }

    let response = match request_builder.send().await {
        Ok(rsp) => {
            debug!("Successfully made a {:?} request to: {}", method, url);
            rsp
        }
        Err(err) => {
            debug!("Request Failed to: {}, due to Error: {:?}", url, err);
            return Err(NetworkError {
                status_code: None,
                error: NetworkErrorType::RequestError(err),
            });
        }
    };

    if !response.status().is_success() {
        return Err(NetworkError {
            status_code: Some(response.status().as_u16()),
            error: NetworkErrorType::Standard,
        });
    }

    response.json::<T>().await.map_err(|err| {
        let msg = format!("JSON Deserialization failed on {}, due to Error: {:?}", url, err);
        error!("{}", msg);
        NetworkError {
            status_code: Some(23),
            error: NetworkErrorType::JsonError(err),
        }
    })
}

#[derive(Debug)]
enum HttpMethod {
    Get { params: Option<HashMap<String, String>> },
    Post { body: Option<String> },
}

/// A generic network error struct. This should be used as a representation of a restful request
/// that carries the resultant object on success.
///
/// An error will be present in the case of a failure whilst performing a network request. This will
/// be possible to identify using the teapot status code.
#[derive(Debug)]
pub struct NetworkError {
    pub status_code: Option<u16>,
    pub error: NetworkErrorType,
}

#[derive(Debug)]
pub enum NetworkErrorType {
    Standard,
    RequestError(reqwest::Error),
    JsonError(reqwest::Error),
}
