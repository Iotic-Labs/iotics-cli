use std::{
    io,
    sync::{Arc, Mutex},
};

use iotics_grpc_client::auth_builder::IntoAuthBuilder;
use iotics_identity::{create_agent_auth_token, Config};
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

#[derive(Clone)]
pub struct AuthBuilder {
    settings: Arc<Mutex<Settings>>,
    token: Arc<Mutex<Option<String>>>,
}

impl AuthBuilder {
    pub fn new(settings: Settings) -> Arc<Self> {
        Arc::new(Self {
            settings: Arc::new(Mutex::new(settings)),
            token: Arc::new(Mutex::new(None)),
        })
    }

    pub fn from(auth_builder: Arc<AuthBuilder>) -> Arc<Self> {
        Arc::new(AuthBuilder::clone(&auth_builder))
    }

    pub fn update_host(&self, new_host_address: String) -> Result<(), anyhow::Error> {
        let mut settings_lock = self
            .settings
            .lock()
            .map_err(|_| anyhow::anyhow!("failed to lock the settings mutex"))?;

        settings_lock.iotics.host_address = new_host_address;

        Ok(())
    }
}

impl IntoAuthBuilder for AuthBuilder {
    fn get_host(&self) -> Result<String, anyhow::Error> {
        let settings_lock = self
            .settings
            .lock()
            .map_err(|_| anyhow::anyhow!("failed to lock the settings mutex"))?;

        Ok(settings_lock.iotics.host_address.clone())
    }

    fn get_token(&self) -> Result<String, anyhow::Error> {
        let mut token_lock = self
            .token
            .lock()
            .map_err(|_| anyhow::anyhow!("failed to lock the token mutex"))?;

        if token_lock.is_none() {
            let settings_lock = self
                .settings
                .lock()
                .map_err(|_| anyhow::anyhow!("failed to lock the settings mutex"))?;

            let identity_config = Config {
                resolver_address: settings_lock.iotics.resolver_address.clone(),
                token_duration: settings_lock.iotics.token_duration as i64,
                user_did: settings_lock.iotics.user_did.clone(),
                agent_did: settings_lock.iotics.agent_did.clone(),
                agent_key_name: settings_lock.iotics.agent_key_name.clone(),
                agent_name: settings_lock.iotics.agent_name.clone(),
                agent_secret: settings_lock.iotics.agent_secret.clone(),
            };

            let token = create_agent_auth_token(&identity_config)?;
            let token = format!("bearer {}", token);

            token_lock.replace(token);
        }

        let token = token_lock.as_ref().expect("this should never happen");

        Ok(token.clone())
    }
}
