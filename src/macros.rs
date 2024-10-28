#[derive(Debug, thiserror::Error)]
pub enum MacroError {
    #[error("Missing Field: {0}")]
    MissingField(String),
}

pub mod private {
    use super::MacroError;

    pub trait GetValue {
        fn get_value(value: serde_json::Value, field_name: &str) -> Result<Self, MacroError>
        where
            Self: Sized;
    }

    impl GetValue for String {
        fn get_value(value: serde_json::Value, field_name: &str) -> Result<Self, MacroError> {
            let value = value[field_name].as_str();
            match value {
                Some(value) => Ok(value.to_string()),
                None => Err(MacroError::MissingField(field_name.to_string())),
            }
        }
    }

    impl GetValue for u64 {
        fn get_value(value: serde_json::Value, field_name: &str) -> Result<Self, MacroError> {
            let value = value[field_name].as_u64();
            match value {
                Some(value) => Ok(value),
                None => Err(MacroError::MissingField(field_name.to_string())),
            }
        }
    }
}

#[macro_export]
macro_rules! aci_struct {
    ($struct_name:ident, $root:expr, { $($field_name:ident : $field_type:ty),* $(,)? }) => {
        #[derive(Debug)]
        pub struct $struct_name {
            $(pub $field_name: $field_type),*
        }

        impl<'de> serde::Deserialize<'de> for $struct_name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let value: serde_json::Value = serde_json::Value::deserialize(deserializer)?;

                use $crate::macros::private::GetValue;
                Ok($struct_name {
                    $(
                        $field_name: <$field_type>::get_value(value[$root]["attributes"].clone(), stringify!($field_name)).map_err(serde::de::Error::custom)?
                    ),*
                })
            }
        }
    };
}

crate::aci_struct!(
    Tenant,
    "fvTenant",
    {
        name: String,
        bytes: u64
    }
);

#[cfg(test)]
mod tests {
    use super::Tenant;

    #[test]
    fn macro_tenant_test() {
        let json_data = r#"
    {
        "fvTenant": {
            "attributes": {
                "name": "TenantName",
                "bytes": 500
            }
        }
    }"#;
        let tenant: Tenant = serde_json::from_str(json_data).expect("Failed to deserialize");
        assert_eq!(tenant.name, "TenantName");
        assert_eq!(tenant.bytes, 500);
    }
}
