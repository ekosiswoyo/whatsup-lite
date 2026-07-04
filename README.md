# WhatsUp Lite

Wrapper desktop WhatsApp Web berbasis Tauri v2. Window utama memakai WebView bawaan OS, sehingga tidak membawa runtime Chromium lengkap seperti Electron. Semua data, cookie, dan konfigurasi berada di komputer pengguna; aplikasi tidak membutuhkan server.

> Proyek ini bukan produk resmi WhatsApp/Meta. Gunakan hanya sebagai client pribadi dan patuhi ketentuan WhatsApp. Tidak ada bot, scraping, auto-reply, atau bypass keamanan.

## Fitur

- Login QR dan sesi persisten melalui profile WebView Tauri
- Single-account, single-window
- System tray: Open, Reload, Settings, Logout/Clear Session, Quit
- Close-to-tray yang dapat dinonaktifkan
- Autostart opsional
- Settings lokal dan tema mengikuti sistem
- Shortcut `Ctrl+R`, `Ctrl+Q`, dan `Ctrl+Shift+L`
- Paket Windows (NSIS/MSI sesuai toolchain) dan Linux (deb/AppImage/RPM sesuai host)

WhatsApp Web sendiri meminta izin dan mengirim Web Notification. Dukungan notifikasi bergantung pada WebView/desktop environment. Badge unread lintas platform tidak diaktifkan karena Tauri/WebView tidak menawarkan API konsisten untuk membaca state WhatsApp tanpa integrasi DOM yang rapuh.

## Struktur

```text
.
├── index.html                 # Settings lokal
├── src/                       # TypeScript/CSS settings
├── src-tauri/
│   ├── capabilities/          # ACL local dan web.whatsapp.com
│   ├── src/lib.rs             # WebView, tray, session, shortcut
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json
└── vite.config.ts
```

## Prasyarat umum

