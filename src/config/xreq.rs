use super::{LoadConfig, ValidateConfig};
use crate::RequestProfile;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReqConfig {
    #[serde(flatten)]
    pub profiles: HashMap<String, RequestProfile>,
}

impl LoadConfig for ReqConfig {}

impl ValidateConfig for ReqConfig {
    fn validate(&self) -> anyhow::Result<()> {
        for (name, profile) in &self.profiles {
            profile
                .validate()
                .with_context(|| format!("profile error [{}]", name.to_string()))?;
        }
        Ok(())
    }
}

impl ReqConfig {
    pub fn get_profile(&self, name: &str) -> Option<&RequestProfile> {
        self.profiles.get(name)
    }

    pub fn new(profiles: HashMap<String, RequestProfile>) -> Self {
        Self { profiles }
    }
}
