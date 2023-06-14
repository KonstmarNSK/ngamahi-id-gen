use std::env::VarError;
use std::fs::File;
use std::io::BufReader;
use std::string::ToString;
use serde::{Deserialize};


const CFG_PATH_ENV_KEY : &str = "ID_GEN_CFG_PATH";
const CFG_DEFAULT_PATH: &str = "configs/default/";
const CFG_PROPS_FILE: &str = "properties.yaml";
const CFG_LOG_FILE: &str = "logs.yaml";


#[derive(Deserialize, Clone)]
pub struct Properties{
    pub etcd_addr: String,
    pub etcd_fetch_range_size: u64,
    pub client_range_max_size: u64,
}

pub struct Configs{
    pub props: Properties,
    pub logs_cfg_path: String,
}


pub fn read_configs() -> Result<Configs, Error> {
    let cfg_path = match std::env::var(&CFG_PATH_ENV_KEY) {
        Ok(path) => {
            println!("Env var {} is set, configs will be read from {}", &CFG_PATH_ENV_KEY, &path);
            path
        }
        Err(VarError::NotPresent) => {
            println!("Env var {} is not set so DEFAULT configs will be used", &CFG_PATH_ENV_KEY);
            CFG_DEFAULT_PATH.to_string()
        }
        Err(VarError::NotUnicode(_)) => {
            println!("Bad env var {} (bad symbols). Must be unicode.", &CFG_PATH_ENV_KEY);
            panic!()
        }
    };

    let cfg_file = File::open(cfg_path.clone() + CFG_PROPS_FILE)?;
    let cfg_reader = BufReader::new(cfg_file);

    let props = serde_yaml::from_reader::<BufReader<File>, Properties>(cfg_reader)?;

    if props.client_range_max_size > props.etcd_fetch_range_size {
        return Err(Error::Validation("Bad configs. client_range_max_size must be less than or equal to etcd_fetch_range_size".to_string()))
    }

    Ok(Configs{
        props,
        logs_cfg_path: cfg_path + CFG_LOG_FILE,
    })
}

#[derive(Debug)]
pub enum Error{
    IO(std::io::Error),
    Deserialization(serde_yaml::Error),
    Validation(String)
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::IO(value)
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(value: serde_yaml::Error) -> Self {
        Error::Deserialization(value)
    }
}
