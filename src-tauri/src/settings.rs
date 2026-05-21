use crate::models::LabelSettings;
use std::fs;
use std::path::PathBuf;

fn settings_dir() -> PathBuf {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("EtiketHazırlayici");
    fs::create_dir_all(&dir).ok();
    dir
}

pub fn save_settings(settings: &LabelSettings, name: &str) -> Result<(), String> {
    let path = settings_dir().join(format!("{}.json", name));
    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Ayarlar serileştirilemedi: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("Ayarlar kaydedilemedi: {}", e))?;
    Ok(())
}

pub fn load_settings(name: &str) -> Result<LabelSettings, String> {
    let path = settings_dir().join(format!("{}.json", name));
    let json = fs::read_to_string(&path).map_err(|e| format!("Ayarlar okunamadı: {}", e))?;
    serde_json::from_str(&json).map_err(|e| format!("Ayarlar ayrıştırılamadı: {}", e))
}

pub fn list_settings() -> Vec<String> {
    let dir = settings_dir();
    let mut names = Vec::new();
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".json") && name != "recent_files.json" {
                    names.push(name.trim_end_matches(".json").to_string());
                }
            }
        }
    }
    names
}

pub fn save_recent_files(files: &[String]) -> Result<(), String> {
    let path = settings_dir().join("recent_files.json");
    let json = serde_json::to_string(files).map_err(|e| format!("Serileştirme hatası: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("Kayıt hatası: {}", e))?;
    Ok(())
}

pub fn load_recent_files() -> Vec<String> {
    let path = settings_dir().join("recent_files.json");
    fs::read_to_string(&path)
        .ok()
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default()
}
