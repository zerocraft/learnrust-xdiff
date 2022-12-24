use anyhow::Result;
use rust_xdiff::DiffConfig;

fn main() -> Result<()> {
    let content = include_str!("../fixtures/test.yml");
    let config = DiffConfig::from_yaml(content)?;

    print!("{:#?}", config);

    Ok(())
}
