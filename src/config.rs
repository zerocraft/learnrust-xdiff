use anyhow::Ok;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io::Write};
use tokio::fs;

use crate::{req::RequestProfile, utils::diff_text, ExtraArgs};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiffConfig {
    #[serde(flatten)]
    pub profiles: HashMap<String, DiffProfile>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiffProfile {
    pub req1: RequestProfile,
    pub req2: RequestProfile,
    pub response: ResponseProfile,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponseProfile {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub skip_headers: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub skip_body: Vec<String>,
}

impl DiffConfig {
    pub fn from_yaml(content: &str) -> anyhow::Result<Self> {
        Ok(serde_yaml::from_str(content)?)
    }

    pub async fn load_yaml(path: &str) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path).await?;
        Self::from_yaml(&content)
    }

    pub fn get_profile(&self, name: &str) -> Option<&DiffProfile> {
        self.profiles.get(name)
    }
}

impl DiffProfile {
    pub async fn diff(&self, args: ExtraArgs) -> anyhow::Result<String> {
        let r1 = self.req1.send(&args).await?;
        //println!("{:?}", r1);
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
}
