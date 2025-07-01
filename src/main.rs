mod plato;

use std::{
    env, fs::{self, File}, io, path::{Path, PathBuf}, sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    }
};
use std::fs::OpenOptions;
use anyhow::{format_err, Context, Error};
use chrono::{DateTime, Datelike, Local, Utc};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::prelude::*;

const SETTINGS_PATH: &str = "Settings.toml";

#[derive(Debug, Deserialize, Serialize)]
enum Value {
    Str(&'static str),
    Int(i32),
    Bool(bool),
}


/// The structure of a dropbox entry. Usually represents a book.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    /// The name of the file.
    pub name: String,
    /// The unique identifier of the book.
    pub id: String,
    /// The date the book was published.
    pub server_modified: Option<DateTime<Utc>>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct DropboxEntries {
    /// Vec of entries
    pub entries: Vec<Entry>,
}

/// Holds the settings for the application converted from a TOML file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
struct Settings {
    /// List of preferred file types to download (i.e. application/x-cbz or application/pdf).
    dropbox_token: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            dropbox_token: "".to_string(),
        }
    }
}

fn get_dropbox_token(client_id: &str, refresh_token: &str) -> Result<String, Error> {
    // curl -k --silent https://api.dropbox.com/oauth2/token \
    // -d grant_type=refresh_token \
    // -d client_id=$client_id \
    // -d refresh_token=$refresh_token
    let client = Client::new();
    let response = client
        .post("https://api.dropbox.com/oauth2/token")
        .form(&[
            ("grant_type", "refresh_token"),
            ("client_id", client_id),
            ("refresh_token", refresh_token),
        ])
        .send()
        .context("Get token: Failed to send token refresh request")?;

    let json: serde_json::Value = response.json().context("Get token: Failed to parse response as JSON")?;
    
    if let Some(access_token) = json.get("access_token").and_then(|v| v.as_str()) {
        Ok(access_token.to_string())
    } else {
        Err(format_err!("Get token: Response missing access token"))
    }
}

fn load_list_folder(token: &String) -> Result<Vec<Entry>, Error> {
    // curl -X POST 'https://api.dropboxapi.com/2/files/list_folder' \
    // --header 'Authorization: Bearer sl.u.AF2WXXy-cQhkIe-FchzjQOnenYB6DbkjypoIaq2EBPZa27ETdBHjAhyjTgqBx7PACSGPWFizz4Vd4WEjny6CCRO30zPgkZFuyAs3ebMB1vhLAJxIxnJ8iUWMds2_sk2jy_2bmR2pAQUKlBGGBbpRUEv5pupO81eOktijxK7C8ifRq4g9a4d2jTc26GU8sULshNl6iK5UcqH93l8z3hFqX-v_QTybEdV7mq3xYoX_lOiXG7GPpp5PIkFEMqQ0YPvWbguvmBQ40b88CtFGj923XIJ4jqtGoibts4esOLqFs-cGEZsBbbfer6-Q9n0dnEkBz7f0Tlobk3-DTlcxoIhkqMR91YpPTmck-qJNkxTksZYeIg1JsNg92DHyZ4chJmdUmjIwKkLVmVpj0zDboncHsrUDvLO_f1tgzZ_euKoH7Y1ROtbYLIYzrp9JCiHm0aF-oFsEZdCrfxjEhza_gT1QES7McbhAxsEJCwP5vyMOaeelSE1ZxDAogWHpFDdPTUitZskguGUtbdx4IkNWFd-9xCo0FnrpQT9svLWDdbISUBcDW0n4RZj8f5Jhlhpr93BqlsiSCeSGAjxNs_fAM5qhp0zPo1kFu22zMOJZXiT_yTTQBN6VGNy-pjrwm98bhDEKRJehOflqylod-FaMvDRwUZS57Aqb-Ur2qWwX9ZgyUBLl76cHeBoSvSW8_MSN3jVk_4plG_hfHmb-MSsFZf_vWEo2IecOzAqJdGeyZy7S20rRo9FSFXA0nUZmZG75o5zBuZXzoi9AWnk1pMZqNKsh2TM_Re82ccib6cOsnjhUCnxjSZxoWxWWt8KbyA-qobmIsHRvYHdT07KroJ__H0O4VFzJX1xsHb_qSv7f9r_8oLPU2nH8JNIx3TazLbsqLW3RyMQkujoX0zRLfnYP-0m_t7-FFrtfv83-t1YpGQw4yi7lt2sj_Fr44JbCFguF_Q9ywDFHNOcfyckfn4-SpLDQZKUt0XFKGcH8o2-4VZbgtVysAADkNUQg2N6az7900_JoMvCaaO6FC-L4uutjYH78F9mpS8xgoSK0XVjlQgoHBNp4R1JwW8NjttSUQjc4xU-X_ZQJ-G8nCRE8eacyKMnfsZS0pRtr-MAVfNDGtcQL9i8rGIMeqZdcFaEGQzdDiCEuFUIEoubl3t6ory0_UWArJgcgVHWS3Pqo3Wlb_rlYorTQwKFWJiRT-XFPAXtcWWtays7dOq3d4Kzi3UrDghAejGvLe-u_WrB8gx1xooMgVIZ2UsOXn6CVLdyHNhS20qcVLzsIWerGRuwpTzz82i_8YTuL' \
    // --header 'Content-Type: application/json' \
    // --data-raw '{"path": "","include_non_downloadable_files": false, "limit": 1000}'
    let client = Client::new();

    // Static JSON payload
    let body = json!({
        "path": "",
        "include_non_downloadable_files": false,
        "limit": 1000
    });

    let response = client
        .post("https://api.dropboxapi.com/2/files/list_folder")
        .header("Authorization", format!("Bearer {}", token))
        .json(&body)
        .send()
        .context("List folder: Failed to send token refresh request")?;
    
    let dropbox_entries: DropboxEntries = response.json().context("List folder: Failed to parse response as JSON")?;
    Ok(dropbox_entries.entries)
}

