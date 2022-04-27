use serde::Deserialize;
use config;
use secrecy::{ExposeSecret, Secret};

#[derive(Deserialize)]
pub struct Settings{
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
}

#[derive(Deserialize)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
}

#[derive(Deserialize)]
pub struct DatabaseSettings{
    pub username: String,

    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

pub fn get_configuration() -> Result<Settings, config::ConfigError>{
    //APP_ENVIRONMENT : local / production
    //histoire de dossiers a resoudre.
    
    let base_path = std::env::current_dir().expect("can't find current dir");
    let configuration_directory = base_path.join("configuration");

    let mut settings = config::Config::default();
    settings.merge(config::File::from(configuration_directory.join("base")).required(true))?;

    let environment: Environment = std::env::var("APP_ENVIRONMENT").unwrap_or_else(|_|"local".into()).as_str().try_into().expect("failed to parse APP_ENVIRONMENT");
    settings.merge(config::File::from(configuration_directory.join(environment.as_str())).required(true))?;

    settings.try_into()
}

enum Environment {
    Local,
    Production
}

impl TryFrom<&str> for Environment {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value{
            "local" => {Ok(Environment::Local)},
            "production" => {Ok(Environment::Production)},
            _ => {Err(format!("environment {} is unknown", value))},
    }
}
}

impl Environment {
    fn as_str(&self) -> &'static str {
        match &self {
            Self::Local => "local",
            Self::Production => "production",
    }
}
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> Secret<String> {
        Secret::new(format!("postgres://{}:{}@{}:{}/{}",
                self.username, 
                self.password.expose_secret(),
                self.host,
                self.port,
                self.database_name))
    }

    pub fn connection_string_without_db(&self) -> Secret<String> {
        Secret::new(format!("postgres://{}:{}@{}:{}",
                self.username, 
                self.password.expose_secret(),
                self.host,
                self.port,
                ))
    }
}

#[test]
fn test_string_connection(){
    let dbs = DatabaseSettings { username: "cedric".to_owned() , password: Secret::new("abcd123".to_owned()), port:5252, host:"monhost".to_owned(), database_name: "madb".to_owned()};
    assert_eq!(dbs.connection_string().expose_secret(), "postgres://cedric:abcd123@monhost:5252/madb");
}

#[test]
fn test_string_connection_without_db(){
    let dbs = DatabaseSettings { username: "cedric".to_owned() , password: Secret::new("abcd123".to_owned()), port:5252, host:"monhost".to_owned(), database_name: "madb".to_owned()};
    assert_eq!(dbs.connection_string_without_db().expose_secret(), "postgres://cedric:abcd123@monhost:5252");
}
