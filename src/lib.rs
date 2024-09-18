use std::str::FromStr;

use anyhow::{anyhow, Result};
use reqwest::Client;
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AciError {
    #[error("Login error")]
    LoginError,
    #[error("Get error")]
    GetError,
    #[error("Post error")]
    PostError,
}

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
    ) -> std::result::Result<Self, AciError> {
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
        let result = aci.login().await;

        match result {
            Ok(()) => Ok(aci),
            Err(e) => Err(e),
        }
    }

    async fn login(&mut self) -> std::result::Result<(), AciError> {
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
            _ => return Err(AciError::LoginError),
        }

        Ok(())
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

    pub async fn post_json(&self, uri: String, data: String) -> Result<()> {
        let url = format!("https://{}/api/{}", self.server, uri);
        let data: Value = serde_json::from_str(data.as_str())?;

        let request = self.client.post(url).json(&data).build()?;
        let response = self.executor.execute_request(request).await?;
        if response.json::<Value>().await?.get("imdata").is_some() {
            return Ok(());
        }
        Err(anyhow!("Error!"))
    }

    // This function creates a snapshot of the ACI fabric
    pub async fn snapshot(&self, description: Option<String>, dn: Option<String>) -> Result<()> {
        let json = get_snapshot_data(description, dn);

        self.post_json(String::from("mo.json"), json.to_string())
            .await
    }

    pub fn get_token(&self) -> &String {
        &self.token
    }
}

impl ACI<Client> {
    pub async fn new(
        server: String,
        username: String,
        password: String,
    ) -> std::result::Result<Self, AciError> {
        let executor = Client::builder()
            .cookie_store(true)
            .danger_accept_invalid_certs(true);
        let executor = executor.build().unwrap();
        ACI::new_with_executor(executor, server, username, password).await
    }
}

fn get_snapshot_data(description: Option<String>, dn: Option<String>) -> Value {
    let description = match description {
        Some(description) => description,
        None => String::from("Snapshot"),
    };

    let dn = dn.unwrap_or_default();

    serde_json::json!({
        "configExportP": {
            "attributes": {
                "adminSt": "triggered",
                "descr": format!("by rustyaci - {description}"),
                "dn": "uni/fabric/configexp-rustyaci",
                "format": "json",
                "includeSecureFields": "yes",
                "maxSnapshotCount": "global-limit",
                "name": "rustyaci",
                "nameAlias": "",
                "snapshot": "yes",
                "targetDn": format!("{dn}")
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use core::panic;
    use std::{fs, str::FromStr};

    use anyhow::anyhow;
    use serde_json::Value;

    use crate::{get_snapshot_data, Executor, ACI};
    pub struct MockClient;

    impl Executor for MockClient {
        async fn execute_request(
            &self,
            request: reqwest::Request,
        ) -> anyhow::Result<reqwest::Response> {
            match request.url().path() {
                "/api/aaaLogin.json" => login_request(),
                "/api/class/fvTenant.json" => bd_request(),
                "/api/mo.json" => mo_request(request),
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

    fn mo_request(request: reqwest::Request) -> anyhow::Result<reqwest::Response> {
        let response_data = fs::read_to_string("tests/json/mo.json")?;
        let data = request.body().unwrap().as_bytes().unwrap();
        let json_data: Value = serde_json::from_slice(data).unwrap();
        println!("{:?}", json_data);
        if !json_data.is_object() {
            return Err(anyhow::anyhow!("JSON is not an Object!"));
        }

        let request_data = json_data.as_object().unwrap();
        if request_data.keys().len() > 1 {
            return Err(anyhow::anyhow!("More than one main key!"));
        }

        let class = request_data.keys().nth(0).unwrap().as_str();
        match class {
            "fvAEPg" => {
                let expected_data = fs::read_to_string("tests/json/post/epg-TEST.json")?;
                let expected_json_data: Value = serde_json::from_str(&expected_data).unwrap();
                assert_eq!(json_data, expected_json_data);

                let response = http::response::Builder::new()
                    .status(200)
                    .body(response_data)
                    .unwrap();
                let response = reqwest::Response::from(response);

                return Ok(response);
            }
            "configExportP" => {
                let expected_data = fs::read_to_string("tests/json/post/configExportP.json")?;
                let expected_json_data: Value = serde_json::from_str(&expected_data).unwrap();
                assert_eq!(json_data, expected_json_data);

                let response = http::response::Builder::new()
                    .status(200)
                    .body(response_data)
                    .unwrap();
                let response = reqwest::Response::from(response);

                return Ok(response);
            }
            _ => {
                return Err(anyhow::anyhow!("Class not supported by mock client"));
            }
        }
    }

    async fn login() -> ACI<MockClient> {
        let executor = MockClient;
        let server = String::from("SERVER");
        let username = String::from("USERNAME");
        let password = String::from("PASSWORD");
        let aci = ACI::new_with_executor(executor, server, username, password).await;

        match aci {
            Ok(aci) => aci,
            Err(e) => panic!("{}", e),
        }
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

    #[tokio::test]
    async fn aci_post_json() {
        let aci = login().await;
        let data = fs::read_to_string("tests/json/post/epg-TEST.json").unwrap();

        match aci.post_json(String::from("mo.json"), data).await {
            Ok(()) => return,
            Err(e) => panic!("{}", e),
        }
    }

    #[tokio::test]
    async fn aci_post_json_inline_data() {
        let aci = login().await;
        let data = r#"
        {
  "fvAEPg": {
    "attributes": {
      "dn": "uni/tn-TEST/ap-TEST/epg-TEST",
      "name": "TEST"
    }
  }
}
        "#;

        match aci
            .post_json(String::from("mo.json"), String::from_str(data).unwrap())
            .await
        {
            Ok(()) => return,
            Err(e) => panic!("{}", e),
        }
    }

    #[tokio::test]
    async fn test_aci_snapshot() {
        let aci = login().await;
        match aci.snapshot(None, None).await {
            Ok(()) => return,
            Err(e) => panic!("{}", e),
        }
    }

    #[tokio::test]
    async fn test_snapshot_data_empty() {
        let data = get_snapshot_data(None, None);
        let expected_data = fs::read_to_string("tests/json/post/configExportP.json").unwrap();
        let expected_json_data: Value = serde_json::from_str(&expected_data).unwrap();

        assert_eq!(data, expected_json_data);
    }

    #[tokio::test]
    async fn test_snapshot_data_with_description() {
        let description = "custom description".to_string();
        let data = get_snapshot_data(Some(description.clone()), None);
        let expected_data = fs::read_to_string("tests/json/post/configExportP.json").unwrap();
        let mut expected_json_data: Value = serde_json::from_str(&expected_data).unwrap();
        let expected_description = String::from("by rustyaci - ") + description.as_str();
        expected_json_data["configExportP"]["attributes"]["descr"] =
            serde_json::Value::String(expected_description);

        assert_eq!(data, expected_json_data);
    }
    #[tokio::test]
    async fn test_snapshot_data_with_dn() {
        let dn = "fvTenant".to_string();
        let data = get_snapshot_data(None, Some(dn.clone()));
        let expected_data = fs::read_to_string("tests/json/post/configExportP.json").unwrap();
        let mut expected_json_data: Value = serde_json::from_str(&expected_data).unwrap();
        expected_json_data["configExportP"]["attributes"]["targetDn"] =
            serde_json::Value::String(dn);

        assert_eq!(data, expected_json_data);
    }
}
