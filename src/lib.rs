use anyhow::{anyhow, Ok, Result};
use reqwest::Client;
use serde_json::Value;

pub struct ACI {
    client: Client,
    server: String,
    username: String,
    password: String,
    token: String,
}

impl ACI {
    pub async fn new(server: String, username: String, password: String) -> Self {
        let client = Client::builder()
            .cookie_store(true)
            .danger_accept_invalid_certs(true);
        let client = client.build().unwrap();
        let mut aci = ACI {
            client,
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
        let response = self.client.execute(request).await;
        self.token = response
            .unwrap()
            .json::<Value>()
            .await
            .unwrap()
            .get("imdata")
            .unwrap()[0]["aaaLogin"]["attributes"]["token"]
            .to_string();

        true
    }

    // async fn refresh_token(&self)

    pub async fn get_json(&self, uri: String) -> Result<Value> {
        let url = format!("https://{}/api/{}", self.server, uri);
        let request = self.client.get(url).build()?;
        let response = self.client.execute(request).await?;
        if let Some(data) = response.json::<Value>().await?.get("imdata") {
            return Ok(data.clone());
        }
        Err(anyhow!("Error!"))
    }

    pub fn get_token(&self) -> &String {
        &self.token
    }
}
