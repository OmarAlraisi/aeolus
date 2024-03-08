use clap::Parser;
use serde::Deserialize;
use std::{fs, io::BufReader, process, u8};

const DEFAULT_LOG_FILE: &str = "/var/log/aeolus.log";
const DEFAULT_NETWORK_INTERFACE: &str = "wlp1s0";

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

fn mac_str_to_bytes(servers: Vec<String>) -> Result<Vec<[u8; 6]>, ConfigError> {
    let mac_err = Err(ConfigError::new("Invalid MAC address."));
    let mut mac_addresses: Vec<[u8; 6]> = Vec::new();
    for mac_str in servers {
        let pairs: Vec<&str> = mac_str.split(':').collect();

        if pairs.len() != 6 {
            return mac_err;
        }

        let mut bytes = [0; 6];
        for (idx, pair) in pairs.iter().enumerate() {
            match u8::from_str_radix(pair, 16) {
                Ok(pair_byte) => {
                    bytes[idx] = pair_byte;
                }
                Err(_) => return mac_err,
            };
        }

        mac_addresses.push(bytes);
    }
    Ok(mac_addresses)
}

#[derive(Parser, Debug)]
struct Options {
    /// Comma Seperated servers' MAC addresses
    #[arg(short, long, value_delimiter = ',', value_name = "MAC")]
    servers: Option<Vec<String>>,

    /// Comma Seperated ports [default: 80]
    #[arg(short, long, value_delimiter = ',', value_name = "PORT")]
    ports: Option<Vec<u16>>,

    /// Network interface to attach eBPF app to
    #[arg(short, long, value_name = "NI", default_value = DEFAULT_NETWORK_INTERFACE)]
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
                Ok(file_config) => {
                    let servers = mac_str_to_bytes(file_config.servers.clone())?;
                    Ok(Config {
                        ports: file_config.ports.unwrap_or(vec![80]),
                        servers,
                        log_file: file_config.log_file.unwrap_or(DEFAULT_LOG_FILE.to_string()),
                        iface: file_config
                            .iface
                            .unwrap_or(DEFAULT_NETWORK_INTERFACE.to_string()),
                    })
                }
                Err(e) => Err(e),
            }
        } else {
            if self.servers.is_none() {
                return Err(ConfigError::new(
                    "must specify 'servers' or provide a 'config' file.",
                ));
            }

            let servers = mac_str_to_bytes(self.servers.clone().unwrap())?;
            Ok(Config {
                ports: self.ports.clone().unwrap_or(vec![80]),
                servers,
                log_file: self.log_file.clone(),
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
    ports: Option<Vec<u16>>,
    servers: Vec<String>,
    log_file: Option<String>,
    iface: Option<String>,
}

#[derive(Debug)]
pub struct Config {
    pub ports: Vec<u16>,
    pub servers: Vec<[u8; 6]>,
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
