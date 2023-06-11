use std::collections::HashMap;
use std::fs;
use std::io::{Error, ErrorKind};
use std::process::{Command, Stdio};
use yaml_rust::{Yaml, YamlLoader};

pub fn build_invalid_data_error_str(text: &str) -> Error {
    Error::new(ErrorKind::InvalidData, text)
}

pub fn build_invalid_data_error_string(text: String) -> Error {
    Error::new(ErrorKind::InvalidData, text)
}

pub fn load_config_file(config_file_name: String) -> Result<Vec<Yaml>, Error> {
    let contents = fs::read_to_string(config_file_name)?;
    return YamlLoader::load_from_str(contents.as_str())
        .map_err(|e|build_invalid_data_error_string(e.to_string()));
}

pub fn build_replaces(replaces_option: Option<&String>) -> Result<HashMap<String, String>, Error> {
    let mut result = HashMap::new();
    if let Some(replaces) = replaces_option {
        if replaces.len() > 0 {
            let parts: Vec<&str> = replaces.split("->").collect();
            let l = parts.len();
            if (l % 1) != 0 {
                return Err(build_invalid_data_error_str("invalid replaces parameter"));
            }
            for i in (0..l).step_by(2) {
                result.insert(parts[i].to_string(), parts[i+1].to_string());
            }
        }
    }
    Ok(result)
}

pub fn replace(text: String, variables: HashMap<String, String>, parameters: &HashMap<String, String>) -> Result<String, Error> {
    let mut result = text;
    for (name, value) in variables {
        result = result.replace(("${".to_string() + name.as_str() + "}").as_str(), value.as_str());
    }
    for (name, value) in parameters {
        result = result.replace(("${".to_string() + name.as_str() + "}").as_str(), value.as_str());
    }
    Ok(result)
}

pub fn apply_replaces(text: &String, replaces: &HashMap<String, String>) -> String {
    let mut result = text.clone();
    for (from, to) in replaces {
        result = result.replace(from, to);
    }
    result
}

pub fn execute(app_name: String, env_vars: HashMap<String, String>) -> Result<(), Error> {
    let status = Command::new(&app_name)
        .envs(env_vars)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    println!("{} finished with status {}", app_name, status);
    Ok(())
}