use serde::{Deserialize, Serialize};

use crate::Configuration;

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Item {
    pub data: rollbar_rust::types::Data,
}

/// Updates an object's fields with those from another object, or with
/// default values, if they are not already set to something.
/// 
/// This macro is used to simplify the code used to prepare a Rollbar
/// event for submission by copying fields from the configuration, or
/// setting appropriate defaults.
macro_rules! set_default {
    ($data:ident [ $field:ident ] from $config:ident) => {
        if $data.$field.is_none() && $config.$field.is_some() {
            $data.$field = $config.$field.clone();
        }
    };

    ($data:ident [ $field:ident ] from $config:ident [ $sfield:ident ]) => {
        if $data.$field.is_none() && $config.$field.is_some() {
            $data.$field = $config.$sfield.clone();
        }
    };

    ($data:ident [ $field:ident ] = $default:expr) => {
        if $data.$field.is_none() {
            $data.$field = Some($default);
        }
    };
}

impl From<(rollbar_rust::types::Data, &Configuration)> for Item {
    fn from((data, config): (rollbar_rust::types::Data, &Configuration)) -> Self {
        let mut data = data;

        set_default!(data[level] = crate::Level::Info);
        set_default!(data[language] = "rust".to_string());

        set_default!(data[environment] from config);
        set_default!(data[code_version] from config);
        set_default!(data[platform] from config);
        set_default!(data[framework] from config);
        set_default!(data[context] from config);
        set_default!(data[custom] from config);

        set_default!(data[platform] = std::env::consts::OS.to_string());
        set_default!(data[uuid] = crate::helpers::new_uuid());

        Item { data }
    }
}