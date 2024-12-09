use serde_json::Value;
use serde_json::Value::String as JsonString;
use std::{collections::HashMap, fs, path::PathBuf};

pub fn get_default_config() -> HashMap<String, Value> {
    HashMap::from([
        (
            "abbreviation_file".to_owned(),
            JsonString(".abv.json".to_owned()),
        ),
        ("extensions".to_owned(), JsonString("None".to_owned())),
    ])
}

pub fn get_default_context() -> HashMap<String, String> {
    HashMap::new()
}

pub fn get_user_config(
    config_path: PathBuf,
) -> Result<HashMap<String, Value>, Box<dyn std::error::Error>> {
    let config_content = fs::read_to_string(config_path)?;
    let config: HashMap<String, Value> = serde_json::from_str(&config_content)?;
    Ok(config)
}
