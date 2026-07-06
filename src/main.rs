#![feature(decl_macro)]

mod element;
mod link;
mod profile;
mod resume;

use std::fs::{
    self,
    File,
    read_to_string,
};
use std::io::{
    Result,
    Write,
};
use std::path::{
    Path,
    PathBuf,
};

use minify_html::{
    Cfg,
    minify,
};
use serde::Deserialize;

use crate::element::*;
use crate::profile::profile;
use crate::resume::resume;

const MAIN_CSS: &str = include_str!("../assets/css/main.css");
const OPEN_SANS_CSS: &str = include_str!("../assets/css/open-sans.css");
const PROFILE_CSS: &str = include_str!("../assets/css/profile.css");
const RESUME_CSS: &str = include_str!("../assets/css/resume.css");

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Config {
    name: String,
    address: String,
    title: String,
    subtitle: String,
    contact: Contact,
    languages: Vec<Language>,
    soft_skills: Vec<SoftSkill>,
    hard_skills: Vec<HardSkill>,
    interests: Vec<Interest>,
    experiences: Vec<Experience>,
    education: Vec<Education>,
    projects: Vec<Project>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Contact {
    email: String,
    phone: String,
    website: String,
    bluesky: String,
    linkedin: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Language {
    name: String,
    level: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct SoftSkill {
    name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct HardSkill {
    name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    examples: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Interest {
    name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    examples: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Experience {
    company: String,
    time: String,
    title: String,
    description: Option<String>,
    link: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    skills: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Education {
    school: String,
    degree: String,
    time: String,
    description: Option<String>,
    link: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Project {
    title: String,
    description: String,
    link: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    skills: Vec<String>,
}

fn copy_dir_all<S, D>(
    src: S,
    dst: D,
) -> Result<()>
where
    S: AsRef<Path>,
    D: AsRef<Path>,
{
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let entry_type = entry.file_type()?;
        let src_entry_path = entry.path();
        let dst_entry_path = dst.as_ref().join(entry.file_name());
        if entry_type.is_dir() {
            copy_dir_all(src_entry_path, dst_entry_path)?;
        } else {
            fs::copy(src_entry_path, dst_entry_path)?;
        }
    }
    Ok(())
}

fn main() {
    let default_lang = "en";
    let assets_directory = PathBuf::from("assets");
    let output_directory = PathBuf::from("public");
    let resume_directory = PathBuf::from("resume");

    let resume_files = resume_directory
        .read_dir()
        .expect("failed to read resume directory")
        .map(|dir_entry| dir_entry.expect("failed to read resume file").path());

    copy_dir_all(&assets_directory, output_directory.join(&assets_directory))
        .expect("failed to copy assets to the output directory");

    for path in resume_files {
        let lang = path
            .file_name()
            .and_then(|s| s.to_str())
            .and_then(|s| s.split_once('.'))
            .map(|(lang, _)| lang)
            .unwrap_or_default();

        let config: Config =
            toml::from_str(&read_to_string(&path).expect("failed to read config file"))
                .expect("failed to parse config file");

        let html = format!(
            "<!doctype html>{}",
            html!(
                @lang = lang,
                head!(
                    title!(format!("CV {}", config.name)),
                    style!(MAIN_CSS, OPEN_SANS_CSS, PROFILE_CSS, RESUME_CSS),
                    meta!(
                        @name = "description",
                        @content = match lang {
                            "fr" => format!("CV de {} pour {}", config.name, config.title),
                            _ => format!("Resume of {} for {}", config.name, config.title),
                        },
                    ),
                ),
                body!(main!(profile(&config, lang), resume(&config, lang),),),
            )
        );

        let cfg = Cfg {
            allow_optimal_entities: true,
            allow_removing_spaces_between_attributes: true,
            minify_doctype: true,
            minify_css: true,
            remove_bangs: true,
            remove_processing_instructions: true,
            ..Default::default()
        };

        let minified = minify(html.as_bytes(), &cfg);

        std::fs::create_dir_all(output_directory.join(lang))
            .expect("failed to create language directory");

        let file_path = output_directory.join(lang).join("index.html");

        let mut file = File::create(&file_path).expect("failed to create {lang}/index.html");
        file.write_all(&minified)
            .expect("failed to write to {lang}/index.html");

        if lang == default_lang {
            std::fs::copy(&file_path, output_directory.join("index.html"))
                .expect("failed to copy {lang}/index.html to index.html");
        }
    }
}
