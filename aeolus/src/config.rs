use clap::Parser;
use serde::Deserialize;
use std::{fs, io::BufReader, net::Ipv4Addr, process, str::FromStr};

const DEFAULT_LOG_FILE: &str = "/var/log/aeolus.log";

struct ConfigError {
    description: String,
}

impl ConfigError {
    fn new(description: &str) -> Self {
        ConfigError {
            description: description.to_string(),
        }
    }

    fn exit(&self) -> ! {
        eprintln!("ERR: {}", self.description);
        process::exit(1);
    }
}

#[derive(Parser, Debug)]
struct Options {
    /// Comma Seperated servers
    #[arg(short, long, value_delimiter = ',', value_name = "IP")]
    servers: Option<Vec<String>>,

    /// Comma Seperated ports [default: 80]
    #[arg(short, long, value_delimiter = ',', value_name = "PORT")]
    ports: Option<Vec<u16>>,

    /// Network interface of the virtual IP
    #[arg(short, long, value_name = "NI", default_value = "wlp1s0")]
    iface: String,

    /// Path to log file
    #[arg(long = "logfile", value_name = "FILE", default_value = DEFAULT_LOG_FILE)]
    log_file: String,

    /// Path to Aeolus configuration file
    #[arg(long, value_name = "FILE")]
    config: Option<String>,
}

impl Options {
    fn validate(&self) -> Result<Config, ConfigError> {
        if let Some(config_file) = &self.config {
            if self.servers.is_some() || self.ports.is_some() || &self.log_file != DEFAULT_LOG_FILE
            {
                return Err(ConfigError::new("cannot specify 'servers', 'ports', or 'logfile' when providing a 'config' file."));
            }

            match Options::parse_config_file(config_file) {
                Ok(file_config) => Ok(Config {
                    ports: file_config.ports,
                    servers: file_config.servers,
                    log_file: file_config.log_file.unwrap_or(DEFAULT_LOG_FILE.to_string()),
                    iface: file_config.iface,
                }),
                Err(e) => Err(e),
            }
        } else {
            if self.servers.is_none() {
                return Err(ConfigError::new(
                    "must specify 'servers' or provide a 'config' file.",
                ));
            }

            let mut servers: Vec<Ipv4Addr> = Vec::new();
            for server_str in self.servers.clone().unwrap().iter() {
                match Ipv4Addr::from_str(server_str) {
                    Ok(server) => {
                        servers.push(server);
                    }
                    Err(e) => return Err(ConfigError::new(e.to_string().as_str())),
                }
            }

            Ok(Config {
                ports: self.ports.clone().unwrap_or(vec![80]),
                servers,
                log_file: self
                    .log_file
                    .clone(),
                iface: self.iface.clone(),
            })
        }
    }

    fn parse_config_file(config_file: &str) -> Result<FileConfig, ConfigError> {
        let file_name = config_file.to_lowercase();
        if !(file_name.ends_with(".yaml") || file_name.ends_with(".yml")) {
            return Err(ConfigError::new("configuration file must be a YAML file."));
        }

        let file = match fs::File::open(config_file) {
            Ok(file) => file,
            Err(e) => {
                return Err(ConfigError::new(e.to_string().as_str()));
            }
        };
        let reader = BufReader::new(file);
        let content: Result<FileConfig, ConfigError> = match serde_yaml::from_reader(reader) {
            Ok(content) => Ok(content),
            Err(e) => Err(ConfigError::new(e.to_string().as_str())),
        };
        content
    }
}

#[derive(Deserialize)]
struct FileConfig {
    ports: Vec<u16>,
    servers: Vec<Ipv4Addr>,
    log_file: Option<String>,
    iface: String,
}

#[derive(Debug)]
pub struct Config {
    pub ports: Vec<u16>,
    pub servers: Vec<Ipv4Addr>,
    pub log_file: String,
    pub iface: String,
}

impl Config {
    pub fn parse() -> Self {
        let options = Options::parse();
        match options.validate() {
            Ok(config) => config,
            Err(e) => e.exit(),
        }
    }
}
