<div align="center">
  <img src="aether-icon.png" width="112" height="112" alt="Aether" />
  <h1>Aether</h1>

  <p><strong>Hızlı, sade ve gizlilik odaklı modern Windows masaüstü tarayıcısı.</strong><br />
  Rust + Tauri v2 + WebView2 ile inşa edildi.</p>

  <p>
    <a href="https://github.com/onturelik-gif/aether-browser/releases/latest"><img src="https://img.shields.io/github/v/release/onturelik-gif/aether-browser?label=s%C3%BCr%C3%BCm&color=0ea5e9" alt="Son Sürüm" /></a>
    <img src="https://img.shields.io/badge/platform-Windows%2010%20%2F%2011-1e293b?logo=windows&logoColor=38bdf8" alt="Windows" />
    <img src="https://img.shields.io/badge/core-Rust%20%2B%20Tauri%20v2-6366f1" alt="Rust + Tauri v2" />
    <a href="LICENSE"><img src="https://img.shields.io/badge/lisans-MPL--2.0-334155" alt="Lisans: MPL-2.0" /></a>
  </p>

  <p>
    <a href="https://github.com/onturelik-gif/aether-browser/releases/latest"><strong>⬇ Aether-Setup.exe İndir (v1.0.0)</strong></a>
  </p>
</div>

---

## Öne Çıkan Özellikler

### İki Farklı Sekme Düzeni (Tek Tıkla Geçiş)
- **Yanda (Kenar Çubuğu):** Modern dikey sekme ağacı, alt köşede sabitlenen uygulamalar ve temiz çalışma alanı.
- **Yukarıda (Klasik):** Alışık olduğun yatay sekme çubuğu, altında yer imleri / sık kullanılanlar çubuğu ve tek tıkla kenar çubuğuna geçiş düğmesi.
- **Sekme Grupları:** Orta tıkla açılan bağlantılar kaynak sekmenin altında otomatik gruplanır, tek tıkla daraltılıp yeniden adlandırılabilir.

### Sürükle-Bırak Split View
- Herhangi bir sekmeyi tutup ekranın sağına/soluna sürükleyerek anında yan yana bölünmüş görünümde çalış.
- Sürükleme sırasında sekmenin kendi logosu imleçle birlikte akıcı biçimde hareket eder.

### Akıllı Sağ Tık Araçları
- **Web'de Ara:** Sayfada seçtiğin herhangi bir metne sağ tıklayıp varsayılan arama motorunda bulunduğun grubun içinde anında ara.
- **Modern Ekran Görüntüsü Aracı:** Herhangi bir sayfada sağ tık → *Ekran görüntüsü al* ile görünür alan, serbest seçim veya **aşağı kaydırarak sınırsız uzunlukta tam sayfa** yakala; sıfır ekran titremesiyle panoya kopyala veya PNG olarak `İndirilenler`'e kaydet.

### Güvenlik, Gizlilik & Performans
- **Yerleşik Reklam & Takipçi Engelleyici:** Rust tabanlı `adblock` motoru ile reklamları ve izleyicileri sayfa yüklenmeden keser.
- **SmartScreen & HTTPS-First:** Zararlı sitelere, phishing girişimlerine ve güvensiz bağlantılara karşı yerleşik koruma.
- **Sıfır Bulut Takibi:** Hesap zorunluluğu ve telemetri yok; yapay zekâ özellikleri tamamen kendi API anahtarınla (BYOK) çalışır.
- **Chrome Eklenti Desteği:** Sevdiğin Chromium eklentilerini doğrudan kurup kullanabilirsin.
- **Akıllı Bellek Yönetimi:** Arka plan sekmeleri 90 saniyede dondurulur, boşta kalan ağır sekmeler otomatik uyutulur ve LTO optimizasyonlu Rust çekirdeğiyle düşük kaynak tüketir.

---

## Kurulum

1. [Releases](https://github.com/onturelik-gif/aether-browser/releases/latest) sayfasından **`Aether-Setup.exe`** dosyasını indir.
2. Çift tıklayıp kurulumu tamamla *(yönetici izni gerektirmez; WebView2 eksikse otomatik indirilir)*.

> **Not:** İlk kurulumda dijital kod imzası olmadığı için Windows SmartScreen *"Bilinmeyen yayımcı"* uyarısı gösterebilir. **"Ek bilgi → Yine de çalıştır"** seçeneğiyle devam edebilirsin.

---

## Kaynaktan Derleme (Windows)

Gereksinimler: **Rust (stable)**, **Node.js 20+**, **pnpm**, **Visual Studio C++ Build Tools**.

```powershell
pnpm install
pnpm -C desktop exec tauri build --bundles nsis
```

---

## Lisans

Bu proje **Mozilla Public License 2.0 (MPL-2.0)** ile lisanslanmıştır. Ayrıntılar için [LICENSE](LICENSE) dosyasına bakabilirsin.
