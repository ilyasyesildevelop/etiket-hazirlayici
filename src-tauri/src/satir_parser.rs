use crate::models::SatirRules;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedSatir {
    pub ebat: String,
    pub adet: String,
    pub metrekare: String,
    pub islem: String,
    pub musteri_adi: String,
    pub diger_aciklamalar: String,
    pub is_enrulo: bool,
    pub print_count: usize,
}

const ISLEM_KEYWORDS: &[&str] = &[
    "SPOR SAÇAKLI",
    "SPOR SAÇAK",
    "SAÇAKLI",
    "SAÇAK",
    "SACAKLI",
    "SACAK",
    "KATLAMALI",
    "KATLAMA",
    "İŞLEMSİZ",
    "ISLEMSIZ",
    "OVERLOKLU",
    "OVERLOK",
    "OVERLOOK",
    "KESİM",
    "KESIM",
    "DİKİM",
    "DIKIM",
    "YUVARLAK",
    "SERPİŞTİRME",
    "SERPISTIRME",
    "BÜZGÜ",
    "BUZGU",
    "PİLİSE",
    "PILISE",
    "SÜRFLE",
    "SURFLE",
    "KAŞTIRMALI",
    "KASTIRMALI",
    "BORDÜRLÜ",
    "BORDURLU",
    "BORDÖR",
    "BORDÜR",
    "BORDUR",
    "KARE",
];

fn standardize_islem(raw: &str) -> String {
    let upper = raw.to_uppercase();
    if upper.contains("SPOR SAÇAK") || upper.contains("SPOR SACAK") { return "SPOR SAÇAK".to_string(); }
    if upper.contains("SAÇAK") || upper.contains("SACAK") { return "SAÇAK".to_string(); }
    if upper.contains("OVERLOK") || upper.contains("OVERLOOK") { return "OVERLOK".to_string(); }
    if upper.contains("KATLAMA") { return "KATLAMA".to_string(); }
    if upper.contains("BORDÜR") || upper.contains("BORDUR") || upper.contains("BORDÖR") { return "BORDÜR".to_string(); }
    if upper.contains("KESİM") || upper.contains("KESIM") { return "KESİM".to_string(); }
    if upper.contains("DİKİM") || upper.contains("DIKIM") { return "DİKİM".to_string(); }
    if upper.contains("YUVARLAK") { return "YUVARLAK".to_string(); }
    if upper.contains("SERPİŞTİRME") || upper.contains("SERPISTIRME") { return "SERPİŞTİRME".to_string(); }
    if upper.contains("BÜZGÜ") || upper.contains("BUZGU") { return "BÜZGÜ".to_string(); }
    if upper.contains("PİLİSE") || upper.contains("PILISE") { return "PİLİSE".to_string(); }
    if upper.contains("SÜRFLE") || upper.contains("SURFLE") { return "SÜRFLE".to_string(); }
    if upper.contains("KAŞTIRMALI") || upper.contains("KASTIRMALI") { return "KAŞTIRMALI".to_string(); }
    if upper.contains("KARE") { return "KARE".to_string(); }
    if upper.contains("İŞLEMSİZ") || upper.contains("ISLEMSIZ") { return "İŞLEMSİZ".to_string(); }
    upper
}

fn build_islem_keyword_list(extra: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut list = Vec::new();
    let mut push = |raw: &str| {
        let t = raw.trim();
        if t.is_empty() || t.eq_ignore_ascii_case("_init") {
            return;
        }
        let key = t.to_uppercase();
        if seen.insert(key) {
            list.push(t.to_string());
        }
    };
    for &k in ISLEM_KEYWORDS {
        push(k);
    }
    for e in extra {
        push(e);
    }
    list.sort_by(|a, b| b.len().cmp(&a.len()));
    list
}

fn match_islem_keywords(remaining: &mut String, text: &str, keywords: &[String]) -> String {
    for keyword in keywords {
        let Ok(re) = Regex::new(&tr_regex(keyword)) else {
            continue;
        };
        if re.is_match(text) {
            *remaining = re.replace(remaining, " ").to_string();
            return standardize_islem(keyword);
        }
    }
    String::new()
}

