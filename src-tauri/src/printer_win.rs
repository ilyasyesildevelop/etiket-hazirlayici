//! Windows RAW yazdırma (Argox PPLB vb.) — Win32 WritePrinter API.

#![cfg(windows)]

use std::ffi::c_void;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{GetLastError, HANDLE};
use windows::Win32::Graphics::Printing::{
    ClosePrinter, EndDocPrinter, EndPagePrinter, OpenPrinterW, StartDocPrinterW, StartPagePrinter,
    WritePrinter, DOC_INFO_1W,
};

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn bool_ok(result: windows::Win32::Foundation::BOOL, context: &str) -> Result<(), String> {
    if result.as_bool() {
        Ok(())
    } else {
        let err = unsafe { GetLastError() };
        Err(format!("{}: Windows hata kodu {:?}", context, err))
    }
}

pub fn send_raw_bytes(printer_name: &str, data: &[u8]) -> Result<(), String> {
    if printer_name.trim().is_empty() {
        return Err("Yazıcı adı boş.".into());
    }
    if data.is_empty() {
        return Err("Yazdırılacak veri yok.".into());
    }

    unsafe {
        let name = to_wide(printer_name);
        let mut h_printer = HANDLE::default();
        OpenPrinterW(PCWSTR(name.as_ptr()), &mut h_printer, None).map_err(|e| {
            format!(
                "Yazıcı açılamadı ({}): {}. Yazıcının Windows'ta kurulu olduğundan emin olun.",
                printer_name, e
            )
        })?;

        let doc_name = to_wide("Etiket Hazırlayıcı");
        let datatype = to_wide("RAW");
        let doc_info = DOC_INFO_1W {
            pDocName: windows::core::PWSTR(doc_name.as_ptr() as *mut u16),
            pOutputFile: windows::core::PWSTR::null(),
            pDatatype: windows::core::PWSTR(datatype.as_ptr() as *mut u16),
        };

        let job = StartDocPrinterW(h_printer, 1, &doc_info);
        if job == 0 {
            let err = GetLastError();
            let _ = ClosePrinter(h_printer);
            return Err(format!(
                "Yazdırma işi başlatılamadı ({}): Windows hata kodu {:?}",
                printer_name, err
            ));
        }

        let result = (|| {
            bool_ok(StartPagePrinter(h_printer), "Sayfa başlatılamadı")?;
            let mut written: u32 = 0;
            bool_ok(
                WritePrinter(
                    h_printer,
                    data.as_ptr() as *const c_void,
                    data.len() as u32,
                    &mut written,
                ),
                "Yazıcıya veri gönderilemedi",
            )?;
            if written as usize != data.len() {
                return Err(format!(
                    "Kısmi yazdırma: {} / {} bayt gönderildi.",
                    written,
                    data.len()
                ));
            }
            bool_ok(EndPagePrinter(h_printer), "Sayfa kapatılamadı")
        })();

        let _ = EndDocPrinter(h_printer);
        let _ = ClosePrinter(h_printer);
        result
    }
}
