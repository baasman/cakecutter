use crate::errors::GenerateFilesError;
use crate::template::{self, Template};
use clap::builder::Str;
use log::{debug, info};
use serde_json::json;
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};
use tera::{Context, Tera};
use walkdir::{DirEntry, WalkDir};
use yash_fnmatch::{without_escape, Pattern};

fn find_child_dir(repo_dir: PathBuf) -> Option<String> {
    if let Ok(dir_entries) = fs::read_dir(repo_dir) {
        for entry in dir_entries {
            if let Ok(entry) = entry {
                let file_type = entry.file_type().unwrap();
                if file_type.is_dir() {
                    let file_name = entry.file_name();
                    let dir_name = file_name.to_str().unwrap();
                    if dir_name.contains("cakecutter") && dir_name.starts_with('{') {
                        return Some(dir_name.to_owned());
                    }
                }
            }
        }
    }
    None
}

fn find_project_dir_name(
    template_data: &Template,
    context: &Context,
) -> Option<Result<String, Box<dyn std::error::Error>>> {
    let child_dir = find_child_dir(template_data.repo_dir.clone());
    if let Some(unren_dir) = child_dir {
        let mut name_template = Tera::default();
        let temp_id = "dir_name";
        name_template.add_raw_template(temp_id, &unren_dir);
        let rendered_dir = name_template.render(&temp_id, &context).unwrap();
        return Some(Ok(rendered_dir));
    }
    None
}

fn is_copy_only_path(entry: &DirEntry, template: &Template) -> bool {
    let p = Pattern::parse(without_escape(entry.path().to_str().unwrap())).unwrap();
    let empty_array = json!([]);
    let dont_render = template
        .template
        .get("_copy_without_render")
        .unwrap_or(&empty_array)
        .as_array()
        .ok_or("Expected _copy_without_render to be an array");
    let mut to_copy_only = false;
    match dont_render {
        Ok(array) => {
            for val in array {
                println!("{}", val.as_str().unwrap());
                let matched_range = p.find(&val.to_string());
                if let Some(_matched_range) = matched_range {
                    to_copy_only = true
                }
            }
        }
        Err(e) => {
            eprintln!("{e}")
        }
    }
    to_copy_only
}

pub fn generate_files(
    template_data: Template,
    output_dir: Option<PathBuf>,
    overwrite_if_exists: bool,
    skip_of_file_exists: bool,
    accept_hooks: bool,
    keep_project_on_failure: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let context = Context::from_serialize(&template_data.template)?;
    info!(
        "Generating project from {}",
        template_data.repo_dir.display()
    );
    let dir_name = find_project_dir_name(&template_data, &context);
    let mut created = false;
    if let Some(dir) = dir_name {
        let dir_to_render = dir?;
        let final_dir = if let Some(out_dir) = output_dir {
            out_dir.join(Path::new(&dir_to_render))
        } else {
            PathBuf::from(&dir_to_render)
        };
        debug!(
            "Output directory {} already exists, will overwrite",
            final_dir.display()
        );
        if final_dir.exists() {
            if overwrite_if_exists {
                debug!(
                    "Output directory {} already exists, will overwrite",
                    final_dir.display()
                );
            } else {
                return Err(Box::new(GenerateFilesError::DirectoryExists(
                    final_dir.display().to_string(),
                )));
            }
        }
        fs::create_dir_all(final_dir)?;
        created = true;
    }
    let delete_project_on_failure = created && !keep_project_on_failure;
    if accept_hooks {};
    let env_path = format!("{}/**/*.cake", template_data.repo_dir.display());
    let env = Tera::new(&env_path).unwrap();
    let all_template_files = env.get_template_names();
    dbg!(&context);
    // iterate through dir and if it's a template THEN render
    for entry in WalkDir::new(&template_data.repo_dir).into_iter() {
        match entry {
            Ok(entry) => {
                let copy_dont_render = is_copy_only_path(&entry, &template_data);
                let template_name = entry.path().display().to_string();
                let rendered_templ = env.render(&template_name, &context)?;
                println!("File or dir: {}", entry.path().display());
                println!("{rendered_templ}");
            }
            Err(e) => {
                // raise custom error here
                eprintln!("Error reading file...")
            }
        }
    }
    Ok(())
}
