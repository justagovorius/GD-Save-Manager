# 🛡️ GD Save Manager (Desktop & Web)

[🇬🇧 English](#english) | [🇷🇺 Русский](#русский)

![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue?style=for-the-badge)
![Python](https://img.shields.io/badge/python-3.8%2B-green?style=for-the-badge)
![Rust](https://img.shields.io/badge/rust-1.75%2B-orange?style=for-the-badge)
![License](https://img.shields.io/badge/license-MIT-purple?style=for-the-badge)

---

## 🇷🇺 Русский

Надежный и безопасный локальный инструмент для резервного копирования сохранений **Geometry Dash** и **GDPS**. Больше никаких потерянных демонов и иконок из-за багов игры или проблем с серверами!

### 📦 Четыре версии на выбор!
Теперь проект доступен на двух языках программирования (Python и Rust) и в двух вариантах интерфейса (Десктоп и Веб):

**На Python (без внешних зависимостей):**
1. **Десктопная (`gd_save_manager.py`)** — классическое окно на Tkinter. Работает автономно.
2. **Веб-версия (`gd_save_manager_web.py`)** — поднимает локальный сервер и открывает интерфейс в браузере.

**На Rust (максимальная скорость и один .exe):**
3. **Десктопная Rust (`GD-Save-Manager-Rust/`)** — нативное приложение с графическим интерфейсом (egui). Запускается мгновенно, не требует Python.
4. **Веб-версия Rust (`GD-Save-Manager-Web-Rust/`)** — невероятно быстрый и легкий веб-сервер (axum), собирается в один крошечный `.exe` файл.

### 🚀 Основные возможности
- **Безопасность:** Полностью локальная работа. Никаких отправок файлов в интернет.
- **Выборочное копирование:** Выбирайте конкретные файлы (основной прогресс, уровни, библиотеки звуков), которые нужно сохранить или загрузить.
- **Умные темы:** Светлая, темная, системная (считывается один раз при запуске) и специальная **AMOLED** (чистый черный цвет).
- **Локализация:** Полная поддержка русского и английского языков (включая автоопределение системного).
- **Встроенный проводник:** Открывайте папки с сохранениями прямо из программы одной кнопкой.
- **Умный веб-сервер:** Веб-версии автоматически выключаются через 1 секунду после закрытия вкладки браузера, не оставляя висящих процессов в памяти (ОЗУ).

### 💾 Как скачать готовую версию
Перейдите в раздел **[Releases](../../releases)** на этом GitHub и скачайте готовые `.exe` файлы для Windows. (Для macOS и Linux используйте запуск из исходников).

### 🛠️ Как запустить (из исходников)
**Python (версия 3.8+):**
1. Скачайте файлы `.py` из корня репозитория.
2. Запустите нужную версию:
   - Десктоп: `python gd_save_manager.py`
   - Веб: `python gd_save_manager_web.py`

**Rust (версия 1.75+):**
1. Установите [Rust](https://rustup.rs/). Для Windows также потребуются [Build Tools для Visual Studio](https://visualstudio.microsoft.com/downloads/) (компонент "Разработка классических приложений на C++").
2. Перейдите в папку нужной версии (`GD-Save-Manager-Rust` или `GD-Save-Manager-Web-Rust`).
3. Выполните в терминале: `cargo run`

### 💻 Как собрать в один .exe (Windows)
**Для Python:** установите PyInstaller (`pip install pyinstaller`) и выполните:
- Десктоп: `python -m PyInstaller --noconsole --onefile gd_save_manager.py`
- Веб: `python -m PyInstaller --noconsole --onefile gd_save_manager_web.py`

**Для Rust:** перейдите в папку с `Cargo.toml` и выполните:
- `cargo build --release`
Готовые скомпилированные файлы будут лежать в папке `target/release/`.

---

## 🇬🇧 English

A safe and reliable local backup tool for **Geometry Dash** and **GDPS** saves. No more lost demons and icons due to game bugs or server issues!

### 📦 Four versions to choose from!
The project is now available in two programming languages (Python and Rust) and two interface types (Desktop and Web):

**Python (no external dependencies):**
1. **Desktop (`gd_save_manager.py`)** — classic Tkinter window. Works standalone.
2. **Web (`gd_save_manager_web.py`)** — launches a local server and opens the UI in your browser.

**Rust (maximum speed and single .exe):**
3. **Desktop Rust (`GD-Save-Manager-Rust/`)** — native application with a graphical interface (egui). Starts instantly, no Python required.
4. **Web Rust (`GD-Save-Manager-Web-Rust/`)** — incredibly fast and lightweight web server (axum), compiles into a tiny standalone `.exe` file.

### 🚀 Features
- **Privacy First:** Completely local operation. No files are sent to the internet.
- **Selective Backup:** Choose specific files (main progress, local levels, sound and music libraries) to save or load.
- **Smart Themes:** Light, Dark, System (read once on startup), and a special **AMOLED** mode (pure black).
- **Localization:** Full support for English and Russian languages (including system auto-detection).
- **Built-in Explorer:** Open save folders directly from the app with one click.
- **Smart Web Server:** Web versions automatically shut down 1 second after you close the browser tab, leaving no hanging processes in memory (RAM).

### 💾 How to download the ready-to-use version
Go to the **[Releases](../../releases)** section on this GitHub and download the pre-compiled `.exe` files for Windows. (For macOS and Linux, run from source).

### 🛠️ How to run (from source)
**Python (version 3.8+):**
1. Download the `.py` files from the repository root.
2. Run the desired version:
   - Desktop: `python gd_save_manager.py`
   - Web: `python gd_save_manager_web.py`

**Rust (version 1.75+):**
1. Install [Rust](https://rustup.rs/). For Windows, you will also need [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/) (component "Desktop development with C++").
2. Go to the folder of the desired version (`GD-Save-Manager-Rust` or `GD-Save-Manager-Web-Rust`).
3. Run in terminal: `cargo run`

### 💻 How to compile to a single .exe (Windows)
**For Python:** install PyInstaller (`pip install pyinstaller`) and run:
- Desktop: `python -m PyInstaller --noconsole --onefile gd_save_manager.py`
- Web: `python -m PyInstaller --noconsole --onefile gd_save_manager_web.py`

**For Rust:** go to the folder containing `Cargo.toml` and run:
- `cargo build --release`
The compiled `.exe` files will be located in the `target/release/` folder.

---
*Created for the Geometry Dash community. Play safe! / Создано для комьюнити Geometry Dash. Играйте безопасно!* 💙