- Node.js 20+ dan npm
- Rust stable melalui [rustup](https://rustup.rs/)
- Git (opsional)

Install dependency proyek:

```bash
npm install
```

## Development

```bash
npm run tauri:dev
```

### Install ke application launcher Ubuntu

Setelah build, salin binary, icon, dan desktop entry untuk user aktif:

```bash
install -Dm755 src-tauri/target/release/whatsup-lite ~/.local/bin/whatsup-lite
install -Dm644 src-tauri/icons/32x32.png ~/.local/share/icons/hicolor/32x32/apps/whatsup-lite.png
sed "s|^Exec=.*|Exec=$HOME/.local/bin/whatsup-lite|" packaging/whatsup-lite.desktop \
  > ~/.local/share/applications/whatsup-lite.desktop
chmod 644 ~/.local/share/applications/whatsup-lite.desktop
```

Vite hanya melayani halaman Settings saat development. WhatsApp Web tetap dimuat langsung dari `https://web.whatsapp.com`.

## Build Windows 10/11

Build Windows harus dilakukan di Windows (cross-compile WebView2/installer dari Linux tidak didukung dengan baik).

1. Install Visual Studio 2022 Build Tools, workload **Desktop development with C++**, Windows 10/11 SDK.
2. Pastikan Microsoft Edge WebView2 Runtime tersedia (sudah ada secara default di Windows 11).
3. Jalankan:

```powershell
npm install
npm run tauri:build
```

Hasil berada di `src-tauri\target\release\bundle\`. Installer umumnya ada pada folder `nsis\` atau `msi\`. Executable portable tanpa installer ada di `src-tauri\target\release\whatsup-lite.exe`; WebView2 Runtime tetap merupakan prasyarat mesin target.

Untuk meminta format tertentu:

```powershell
npx tauri build --bundles nsis
npx tauri build --bundles msi
```

MSI memerlukan WiX Toolset yang sesuai. NSIS biasanya pilihan distribusi termudah.

## Build Ubuntu/Linux

Ubuntu 22.04/24.04:

```bash
sudo apt update
sudo apt install -y libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev patchelf
npm install
npm run tauri:build
```

Hasil ada di `src-tauri/target/release/bundle/`. Build format tertentu:

```bash
npx tauri build --bundles deb
npx tauri build --bundles appimage
```

`deb` adalah installer yang paling natural untuk Ubuntu. AppImage lebih portable, tetapi tetap bergantung pada kompatibilitas kernel, glibc, WebKitGTK, tray implementation, dan desktop environment host. Build di distro Linux tertua yang ingin didukung agar kompatibilitas glibc lebih luas.

## Data lokal dan keamanan

- Cookie/cache WebView memakai data directory persisten milik aplikasi yang dikelola Tauri/WebView.
- Settings aplikasi disimpan sebagai `settings.json` di app config directory OS.
- **Logout / Clear Session** menghapus seluruh browsing data WebView, bukan sekadar me-reload halaman.
- ACL hanya memberikan command aplikasi kepada window Settings dan origin persis `https://web.whatsapp.com`.
- Navigasi utama menggunakan HTTPS langsung ke WhatsApp; kredensial tidak diteruskan ke server lain.

Update Tauri/plugin secara berkala dan audit perubahan sebelum merilis binary. Untuk distribusi publik, sign installer Windows dan paket Linux sesuai proses organisasi Anda.

## Hardware acceleration

WebView2 (Windows) dan WebKitGTK (Linux) mengelola hardware acceleration. Toggle runtime yang benar-benar konsisten tidak tersedia di Tauri v2 lintas platform, sehingga Settings tidak menawarkan switch palsu. Bila driver GPU bermasalah, gunakan pengaturan/variable WebView platform ketika menjalankan aplikasi; konsekuensinya dapat berbeda per versi OS dan WebView. Opsi ini sengaja tidak dipersistenkan sebagai fitur aplikasi.

## Troubleshooting

### QR tidak muncul

- Pastikan waktu/tanggal OS benar dan `https://web.whatsapp.com` dapat dibuka tanpa proxy/filter jaringan.
- Pilih tray **Reload**.
- Update WebView2 di Windows atau paket WebKitGTK di Linux.
- Pilih **Logout / Clear Session**, lalu buka kembali. Ini menghapus login lama.
- Jika WhatsApp menolak user agent, perbarui string `.user_agent(...)` di `src-tauri/src/lib.rs` mengikuti browser stabil yang saat itu didukung WhatsApp.

### Session tidak tersimpan

- Jangan menjalankan binary dari sandbox/read-only filesystem yang mencegah WebView menulis profile.
- Jangan memilih Clear Session saat keluar.
- Pastikan app identifier `com.whatsuplite.desktop` tidak berubah antar-build; identifier berbeda menghasilkan data directory berbeda.
- Periksa permission app-data/config user dan ruang disk.

### Notification tidak muncul

- Aktifkan notification di Settings WhatsApp Web dan izinkan permission saat diminta.
- Pastikan WhatsUp Lite tidak diblokir di Windows Notification Settings atau pengaturan notifikasi desktop Linux.
- WebKitGTK dan beberapa compositor Linux tidak selalu menjembatani Web Notification dengan lengkap. Alternatif ringan adalah tetap mengandalkan bunyi/unread title WhatsApp Web; notifikasi native yang andal memerlukan bridge DOM khusus yang mudah rusak ketika WhatsApp mengubah UI.

### WebView blank

- Windows: repair/install Microsoft Edge WebView2 Runtime.
- Linux: verifikasi `libwebkit2gtk-4.1-0` terpasang dan coba jalankan dari terminal untuk melihat error GPU/WebKit.
- Nonaktifkan sementara VPN, TLS inspection, ad blocker DNS, atau proxy korporat.
- Hapus sesi dari tray. Jika tetap blank, hapus app-data WhatsUp Lite secara manual setelah membuat backup.

### Build gagal di Ubuntu

- Pastikan paket `libwebkit2gtk-4.1-dev` (bukan hanya runtime) dan `libayatana-appindicator3-dev` terpasang.
- Jalankan `rustup update stable` dan cek `rustc --version`.
- Pada Ubuntu lama yang hanya menyediakan WebKitGTK 4.0, gunakan distro/container build yang menyediakan 4.1; jangan mencampur library lintas rilis Ubuntu.
- Jalankan `npm run build` lalu `cargo check --manifest-path src-tauri/Cargo.toml` untuk memisahkan error frontend dan Rust.

### Build gagal di Windows

- Jalankan dari **Developer PowerShell for VS 2022** dan pastikan MSVC + Windows SDK terpasang.
- Jalankan `rustup default stable-x86_64-pc-windows-msvc`.
- Untuk error bundler MSI, install WiX atau build NSIS saja dengan `npx tauri build --bundles nsis`.
- Untuk error frontend, hapus `node_modules` lalu jalankan `npm install` kembali. Jangan menghapus data directory aplikasi karena itu berisi sesi login.

## Batasan yang disengaja

- Tidak ada updater/background service agar footprint kecil.
- Tidak ada multi-account/multi-window.
- Tidak ada unread badge yang mengandalkan selector internal WhatsApp.
- User agent statis mungkin perlu diperbarui ketika kebijakan WhatsApp berubah.
- Konsumsi RAM terutama ditentukan oleh WhatsApp Web dan WebView sistem; wrapper ini mengurangi overhead runtime, tetapi tidak dapat mengendalikan memory aplikasi web WhatsApp.
