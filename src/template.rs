use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use clap::parser::Values;
use log::{debug, error, info, trace, warn};
use serde_json::Value;
use url::Url;

fn is_zip_file(path: &PathBuf) -> bool {
    path.extension()
        .map_or(false, |ext| ext.eq_ignore_ascii_case("zip"))
}

fn is_git_repo(url: &str) -> bool {
    if let Ok(parsed_url) = Url::parse(url) {
        match parsed_url.scheme() {
            "http" | "https" | "git" | "ssh" => true,
            _ => false,
        }
    } else {
        url.contains('@') && url.contains(':')
    }
}

pub fn parse_template_input(input: String, directory: Option<PathBuf>) -> TemplateType {
    let path = PathBuf::from(&input);
    if path.exists() {
        if path.is_file() {
            panic!("Must provide a directory, not a file: {}", &input)
        }
        if is_zip_file(&path) {
            return TemplateType::ZipPath(path);
        }
        if let Some(dir) = directory {
            if !dir.exists() {
                panic!("Directory {} is given but can not be found", dir.display());
            }
            return TemplateType::Path(dir);
        }
        TemplateType::Path(path)
    } else {
        if !is_git_repo(&input) {
            panic!("Path {} is given but is not a valid git url", input);
        }
        TemplateType::RepoURL(input)
    }
}

fn _load_abbreviations(
    abbrev_loc: PathBuf,
) -> Result<HashMap<String, Value>, Box<dyn std::error::Error>> {
    let abbrev_content = fs::read_to_string(abbrev_loc)?;
    let abbrev_data = serde_json::from_str(&abbrev_content).unwrap_or_default();
    Ok(abbrev_data)
}

pub enum RepoDir {
    Path(PathBuf),
    RepoUrl(String),
}

#[derive(Debug)]
pub struct Template {
    pub template: HashMap<String, Value>,
    pub template_original: HashMap<String, Value>,
    pub abbreviations: HashMap<String, Value>,
    pub repo_dir: PathBuf,
}

pub enum TemplateType {
    Path(PathBuf),
    ZipPath(PathBuf),
    RepoURL(String),
}

impl Template {
    pub fn new(template: HashMap<String, Value>, repo_dir: PathBuf) -> Self {
        Self {
            template,
            template_original: HashMap::new(),
            abbreviations: HashMap::new(),
            repo_dir,
        }
    }

    pub fn generate_original_context(&mut self) {
        self.template_original = self
            .template
            .iter()
            .filter(|(key, _)| !key.starts_with('_'))
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
    }

    pub fn set_abbreviations(&mut self, abbv_file: &str) {
        let path_abbv = PathBuf::from(abbv_file);
        let abbreviations = _load_abbreviations(path_abbv);
        self.abbreviations = abbreviations.unwrap_or(HashMap::new());
    }

    pub fn should_cleanup_dir(&self, template: &TemplateType) -> bool {
        match template {
            TemplateType::Path(_) => false,
            TemplateType::ZipPath(_) => true,
            TemplateType::RepoURL(_) => false,
        }
    }
}
