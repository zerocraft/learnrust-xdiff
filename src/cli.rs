use anyhow::*;

use crate::ExtraArgs;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug, Clone)]
#[clap(version,author,about,long_about=None)]
pub struct Args {
    #[clap(subcommand)]
    pub action: Action,
}

#[derive(Subcommand, Debug, Clone)]
#[non_exhaustive]
pub enum Action {
    /// Diff two responses based on profile
    Run(RunArgs),
    Parse,
}

#[derive(Parser, Debug, Clone)]
pub struct RunArgs {
    /// profile name
    #[clap(short, long, value_parser)]
    pub profile: String,

    /// Override args
    /// For query, like '-e key=value'
    /// For header, like '-e %key=value'
    /// For body, like '-e @key=value'
    #[clap(short, long, value_parser=parse_key_val,number_of_values=1)]
    pub extra_params: Vec<KeyVal>,

    /// config file path
    #[clap(short, long, value_parser)]
    pub config: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyVal {
    key: String,
    value: String,
    key_type: KeyValType,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyValType {
    Query,
    Header,
    Body,
}

pub fn parse_key_val(s: &str) -> Result<KeyVal> {
    let mut parts = s.splitn(2, '=');

    // let retrieve = |v: Option<&str>, str: String| -> Result<&str> {
    //     Ok(v.ok_or_else(|| anyhow!("Invalid key value:{}", str))?
    //         .trim())
    // };
    // let key = retrieve(parts.next(), s.to_string())?;
    // let value = retrieve(parts.next(), s.to_string())?;

    let key = parts
        .next()
        .ok_or_else(|| anyhow!("Invalid key :{}", s))?
        .trim();
    let value = parts
        .next()
        .ok_or_else(|| anyhow!("Invalid value :{}", s))?
        .trim();

    let (key_type, key) = match key.chars().next() {
        Some('%') => (KeyValType::Header, &key[1..]),
        Some('@') => (KeyValType::Body, &key[1..]),
        Some(v) if v.is_ascii_alphabetic() => (KeyValType::Query, key),
        _ => return Err(anyhow!("Invalid key value type:{}", key)),
    };

    Ok(KeyVal {
        key: key.to_string(),
        value: value.to_string(),
        key_type: key_type,
    })
}

impl From<Vec<KeyVal>> for ExtraArgs {
    fn from(args: Vec<KeyVal>) -> Self {
        let mut headers = vec![];
        let mut querys = vec![];
        let mut bodys = vec![];

        for arg in args {
            match arg.key_type {
                KeyValType::Query => querys.push((arg.key, arg.value)),
                KeyValType::Header => headers.push((arg.key, arg.value)),
                KeyValType::Body => bodys.push((arg.key, arg.value)),
            }
        }
        Self {
            headers: headers,
            bodys: bodys,
            querys: querys,
        }
    }
}
