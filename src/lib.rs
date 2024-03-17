use std::str::FromStr;

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

        // Parse the token out of the response
        let token = response.unwrap().json::<Value>().await.unwrap();
        let token = token.get("imdata").unwrap()[0]["aaaLogin"]["attributes"]["token"].as_str();

        match token {
            Some(token) => {
                self.token = String::from_str(token).unwrap();
            }
            _ => return false,
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
                "/api/aaaLogin.json" => login_request(),
                "/api/class/fvTenant.json" => bd_request(),
                _ => Err(anyhow!("not supported in MockClient!")),
            }
        }
    }

    fn login_request() -> anyhow::Result<reqwest::Response> {
        let data = fs::read_to_string("tests/json/aaaLogin.json")?;
        let response = http::response::Builder::new()
            .status(200)
            .body(data)
            .unwrap();
        let response = reqwest::Response::from(response);

        Ok(response)
    }

    fn bd_request() -> anyhow::Result<reqwest::Response> {
        let data = fs::read_to_string("tests/json/fvTenant.json")?;
        let response = http::response::Builder::new()
            .status(200)
            .body(data)
            .unwrap();
        let response = reqwest::Response::from(response);

        Ok(response)
    }

    async fn login() -> ACI<MockClient> {
        let executor = MockClient;
        let server = String::from("SERVER");
        let username = String::from("USERNAME");
        let password = String::from("PASSWORD");
        let aci = ACI::new_with_executor(executor, server, username, password).await;

        aci
    }

    #[tokio::test]
    async fn aci_login() {
        let aci = login().await;

        assert_eq!("TOKEN", aci.token);
        assert_eq!("TOKEN", aci.get_token());
    }

    #[tokio::test]
    #[should_panic]
    async fn get_invalid_json() {
        let aci = login().await;

        aci.get_json(String::from("this_is_nonsense"))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn aci_get_json() {
        let aci = login().await;

        match aci.get_json(String::from("class/fvTenant.json")).await {
            Ok(bds) => {
                let bd_array = bds
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|bd| bd["fvTenant"]["attributes"]["name"].as_str().unwrap())
                    .collect::<Vec<_>>();
                assert_eq!(bd_array, vec!["infra", "common"]);
                assert_ne!(bd_array, vec!["infra", "common", "test"])
            }
            Err(e) => panic!("{}", e),
        }
    }
}
