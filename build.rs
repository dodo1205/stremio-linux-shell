use std::{
    fs::{self, File},
    io::BufReader,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Error, Ok, Result};
use bzip2::bufread::BzDecoder;
use dircpy::copy_dir;
use globset::{Glob, GlobBuilder};
use serde::{Deserialize, Serialize};
use toml::Value;

const CEF_CDN: &str = "https://cef-builds.spotifycdn.com";
const CEF_CDN_INDEX: &str = "index.json";
const CEF_FILE_TYPE: &str = "minimal";
const CEF_OUT: &str = "cef";
const CEF_ARCHIVE_FILES: &[[&str; 2]] = &[
    ["*/Resources/locales/**", "locales"],
    ["*/Resources/*.pak", ""],
    ["*/Resources/icudtl.dat", ""],
    ["*/Release/libcef.so", ""],
    ["*/Release/libEGL.so", ""],
    ["*/Release/libGLESv2.so", ""],
    ["*/Release/v8_context_snapshot.bin", ""],
];

#[derive(Deserialize, Serialize, Debug)]
pub struct CefFile {
    #[serde(rename = "type")]
    pub file_type: String,
    pub name: String,
    pub sha1: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CefVersion {
    pub cef_version: String,
    pub files: Vec<CefFile>,
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct CefPlatform {
    pub versions: Vec<CefVersion>,
}

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct CefIndex {
    pub linux64: CefPlatform,
}

fn main() -> Result<()> {
    let cef_version = get_version()?;

    let cef_path = PathBuf::from(CEF_OUT);
    let debug_path = cef_path.join("debug");
    let release_path = cef_path.join("release");

    if !cef_path.exists() {
        let archive_name = get_archive_name(cef_version)?;
        let archive_url = format!("{CEF_CDN}/{}", archive_name);
        let archive_path = cef_path.join(archive_name);
        let archive_out_path = cef_path.join("archive");

        fs::create_dir_all(&cef_path)?;
        download_archive(&archive_url, &archive_path)?;
        unpack_archive(&archive_path, &archive_out_path)?;
        fs::remove_file(&archive_path)?;

        copy_dir(&archive_out_path, &debug_path)?;
        copy_dir(&archive_out_path, &release_path)?;
        fs::remove_dir_all(archive_out_path)?;

        strip_symbols(&release_path, "*.so")?;
    }

    let ld_library_path = match cfg!(debug_assertions) {
        true => debug_path.to_str().unwrap(),
        false => release_path.to_str().unwrap(),
    };

    println!("cargo:rustc-env=LD_LIBRARY_PATH={}", ld_library_path);

    Ok(())
}

fn get_version() -> Result<String> {
    let cargo_toml = fs::read_to_string("Cargo.toml")?;
    let value: Value = toml::from_str(&cargo_toml)?;

    value
        .get("dependencies")
        .and_then(|deps| deps.get("cef"))
        .and_then(|dep| {
            if let Value::Table(table) = dep {
                table.get("version")
            } else {
                Some(dep)
            }
        })
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or(Error::msg("Failed to get cef version"))
}

fn get_archive_name(cef_version: String) -> Result<String> {
    println!("Fetch archive name...");

    let index_url = format!("{CEF_CDN}/{CEF_CDN_INDEX}");
    let index = ureq::get(index_url)
        .call()?
        .into_body()
        .read_json::<CefIndex>()?;

    let version = index
        .linux64
        .versions
        .iter()
        .find(|version| version.cef_version.starts_with(&cef_version))
        .ok_or(Error::msg("Failed to find version"))?;

    let file = version
        .files
        .iter()
        .find(|file| file.file_type == CEF_FILE_TYPE)
        .ok_or(Error::msg("Failed to find file"))?;

    Ok(file.name.to_owned())
}

fn download_archive(url: &str, out: &Path) -> Result<()> {
    println!("Downloading archive...");

    if !out.exists() {
        let resp = ureq::get(url).call()?;
        let mut file = File::create(out)?;

        std::io::copy(&mut resp.into_body().into_reader(), &mut file)?;
    }

    Ok(())
}

fn unpack_archive(path: &Path, out: &Path) -> Result<()> {
    println!("Unpacking archive...");

    if path.exists() {
        let decoder = BzDecoder::new(BufReader::new(File::open(path)?));
        let mut archive = tar::Archive::new(decoder);

        for entry in archive.entries()? {
            let mut entry = entry?;

            let file_path = entry.header().path()?.into_owned();

            if let Some(file_name) = file_path.file_name() {
                for archive_file in CEF_ARCHIVE_FILES {
                    let glob = GlobBuilder::new(archive_file[0])
                        .literal_separator(true)
                        .build()?
                        .compile_matcher();

                    if glob.is_match(&file_path) {
                        let dest_path = out.join(archive_file[1]);
                        let dest_file_path = dest_path.join(file_name);
                        println!("Writting {:?} to {:?}", file_path, dest_file_path);

                        fs::create_dir_all(&dest_path)?;
                        entry.unpack(dest_file_path)?;
                    }
                }
            }
        }
    }

    Ok(())
}

fn strip_symbols(path: &Path, glob: &str) -> Result<()> {
    println!("Stripping symbols...");

    let glob = Glob::new(glob)?.compile_matcher();

    for entry in fs::read_dir(path)? {
        let path = entry?.path();

        if glob.is_match(&path) {
            let mut command = Command::new("strip");
            command.arg("-s");
            command.arg(path.to_str().unwrap());
            command.spawn().context("Failed to strip symbols")?;
        }
    }

    Ok(())
}
