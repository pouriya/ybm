mod otp;
mod settings;
mod tui;

fn main() -> anyhow::Result<()> {
    match settings::parse().maybe_subcommand.unwrap_or_default() {
        settings::CliSubCommand::Tui { config_file } => {
            tui::run(&mut crate::settings::ensure_config(config_file)?)?;
        }
    }
    Ok(())
}
