use git2::Repository;
use log::{debug, error, info, trace, warn};
use serde_json::Value;
use std::{
    collections::HashMap,
    fs::{self},
    os::unix::process,
    path::PathBuf,
};

use crate::{
    generate::generate_files,
    template::Template,
    template::TemplateType,
    user_config::{get_default_config, get_default_context, get_user_config},
};

fn get_cakecutter_json_content(
    template_type: TemplateType,
    default_template_values: HashMap<String, String>,
    abbreviation_file: Option<&serde_json::Value>,
) -> Result<Template, Box<dyn std::error::Error>> {
    let mut template_data;
    match template_type {
        TemplateType::Path(path) => {
            let mut template_loc = PathBuf::from(&path);
            template_loc.push("cakecutter.json");
            info!("Using file {}", template_loc.display());
            let template_content = fs::read_to_string(template_loc)?;
            let template = serde_json::from_str(&template_content)?;
            template_data = Template::new(template, PathBuf::from(path));
        }
        TemplateType::ZipPath(path) => {
            // pull and unzip into temp dir
            // look for json file
            let template_loc = &path;
            let template = HashMap::from([(
                "version".to_owned(),
                serde_json::Value::String("1".to_owned()),
            )]);
            template_data = Template::new(template, PathBuf::from(path));
        }
        TemplateType::RepoURL(url) => {
            let temp_dir = tempfile::tempdir().expect("Failed to create a temporary directory");
            let temp_path = temp_dir.path();
            info!("Name of temp dir: {}", temp_path.display());
            _ = Repository::clone(&url, temp_path);
            let template = HashMap::from([(
                "version".to_owned(),
                serde_json::Value::String("1".to_owned()),
            )]);
            template_data = Template::new(template, PathBuf::new());
        }
    }
    for (key, value) in default_template_values.into_iter() {
        if let None = template_data.template.get(&key) {
            template_data.template.insert(key, Value::String(value));
        }
    }
    template_data.generate_original_context();
    if let Some(abbrev_file) = abbreviation_file {
        template_data.set_abbreviations(abbrev_file.as_str().unwrap());
    }
    Ok(template_data)
}

pub fn cakecutter(
    template_type: TemplateType,
    output_dir: Option<PathBuf>,
    config_path: PathBuf,
    replay: bool,
    no_input: bool,
    overwrite_if_exists: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = get_user_config(config_path).unwrap_or_else(|_| get_default_config());
    let default_template_values = get_default_context();
    let abbreviation_file = config.get("abbreviation_file");
    let template_data =
        get_cakecutter_json_content(template_type, default_template_values, abbreviation_file)?;

    if replay {
    } else {
    }

    match generate_files(
        template_data,
        output_dir,
        overwrite_if_exists,
        false,
        false,
        false,
    ) {
        Ok(_) => {
            info!("Successfully generated files");
        }
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1)
        }
    }

    dbg!(config);
    dbg!(replay);
    Ok(())
}
