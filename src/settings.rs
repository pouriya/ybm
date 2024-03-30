use anyhow::Context;
use clap::{Parser, Subcommand};
use crossterm::style::Color;
use serde::{Deserialize, Deserializer};
use std::{collections::HashMap, fs, path::Path};

#[derive(Parser)]
#[command(version, about, author, long_about = None)]
pub struct CliArg {
    /// Enables tracing.
    #[arg(global = true, long)]
    pub trace: bool,
    #[command(subcommand)]
    pub maybe_subcommand: Option<CliSubCommand>,
}

#[derive(Subcommand)]
pub enum CliSubCommand {
    /// Runs TUI from provided or default configuration file
    Tui {
        /// A Toml configuration file containing issuers and secrets
        #[arg(default_value_t = default_config_file())]
        config_file: String,
    },
}

impl Default for CliSubCommand {
    fn default() -> Self {
        Self::Tui {
            config_file: default_config_file(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) struct Config {
    #[serde(default, rename = "THEME")]
    pub theme: Theme,
    #[serde(flatten)]
    pub accounts: HashMap<String, Account>,
}

#[derive(Debug, Deserialize)]
pub struct Theme {
    #[serde(default = "default_color_name")]
    pub name: Color,
    #[serde(default = "default_color_code")]
    pub code: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: default_color_name(),
            code: default_color_code(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct Account {
    #[serde(deserialize_with = "base32_de")]
    pub secret: (String, Vec<u8>),
    #[serde(default)]
    pub algorithm: Algorithm,
    #[serde(default = "default_password_length")]
    pub length: u32,
    #[serde(default, skip)]
    pub code: String,
}

#[derive(Debug, Default, Deserialize)]
pub enum Algorithm {
    #[default]
    Sha1,
}

struct Base32Parser;

impl<'de> serde::de::Visitor<'de> for Base32Parser {
    type Value = (String, Vec<u8>);

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("base-32 encoded secret for your account")
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_str(v.as_str())
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visit_str(v)
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if s.is_empty() {
            Err(serde::de::Error::invalid_length(
                0,
                &"non-empty base-32 encoded secret for your account",
            ))
        } else if let Some(bytes) = base32::decode(base32::Alphabet::RFC4648 { padding: false }, s)
        {
            Ok((s.to_string(), bytes))
        } else {
            Err(serde::de::Error::custom(
                "base-32 encoded secret for your account",
            ))
        }
    }
}

pub fn base32_de<'de, D>(deserializer: D) -> Result<(String, Vec<u8>), D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(Base32Parser)
}

fn default_config_file() -> String {
    format!(
        "{:?}",
        dirs::config_dir()
            .map(|dir| dir.join("ybm.toml"))
            .expect("System configuration directory")
    )
}

fn default_password_length() -> u32 {
    6
}

fn default_color_name() -> Color {
    Color::Blue
}

fn default_color_code() -> Color {
    Color::White
}

pub fn parse() -> CliArg {
    CliArg::parse()
}

pub fn ensure_config<P: AsRef<Path>>(config: P) -> anyhow::Result<Config> {
    let config_file = config.as_ref();
    if !config.as_ref().exists() {
        fs::write(config_file, include_str!("ybm.toml"))
            .map(|_| eprintln!("Default configuration created at {config_file:?}"))
            .with_context(|| format!("Could not write default config inside {config_file:?}"))?;
    }
    let config_contents = fs::read_to_string(config_file)
        .with_context(|| format!("Could not read configuration file {config_file:?}"))?;
    toml_edit::de::from_str(&config_contents)
        .with_context(|| format!("Could not decode configuration from {config_file:?}"))
}
