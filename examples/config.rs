use anyhow::Result;
use rust_xlearn::{DiffConfig, LoadConfig, ReqConfig};

fn main() -> Result<()> {
    let content = include_str!("../fixtures/test.yml");
    let config = DiffConfig::from_yaml(content)?;
    println!("{:#?}", config);

    let content = include_str!("../fixtures/req.yml");
    let config = ReqConfig::from_yaml(content)?;
    println!("{:#?}", config);

    Ok(())
}
