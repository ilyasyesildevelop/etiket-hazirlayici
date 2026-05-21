# Etiket Hazırlayıcı - Proje Dokümantasyonu

## 1. Uygulamanın Amacı
Etiket Hazırlayıcı; halı üretimi, satışı veya lojistiği yapan firmaların karmaşık Excel sipariş, üretim veya sevk listelerinden verileri okuyup akıllı algoritmalarla ayrıştırarak, termal yazıcılar için otomatik ve standartlaştırılmış etiketler (doğrudan yazıcı çıktısı veya PDF) oluşturmasını sağlayan modern bir masaüstü uygulamasıdır. İnsan kaynaklı yazım hatalarını ve karmaşık açıklamaları düzelterek etiketleme sürecini hatasız ve saniyeler içinde tamamlamayı hedefler.

## 2. Uygulamanın Arayüz Tasarım Özellikleri
Modern ve kullanıcı dostu bir arayüze sahip olan uygulama şu temel bölümlerden oluşur:

* **Üst Kontrol Çubuğu:** Kurumsal mavi tonda tasarlanmış bu alanda uygulama versiyonu, Excel dosya yükleme butonu ("Dosya Aç"), Excel içindeki çalışma sayfaları ("Sheet") arasında geçiş yapmayı sağlayan seçici, "Yükle" butonu ve geçmiş dosyalara hızlı erişim için "Son Yüklenenler" bölümü bulunur.
* **Sol Panel (Veri Listesi):** Yüklenen Excel dosyasındaki verilerin satır satır listelendiği ana tablodur. Satırlar seçilebilir, sayfalama (pagination) özelliği ile sayfalar arası geçiş yapılabilir ve arama kutusu ile binlerce satır içinde hızlıca istenen veriye ulaşılabilir.
* **Sağ Panel (Canlı Önizleme):** Seçili olan etiketin, o anki boyut, font ve alan genişliği ayarlarına göre yazıcıdan tam olarak nasıl çıkacağını gösteren gerçek zamanlı, interaktif bir önizleme alanıdır. Altındaki kontroller ile yakınlaştırma/uzaklaştırma yapılabilir ve seçilen etiketler arasında gezilebilir.
* **Alt Panel (Ayarlar Alanı):** 3 ana sekmeden ("Etiket Ayarları", "Alan Genişlikleri", "Genel Ayarlar") oluşur. Etiket boyutları, font, başlık metni, kenar boşlukları ve yazıcı seçimi bu alandan yapılır.
* **Sağ Alt İşlem Butonları:** Sistemin ana fonksiyonlarını tetikleyen butonlardır: PDF Oluştur, Yazdır, Ayarları Kaydet, Ayarları Yükle, Varsayılanlara Dön ve Hakkında.
* **Durum Çubuğu (Footer):** En altta yer alan bu bantta uygulamanın çalışma durumu (Hazır, Yükleniyor vs.), seçili etiket boyutu, toplam kayıt sayısı ve sağ köşede geliştirici telif bilgisi yer alır.

## 3. Uygulama Özellikleri
* **Excel'den Akıllı Okuma:** Farklı isimlerle oluşturulmuş sütunları (Örn: CARİ, ÜRÜN, AÇIKLAMA, SİPARİŞ vb.) otomatik tanır.
* **Canlı Tasarım:** Ayarlarda yapılan milimetrik bir değişikliği bile anında önizleme ekranına yansıtır.
* **Termal Yazıcı Entegrasyonu:** PPLB formatını destekleyerek Argox ve benzeri termal yazıcılara doğrudan ham veri gönderir, kusursuz kalitede hızlı baskı alınmasını sağlar.
* **PDF Dışa Aktarma:** Seçilen etiketleri belirtilen ayarlarla çok sayfalı PDF'e dönüştürür.
* **Otomatik Veri Ayrıştırma:** Uzun ve düzensiz açıklama metinlerinin içinden ebat, adet, müşteri ismi, işlem tipi gibi verileri çekip çıkararak alanlara böler.
* **Kalıcı Ayarlar:** Yapılan tüm etiket tasarımı ayarlarının kalıcı olarak kaydedilebilmesini ve uygulamanın her açılışında geri yüklenmesini sağlar.

