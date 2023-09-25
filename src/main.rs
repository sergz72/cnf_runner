mod yaml_handler;
mod utilities;

use std::collections::HashMap;
use std::env::args;
use std::io::Error;
use env_file::parse_env_file;
use crate::utilities::{apply_replaces, build_invalid_data_error_str, build_replaces, execute, load_config_file, replace};
use crate::yaml_handler::{build_var_list, find_source_doc, get_procedure};

fn main() -> Result<(), Error> {
    let mut arguments = args();
    if arguments.len() != 4 {
        println!("Usage: cnf_runner config_file_name env_file_name exec_file");
        return Ok(());
    }
    let config_file_name = arguments.nth(1).unwrap();
    let env_file_name = arguments.next().unwrap();
    let exec_file = arguments.next().unwrap();
    let parameters = parse_env_file(env_file_name)?;
    let source = parameters.get("source")
        .ok_or(build_invalid_data_error_str("Source parameter is absent in the env file"))?;
    let replaces = build_replaces(parameters.get("replace"))?;
    if source.len() == 0 {
        println!("Source parameter is empty in the env file");
        return Ok(());
    }
    let docs = load_config_file(config_file_name)?;
    let doc = &docs[0];
    let resources = doc["Resources"].as_hash()
        .ok_or(build_invalid_data_error_str("No resources found"))?;
    let mappings = doc["Mappings"].as_hash()
        .ok_or(build_invalid_data_error_str("No mappings found"))?;
    let source_doc = find_source_doc(doc, source)?;
    let var_list = build_var_list(source_doc)?;
    let mut env_vars: HashMap<String, String> = HashMap::new();
    for (name, procedure_name) in var_list {
        if let Some(value) = parameters.get(&procedure_name) {
            println!("{} {}", name, value);
            env_vars.insert(name, value.clone());
        } else {
            let (text, variables) =
                get_procedure(procedure_name, resources, mappings, &parameters)?;
            let final_variables: HashMap<String, String> = variables.iter()
                .map(|(name, value)| (name.clone(), apply_replaces(value, &replaces)))
                .collect();
            let value = replace(text, final_variables, &parameters)?;
            println!("{} {}", name, value);
            env_vars.insert(name, value);
        }
    }
    if let Some(secrets_env_file) = parameters.get("secretsEnvFile") {
        let parameters = parse_env_file(secrets_env_file.clone())?;
        for (name, value) in parameters {
            println!("{} {}", name, value);
            env_vars.insert(name, value);
        }
    }
    execute(exec_file, env_vars)
}
