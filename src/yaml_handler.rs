use std::collections::HashMap;
use std::io::Error;
use std::str::FromStr;
use yaml_rust::Yaml;
use yaml_rust::yaml::{Array, Hash};
use crate::utilities::{build_invalid_data_error_str, build_invalid_data_error_string};

pub fn get_procedure(procedure_name: String, resources: &Hash, mappings: &Hash,
                     parameters: &HashMap<String, String>) -> Result<(String, HashMap<String, String>), Error> {
    let procedure = resources.get(&Yaml::from_str(procedure_name.as_str()))
        .ok_or(build_invalid_data_error_string(format!("Procedure {} not found", procedure_name)))?;
    let props = procedure["Properties"].as_hash()
        .ok_or(build_invalid_data_error_string(format!("Procedure {} properties not found", procedure_name)))?;
    let value = props.get(&Yaml::from_str("Value"))
        .ok_or(build_invalid_data_error_string(format!("Procedure {} value not found", procedure_name)))?
        .as_vec()
        .ok_or(build_invalid_data_error_string(format!("Procedure {} value should be a vector", procedure_name)))?;
    let text = value.get(0)
        .ok_or(build_invalid_data_error_string(format!("Procedure {} value[0] should be present", procedure_name)))?
        .as_str()
        .ok_or(build_invalid_data_error_string(format!("Procedure {} value[] should be a string", procedure_name)))
        .map(|s|s.to_string())?;
    let h = value.get(1)
        .ok_or(build_invalid_data_error_string(format!("Procedure {} value[1] should be present", procedure_name)))?
        .as_hash()
        .ok_or(build_invalid_data_error_string(format!("Procedure {} value[1] should be a hash", procedure_name)))?;
    let mut variables= HashMap::new();
    for (name_yaml, value_yaml) in h {
        let name = name_yaml.as_str()
            .ok_or(build_invalid_data_error_string(format!("Procedure {} parameter name is not string", procedure_name)))?;
        if let Some(value) = value_yaml.as_vec() {
            variables.insert(name.to_string(), find_array_variable_value(value, mappings, parameters)?);
        } else if let Some(value) = value_yaml.as_str() {
            variables.insert(name.to_string(), find_string_variable_value(value, parameters)?);
        } else {
            return Err(build_invalid_data_error_string(
                format!("Procedure {} parameter value is not vector or string", procedure_name)));
        }
    }
    Ok((text, variables))
}

fn find_array_variable_value(array: &Array, mappings: &Hash, parameters: &HashMap<String, String>) -> Result<String, Error> {
    if array.len() != 3 {
        return Err(build_invalid_data_error_str("variable array length should be = 3"));
    }
    let hash_name = array.get(0).unwrap().as_str()
        .ok_or(build_invalid_data_error_str("variable hash name should be string"))?;
    let section_key = array.get(1).unwrap().as_str()
        .ok_or(build_invalid_data_error_str("variable section name should be string"))?;
    let value_key = array.get(2).unwrap().as_str()
        .ok_or(build_invalid_data_error_str("variable value key should be string"))?;
    //println!("{} {} {}", hash_name, section_key, value_key);
    let hash = mappings.get(&Yaml::from_str(hash_name))
        .ok_or(build_invalid_data_error_string(format!("hash {} not found", hash_name)))?
        .as_hash()
        .ok_or(build_invalid_data_error_string(format!("{} should be a hash", hash_name)))?;
    let section_name = parameters.get(section_key)
        .ok_or(build_invalid_data_error_string(format!("parameter {} not found", section_key)))?;
    let section = hash.get(&Yaml::from_str(section_name))
        .ok_or(build_invalid_data_error_string(format!("key {} not found in {}", section_name, hash_name)))?
        .as_hash()
        .ok_or(build_invalid_data_error_string(format!("key {} in {} should be a hash", section_name, hash_name)))?;
    let value_yaml = section.get(&Yaml::from_str(value_key))
        .ok_or(build_invalid_data_error_string(format!("key {} not found in {}", value_key, section_name)))?;
    if let Some(s) = value_yaml.as_str() {
        return Ok(s.to_string());
    }
    if let Some(b) = value_yaml.as_bool() {
        return Ok(b.to_string());
    }
    if let Some(i) = value_yaml.as_i64() {
        return Ok(i.to_string());
    }
    Err(build_invalid_data_error_string(format!("key {} in {} should be a string", value_key, section_name)))
}

fn find_string_variable_value(name: &str, parameters: &HashMap<String, String>) -> Result<String, Error> {
    let sname = name.replace("${", "").replace("}", "");
    parameters.get(&sname)
        .map(|s|s.clone())
        .ok_or(build_invalid_data_error_string(format!("parameter {} not found", name)))
}

pub fn build_var_list(doc: &Yaml, source_doc: &Yaml) -> Result<HashMap<String, String>, Error>{
    let vars = source_doc.as_vec().ok_or(build_invalid_data_error_str("Source parameter should be a vector"))?;
    let mut result = HashMap::new();
    for var in vars {
        let h = var.as_hash().ok_or(build_invalid_data_error_str("Variable should be a hash"))?;
        let name_yaml = h.get(&Yaml::from_str("Name"))
            .ok_or(build_invalid_data_error_str("Variable name is absent"))?;
        let value_yaml = h.get(&Yaml::from_str("Value"))
            .ok_or(build_invalid_data_error_str("Variable name is absent"))?;
        let name = name_yaml.as_str().ok_or(build_invalid_data_error_str("Variable name should be a string"))?;
        let value = value_yaml.as_str().ok_or(build_invalid_data_error_str("Variable name should be a string"))?;
        let svalue = value.replace("${", "").replace(".Value}", "");
        //println!("{} {}", name, svalue);
        result.insert(name.to_string(), svalue);
    }
    Ok(result)
}


pub fn find_source_doc<'a>(doc: &'a Yaml, source: &String) -> Result<&'a Yaml, Error> {
    let mut source_doc = doc;
    for part in source.split('.') {
        if let Ok(idx) = usize::from_str(part) {
            if let Some(v) = source_doc.as_vec() {
                source_doc = &v[idx];
            } else {
                return Err(build_invalid_data_error_string(format!("expected vector yaml element for {} source parameter", part)))
            }
        } else {
            source_doc = &source_doc[part];
        }
        if source_doc.is_null() || source_doc.is_badvalue() {
            return Err(build_invalid_data_error_string(format!("{} source parameter not found in the env file", part)))
        }
    }
    Ok(source_doc)
}