pub fn parse_satir_aciklama(
    satir: &str,
    malz: &str,
    bekleyen: &str,
    dokumanizleme: &str,
    cari: &str,
    rules: &SatirRules,
    extra_islem_keywords: &[String],
) -> ParsedSatir {
    let satir = satir.trim();
    let malz = malz.trim();
    let is_enrulo = malz.to_uppercase().contains("ENRULO");
    let mut remaining = satir.to_string();

    // 1. Extract EBAT
    let ebat = if is_enrulo {
        // ENRULO: dimensions from SATIR_ACIKLAMA
        extract_ebat_from_text(&mut remaining)
    } else {
        // Standard: dimensions from MALZ_ACIKLAMA
        let mut malz_copy = malz.to_string();
        let e = extract_ebat_from_text(&mut malz_copy);
        if e.is_empty() {
            // Fallback: try SATIR
            extract_ebat_from_text(&mut remaining)
        } else {
            e
        }
    };

    // 2. Extract ADET
    let adet = extract_adet(&mut remaining);

    // 3. Determine adet vs m² from BEKLEYEN_SIPARIS
    // Satır açıklamasından çıkarılan "X ADET" bilgisindeki X'i bulalım
    let extracted_count = if let Some(cap) = regex::Regex::new(r"(\d+)").unwrap().captures(&adet) {
        cap[1].parse::<usize>().unwrap_or(1)
    } else {
        1
    };

    let (adet_display, metrekare, print_count) = if is_enrulo {
        // ENRULO: BEKLEYEN = m², adet from SATIR
        let m2 = format_metrekare(bekleyen);
        // ENRULO için Satır açıklamasında bulunan adet kadar etiket çıkar
        let p_count = if extracted_count > 0 { extracted_count } else { 1 };
        (adet, m2, p_count)
    } else {
        // Standard: BEKLEYEN = adet count
        let bek = bekleyen.trim().replace(',', ".");
        let mut p_count = 1;
        let mut bek_adet = adet.clone();
        
        if let Ok(val) = bek.parse::<f64>() {
            if val > 0.0 {
                p_count = val as usize;
                bek_adet = "1 ADET".to_string();
            }
        } else if extracted_count > 0 {
            // BEKLEYEN sütununda geçerli bir değer yoksa (boşsa vs.) satırdaki adeti kullan
            p_count = extracted_count;
            if p_count > 0 {
                bek_adet = "1 ADET".to_string();
            }
        }
        
        (bek_adet, String::new(), p_count)
    };

    // 4. Extract İŞLEM (sabit anahtar kelimeler + Firebase IslemList)
    let islem_keywords = build_islem_keyword_list(extra_islem_keywords);
    let islem = extract_islem(&mut remaining, &islem_keywords);

    // 5. Extract MÜŞTERİ ADI
    let musteri_adi = extract_musteri(&mut remaining, dokumanizleme, cari);

    // 6. Clean remaining
    let remaining = remaining
        .replace('\r', " ")
        .replace('\n', " ")
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
        .trim_matches(|c: char| "/ ,:-*()".contains(c) || c.is_whitespace())
        .to_string();

    let diger_aciklamalar = if rules.move_long_text && remaining.len() > rules.max_chars {
        let pos = remaining[..rules.max_chars]
            .rfind(' ')
            .unwrap_or(rules.max_chars);
        remaining[..pos].trim().to_string()
    } else {
        remaining
    };

    ParsedSatir {
        ebat,
        adet: adet_display,
        metrekare,
        islem,
        musteri_adi,
        diger_aciklamalar,
        is_enrulo,
        print_count,
    }
}

fn extract_ebat_from_text(text: &mut String) -> String {
    static RECT_RE: OnceLock<Regex> = OnceLock::new();
    static Q_SUFFIX_RE: OnceLock<Regex> = OnceLock::new();
    static Q_PREFIX_RE: OnceLock<Regex> = OnceLock::new();

    let rect_re = RECT_RE.get_or_init(|| Regex::new(r"(\d+)\s*[xX×\*]\s*(\d+)").unwrap());
    let q_suffix_re = Q_SUFFIX_RE.get_or_init(|| Regex::new(r"(?i)\b(\d{2,})\s*([QS])\b").unwrap());
    let q_prefix_re = Q_PREFIX_RE.get_or_init(|| Regex::new(r"(?i)\b([QS])\s*(\d{2,})\b").unwrap());

    if let Some(cap) = rect_re.captures(&text.clone()) {
        let matched = cap.get(0).unwrap().as_str().to_string();
        *text = text.replace(&matched, " ");
        return format!("{}*{}", &cap[1], &cap[2]);
    }
    if let Some(cap) = q_suffix_re.captures(&text.clone()) {
        let matched = cap.get(0).unwrap().as_str().to_string();
        *text = text.replace(&matched, " ");
        return format!("{}{}", cap[2].to_uppercase(), &cap[1]);
    }
    if let Some(cap) = q_prefix_re.captures(&text.clone()) {
        let matched = cap.get(0).unwrap().as_str().to_string();
        *text = text.replace(&matched, " ");
        return format!("{}{}", cap[1].to_uppercase(), &cap[2]);
    }
    String::new()
}

