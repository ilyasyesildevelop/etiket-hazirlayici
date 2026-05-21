//! Argox CP-2140 PPLB komut üretici (PDF/önizleme ile aynı mantık).

use crate::models::{LabelSettings, ParsedLabel};

const DOTS_PER_MM: f64 = 8.0; // 203 dpi ≈ 8 dots/mm
const DEFAULT_GAP_MM: f64 = 3.0;

/// PPLB ham komutlarını Windows-1254 baytlarına çevirir (Türkçe + Argox uyumu).
pub fn build_raw_bytes(labels: &[ParsedLabel], settings: &LabelSettings) -> Vec<u8> {
    let commands = build_commands(labels, settings);
    encode_windows_1254(&commands)
}

fn build_commands(labels: &[ParsedLabel], settings: &LabelSettings) -> String {
    if labels.is_empty() {
        return String::new();
    }

    let (label_w, label_h) = label_dots(settings);
    let gap = (DEFAULT_GAP_MM * DOTS_PER_MM) as i32;
    let margin = (settings.label_margin * DOTS_PER_MM) as i32;

    let mut out = String::new();
    // Boyut ayarı yalnızca bir kez — her etikette tekrarlanınca çift besleme / boş etiket oluşuyordu.
    out.push_str("N\n");
    out.push_str(&format!("q{}\n", label_w));
    out.push_str(&format!("Q{},{}\n", label_h, gap));
    out.push_str("S3\n");
    out.push_str("D8\n");
    out.push_str("R0,0\n");

    let copies = settings.copies.max(1) as usize;
    let mut first_page = true;
    for (idx, label) in labels.iter().enumerate() {
        for _ in 0..copies {
            if !first_page {
                out.push_str("N\n");
            }
            first_page = false;
            append_label(&mut out, label, idx, settings, label_w, label_h, margin);
            out.push_str("P1\n");
        }
    }

    out
}

fn label_dots(settings: &LabelSettings) -> (i32, i32) {
    let w = (settings.width_mm * DOTS_PER_MM) as i32;
    let h = (settings.height_mm * DOTS_PER_MM) as i32;
    (w, h)
}

fn append_label(
    out: &mut String,
    label: &ParsedLabel,
    idx: usize,
    settings: &LabelSettings,
    label_w: i32,
    label_h: i32,
    margin: i32,
) {
    let header_h = 28_i32;
    let body_top = margin + header_h;
    let body_bottom = label_h - margin;
    let body_h = (body_bottom - body_top).max(40);
    let inner_w = label_w - margin * 2;

    // --- Üst şerit (başlık, tarih, sayfa) — yatay, rotation 0 ---
    if settings.show_page_number {
        let page = format!("- {} -", idx + 1);
        let x = margin + inner_w / 6;
        pplb_text(out, x, margin + 4, 0, 2, &page);
    }
    let header_x = margin + inner_w / 2;
    pplb_text(out, header_x, margin + 4, 0, 2, &settings.header_text);
    if settings.show_date {
        let date = super::chrono_date();
        let x = margin + (inner_w * 5) / 6;
        pplb_text(out, x, margin + 4, 0, 2, &format!("- {} -", date));
    }

    // --- Gövde sütunları — dikey metin (rotation 1), PDF ile aynı ---
    let fw = &settings.field_widths;
    let fs = &settings.field_font_sizes;
    let adet_m2 = if label.metrekare.is_empty() {
        label.adet.clone()
    } else {
        label.metrekare.clone()
    };

    let fields: [(&str, f64, f64, bool); 7] = [
        (&label.cari_unvan, fw.cari_unvan, fs.cari_unvan, true),
        (&label.malz_aciklama, fw.malz_aciklama, fs.malz_aciklama, true),
        (&label.ebat, fw.ebat, fs.ebat, true),
        (&adet_m2, fw.adet_metrekare, fs.adet_metrekare, false),
        (&label.islem, fw.islem, fs.islem, true),
        (&label.musteri_adi, fw.musteri_adi, fs.musteri_adi, false),
        (
            &label.diger_aciklamalar,
            fw.diger_aciklamalar,
            fs.diger_aciklamalar,
            false,
        ),
    ];

    let total_pct: f64 = fields.iter().map(|(_, p, _, _)| p).sum();
    let mut x_pos = margin;

    for (text, pct, pt, _bold) in &fields {
        if text.trim().is_empty() {
            continue;
        }
        let col_w = ((pct / total_pct) * inner_w as f64) as i32;
        let center_x = x_pos + col_w / 2;
        let font = pt_to_pplb_font(*pt);
        // rotation 1 = 90° — sütun içi dikey yazı (önizleme/PDF ile uyumlu)
        let y_center = body_top + body_h / 2;
        pplb_text(out, center_x, y_center, 1, font, text);
        x_pos += col_w;
    }

    // Sıra/adet (sağ alt)
    let seq = format!("1/{}", label.print_count.max(1));
    let seq_font = pt_to_pplb_font(settings.sequence_font_size);
    pplb_text(
        out,
        label_w - margin - 8,
        label_h - margin - 8,
        0,
        seq_font,
        &seq,
    );
}

fn pt_to_pplb_font(pt: f64) -> i32 {
    // PPLB dahili font 1–5; pt boyutuna yaklaşık eşleme
    if pt >= 28.0 {
        5
    } else if pt >= 22.0 {
        4
    } else if pt >= 16.0 {
        3
    } else if pt >= 12.0 {
        2
    } else {
        1
    }
}

fn pplb_text(out: &mut String, x: i32, y: i32, rotation: i32, font: i32, text: &str) {
    let safe = escape_pplb_string(text);
    if safe.is_empty() {
        return;
    }
    out.push_str(&format!(
        "A{},{},{},{},1,1,N,\"{}\"\n",
        x.max(0),
        y.max(0),
        rotation,
        font,
        safe
    ));
}

/// Em-dash ve benzeri karakterler Argox'ta â€… olarak basılıyordu.
fn escape_pplb_string(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('—', "-")
        .replace('–', "-")
        .replace('…', "...")
        .replace('“', "\"")
        .replace('”', "\"")
        .replace('’', "'")
}

fn encode_windows_1254(text: &str) -> Vec<u8> {
    encoding_rs::WINDOWS_1254.encode(text).0.into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;

    fn sample_settings() -> LabelSettings {
        LabelSettings::default()
    }

    #[test]
    fn q_command_once_per_job() {
        let labels = vec![ParsedLabel {
            cari_unvan: "TEST".into(),
            malz_aciklama: String::new(),
            ebat: String::new(),
            islem: String::new(),
            adet: "1 ADET".into(),
            metrekare: String::new(),
            musteri_adi: String::new(),
            diger_aciklamalar: String::new(),
            bekleyen_siparis: String::new(),
            print_count: 1,
        }];
        let cmds = build_commands(&labels, &sample_settings());
        assert_eq!(cmds.matches("Q").count(), 1);
        assert_eq!(cmds.matches("\nq").count(), 1);
    }

    #[test]
    fn no_em_dash_in_output() {
        let mut s = LabelSettings::default();
        s.header_text = "İyi günlerde — test".into();
        let labels = vec![ParsedLabel {
            cari_unvan: "A".into(),
            malz_aciklama: String::new(),
            ebat: String::new(),
            islem: String::new(),
            adet: String::new(),
            metrekare: String::new(),
            musteri_adi: String::new(),
            diger_aciklamalar: String::new(),
            bekleyen_siparis: String::new(),
            print_count: 1,
        }];
        let cmds = build_commands(&labels, &s);
        assert!(!cmds.contains('—'));
        assert!(cmds.contains("- test"));
    }
}