fn load_and_process_dropbox() -> Result<(), Error> {
    let mut args = env::args().skip(1);
    let library_path = PathBuf::from(
        args.next()
            .ok_or_else(|| format_err!("missing argument: library path"))?,
    );
    let save_path = PathBuf::from(
        args.next()
            .ok_or_else(|| format_err!("missing argument: save path"))?,
    );
    let wifi = args
        .next()
        .ok_or_else(|| format_err!("missing argument: wifi status"))
        .and_then(|v| v.parse::<bool>().map_err(Into::into))?;
    let online = args
        .next()
        .ok_or_else(|| format_err!("missing argument: online status"))
        .and_then(|v| v.parse::<bool>().map_err(Into::into))?;
    let settings: Settings = load_toml::<Settings, _>(SETTINGS_PATH)
        .with_context(|| format!("can't load settings from {}", SETTINGS_PATH))?;

    if !online {
        if !wifi {
            plato::show_notification("Establishing a network connection.");
            plato::set_wifi(true);
        } else {
            plato::show_notification("Waiting for the network to come up.");
        }
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
    }

    if !save_path.exists() {
        fs::create_dir(&save_path)?;
    }

    let client = Client::builder().user_agent("Plato-Dropbox/1.0.0").build()?;
    let sigterm = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&sigterm))?;

    // split to client_id and refresh_token dropbox-token = client_id:refresh_token
    let (client_id, refresh_token) = settings.dropbox_token
        .split_once(':')
        .ok_or(format_err!("Invalid dropbox token"))?;
    
    let token = get_dropbox_token(
        client_id,
        refresh_token,
    )?;

    let entries = load_list_folder(&token)?;
    let len = entries.len();
    plato::show_notification(&format!("Sync {len} files"));

    for entry in entries {
        if sigterm.load(Ordering::Relaxed) {
            break;
        }
        
        // Support epub files only
        if !entry.name.to_lowercase().ends_with(".epub") {
            continue;
        }
            
        let doc_path = save_path.join(&entry.name);
        let path = doc_path.display();
        if doc_path.exists() {
            plato::show_notification(&format!("Skip {path} - already existed"));
            continue;
        }
        plato::show_notification(&format!("Start sync {path} "));

        let mut response = client
            .post("https://content.dropboxapi.com/2/files/download")
            .header("Authorization", format!("Bearer {}", token))
            .header("Dropbox-API-Arg", format!(r#"{{"path": "{}"}}"#, entry.id))
            .send()?;

        if !response.status().is_success() {
            plato::show_notification(&format!("Error downloading '{}': {} - {}.", 
                entry.name, response.status(), response.text().unwrap_or_default() 
            ));
            continue;
        }
        
        let mut file = File::create(&doc_path)?;
        let write_file_result = response.copy_to(&mut file);

        if let Err(err) = write_file_result {
            plato::show_notification(&format!("Error downloading '{}': {:#}.", entry.name, err));
            fs::remove_file(doc_path).ok();
            continue;
        }

        if let Ok(path) = doc_path.strip_prefix(&library_path) {
            let file_info = json!({
                "path": path,
                "kind": "epub",
                "size": file.metadata().ok().map_or(0, |m| m.len()),
            });

           
            let year = match entry.server_modified {
                Some(date) => date.year().to_string(),
                None => "".to_string(),
            };

            // Get the current time.
            let updated_at = Utc::now();

            let read_state = json!({
                "opened": updated_at.with_timezone(&Local)
                                   .format("%Y-%m-%d %H:%M:%S")
                                   .to_string(),
                "currentPage": 0,
                "PagesCount": 1,
                "finished": false,
                "dithered": "false"
            });

            let info = json!({
                "title": entry.name,
                "author": "Unknown",
                "year": year,
                "identifier": entry.id,
                "added": updated_at.with_timezone(&Local)
                                   .format("%Y-%m-%d %H:%M:%S")
                                   .to_string(),
                "file": file_info,
                "reader": read_state
            });

            plato::add_document(info);
        }
    }
    plato::show_notification("Finished syncing with ");
    Ok(())
}

fn main() -> Result<(), Error> {
    log_panics::init();
    if let Err(err) = load_and_process_dropbox() {
        eprintln!("Error: {:#}", err);
        plato::show_notification(&format!("Error: {err}"));
        write_log(&format!("{:#}", err));
        return Err(err);
    }

    Ok(())
}


fn write_log(message: &str) {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open("plato-dropbox.log")
        .unwrap();
    writeln!(file, "{}", message).ok();
}

pub fn load_toml<T, P: AsRef<Path>>(path: P) -> Result<T, Error>
where
    for<'a> T: Deserialize<'a>,
{
    let s = fs::read_to_string(path.as_ref())
        .with_context(|| format!("can't read file {}", path.as_ref().display()))?;
    toml::from_str(&s)
        .with_context(|| format!("can't parse TOML content from {}", path.as_ref().display()))
        .map_err(Into::into)
}