fn extract_adet(remaining: &mut String) -> String {
    static ADET_RE: OnceLock<Regex> = OnceLock::new();
    static COLON_RE: OnceLock<Regex> = OnceLock::new();

    let adet_re = ADET_RE.get_or_init(|| Regex::new(r"(?i)(\d+)\s*adet").unwrap());
    let colon_re = COLON_RE.get_or_init(|| Regex::new(r":\s*(\d+)\s*(?i:adet)?").unwrap());

    let clone = remaining.clone();
    if let Some(cap) = adet_re.captures(&clone) {
        let matched = cap.get(0).unwrap().as_str().to_string();
        let num = cap[1].to_string();
        *remaining = remaining.replace(&matched, " ");
        return format!("{} ADET", num);
    }
    let clone = remaining.clone();
    if let Some(cap) = colon_re.captures(&clone) {
        let matched = cap.get(0).unwrap().as_str().to_string();
        let num = cap[1].to_string();
        *remaining = remaining.replace(&matched, " ");
        return format!("{} ADET", num);
    }
    String::new()
}

fn tr_regex(word: &str) -> String {
    let mut pattern = String::new();
    for c in word.chars() {
        match c {
            'I' | 'ı' | 'İ' | 'i' => pattern.push_str("[Iıİi]"),
            'C' | 'c' | 'Ç' | 'ç' => pattern.push_str("[CcÇç]"),
            'S' | 's' | 'Ş' | 'ş' => pattern.push_str("[SsŞş]"),
            'G' | 'g' | 'Ğ' | 'ğ' => pattern.push_str("[GgĞğ]"),
            'U' | 'u' | 'Ü' | 'ü' => pattern.push_str("[UuÜü]"),
            'O' | 'o' | 'Ö' | 'ö' => pattern.push_str("[OoÖö]"),
            _ => pattern.push_str(&regex::escape(&c.to_string())),
        }
    }
    format!(r"(?i)\b{}\b", pattern)
}

