use anyhow::{Ok, Result};
use clap::Parser;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Input;
use rust_xlearn::{
    cli::*, get_body_text, get_header_text, get_status_text, highlight_text, ExtraArgs, LoadConfig,
    ReqConfig,
};
use std::fmt::Write as _;
use std::io::Write as _;

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
        .with_prompt("Url")
        .interact_text()?;
    let req: rust_xlearn::RequestProfile = url1.parse()?;

    let profile_name: String = Input::with_theme(&theme)
        .with_prompt("Profile")
        .interact_text()?;

    let config = ReqConfig::new(vec![(profile_name, req)].into_iter().collect());
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
    let config_file = args.config.unwrap_or_else(|| "./xreq.yml".to_string());
    let config = ReqConfig::load_yaml(&config_file).await?;
    let profile = config
        .get_profile(&args.profile)
        .ok_or_else(|| anyhow::anyhow!("no profile {} in config {}", args.profile, config_file))?;
    let extra_args: ExtraArgs = args.extra_params.into();
    let url = profile.get_url(&extra_args)?;
    let res = profile.send(&extra_args).await?.into_inner();

    let status = get_status_text(&res)?;
    let headers = get_header_text(&res, &[])?;
    let body = get_body_text(res, &[]).await?;

    let mut output = String::new();
    write!(
        &mut output,
        "{}\n",
        highlight_text(&format!("Url: {}\n", url), "yaml", None)?
    )?;
    write!(
        &mut output,
        "{}\n",
        highlight_text(
            &format!("Result: {}", status),
            "yaml",
            Some("Solarized (dark)")
        )?
    )?;
    write!(
        &mut output,
        "{}\n",
        highlight_text(&headers, "yaml", Some("InspiredGitHub"))?
    )?;
    write!(&mut output, "{}", highlight_text(&body, "json", None)?)?;

    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    writeln!(stdout, "{}", output)?;

    Ok(())
}
