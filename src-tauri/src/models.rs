use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawRow {
    pub row_index: usize,
    pub cari_unvan: String,
    pub malz_aciklama: String,
    pub satir_aciklama: String,
    pub bekleyen_siparis: String,
    pub dokumanizleme_no: String,
    pub sevkiyat_adi: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedLabel {
    pub cari_unvan: String,
    pub malz_aciklama: String,
    pub ebat: String,
    pub islem: String,
    pub adet: String,
    pub metrekare: String,
    pub musteri_adi: String,
    pub diger_aciklamalar: String,
    pub bekleyen_siparis: String,
    pub print_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelSheet {
    pub name: String,
    pub row_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMapping {
    pub cari_unvan_col: Option<usize>,
    pub malz_aciklama_col: Option<usize>,
    pub satir_aciklama_col: Option<usize>,
    pub bekleyen_siparis_col: Option<usize>,
    pub dokumanizleme_no_col: Option<usize>,
    pub sevkiyat_adi_col: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelSettings {
    pub width_mm: f64,
    pub height_mm: f64,
    pub field_widths: FieldWidths,
    pub field_font_sizes: FieldFontSizes,
    pub global_font_family: String,
    pub global_color: String,
    pub satir_rules: SatirRules,
    pub printer_name: String,
    pub copies: u32,
    pub alignment: String,
    pub header_text: String,
    #[serde(default = "default_cari_max_chars")]
    pub cari_max_chars: usize,
    pub show_date: bool,
    pub show_page_number: bool,
    pub label_margin: f64,
    pub sequence_font_size: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldWidths {
    pub cari_unvan: f64,
    pub malz_aciklama: f64,
    pub ebat: f64,
    pub adet_metrekare: f64,
    pub islem: f64,
    pub musteri_adi: f64,
    pub diger_aciklamalar: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldFontSizes {
    pub cari_unvan: f64,
    pub malz_aciklama: f64,
    pub ebat: f64,
    pub adet_metrekare: f64,
    pub islem: f64,
    pub musteri_adi: f64,
    pub diger_aciklamalar: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SatirRules {
    pub split_char: String,
    pub move_long_text: bool,
    pub max_chars: usize,
}

fn default_cari_max_chars() -> usize {
    45
}

impl Default for LabelSettings {
    fn default() -> Self {
        Self {
            width_mm: 80.0,
            height_mm: 50.0,
            field_widths: FieldWidths {
                cari_unvan: 20.0,
                malz_aciklama: 35.0,
                ebat: 10.0,
                adet_metrekare: 6.0,
                islem: 10.0,
                musteri_adi: 18.0,
                diger_aciklamalar: 20.0,
            },
            field_font_sizes: FieldFontSizes {
                cari_unvan: 25.0,
                malz_aciklama: 32.0,
                ebat: 30.0,
                adet_metrekare: 16.0,
                islem: 24.0,
                musteri_adi: 25.0,
                diger_aciklamalar: 22.0,
            },
            global_font_family: "Tahoma".into(),
            global_color: "#000000".into(),
            alignment: "center".into(),
            satir_rules: SatirRules::default(),
            printer_name: String::new(),
            header_text: "".into(),
            cari_max_chars: 45,
            copies: 1,
            show_date: true,
            show_page_number: true,
            label_margin: 1.5,
            sequence_font_size: 7.0,
        }
    }
}

impl Default for FieldWidths {
    fn default() -> Self {
        Self {
            cari_unvan: 20.0,
            malz_aciklama: 35.0,
            ebat: 10.0,
            adet_metrekare: 6.0,
            islem: 10.0,
            musteri_adi: 18.0,
            diger_aciklamalar: 20.0,
        }
    }
}

impl Default for FieldFontSizes {
    fn default() -> Self {
        Self {
            cari_unvan: 25.0,
            malz_aciklama: 32.0,
            ebat: 30.0,
            adet_metrekare: 16.0,
            islem: 24.0,
            musteri_adi: 25.0,
            diger_aciklamalar: 22.0,
        }
    }
}

impl Default for SatirRules {
    fn default() -> Self {
        Self {
            split_char: "/".into(),
            move_long_text: true,
            max_chars: 100,
        }
    }
}