fn extract_islem(remaining: &mut String, islem_keywords: &[String]) -> String {
    let mut clone = remaining.clone();
    
    // 1. Önce "OVAL" kelimesini arayalım
    static OVAL_RE: OnceLock<Regex> = OnceLock::new();
    let oval_re = OVAL_RE.get_or_init(|| Regex::new(r"(?i)\bOVAL\b").unwrap());
    
    let mut has_oval = false;
    if let Some(mat) = oval_re.find(&clone) {
        has_oval = true;
        let matched = mat.as_str().to_string();
        clone = clone.replace(&matched, " ").to_string();
        *remaining = remaining.replace(&matched, " ").to_string();
    }
    
    // 2. Çift bordür (İç/Dış veya Kalın/İnce) kontrolü
    static IC_BRD_RE: OnceLock<Regex> = OnceLock::new();
    static DIS_BRD_RE: OnceLock<Regex> = OnceLock::new();
    static CIFT_KELIME_RE: OnceLock<Regex> = OnceLock::new();
    
    let ic_re = IC_BRD_RE.get_or_init(||
        Regex::new(r"(?i)(?:^|\s)(?:[iıİI][çcÇC]|[iıİI]nc[eE])\s*[-:.]?\s*(?:b[oöOÖ]rd[üuÜU]r(?:l[üuÜU])?|bodrur|brd)?\s*(\d+)").unwrap()
    );
    let dis_re = DIS_BRD_RE.get_or_init(||
        Regex::new(r"(?i)(?:^|\s)(?:d[iıİI][şsŞS]|k[aA]l[ıiIİ]n)\s*[-:.]?\s*(?:b[oöOÖ]rd[üuÜU]r(?:l[üuÜU])?|bodrur|brd)?\s*(\d+)").unwrap()
    );
    let cift_kelime = CIFT_KELIME_RE.get_or_init(|| 
        Regex::new(r"(?i)(?:^|\s)[çcÇC][iıİI]ft\s+(?:b[oöOÖ]rd[üuÜU]r(?:l[üuÜUiıİI])?|bodrur|brd)").unwrap()
    );

    let ic_cap = ic_re.captures(&clone);
    let dis_cap = dis_re.captures(&clone);
    
    if ic_cap.is_some() || dis_cap.is_some() {
        let ic_num = ic_cap.map(|c| c[1].to_string());
        let dis_num = dis_cap.map(|c| c[1].to_string());
        
        // Cümle içindeki tüm bu kalıpları silelim
        *remaining = cift_kelime.replace(remaining, " ").to_string();
        *remaining = ic_re.replace(remaining, " ").to_string();
        *remaining = dis_re.replace(remaining, " ").to_string();
        
        let islem = match (ic_num, dis_num) {
            (Some(ic), Some(dis)) => format!("ÇİFT BRD\nİÇ {} - DIŞ {}", ic, dis),
            (Some(ic), None) => format!("ÇİFT BRD\nİÇ {}", ic),
            (None, Some(dis)) => format!("ÇİFT BRD\nDIŞ {}", dis),
            (None, None) => String::new(),
        };
        
        if !islem.is_empty() {
            return if has_oval { format!("OVAL - {}", islem) } else { islem };
        }
    }
    
    // 3. Banko ve Henna Nubuk kontrolü
    let mut islem = String::new();
    
    static HENNA_RE: OnceLock<Regex> = OnceLock::new();
    static BANKO_RE: OnceLock<Regex> = OnceLock::new();
    
    let henna_re = HENNA_RE.get_or_init(|| Regex::new(r"(?i)(?:brd\s+)?h[eE]nn[aAeE]\s+nubuk\s*(\d+)").unwrap());
    let banko_re = BANKO_RE.get_or_init(|| Regex::new(r"(?i)banko\s*(\d+)").unwrap());
    
    if let Some(cap) = henna_re.captures(&clone) {
        let matched = cap.get(0).unwrap().as_str().to_string();
        *remaining = remaining.replace(&matched, " ");
        islem = format!("HENNA NUBUK {}", &cap[1]);
    } else if let Some(cap) = banko_re.captures(&clone) {
        let matched = cap.get(0).unwrap().as_str().to_string();
        *remaining = remaining.replace(&matched, " ");
        islem = format!("BANKO {}", &cap[1]);
    } else {
        // 4. Tek BRD kontrolü
        static BRD_RE: OnceLock<Regex> = OnceLock::new();
        static BRD_REV_RE: OnceLock<Regex> = OnceLock::new();
        // BRD 41153, BORDÜR 41153, BORDÜRLÜ 41153 vs.
        let brd_re = BRD_RE.get_or_init(|| Regex::new(r"(?i)\b(?:BRD|BORD[ÜUuüÖö]R(?:L[ÜUuü])?)\s*(\d+)\b").unwrap());
        // 41153 BRD, 41153 BORDÜR vs.
        let brd_rev_re = BRD_REV_RE.get_or_init(|| Regex::new(r"(?i)\b(\d+)\s*(?:BRD|BORD[ÜUuüÖö]R(?:L[ÜUuü])?)\b").unwrap());

        if let Some(cap) = brd_re.captures(&clone) {
            let matched = cap.get(0).unwrap().as_str().to_string();
            *remaining = remaining.replace(&matched, " ");
            islem = format!("BRD {}", &cap[1]);
        } else if let Some(cap) = brd_rev_re.captures(&clone) {
            let matched = cap.get(0).unwrap().as_str().to_string();
            *remaining = remaining.replace(&matched, " ");
            islem = format!("BRD {}", &cap[1]);
        } else {
            // 5. Genel keyword kontrolü (sabit + Firebase listesi, uzun eşleşme önce)
            islem = match_islem_keywords(remaining, &clone, islem_keywords);
        }
    }
    
    if has_oval {
        if islem.is_empty() {
            "OVAL".to_string()
        } else {
            format!("OVAL - {}", islem)
        }
    } else {
        islem
    }
}

