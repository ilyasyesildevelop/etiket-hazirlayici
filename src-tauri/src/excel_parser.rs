use crate::models::*;
use calamine::{open_workbook_auto, Data, Reader};

pub fn get_sheets(file_path: &str) -> Result<Vec<ExcelSheet>, String> {
    let mut workbook =
        open_workbook_auto(file_path).map_err(|e| format!("Excel açılamadı: {}", e))?;
    let sheet_names = workbook.sheet_names().to_vec();
    let mut result = Vec::new();
    for name in sheet_names {
        if let Ok(range) = workbook.worksheet_range(&name) {
            result.push(ExcelSheet {
                name: name.clone(),
                row_count: range.height(),
            });
        }
    }
    Ok(result)
}

pub fn parse_excel(
    file_path: &str,
    sheet_name: &str,
) -> Result<(Vec<RawRow>, ColumnMapping), String> {
    let mut workbook =
        open_workbook_auto(file_path).map_err(|e| format!("Excel açılamadı: {}", e))?;
    let range = workbook
        .worksheet_range(sheet_name)
        .map_err(|e| format!("Sayfa açılamadı: {}", e))?;

    let (header_row, mapping) = find_columns(&range)?;

    let mut rows = Vec::new();
    for (idx, row) in range.rows().enumerate() {
        if idx <= header_row {
            continue;
        }
        let cari = get_cell_value(row, mapping.cari_unvan_col);
        let malz = get_cell_value(row, mapping.malz_aciklama_col);
        let satir = get_cell_value(row, mapping.satir_aciklama_col);
        let bekleyen = get_cell_value(row, mapping.bekleyen_siparis_col);
        let siparis_tarihi = get_date_value(row, mapping.siparis_tarihi_col);
        let dokuman = get_cell_value(row, mapping.dokumanizleme_no_col);
        let sevkiyat = get_cell_value(row, mapping.sevkiyat_adi_col);
        let is_selected = is_cell_selected(row, mapping.secim_col);

        if cari.is_empty() && malz.is_empty() && satir.is_empty() {
            continue;
        }

        rows.push(RawRow {
            row_index: idx,
            cari_unvan: cari,
            malz_aciklama: malz,
            satir_aciklama: satir,
            bekleyen_siparis: bekleyen,
            siparis_tarihi,
            dokumanizleme_no: dokuman,
            sevkiyat_adi: sevkiyat,
            is_selected,
        });
    }

    Ok((rows, mapping))
}

fn find_columns(range: &calamine::Range<Data>) -> Result<(usize, ColumnMapping), String> {
    for (row_idx, row) in range.rows().enumerate().take(10) {
        let mut mapping = ColumnMapping {
            cari_unvan_col: None,
            malz_aciklama_col: None,
            satir_aciklama_col: None,
            bekleyen_siparis_col: None,
            siparis_tarihi_col: None,
            dokumanizleme_no_col: None,
            sevkiyat_adi_col: None,
            secim_col: None,
        };

        for (col_idx, cell) in row.iter().enumerate() {
            let raw_str = cell.to_string().trim().to_string();
            let val = normalize_header(&raw_str);

            if raw_str == "." || val == "SEC" || val == "SECIM" || raw_str == "✓" || raw_str == "✔" {
                mapping.secim_col = Some(col_idx);
            } else if val.contains("CARI") || val.contains("FIRMA") {
                mapping.cari_unvan_col = Some(col_idx);
            } else if val.contains("MALZ") || val.contains("URUN") {
                mapping.malz_aciklama_col = Some(col_idx);
            } else if val.contains("SATIR") || val.contains("ACIKLAMA") || val.contains("ACK") {
                mapping.satir_aciklama_col = Some(col_idx);
            } else if val.contains("SIPTARIH") || val.contains("SIPARISTARIH") {
                mapping.siparis_tarihi_col = Some(col_idx);
            } else if val.contains("SIPARIS") || val == "SIP" {
                mapping.bekleyen_siparis_col = Some(col_idx);
            } else if val.contains("MUSTERI") || val == "MS" {
                mapping.dokumanizleme_no_col = Some(col_idx);
            } else if val.contains("SEVK") || val.contains("ADRES") || val.contains("SUBE") {
                mapping.sevkiyat_adi_col = Some(col_idx);
            }
        }

        let found = [
            mapping.cari_unvan_col,
            mapping.malz_aciklama_col,
            mapping.satir_aciklama_col,
            mapping.bekleyen_siparis_col,
        ]
        .iter()
        .filter(|x| x.is_some())
        .count();

        if found >= 2 {
            return Ok((row_idx, mapping));
        }
    }

    Err("Gerekli sütunlar bulunamadı. Lütfen sütun isimlerini kontrol edin (Örn: CARI/FIRMA, MALZ/URUN, ACIKLAMA/SATIR/ACK, SIPARIS/SIP).".into())
}

fn normalize_header(s: &str) -> String {
    s.to_uppercase()
        .replace(' ', "")
        .replace('_', "")
        .replace('.', "")
        .replace('-', "")
        .replace('İ', "I")
        .replace('Ş', "S")
        .replace('Ç', "C")
        .replace('Ö', "O")
        .replace('Ü', "U")
        .replace('Ğ', "G")
}

fn get_cell_value(row: &[Data], col: Option<usize>) -> String {
    match col {
        Some(idx) if idx < row.len() => match &row[idx] {
            Data::Empty => String::new(),
            Data::Float(f) => {
                if *f == (*f as i64) as f64 {
                    format!("{}", *f as i64)
                } else {
                    format!("{:.2}", f).replace('.', ",")
                }
            }
            Data::Int(i) => format!("{}", i),
            other => other.to_string().trim().to_string(),
        },
        _ => String::new(),
    }
}

fn get_date_value(row: &[Data], col: Option<usize>) -> String {
    match col.and_then(|idx| row.get(idx)) {
        Some(Data::DateTime(value)) => {
            let serial = value.as_f64().floor() as i64;
            chrono::NaiveDate::from_ymd_opt(1899, 12, 30)
                .and_then(|date| date.and_hms_opt(0, 0, 0))
                .and_then(|date| date.checked_add_signed(chrono::Duration::days(serial)))
                .map(|date| date.format("%d.%m.%Y").to_string())
                .unwrap_or_default()
        }
        Some(Data::String(value)) => value.trim().to_string(),
        Some(Data::Int(value)) => value.to_string(),
        Some(Data::Float(value)) => format!("{}", value),
        _ => String::new(),
    }
}

fn is_cell_selected(row: &[Data], col: Option<usize>) -> bool {
    match col {
        Some(idx) if idx < row.len() => match &row[idx] {
            Data::Bool(b) => *b,
            Data::Int(i) => *i == 1,
            Data::Float(f) => (*f - 1.0).abs() < f64::EPSILON,
            Data::String(s) => {
                let lower = s.trim().to_lowercase();
                lower == "true"
                    || lower == "doğru"
                    || lower == "dogru"
                    || lower == "1"
                    || lower == "evet"
                    || lower == "x"
                    || lower == "v"
                    || lower == "✓"
                    || lower == "✔"
            }
            _ => false,
        },
        _ => true,
    }
}

