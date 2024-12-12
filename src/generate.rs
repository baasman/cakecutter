use crate::errors::GenerateFilesError;
use crate::template::{self, Template};
use clap::builder::Str;
use log::{debug, info};
use serde_json::json;
use std::io::Write;
use std::{
    error::Error,
    fs, path,
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

fn render_rel_path(rel_path: &str, context: &Context) -> String {
    let mut name_template = Tera::default();
    let temp_id = "rel_path";
    name_template.add_raw_template(temp_id, rel_path);
    let rendered_dir = name_template.render(&temp_id, &context).unwrap();
    rendered_dir
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

fn create_dir(
    template_data: &Template,
    context: &Context,
    output_dir: Option<PathBuf>,
    overwrite_if_exists: bool,
) -> Result<PathBuf, Box<dyn Error>> {
    let dir_name = find_project_dir_name(template_data, context);
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
        fs::create_dir_all(&final_dir).map_err(|e| Box::new(GenerateFilesError::IoError(e)))?;
        return Ok(final_dir);
    } else {
        return Err(Box::new(GenerateFilesError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Unable to find the directory to create",
        ))));
    }
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

    let final_out_dir = create_dir(&template_data, &context, output_dir, overwrite_if_exists)?;
    let delete_project_on_failure = true && !keep_project_on_failure;
    if accept_hooks {};
    let mut env = Tera::default();
    let repo_dir = path::absolute(&template_data.repo_dir)?;
    for entry in WalkDir::new(&repo_dir).into_iter() {
        match entry {
            Ok(entry) => {
                let abs_path = path::absolute(entry.path())?;
                let abs_path_str = abs_path.to_str().unwrap();
                if let Ok(rel_path) = abs_path.strip_prefix(&repo_dir) {
                    let template_name = rel_path.to_str().unwrap();
                    let rendered_template_name = render_rel_path(template_name, &context);
                    let ret = env.add_template_file(abs_path_str, Some(template_name));
                    if let Ok(_) = ret {
                        let copy_dont_render = is_copy_only_path(&entry, &template_data);
                        if !copy_dont_render {
                            let rendered_templ = env.render(template_name, &context)?;
                            let output_file_path = final_out_dir.join(rendered_template_name);
                            let mut output_file = fs::File::create(&output_file_path)?;
                            output_file.write_all(rendered_templ.as_bytes())?;
                            println!("{}", rendered_templ);
                        }
                    }
                }
            }
            Err(e) => {
                // raise custom error here
                eprintln!("Error reading file: {}", e)
            }
        }
    }
    Ok(())
}