/// MŞ / MS / Müşteri etiketinden isim çıkarır (satırın herhangi bir yerinde).
fn find_musteri_in_text(text: &str) -> Option<(String, String)> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    static MARKER_RE: OnceLock<Regex> = OnceLock::new();
    let marker_re = MARKER_RE.get_or_init(|| Regex::new(r"(?i)[*]?\s*(?:müşteri|musteri|m[sş][tş]?)\s*[:.\s-]*").unwrap());
    
    let marker = marker_re.find(text)?;
    let (name, consumed) = take_customer_name_segment(&text[marker.end()..]);
    
    let matched = text[marker.start()..marker.end() + consumed].to_string();
    Some((matched, name))
}

fn take_customer_name_segment(rest: &str) -> (String, usize) {
    let rest_trimmed = rest.trim_start();
    if rest_trimmed.is_empty() {
        return (String::new(), rest.len() - rest_trimmed.len());
    }
    
    static STOP_RE: OnceLock<Vec<Regex>> = OnceLock::new();
    let stop_re = STOP_RE.get_or_init(|| {
        let stop_patterns = [
            r"\d{4,}",
            r"(?i)\d+\s*[xX×*]\s*\d+",
            r"(?i)\d+\s*adet\b",
            r"(?i)\s+(?:OVERLOK|KES[Iıİi]M|SA[CcÇç]AK|SPOR|ADET|[Iıİi][SsŞş]LEM)\b",
            r"/",
            r"\|",
            r";",
        ];
        stop_patterns.iter().filter_map(|pat| Regex::new(pat).ok()).collect()
    });

    let mut end = rest_trimmed.len();
    for re in stop_re {
        if let Some(m) = re.find(rest_trimmed) {
            end = end.min(m.start());
        }
    }
    let raw = rest_trimmed[..end].trim();
    let name = clean_customer_name(raw);
    let consumed = (rest.len() - rest_trimmed.len()) + end;
    
    (name, consumed)
}

fn clean_customer_name(name: &str) -> String {
    // 1. Telefon numaralarını temizle (Örn: 0554..., 0 554..., +90 554...)
    static PHONE_RE: OnceLock<Regex> = OnceLock::new();
    let phone_re = PHONE_RE.get_or_init(|| Regex::new(r"(?i)\s*(?:\+?90)?\s*[0o]?[\s-]*\d{3,}[\s\d-]*$").unwrap());
    let cleaned = phone_re.replace(name.trim(), "").to_string();
    
    // 2. Geriye kalan tek/çift haneli rakamları da sil (ör: sondaki "0", "05" vb.)
    static TRAILING_DIGIT_RE: OnceLock<Regex> = OnceLock::new();
    let trailing_re = TRAILING_DIGIT_RE.get_or_init(|| Regex::new(r"\s+\d{1,2}\s*$").unwrap());
    let cleaned = trailing_re.replace(&cleaned, "").to_string();
    
    cleaned
        .trim_matches(|c: char| "/ ,:-*()".contains(c) || c.is_whitespace())
        .trim()
        .to_string()
}

fn format_musteri_label(name: &str) -> String {
    format!("MŞ: {}", clean_customer_name(name).to_uppercase())
}

