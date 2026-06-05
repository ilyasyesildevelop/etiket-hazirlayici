mod admin_config;
mod excel_parser;
mod models;
mod pplb;
#[cfg(windows)]
mod printer_win;
mod satir_parser;
mod settings;

use admin_config::LicenseStatus;
use models::*;
use satir_parser::ParsedSatir;
use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{Emitter, State};

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[cfg(windows)]
fn hide_console_window(command: &mut Command) -> &mut Command {
    use std::os::windows::process::CommandExt;
    command.creation_flags(CREATE_NO_WINDOW)
}

#[cfg(not(windows))]
fn hide_console_window(command: &mut Command) -> &mut Command {
    command
}

struct AdminSession {
    token: String,
    expires_at: Instant,
}

struct AppState {
    rows: Mutex<Vec<RawRow>>,
    manual_labels: Mutex<Vec<ParsedLabel>>,
    current_file: Mutex<String>,
    admin_session: Mutex<Option<AdminSession>>,
}

const ADMIN_SESSION_HOURS: u64 = 2;

fn validate_admin_token(state: &State<AppState>, token: &str) -> Result<(), String> {
    let session = state.admin_session.lock().unwrap();
    match session.as_ref() {
        Some(s) if s.token == token && s.expires_at > Instant::now() => Ok(()),
        Some(_) => Err("Oturum süresi doldu. Tekrar giriş yapın.".into()),
        None => Err("Geçersiz oturum. Tekrar giriş yapın.".into()),
    }
}

fn new_admin_token(state: &State<AppState>) -> String {
    let token = format!(
        "{:x}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    *state.admin_session.lock().unwrap() = Some(AdminSession {
        token: token.clone(),
        expires_at: Instant::now() + Duration::from_secs(ADMIN_SESSION_HOURS * 3600),
    });
    token
}

#[tauri::command]
fn get_license_status() -> LicenseStatus {
    admin_config::license_status()
}

#[tauri::command]
fn admin_login(
    username: String,
    password: String,
    state: State<AppState>,
) -> Result<serde_json::Value, String> {
    admin_config::verify_login(&username, &password)?;
    let token = new_admin_token(&state);
    let (user, expiry) = admin_config::admin_info();
    Ok(serde_json::json!({
        "token": token,
        "username": user,
        "expiry_date": expiry,
    }))
}

#[tauri::command]
fn admin_logout(token: String, state: State<AppState>) {
    let mut session = state.admin_session.lock().unwrap();
    if session.as_ref().map(|s| s.token.as_str()) == Some(token.as_str()) {
        *session = None;
    }
}

#[tauri::command]
fn admin_get_info(token: String, state: State<AppState>) -> Result<serde_json::Value, String> {
    validate_admin_token(&state, &token)?;
    let (username, expiry_date) = admin_config::admin_info();
    Ok(serde_json::json!({ "username": username, "expiry_date": expiry_date }))
}

#[tauri::command]
fn admin_set_expiry(token: String, expiry_date: String, state: State<AppState>) -> Result<(), String> {
    validate_admin_token(&state, &token)?;
    admin_config::set_expiry_date(&expiry_date)
}

#[tauri::command]
fn admin_change_credentials(
    token: String,
    current_password: String,
    new_username: String,
    new_password: String,
    state: State<AppState>,
) -> Result<(), String> {
    validate_admin_token(&state, &token)?;
    admin_config::change_credentials(&current_password, &new_username, &new_password)
}

#[tauri::command]
fn open_file_dialog(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog()
        .file()
        .add_filter("Excel Dosyaları", &["xlsx", "xlsm", "xls"])
        .pick_file(move |file_path| {
            let result = file_path.map(|p| p.to_string());
            tx.send(result).ok();
        });
    rx.recv()
        .map_err(|_| "Dosya seçme iptal edildi".to_string())
}

#[tauri::command]
fn get_sheets(file_path: String) -> Result<Vec<ExcelSheet>, String> {
    excel_parser::get_sheets(&file_path)
}

