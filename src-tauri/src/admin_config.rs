use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

use crate::settings;

const DEFAULT_USERNAME: &str = "eko";
const DEFAULT_PASSWORD: &str = "eko2026.iy";
const DEFAULT_EXPIRY: &str = "2027-01-01";
const SALT: &str = "etiket-hazirlayici-admin-v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminConfigFile {
    pub username: String,
    pub password_hash: String,
    /// YYYY-MM-DD — bu güne kadar kullanım; bu gün kilitlenir
    pub expiry_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseStatus {
    pub is_locked: bool,
    pub expiry_date: String,
    pub days_remaining: i64,
    pub message: String,
}

fn config_path() -> PathBuf {
    settings::settings_dir().join("admin_config.json")
}

pub fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(SALT.as_bytes());
    hasher.update(password.as_bytes());
    hex::encode(hasher.finalize())
}

fn verify_password(password: &str, stored_hash: &str) -> bool {
    hash_password(password) == stored_hash
}

pub fn load_or_create() -> AdminConfigFile {
    let path = config_path();
    if let Ok(json) = fs::read_to_string(&path) {
        if let Ok(cfg) = serde_json::from_str::<AdminConfigFile>(&json) {
            if !cfg.username.is_empty() && !cfg.password_hash.is_empty() && !cfg.expiry_date.is_empty()
            {
                return cfg;
            }
        }
    }
    let cfg = AdminConfigFile {
        username: DEFAULT_USERNAME.into(),
        password_hash: hash_password(DEFAULT_PASSWORD),
        expiry_date: DEFAULT_EXPIRY.into(),
    };
    save(&cfg).ok();
    cfg
}

pub fn save(cfg: &AdminConfigFile) -> Result<(), String> {
    let path = config_path();
    let json = serde_json::to_string_pretty(cfg)
        .map_err(|e| format!("Yapılandırma serileştirilemedi: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("Yapılandırma kaydedilemedi: {}", e))?;
    Ok(())
}

fn parse_expiry(date_str: &str) -> Result<chrono::NaiveDate, String> {
    chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map_err(|_| "Geçersiz tarih formatı (YYYY-MM-DD bekleniyor)".into())
}

/// Son kullanma tarihinde ve sonrasında kilitli.
pub fn is_locked(cfg: &AdminConfigFile) -> bool {
    let today = chrono::Local::now().date_naive();
    match parse_expiry(&cfg.expiry_date) {
        Ok(expiry) => today >= expiry,
        Err(_) => false,
    }
}

pub fn license_status() -> LicenseStatus {
    let cfg = load_or_create();
    let today = chrono::Local::now().date_naive();
    let expiry = parse_expiry(&cfg.expiry_date).unwrap_or(today);
    let days_remaining = (expiry - today).num_days();
    let locked = is_locked(&cfg);

    let message = if locked {
        format!(
            "Lisans süresi {} tarihinde sona erdi. Devam etmek için yönetici girişi yapın.",
            format_date_tr(&cfg.expiry_date)
        )
    } else if days_remaining <= 30 {
        format!(
            "Lisans {} tarihine kadar geçerli ({} gün kaldı).",
            format_date_tr(&cfg.expiry_date),
            days_remaining
        )
    } else {
        format!("Lisans {} tarihine kadar geçerli.", format_date_tr(&cfg.expiry_date))
    };

    LicenseStatus {
        is_locked: locked,
        expiry_date: cfg.expiry_date.clone(),
        days_remaining,
        message,
    }
}

fn format_date_tr(iso: &str) -> String {
    if let Ok(d) = chrono::NaiveDate::parse_from_str(iso, "%Y-%m-%d") {
        return d.format("%d.%m.%Y").to_string();
    }
    iso.to_string()
}

pub fn verify_login(username: &str, password: &str) -> Result<(), String> {
    let cfg = load_or_create();
    if username.trim() != cfg.username {
        return Err("Kullanıcı adı veya şifre hatalı.".into());
    }
    if !verify_password(password, &cfg.password_hash) {
        return Err("Kullanıcı adı veya şifre hatalı.".into());
    }
    Ok(())
}

pub fn admin_info() -> (String, String) {
    let cfg = load_or_create();
    (cfg.username.clone(), cfg.expiry_date.clone())
}

pub fn set_expiry_date(date: &str) -> Result<(), String> {
    parse_expiry(date)?;
    let mut cfg = load_or_create();
    cfg.expiry_date = date.to_string();
    save(&cfg)
}

pub fn change_credentials(
    current_password: &str,
    new_username: &str,
    new_password: &str,
) -> Result<(), String> {
    let mut cfg = load_or_create();
    if !verify_password(current_password, &cfg.password_hash) {
        return Err("Mevcut şifre hatalı.".into());
    }
    let user = new_username.trim();
    if user.is_empty() {
        return Err("Kullanıcı adı boş olamaz.".into());
    }
    if new_password.len() < 4 {
        return Err("Yeni şifre en az 4 karakter olmalı.".into());
    }
    cfg.username = user.to_string();
    cfg.password_hash = hash_password(new_password);
    save(&cfg)
}
