use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone)]
pub struct Tenant {
    pub dn: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct APIResponse<T> {
    #[serde(rename = "totalCount")]
    total_count: String,
    imdata: Vec<T>,
}

#[derive(Deserialize)]
struct Wrapper<T> {
    attributes: T,
}

#[derive(Deserialize)]
struct TenantWrapper<T> {
    #[serde(rename = "fvTenant")]
    fv_tenant: T,
}

#[cfg(test)]
mod tests {
    use super::*;

    const DATA: &str = r#"{
    "totalCount": "7",
    "imdata": [
        {
            "fvTenant": {
                "attributes": {
                    "annotation": "",
                    "childAction": "",
                    "descr": "",
                    "dn": "uni/tn-t-COOP-SRV",
                    "extMngdBy": "",
                    "lcOwn": "local",
                    "modTs": "2023-04-12T07:56:13.076+01:00",
                    "monPolDn": "uni/tn-common/monepg-default",
                    "name": "t-COOP-SRV",
                    "nameAlias": "",
                    "ownerKey": "",
                    "ownerTag": "",
                    "status": "",
                    "uid": "15374",
                    "userdom": "all"
                }
            }
        },
        {
            "fvTenant": {
                "attributes": {
                    "annotation": "",
                    "childAction": "",
                    "descr": "",
                    "dn": "uni/tn-t-COOP-TEST",
                    "extMngdBy": "",
                    "lcOwn": "local",
                    "modTs": "2023-04-12T07:56:13.076+01:00",
                    "monPolDn": "uni/tn-common/monepg-default",
                    "name": "t-COOP-TEST",
                    "nameAlias": "",
                    "ownerKey": "",
                    "ownerTag": "",
                    "status": "",
                    "uid": "15374",
                    "userdom": "all"
                }
            }
        }
        ]}"#;

    #[test]
    fn parse() {
        let data =
            serde_json::from_str::<APIResponse<TenantWrapper<Wrapper<Tenant>>>>(DATA).unwrap();
        for tenant in data.imdata {
            assert_ne!(tenant.fv_tenant.attributes.dn, "")
        }
    }
}

pub async fn get_tenants(&self) -> Vec<Tenant> {
    let url = format!("https://{}/api/class/fvTenant.json", self.server);
    let request = self.client.get(url).build().unwrap();
    let response = self.client.execute(request).await;
    response
        .unwrap()
        .json::<APIResponse<TenantWrapper<Wrapper<Tenant>>>>()
        .await
        .unwrap()
        .imdata
        .iter()
        .map(|t| t.fv_tenant.attributes.clone())
        .collect::<Vec<_>>()
}