#[tauri::command]
fn load_excel(
    file_path: String,
    sheet_name: String,
    state: State<AppState>,
) -> Result<serde_json::Value, String> {
    let (rows, mapping) = excel_parser::parse_excel(&file_path, &sheet_name)?;

    *state.rows.lock().unwrap() = rows.clone();
    state.manual_labels.lock().unwrap().clear();
    *state.current_file.lock().unwrap() = file_path.clone();

    let mut recent = settings::load_recent_files();
    recent.retain(|f| f != &file_path);
    recent.insert(0, file_path);
    recent.truncate(10);
    settings::save_recent_files(&recent).ok();

    Ok(serde_json::json!({
        "rows": rows,
        "mapping": mapping,
        "total": rows.len(),
    }))
}

#[tauri::command]
fn parse_satir(
    text: String,
    malz: String,
    bekleyen: String,
    dokumanizleme: String,
    rules: SatirRules,
    extra_islem_keywords: Vec<String>,
) -> ParsedSatir {
    satir_parser::parse_satir_aciklama(
        &text,
        &malz,
        &bekleyen,
        &dokumanizleme,
        "",
        &rules,
        &extra_islem_keywords,
    )
}

#[tauri::command]
fn parse_all_labels(
    state: State<AppState>,
    rules: SatirRules,
    cari_max_chars: usize,
    extra_islem_keywords: Vec<String>,
) -> Vec<ParsedLabel> {
    let rows = state.rows.lock().unwrap();
    let mut result: Vec<ParsedLabel> = rows
        .iter()
        .map(|row| {
            let parsed = satir_parser::parse_satir_aciklama(
                &row.satir_aciklama,
                &row.malz_aciklama,
                &row.bekleyen_siparis,
                &row.dokumanizleme_no,
                &row.cari_unvan,
                &rules,
                &extra_islem_keywords,
            );
            ParsedLabel {
                cari_unvan: satir_parser::truncate_cari(&row.cari_unvan, cari_max_chars),
                malz_aciklama: row.malz_aciklama.clone(),
                ebat: parsed.ebat,
                islem: parsed.islem,
                adet: parsed.adet,
                metrekare: parsed.metrekare,
                musteri_adi: parsed.musteri_adi,
                diger_aciklamalar: parsed.diger_aciklamalar,
                bekleyen_siparis: row.bekleyen_siparis.clone(),
                print_count: parsed.print_count,
            }
        })
        .collect();

    // Manuel eklenen etiketleri listenin sonuna iliştir
    let manual = state.manual_labels.lock().unwrap();
    result.extend(manual.iter().cloned());

    result
}

#[tauri::command]
fn add_manual_label(label: ParsedLabel, state: State<AppState>) {
    state.manual_labels.lock().unwrap().push(label);
}

#[tauri::command]
fn update_manual_label(index: usize, label: ParsedLabel, state: State<AppState>) {
    let mut manual = state.manual_labels.lock().unwrap();
    if index < manual.len() {
        manual[index] = label;
    }
}

#[tauri::command]
fn remove_manual_label(index: usize, state: State<AppState>) {
    let mut manual = state.manual_labels.lock().unwrap();
    if index < manual.len() {
        manual.remove(index);
    }
}

#[tauri::command]
fn remove_excel_row(index: usize, state: State<AppState>) {
    let mut rows = state.rows.lock().unwrap();
    if index < rows.len() {
        rows.remove(index);
    }
}

#[tauri::command]
fn clear_all_data(state: State<AppState>) {
    state.rows.lock().unwrap().clear();
    state.manual_labels.lock().unwrap().clear();
    *state.current_file.lock().unwrap() = String::new();
}

#[tauri::command]
fn save_label_settings(settings_data: LabelSettings, name: String) -> Result<(), String> {
    settings::save_settings(&settings_data, &name)
}

#[tauri::command]
fn load_label_settings(name: String) -> Result<LabelSettings, String> {
    settings::load_settings(&name)
}

#[tauri::command]
fn save_settings_to_file(
    app: tauri::AppHandle,
    settings_data: LabelSettings,
) -> Result<String, String> {
    use tauri_plugin_dialog::DialogExt;
    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog()
        .file()
        .add_filter("Etiket Ayarı", &["json"])
        .save_file(move |file_path| {
            tx.send(file_path.map(|p| p.to_string())).ok();
        });
    let path = rx
        .recv()
        .map_err(|_| "İptal edildi".to_string())?
        .ok_or("İptal edildi")?;

    let json = serde_json::to_string_pretty(&settings_data).unwrap();
    std::fs::write(&path, json).map_err(|e| format!("Dosya yazılamadı: {}", e))?;
    Ok(path)
}

