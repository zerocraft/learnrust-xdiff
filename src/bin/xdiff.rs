use anyhow::{Ok, Result};

use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Input, MultiSelect};
use rust_xlearn::{
    cli::*, DiffConfig, DiffProfile, ExtraArgs, LoadConfig, RequestProfile, ResponseProfile,
};
use std::io::Write;

#[tokio::main]
pub async fn main() -> Result<()> {
    let args = Args::parse();

    match args.action {
        Action::Run(args) => run(args).await?,
        Action::Parse => parse().await?,
        _ => panic!("Not implemented"),
    }
    Ok(())
}

async fn parse() -> Result<()> {
    let theme = ColorfulTheme::default();
    let url1: String = Input::with_theme(&theme)
        .with_prompt("Url1")
        .interact_text()?;
    let url2: String = Input::with_theme(&theme)
        .with_prompt("Url2")
        .interact_text()?;
    let req1: RequestProfile = url1.parse()?;
    let req2 = url2.parse()?;

    let profile_name: String = Input::with_theme(&theme)
        .with_prompt("Profile")
        .interact_text()?;

    let res1 = req1.send(&ExtraArgs::default()).await?;

    let mut headers = res1.get_header_keys();
    headers.sort();

    let chosen = MultiSelect::with_theme(&theme)
        .with_prompt("Skip Headers")
        .items(&headers)
        .interact()?;
    let skip_headers = chosen.iter().map(|i| headers[*i].to_string()).collect();
    let response = ResponseProfile::new(skip_headers, vec![]);
    let profile = DiffProfile::new(req1, req2, response);
    let config = DiffConfig::new(vec![(profile_name, profile)].into_iter().collect());
    let result = serde_yaml::to_string(&config)?;

    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    write!(
        stdout,
        "---\n{}",
        rust_xlearn::highlight_text(&result, "yaml", None).unwrap()
    )?;
    Ok(())
}

async fn run(args: RunArgs) -> Result<()> {
    let config_file = args.config.unwrap_or_else(|| "./dif.yml".to_string());
    let config = DiffConfig::load_yaml(&config_file).await?;
    let profile = config
        .get_profile(&args.profile)
        .ok_or_else(|| anyhow::anyhow!("no profile {} in config {}", args.profile, config_file))?;
    let extra_args: ExtraArgs = args.extra_params.into();
    profile.diff(extra_args).await?;
    Ok(())
}