fn extract_musteri(remaining: &mut String, dokumanizleme: &str, cari_unvan: &str) -> String {
    // 1. Önce Doküman İzleme'de "MŞ:" vs. gibi bir işaretçi ile arıyoruz
    if let Some((_matched, name)) = find_musteri_in_text(dokumanizleme) {
        if !name.is_empty() {
            return format_musteri_label(&name);
        }
    }

    // 2. Eğer Doküman İzleme'de MŞ yoksa bile doğrudan müşteri adını çıkarmayı dene
    let dok = dokumanizleme.trim();
    if !dok.is_empty() {
        // Eğer ebat ile başlıyorsa bu bir satır açıklamasıdır, isim arama.
        let starts_with_ebat = Regex::new(r"^\d+\s*[xX×*]\s*\d+").unwrap().is_match(dok);
        if !starts_with_ebat {
            let (name, _) = take_customer_name_segment(dok);
            if name.len() >= 3 {
                // İşlem kelimesi veya Cari Ünvan'ın kendisi müşteri ismi sayılmasın
                let is_op = ISLEM_KEYWORDS.iter().any(|&k| name.to_uppercase() == k.to_uppercase());
                
                // Cari ünvan benzerlik kontrolü ("DOKUYAN HALI" cari ünvan ise, dokümanda "DOKUYAN HALI" yazıyorsa alma)
                let cari_upper = cari_unvan.to_uppercase();
                let name_upper = name.to_uppercase();
                let is_cari = cari_upper.contains(&name_upper) || name_upper.contains(&cari_upper);
                
                if !is_op && !is_cari {
                    return format_musteri_label(&name);
                }
            }
        }
    }

    // 3. Bulunamazsa Satır Açıklamasına bakarız (Varsa MŞ marker'ı ile sileriz)
    if let Some((matched, name)) = find_musteri_in_text(remaining) {
        *remaining = remaining.replace(&matched, " ");
        if !name.is_empty() {
            return format_musteri_label(&name);
        }
    }

    // 4. Nadir durum: PDF GELECEK formatı (Satır açıklamasında kalmış olabilir)
    let pdf_re = Regex::new(r"(?i)\(?\s*PDF\s+GELECEK\s*[-:]\s*([^)]+)\)?").unwrap();
    let clone = remaining.clone();
    if let Some(cap) = pdf_re.captures(&clone) {
        let matched = cap.get(0).unwrap().as_str().to_string();
        let name = clean_customer_name(&cap[1]);
        *remaining = remaining.replace(&matched, " ");
        if name.len() >= 2 {
            return format_musteri_label(&name);
        }
    }

    String::new()
}

fn format_metrekare(bekleyen: &str) -> String {
    let bek = bekleyen.trim().replace(',', ".");
    if let Ok(val) = bek.parse::<f64>() {
        if val > 0.0 {
            let s = format!("{:.2}", val).replace('.', ",");
            let s = s.trim_end_matches('0').trim_end_matches(',');
            return format!("{} m²", s);
        }
    }
    String::new()
}

