use std::io;

use iotics_identity::{create_agent_auth_token, Config, IdentityLibError};
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;
use yansi::Paint;

#[derive(Deserialize, Clone)]
pub struct Settings {
    pub iotics: IoticsSettings,
}

#[derive(serde::Deserialize, Clone)]
pub struct IoticsSettings {
    pub host_address: String,
    pub resolver_address: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub token_duration: usize,
    pub user_did: String,
    pub agent_did: String,
    pub agent_key_name: String,
    pub agent_name: String,
    pub agent_secret: String,
}

impl Settings {
    pub fn new(config_name: &str, stdout: &mut dyn io::Write) -> Result<Settings, anyhow::Error> {
        let mut settings = config::Config::default();
        let base_path = std::env::current_dir().expect("Failed to determine the current directory");
        let configuration_directory = base_path.join("configuration");

        writeln!(stdout, "Loading base configuration...")?;
        stdout.flush()?;
        // Read the "base" configuration file
        settings.merge(config::File::from(configuration_directory.join("base")).required(true))?;

        // Layer on the config-specific values.
        let mut config_path = configuration_directory.join(config_name);
        config_path.set_extension("yaml");
        writeln!(
            stdout,
            "Loading configuration {:#?}...",
            Paint::blue(&config_path),
        )?;
        stdout.flush()?;
        settings.merge(config::File::from(config_path).required(true))?;

        let settings: Settings = settings.try_into()?;

        writeln!(
            stdout,
            "host {}",
            Paint::blue(&settings.iotics.host_address),
        )?;
        writeln!(
            stdout,
            "resolver {}",
            Paint::blue(&settings.iotics.resolver_address),
        )?;
        writeln!(
            stdout,
            "user did {}",
            Paint::blue(&settings.iotics.user_did),
        )?;
        writeln!(
            stdout,
            "agent did {}",
            Paint::blue(&settings.iotics.agent_did),
        )?;
        stdout.flush()?;

        Ok(settings)
    }
}

pub fn get_token(settings: &Settings) -> Result<String, IdentityLibError> {
    let identity_config = Config {
        resolver_address: settings.iotics.resolver_address.clone(),
        token_duration: settings.iotics.token_duration as i64,
        user_did: settings.iotics.user_did.clone(),
        agent_did: settings.iotics.agent_did.clone(),
        agent_key_name: settings.iotics.agent_key_name.clone(),
        agent_name: settings.iotics.agent_name.clone(),
        agent_secret: settings.iotics.agent_secret.clone(),
    };

    let token = create_agent_auth_token(&identity_config)?;
    Ok(format!("bearer {}", token))
}