#[tauri::command]
fn load_settings_from_file(app: tauri::AppHandle) -> Result<LabelSettings, String> {
    use tauri_plugin_dialog::DialogExt;
    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog()
        .file()
        .add_filter("Etiket Ayarı", &["json"])
        .pick_file(move |file_path| {
            tx.send(file_path.map(|p| p.to_string())).ok();
        });
    let path = rx
        .recv()
        .map_err(|_| "İptal edildi".to_string())?
        .ok_or("İptal edildi")?;

    let json = std::fs::read_to_string(&path).map_err(|e| format!("Dosya okunamadı: {}", e))?;
    let settings: LabelSettings =
        serde_json::from_str(&json).map_err(|e| format!("Geçersiz ayar dosyası: {}", e))?;
    Ok(settings)
}

#[tauri::command]
fn list_saved_settings() -> Vec<String> {
    settings::list_settings()
}

#[tauri::command]
fn get_default_settings() -> LabelSettings {
    LabelSettings::default()
}

#[tauri::command]
fn load_startup_settings() -> LabelSettings {
    settings::load_startup_settings().unwrap_or_default()
}

#[tauri::command]
fn save_startup_settings(settings_data: LabelSettings) -> Result<String, String> {
    settings::save_startup_settings(&settings_data)?;
    Ok(settings::startup_settings_path().to_string_lossy().into_owned())
}

#[tauri::command]
fn get_recent_files() -> Vec<String> {
    settings::load_recent_files()
}

#[tauri::command]
fn list_printers() -> Vec<String> {
    let mut command = Command::new("powershell");
    let output = hide_console_window(command.args([
        "-Command",
        "Get-Printer | Select-Object -ExpandProperty Name",
    ]))
    .output();
    match output {
        Ok(out) => String::from_utf8_lossy(&out.stdout)
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect(),
        Err(_) => vec![],
    }
}

#[tauri::command]
fn generate_pplb(labels: Vec<ParsedLabel>, settings_data: LabelSettings) -> Result<Vec<u8>, String> {
    Ok(pplb::build_raw_bytes(&labels, &settings_data))
}

pub(crate) fn chrono_date() -> String {
    let now = std::time::SystemTime::now();
    let duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs() as i64;
    let days = secs / 86400;
    // Simple date calculation
    let (y, m, d) = days_to_date(days + 719468);
    format!("{:02}.{:02}.{}", d, m, y)
}

