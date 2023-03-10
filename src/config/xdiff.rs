use super::{is_default, LoadConfig, RequestProfile, ValidateConfig};
use crate::{utils::diff_text, ExtraArgs};
use anyhow::{Context, Ok};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io::Write};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiffConfig {
    #[serde(flatten)]
    pub profiles: HashMap<String, DiffProfile>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiffProfile {
    pub req1: RequestProfile,
    pub req2: RequestProfile,
    #[serde(skip_serializing_if = "is_default", default)]
    pub response: ResponseProfile,
}

impl LoadConfig for DiffConfig {}

impl ValidateConfig for DiffConfig {
    fn validate(&self) -> anyhow::Result<()> {
        for (name, profile) in &self.profiles {
            profile
                .validate()
                .context(format!("profile error [{}]", name.to_string()))?;
        }
        Ok(())
    }
}

impl DiffConfig {
    pub fn get_profile(&self, name: &str) -> Option<&DiffProfile> {
        self.profiles.get(name)
    }

    pub fn new(profiles: HashMap<String, DiffProfile>) -> Self {
        Self { profiles }
    }
}

impl DiffProfile {
    pub async fn diff(&self, args: ExtraArgs) -> anyhow::Result<String> {
        let r1 = self.req1.send(&args).await?;
        let r2 = self.req2.send(&args).await?;

        let t1 = r1.filter_text(&self.response).await?;
        let t2 = r2.filter_text(&self.response).await?;

        let output = diff_text(t1.as_str(), t2.as_str())?;

        let stdout = std::io::stdout();
        let mut stdout = stdout.lock();
        write!(stdout, "{}", output)?;

        //println!("{}", t1);
        //println!("{}", t2);

        //println!("{:?}", args);
        //println!("{:?}", &self.response);

        Ok("".to_string())
    }

    pub fn new(req1: RequestProfile, req2: RequestProfile, res: ResponseProfile) -> Self {
        Self {
            req1,
            req2,
            response: res,
        }
    }
}

impl ValidateConfig for DiffProfile {
    fn validate(&self) -> anyhow::Result<()> {
        _ = &self.req1.validate().context("req1 error")?;
        _ = &self.req2.validate().context("req2 error")?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
pub struct ResponseProfile {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub skip_headers: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub skip_body: Vec<String>,
}

impl ResponseProfile {
    pub fn new(skip_headers: Vec<String>, skip_body: Vec<String>) -> Self {
        Self {
            skip_headers,
            skip_body,
        }
    }
}
