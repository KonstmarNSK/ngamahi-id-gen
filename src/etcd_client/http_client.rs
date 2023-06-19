use std::sync::Arc;
use awc::error::JsonPayloadError;
use serde::de::DeserializeOwned;
use crate::etcd_client::operations::{DeserializeErr, EtcdInteropErr};

#[cfg(not(test))]
pub type HttpClient = awc::Client;

#[cfg(not(test))]
pub async fn make_request<TResp>(body: String, url: String, client: &HttpClient) -> Result<TResp, EtcdInteropErr>
    where
        TResp: DeserializeOwned
{
    let req = client.post(url).insert_header(("User-Agent", "id-gen/1.0"));

    let mut res = req.send_body(body).await?;
    Ok(res.json::<TResp>().limit(2000).await.map_err(|e| DeserializeErr::JsonPayload(e))?)
}


// mocking http client for tests
#[cfg(test)]
#[derive(Clone)]
pub struct MockClient{
    pub must_fail: Arc<Box<dyn Fn(String, String) -> Option<EtcdInteropErr>>>,
    pub get_response: Arc<Box<dyn Fn(String, String) -> String>>,
}

#[cfg(test)]
pub type HttpClient = MockClient;

#[cfg(test)]
pub async fn make_request<TResp>(body: String, url: String, client: &HttpClient) -> Result<TResp, EtcdInteropErr>
    where
        TResp: DeserializeOwned
{
    if let Some(err) = (client.must_fail)(body.clone(), url.clone()) {
        return Err(err);
    };

    let response = serde_json::from_str((client.get_response)(body, url).as_str())
        .map_err(|e| EtcdInteropErr::DeserializationErr(DeserializeErr::JsonPayload(JsonPayloadError::Deserialize(e))))?;

    Ok(response)
}

// factory function
#[cfg(not(test))]
pub fn new_http_client(client: awc::Client) -> HttpClient {
    client
}

#[cfg(test)]
pub fn new_http_client(client: MockClient) -> HttpClient {
    client
}