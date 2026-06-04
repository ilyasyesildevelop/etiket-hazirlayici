use crate::models::LabelSettings;
use std::fs;
use std::path::PathBuf;

const STARTUP_SETTINGS_FILE: &str = "startup_default_settings.json";

pub fn settings_dir() -> PathBuf {
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
                if name.ends_with(".json")
                    && name != "recent_files.json"
                    && name != "admin_config.json"
                    && name != STARTUP_SETTINGS_FILE
                {
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

pub fn save_startup_settings(settings: &LabelSettings) -> Result<(), String> {
    let path = settings_dir().join(STARTUP_SETTINGS_FILE);
    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Varsayılan ayarlar serileştirilemedi: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("Varsayılan ayarlar kaydedilemedi: {}", e))?;
    Ok(())
}

pub fn load_startup_settings() -> Option<LabelSettings> {
    let path = settings_dir().join(STARTUP_SETTINGS_FILE);
    let json = fs::read_to_string(&path).ok()?;
    if let Ok(settings) = serde_json::from_str::<LabelSettings>(&json) {
        return Some(settings);
    }
    // Eski kayıtlar (ör. cari_max_words) veya kısmi JSON için geçiş
    let mut value: serde_json::Value = serde_json::from_str(&json).ok()?;
    let obj = value.as_object_mut()?;
    if obj.get("cari_max_chars").is_none() {
        if let Some(w) = obj.get("cari_max_words").and_then(|v| v.as_u64()) {
            let chars = if w > 0 && w <= 20 { w.saturating_mul(10) } else { 45 };
            obj.insert("cari_max_chars".into(), serde_json::json!(chars));
        } else {
            obj.insert("cari_max_chars".into(), serde_json::json!(45));
        }
    }
    obj.remove("cari_max_words");
    serde_json::from_value(value).ok()
}

pub fn startup_settings_path() -> PathBuf {
    settings_dir().join(STARTUP_SETTINGS_FILE)
}