## 4. Teknik Detaylar (Alanların Tespit Edilmesi)
Uygulamanın arka planında (Rust dilinde yazılmış) gelişmiş bir Ayrıştırıcı (Parser) motoru çalışır. Bu motor, metinlerdeki kelimeleri ve kalıpları (Regex) analiz eder:

* **EBAT:** Metin içindeki `120*200`, `80x150` gibi kalıpları bulur. "ENRULO" ibaresi geçen ürünlerde ebat bilgisi `Satır Açıklaması` sütunundan, diğer standart ürünlerde ise `Malz. Açıklama` sütunundan öncelikli olarak çekilir.
* **ADET / m²:** Metin içindeki "X ADET" bilgisini çıkarır. ENRULO tipi ürünlerde `Bekleyen Sipariş` sütunundaki değer m² olarak algılanırken, normal ürünlerdeki değer doğrudan kopya sayısı (Adet) olarak algılanır.
* **İŞLEM:** Satır açıklamasındaki metni tarar. OVERLOK, SAÇAK, KATLAMA vb. kelimeleri standartlaştırır. Müşterilerin karmaşık şekilde yazdığı "İç bordür 50000 / Dış bordür 42451" ya da "Kalın/İnce" gibi Çift Bordür işlemlerini algılayıp `ÇİFT BRD` alt satırına `İÇ 50000 - DIŞ 42451` şeklinde muntazam bir şekilde formatlar. Ayrıca "BODRUR" gibi yazım yanlışlarını tolere eder.
* **MÜŞTERİ ADI (MŞ):** `Doküman İzleme` ve `Satır Açıklaması` sütunlarını tarar. "MŞ:" gibi bir işaretçi arar. Eğer işaretçi yoksa bile, salt isim ve telefon numarasından oluşan metinleri tespit edip, telefon numaralarını temizleyerek sadece adı ve soyadı kısmını çıkarır (Örn: MŞ: AYŞEGÜL TAŞKIN). Bayi (Cari) ünvanlarının yanlışlıkla son kullanıcı olarak yazılmasını önleyen güvenlik filtresi içerir.
* **Cari Ünvan:** Çok uzun şirket ünvanlarını etikete sığdırabilmek için ayarlardan girilen "Cari Maks. Kelime" sınırına göre otomatik kırpar.
* **Diğer Açıklamalar:** Ebat, adet, işlem, müşteri adı, telefon numarası gibi bilgiler metinlerden ayıklandıktan sonra geriye kalan saf metni bu alana taşır. Uzun metinlerde otomatik olarak font küçültme (auto-fit) uygular.

## 5. Etiket Özellikleri (Varsayılanlar)
Uygulama ilk yüklendiğinde etiket tasarımı şu varsayılan ayarlarla gelir:

* **Fiziksel Boyut:** 80 mm Genişlik × 50 mm Yükseklik (Kenar boşluğu: 1,5 mm)
* **Yazı Tipi:** Tahoma, Siyah Renk (#000000)
* **Üst Başlık:** "İyi günlerde kullanmanız dileğiyle" (6 Punto)
* **Etiket Alanı Genişlik Oranları:**
  * Malzeme Açıklama: 25%
  * Diğer Açıklamalar: 25%
  * Cari Ünvan: 22%
  * Müşteri Adı: 12%
  * Ebat: 11%
  * İşlem: 10%
  * Adet / m²: 8%
* **Varsayılan Puntolar:** Ebat: 35 | Malzeme Açıklama: 30 | Cari Ünvan: 25 | Müşteri Adı: 25 | Diğer Açıklamalar: 25 | İşlem: 18 | Adet / m²: 18
* **Ekstralar:** Etiketin sağ alt köşesine sıra numarası (Örn: 1/1) ve kenarlara güncel tarih otomatik olarak yazdırılır.

## 6. Geliştirici ve Telif Hakları
Bu uygulamanın mimarisi, arayüz tasarımı ve ayrıştırma (parsing) algoritmaları özel olarak geliştirilmiştir.
* **Geliştirici:** İlyas Yeşil
* **Altyapı:** Tauri, Rust, Vite, Vanilla JS/CSS
* **Telif Hakkı:** © 2026 Tüm hakları saklıdır. Uygulamanın izinsiz kopyalanması, dağıtılması ve kaynak kodlarının değiştirilmesi telif hakkı ihlali sayılır.
