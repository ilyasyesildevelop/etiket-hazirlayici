# Etiket Hazırlayıcı

**Etiket Hazırlayıcı**, işletmelerin ERP veya ön muhasebe sistemlerinden (örneğin Nebim) çektikleri Excel (`.xlsx`, `.xls`, `.xlsm`) sipariş listelerini otomatik olarak analiz edip, sevkiyat / üretim için standart, hatasız ve temiz etiket çıktısına (PDF ve Termal Yazıcı uyumlu) dönüştüren modern bir masaüstü uygulamasıdır.

## 🚀 Teknolojik Altyapı
- **Masaüstü Kabuk:** Tauri v2
- **Arka Plan Mantığı:** Rust (Yüksek performanslı ve güvenli sistem yönetimi, veri ayrıştırma)
- **Arayüz (Frontend):** Vite + Vanilla JS/CSS (Saf JavaScript ve CSS ile yüksek hız)

## ✨ Öne Çıkan Özellikler

* **Akıllı Veri Ayrıştırma (Parser):** Karışık satır açıklamalarından (Örn: "OVERLOK İÇ BORDÜR 120*200 5 ADET MŞ: ALİ") ebat, m², adet, işlem tipi (saçak, overlok vb.) ve müşteri adını otomatik çıkarır.
* **Sürükle & Bırak Desteği:** Excel dosyalarını doğrudan uygulama penceresine VEYA **uygulama kapalıyken kısayol/EXE dosyası üzerine** sürükleyerek anında listeye yükleyebilirsiniz.
* **Gelişmiş Önizleme:** Sağ tarafta yer alan etkileşimli alanda, etiketinizin yazıcıdan çıkmadan önceki birebir halini görebilir ve tek tıkla döndürebilirsiniz.
* **Özel Etiket (Manuel Kayıt):** Excel olmadan da sıfırdan manuel etiket ekleme, mevcut etiketleri düzenleme ve kopyalama imkanı.
* **Sütun Sıralama ve Boyutlandırma:** Tablo üzerindeki sütunları daraltıp genişletebilir, başlıklara tıklayarak artan/azalan düzende sıralayabilirsiniz.
* **İnce Ayarlar:** Yazı fontu, punto büyüklükleri, etiket kenar boşluğu, tarih ve N/M sıra numarası yazdırma seçeneklerini kişiselleştirebilir ve bu ayarları JSON olarak kaydedebilirsiniz.
* **PDF Çıktı:** Seçili etiketleri, yazdırma ayarlarınıza uygun şekilde tarayıcıda PDF olarak tek sayfa-tek etiket formatında oluşturur.

## 📦 Kurulum ve Başlangıç

### Geliştirici Ortamı (Derleme)
Projeyi kendi ortamınızda derlemek için:
1. Gerekli bileşenleri kurun (Node.js, Rust).
2. Bağımlılıkları yükleyin: `npm install`
3. Portable (Taşınabilir) EXE oluşturmak için: `npm run package:portable`
4. Inno Setup kurulum dosyası (Installer) oluşturmak için: `npm run package:installer` (Sisteminizde Inno Setup yüklü olmalıdır).

### Temel Kullanım
1. Excel dosyanızı uygulamaya sürükleyip bırakın veya **Dosya Aç** butonu ile seçin.
2. Excel'deki sayfayı (Sheet) seçin.
3. Listeden etiketini yazdırmak istediğiniz satırları seçin (checkbox).
4. Alt kısımdan etiket boyutu, font vb. ayarlarınızı yapıp **Canlı Önizleme** alanından sonuca bakın.
5. Hazır olduğunuzda **PDF Oluştur** butonuna tıklayarak çıktılarınızı alın.

## 📋 Excel Sütun Eşleşmesi
Sistem Excel içindeki verileri belirli başlık kelimelerine göre otomatik tanır:
- `CARI`, `FIRMA` -> Cari Ünvan
- `MALZ`, `URUN` -> Malzeme / Ürün Adı
- `ACK`, `SATIR` -> Satır Açıklaması
- `SIP` -> Adet / Miktar
- `MS` VEYA `MUSTERI` -> Müşteri Adı

## 👨‍💻 Geliştirici
**İlyas Yeşil**
Telif Hakkı © 2026 — Tüm hakları saklıdır.
Mülk (Proprietary) yazılım lisansına tabidir. İzinsiz kopyalama veya dağıtım yapılamaz.