static CARI_REPLACEMENTS: OnceLock<Vec<(Regex, &'static str)>> = OnceLock::new();
static UNVAN_TOKEN_RE: OnceLock<Regex> = OnceLock::new();

fn is_unvan_token(word: &str) -> bool {
    let w = word.trim().trim_end_matches('.');
    if w.is_empty() {
        return false;
    }
    if w.eq_ignore_ascii_case("VE") {
        return true;
    }
    UNVAN_TOKEN_RE
        .get_or_init(|| {
            Regex::new(
                r"(?i)^(İNŞ|INŞ|INSAAT|INŞAAT|TUR|TURIZM|TURİZM|SAN|SANAYI|SANAYİ|TİC|TIC|TICARET|TİCARET|LTD|LIMITED|LİMİTED|ŞTİ|STI|SIRKETI|ŞİRKETİ|A\.Ş|A\.S|TAAH|MOB|MOBILYA|MOBİLYA|TEKS|TEKSTIL|TEKSTİL|OTO|OTOMOTIV|OTOMOTİV|LOJ|LOJISTIK|LOJİSTİK|MÜH|ITH|İTH|IHR|İHR|HİZ|DAY|TÜK|ORG|PAZ|ORT|GER|ÜRÜN|MEF|MALL|TAŞ|DANIŞM|ZEM)$",
            )
            .unwrap()
        })
        .is_match(w)
}

/// Ünvan parçalarını nokta ile birleştirir: İNŞ.TUR.SAN.VE TİC.LTD.ŞTİ
fn format_unvan_groups(legal: &[&str]) -> String {
    if legal.is_empty() {
        return String::new();
    }
    let mut groups: Vec<Vec<String>> = vec![];
    let mut current: Vec<String> = vec![];

    for (i, &word) in legal.iter().enumerate() {
        let piece = word.trim().trim_end_matches('.').to_string();
        let is_ve = piece.eq_ignore_ascii_case("VE");
        current.push(if is_ve {
            "VE".to_string()
        } else {
            piece
        });

        if is_ve {
            let next_is_tic = legal.get(i + 1).map(|w| {
                let t = w.trim().trim_end_matches('.').to_uppercase();
                t.starts_with("TİC") || t.starts_with("TIC")
            }).unwrap_or(false);
            if next_is_tic {
                groups.push(current);
                current = vec![];
            }
        }
    }
    if !current.is_empty() {
        groups.push(current);
    }

    groups
        .iter()
        .map(|g| g.join("."))
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_cari_unvan(s: &str) -> String {
    let words: Vec<&str> = s.split_whitespace().collect();
    if words.is_empty() {
        return String::new();
    }

    let Some(start) = words.iter().position(|w| is_unvan_token(w)) else {
        return s.to_string();
    };

    let company = words[..start].join(" ");
    let legal = format_unvan_groups(&words[start..]);

    if company.is_empty() {
        legal
    } else if legal.is_empty() {
        company
    } else {
        format!("{} {}", company, legal)
    }
}

pub fn abbreviate_cari(cari: &str) -> String {
    let replacements = CARI_REPLACEMENTS.get_or_init(|| {
        vec![
            (Regex::new(r"(?i)DUBA[İI]\s+HALIBANK\s+HALI\s+MOB(?:\.|İLYA)?\s+EV\s+TEKST[İI]L\s+Z[ÜU]CCAC[İI]YE\s+HED[İI]YEL[İI]K\s+E[ŞS]YA\s+OTOMOT[İI]V\s+[İI]N[ŞS]\.?\s+TAAH\.?\s+GIDA\s+NALBUR[İI]YE\s+SAN\.?\s+VE\s+T[İI]C\.?\s+LTD\.?\s+[ŞS]T[İI]\.?").unwrap(), "DUBAİ HALIBANK HALI MOB."),
            (Regex::new(r"(?i)\bDAYANIKLI\b").unwrap(), "DAY."),
            (Regex::new(r"(?i)\bT[ÜU]KET[İI]M\b").unwrap(), "TÜK."),
            (Regex::new(r"(?i)\bMALLARI\b").unwrap(), "MALL."),
            (Regex::new(r"(?i)\bMEFRU[ŞS]AT\b").unwrap(), "MEF."),
            (Regex::new(r"(?i)\bTA[ŞS]IMACILIK\b").unwrap(), "TAŞ."),
            (Regex::new(r"(?i)\b[İI]N[ŞS]AAT\b").unwrap(), "İNŞ"),
            (Regex::new(r"(?i)\bTAAHH[ÜU]T\b").unwrap(), "TAAH"),
            (Regex::new(r"(?i)\bSANAY[İI]\b").unwrap(), "SAN"),
            (Regex::new(r"(?i)\bT[İI]CARET\b").unwrap(), "TİC"),
            (Regex::new(r"(?i)\bL[İI]M[İI]TED\b").unwrap(), "LTD"),
            (Regex::new(r"(?i)\bANON[İI]M\s+[ŞS][İI]RKET[İI]\b").unwrap(), "A.Ş"),
            (Regex::new(r"(?i)\b[ŞS][İI]RKET[İI]\b").unwrap(), "ŞTİ"),
            (Regex::new(r"(?i)\bDANI[ŞS]MANLIK\b").unwrap(), "DANIŞM"),
            (Regex::new(r"(?i)\bORGAN[İI]ZASYON\b").unwrap(), "ORG"),
            (Regex::new(r"(?i)\bPAZARLAMA\b").unwrap(), "PAZ"),
            (Regex::new(r"(?i)\bORTAKLI[ĞG]I\b").unwrap(), "ORT"),
            (Regex::new(r"(?i)\bMOB[İI]LYA\b").unwrap(), "MOB"),
            (Regex::new(r"(?i)\bZEM[İI]N\s+KAPLAMALARI\b").unwrap(), "ZEM KAPL"),
            (Regex::new(r"(?i)\bENRULO\s+N\b").unwrap(), "EN"),
            (Regex::new(r"(?i)\bENRULO\b").unwrap(), "EN"),
            (Regex::new(r"(?i)\b[İI]THALAT\b").unwrap(), "İTH"),
            (Regex::new(r"(?i)\b[İI]HRACAT\b").unwrap(), "İHR"),
            (Regex::new(r"(?i)\bH[İI]ZMETLER[İI]\b").unwrap(), "HİZ"),
            (Regex::new(r"(?i)\bTEKST[İI]L\b").unwrap(), "TEKS"),
            (Regex::new(r"(?i)\bTUR[İI]ZM\b").unwrap(), "TUR"),
            (Regex::new(r"(?i)\bOTOMOT[İI]V\b").unwrap(), "OTO"),
            (Regex::new(r"(?i)\bLOJ[İI]ST[İI]K\b").unwrap(), "LOJ"),
            (Regex::new(r"(?i)\bM[ÜU]HEND[İI]SL[İI]K\b").unwrap(), "MÜH"),
            (Regex::new(r"(?i)\b[ÜU]R[ÜU]NLER[İI]\b").unwrap(), "ÜRÜN"),
            (Regex::new(r"(?i)\bGERE[ÇC]LER[İI]\b").unwrap(), "GER"),
        ]
    });

    // Nokta = boşluk (kelime ayırıcı)
    let mut s = cari.replace('.', " ").replace(',', " ");
    for (re, repl) in replacements {
        s = re.replace_all(&s, *repl).to_string();
    }

    static SPACE_RE: OnceLock<Regex> = OnceLock::new();
    let space_re = SPACE_RE.get_or_init(|| Regex::new(r"\s+").unwrap());
    let normalized = space_re.replace_all(&s, " ").trim().to_string();

    format_cari_unvan(&normalized)
}

pub fn truncate_cari(cari: &str, max_chars: usize) -> String {
    const SUFFIX: &str = "...";
    let abbr = abbreviate_cari(cari);
    if max_chars == 0 {
        return abbr;
    }
    let char_count = abbr.chars().count();
    if char_count <= max_chars {
        return abbr;
    }
    let suffix_len = SUFFIX.chars().count();
    let keep = max_chars.saturating_sub(suffix_len);
    if keep == 0 {
        return SUFFIX.to_string();
    }
    format!("{}{}", abbr.chars().take(keep).collect::<String>(), SUFFIX)
}

#[cfg(test)]
mod cari_tests {
    use super::*;

    #[test]
    fn truncate_cari_adds_ellipsis() {
        let long = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789EXTRA";
        let out = truncate_cari(long, 45);
        assert!(out.ends_with("..."));
        assert_eq!(out.chars().count(), 45);
    }

    #[test]
    fn dot_is_word_separator() {
        let out = abbreviate_cari("HALI MOB. İNŞ.TUR.SAN.VE TİC.LTD.ŞTİ");
        assert!(out.contains("İNŞ.TUR") || out.contains("INŞ.TUR"));
    }

    #[test]
    fn full_unvan_abbreviation() {
        let out = abbreviate_cari(
            "HALICIOĞULLARI HALI MOBİLYA İNŞAAT TURİZM SANAYİ VE TİCARET LİMİTED ŞİRKETİ",
        );
        assert!(out.contains("HALICIOĞULLARI") || out.contains("HALICIO"));
        assert!(out.contains("MOB"));
        assert!(out.contains("İNŞ.TUR") || out.contains("INŞ.TUR"));
        assert!(out.contains("TİC.LTD") || out.contains("TIC.LTD"));
    }
}

#[cfg(test)]
mod musteri_tests {
    use super::*;

    fn parse_musteri(satir: &str, dok: &str) -> String {
        let mut rem = satir.to_string();
        extract_musteri(&mut rem, dok, "")
    }

    #[test]
    fn ms_formats_variants() {
        assert_eq!(
            parse_musteri("150*200 5 ADET MŞ Mehmet Kaygusuz", ""),
            "MŞ: MEHMET KAYGUSUZ"
        );
        assert_eq!(
            parse_musteri("OVERLOK MŞ: Mehmet Kaygusuz", ""),
            "MŞ: MEHMET KAYGUSUZ"
        );
        assert_eq!(
            parse_musteri("MŞ:Mehmet Kaygusuz 150*200", ""),
            "MŞ: MEHMET KAYGUSUZ"
        );
        assert_eq!(parse_musteri("MŞ.Mehmet Kaygusuz", ""), "MŞ: MEHMET KAYGUSUZ");
        assert_eq!(parse_musteri("mş:mehmet kaygusuz", ""), "MŞ: MEHMET KAYGUSUZ");
        assert_eq!(parse_musteri("mş.mehmet kaygusuz", ""), "MŞ: MEHMET KAYGUSUZ");
        assert_eq!(parse_musteri("mş mehmet kaygusuz", ""), "MŞ: MEHMET KAYGUSUZ");
        assert_eq!(parse_musteri("MS mehmet kaygusuz", ""), "MŞ: MEHMET KAYGUSUZ");
    }

    #[test]
    fn musteri_from_dokumanizleme() {
        assert_eq!(
            parse_musteri("150*200 ADET", "MŞ Mehmet Kaygusuz"),
            "MŞ: MEHMET KAYGUSUZ"
        );
    }
}
