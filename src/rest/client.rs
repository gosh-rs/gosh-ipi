// [[file:../../ipi.note::8bb618e6][8bb618e6]]
use super::*;
use reqwest;
// 8bb618e6 ends here

// [[file:../../ipi.note::d2c8de54][d2c8de54]]
/// Client for remote execution
pub struct Client {
    client: reqwest::Client,
    service_uri: String,
}

impl Client {
    /// Connect to remote service using address like "localhost:12345"
    pub fn connect(address: impl std::fmt::Display) -> Self {
        // by the default there is no timeout
        let client = reqwest::Client::builder().build().expect("reqwest client");
        let service_uri = format!("http://{}", address);
        Self { client, service_uri }
    }
}
// d2c8de54 ends here

// [[file:../../ipi.note::743b32f9][743b32f9]]
impl Client {
    pub(super) async fn post(&self, end_point: &str, data: impl serde::Serialize) -> Result<String> {
        let uri = format!("{}/{end_point}", self.service_uri);
        let resp = self.client.post(&uri).json(&data).send().await?.text().await?;

        Ok(resp)
    }
}
// 743b32f9 ends here
