use anyhow::{anyhow, Result};
use reqwest::Client;
use serde_json::Value;

pub struct ACI<E: Executor> {
    client: Client,
    executor: E,
    server: String,
    username: String,
    password: String,
    token: String,
}

impl Executor for Client {
    async fn execute_request(&self, request: reqwest::Request) -> Result<reqwest::Response> {
        match self.execute(request).await {
            Ok(response) => Ok(response),
            Err(error) => Err(error.into()),
        }
    }
}

#[trait_variant::make(Executor: Send)]
pub trait LocalExecutor {
    async fn execute_request(&self, request: reqwest::Request) -> Result<reqwest::Response>;
}

impl<E: Executor> ACI<E> {
    pub async fn new_with_executor(
        executor: E,
        server: String,
        username: String,
        password: String,
    ) -> Self {
        let client = Client::builder()
            .cookie_store(true)
            .danger_accept_invalid_certs(true);
        let client = client.build().unwrap();
        let mut aci = ACI {
            client,
            executor,
            server,
            username,
            password,
            token: String::new(),
        };
        aci.login().await;

        aci
    }

    async fn login(&mut self) -> bool {
        let url = format!("https://{}/api/aaaLogin.json", self.server);
        let request = self.client.post(url);
        let json = &serde_json::json!({
          "aaaUser" : {
            "attributes" : {
              "name" : self.username,
              "pwd" : self.password
            }
          }
        });
        let request = request.json(json).build().unwrap();
        let response = self.executor.execute_request(request).await;
        self.token = response
            .unwrap()
            .json::<Value>()
            .await
            .unwrap()
            .get("imdata")
            .unwrap()[0]["aaaLogin"]["attributes"]["token"]
            .to_string();

        if self.token == String::from("null") {
            return false;
        }

        true
    }

    // async fn refresh_token(&self)

    pub async fn get_json(&self, uri: String) -> Result<Value> {
        let url = format!("https://{}/api/{}", self.server, uri);
        let request = self.client.get(url).build()?;
        let response = self.executor.execute_request(request).await?;
        if let Some(data) = response.json::<Value>().await?.get("imdata") {
            return Ok(data.clone());
        }
        Err(anyhow!("Error!"))
    }

    pub fn get_token(&self) -> &String {
        &self.token
    }
}

impl ACI<Client> {
    pub async fn new(server: String, username: String, password: String) -> Self {
        let executor = Client::builder()
            .cookie_store(true)
            .danger_accept_invalid_certs(true);
        let executor = executor.build().unwrap();
        ACI::new_with_executor(executor, server, username, password).await
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use anyhow::anyhow;

    use crate::{Executor, ACI};
    pub struct MockClient;

    impl Executor for MockClient {
        async fn execute_request(
            &self,
            request: reqwest::Request,
        ) -> anyhow::Result<reqwest::Response> {
            match request.url().path() {
                "/api/aaaLogin.json" => login_request(request),
                _ => Err(anyhow!("not supported in MockClient!")),
            }
        }
    }

    fn login_request(request: reqwest::Request) -> anyhow::Result<reqwest::Response> {
        let data = fs::read_to_string("tests/json/aaaLogin.json")?;
        let response = http::response::Builder::new()
            .status(200)
            .body(data)
            .unwrap();
        let response = reqwest::Response::from(response);
        Ok(response)
    }

    #[tokio::test]
    async fn aci_login() {
        let executor = MockClient;
        let server = String::from("SERVER");
        let username = String::from("USERNAME");
        let password = String::from("PASSWORD");
        let aci = ACI::new_with_executor(executor, server, username, password).await;

        println!("{}", aci.token)
    }
}