fn days_to_date(days: i64) -> (i64, i64, i64) {
    let era = if days >= 0 { days } else { days - 146096 } / 146097;
    let doe = days - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

#[tauri::command]
async fn open_html_in_browser(html_content: String, sheet_name: String) -> Result<String, String> {
    let temp_dir = std::env::temp_dir();
    let html_file = temp_dir.join("etiketler.html");

    // Create Desktop/Etiket folder
    let desktop = dirs::desktop_dir().unwrap_or_else(|| {
        std::path::PathBuf::from(std::env::var("USERPROFILE").unwrap_or_default()).join("Desktop")
    });
    let etiket_dir = desktop.join("Etiket");
    std::fs::create_dir_all(&etiket_dir).map_err(|e| format!("Klasör oluşturulamadı: {}", e))?;

    // PDF filename: date-sheetname.pdf
    let date_str = chrono_date().replace('.', ".");
    let clean_sheet = sheet_name
        .replace(
            |c: char| !c.is_alphanumeric() && c != ' ' && c != '-' && c != '_',
            "",
        )
        .trim()
        .to_string();
    let sheet_part = if clean_sheet.is_empty() {
        "etiketler".to_string()
    } else {
        clean_sheet
    };
    let pdf_name = format!("{}-{}.pdf", date_str, sheet_part);
    let pdf_file = etiket_dir.join(&pdf_name);

    std::fs::write(&html_file, &html_content)
        .map_err(|e| format!("HTML dosyası yazılamadı: {}", e))?;

    // Try Edge first, then Chrome
    let browsers = [
        r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
        r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
        r"C:\Program Files\Google\Chrome\Application\chrome.exe",
        r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
    ];

    let browser = browsers
        .iter()
        .find(|b| std::path::Path::new(b).exists())
        .ok_or("Edge veya Chrome bulunamadı.")?;

    let html_path = html_file.to_string_lossy().to_string();
    let pdf_path = pdf_file.to_string_lossy().to_string();

    let output = Command::new(browser)
        .args([
            "--headless",
            "--disable-gpu",
            "--no-sandbox",
            &format!("--print-to-pdf={}", pdf_path),
            "--print-to-pdf-no-header",
            &format!("file:///{}", html_path.replace('\\', "/")),
        ])
        .output()
        .map_err(|e| format!("PDF oluşturulamadı: {}", e))?;

    if !pdf_file.exists() {
        return Err(format!(
            "PDF dosyası oluşturulamadı: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // Open the PDF
    Command::new("cmd")
        .args(["/C", "start", "", &pdf_path])
        .spawn()
        .map_err(|e| format!("PDF açılamadı: {}", e))?;

    Ok(pdf_path)
}

#[tauri::command]
fn send_to_printer(printer_name: String, pplb_data: Vec<u8>) -> Result<String, String> {
    #[cfg(windows)]
    {
        printer_win::send_raw_bytes(&printer_name, &pplb_data)?;
        return Ok("Yazdırma başarılı".to_string());
    }
    #[cfg(not(windows))]
    {
        let _ = (printer_name, pplb_data);
        return Err("Doğrudan yazdırma yalnızca Windows'ta desteklenir.".into());
    }
}

#[tauri::command]
fn get_startup_file() -> Option<String> {
    for arg in std::env::args().skip(1) {
        let lower = arg.to_lowercase();
        if lower.ends_with(".xlsx") || lower.ends_with(".xls") || lower.ends_with(".xlsm") {
            if std::path::Path::new(&arg).exists() {
                return Some(arg);
            }
        }
    }
    None
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            use tauri::Manager;
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
                
                // Gelen argümanlar arasında excel dosyası varsa frontend'e bildir
                for arg in args {
                    let lower = arg.to_lowercase();
                    if lower.ends_with(".xlsx") || lower.ends_with(".xls") || lower.ends_with(".xlsm") {
                        if std::path::Path::new(&arg).exists() {
                            let _ = window.emit("startup-file", arg);
                            break;
                        }
                    }
                }
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            use tauri::Manager;
            let quit_i =
                tauri::menu::MenuItem::with_id(app, "quit", "Tamamen Kapat", true, None::<&str>)?;
            let show_i = tauri::menu::MenuItem::with_id(app, "show", "Aç", true, None::<&str>)?;
            let menu = tauri::menu::Menu::with_items(app, &[&show_i, &quit_i])?;

            let _tray = tauri::tray::TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| {
                    if event.id() == "quit" {
                        app.exit(0);
                    } else if event.id() == "show" {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        button_state: tauri::tray::MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .manage(AppState {
            rows: std::sync::Mutex::new(Vec::new()),
            manual_labels: std::sync::Mutex::new(Vec::new()),
            current_file: std::sync::Mutex::new(String::new()),
            admin_session: std::sync::Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            get_license_status,
            admin_login,
            admin_logout,
            admin_get_info,
            admin_set_expiry,
            admin_change_credentials,
            open_file_dialog,
            get_sheets,
            load_excel,
            parse_satir,
            parse_all_labels,
            add_manual_label,
            update_manual_label,
            remove_manual_label,
            remove_excel_row,
            clear_all_data,
            save_label_settings,
            load_label_settings,
            save_settings_to_file,
            load_settings_from_file,
            list_saved_settings,
            get_default_settings,
            load_startup_settings,
            save_startup_settings,
            get_recent_files,
            list_printers,
            generate_pplb,
            send_to_printer,
            open_html_in_browser,
            get_startup_file,
        ])
        .run(tauri::generate_context!())
        .expect("Uygulama başlatılırken hata oluştu");
}
