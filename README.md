# 🛡️ GD Save Manager (Desktop & Web)

[🇬🇧 English](#english) | [🇷🇺 Русский](#русский)

![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue?style=for-the-badge)
![Python](https://img.shields.io/badge/python-3.8%2B-green?style=for-the-badge)
![License](https://img.shields.io/badge/license-MIT-purple?style=for-the-badge)

---

## 🇷🇺 Русский

Надежный и безопасный локальный инструмент для резервного копирования сохранений **Geometry Dash** и **GDPS**. Больше никаких потерянных демонов и иконок из-за багов игры или проблем с серверами!

### 📦 Две версии на выбор
1. **Десктопная (`gd_save_manager.py`)** — классическое окно программы, написанное на Tkinter. Работает автономно, не требует браузера.
2. **Веб-версия (`gd_save_manager_web.py`)** — поднимает локальный сервер и открывает современный красивый интерфейс прямо в вашем браузере.

### 🚀 Основные возможности
- **Безопасность:** Полностью локальная работа. Никаких отправок файлов в интернет, никаких сторонних библиотек.
- **Выборочное копирование:** Выбирайте конкретные файлы (основной прогресс, локальные уровни, библиотеки звуков и музыки), которые нужно сохранить или загрузить.
- **Умные темы:** Светлая, темная, системная (считывается один раз при запуске) и специальная **AMOLED** (чистый черный цвет для экономии батареи).
- **Локализация:** Полная поддержка русского и английского языков (включая автоопределение системного).
- **Встроенный проводник:** Открывайте папки с сохранениями прямо из программы одной кнопкой.
- **Умный веб-сервер:** Веб-версия автоматически выключается через 1 секунду после закрытия вкладки браузера, не оставляя висящих процессов в памяти (ОЗУ).

### 💾 Как скачать готовую версию
Перейдите в раздел **[Releases](../../releases)** на этом GitHub и скачайте готовые `.exe` файлы для Windows. (Для macOS и Linux используйте запуск из исходников).

### 🛠️ Как запустить (из исходников)
1. Установите [Python](https://www.python.org/downloads/) (версия 3.8 или выше).
2. Скачайте файлы `gd_save_manager.py` и/или `gd_save_manager_web.py`.
3. Запустите нужную версию:
   - Десктопная версия: `python gd_save_manager.py`
   - Веб-версия: `python gd_save_manager_web.py`

*(Никакие сторонние библиотеки типа `pip install ...` не требуются, всё работает на стандартной библиотеке Python!)*

### 💻 Как собрать в один .exe (Windows)
Если вы хотите получить один исполняемый файл без необходимости установки Python:
1. Установите PyInstaller: `pip install pyinstaller`
2. Выполните команду в папке с файлами:
   - Для десктопной версии: `python -m PyInstaller --noconsole --onefile gd_save_manager.py`
   - Для веб-версии: `python -m PyInstaller --noconsole --onefile gd_save_manager_web.py`
3. Готовые `.exe` файлы появятся в папке `dist`.

---

## 🇬🇧 English

A safe and reliable local backup tool for **Geometry Dash** and **GDPS** saves. No more lost demons and icons due to game bugs or server issues!

### 📦 Two versions to choose from
1. **Desktop (`gd_save_manager.py`)** — a classic Tkinter window application. Works standalone, no browser needed.
2. **Web (`gd_save_manager_web.py`)** — launches a local server and opens a modern, beautiful interface right in your browser.

### 🚀 Features
- **Privacy First:** Completely local operation. No files are sent to the internet, and no external libraries are required.
- **Selective Backup:** Choose specific files (main progress, local levels, sound and music libraries) to save or load.
- **Smart Themes:** Light, Dark, System (read once on startup), and a special **AMOLED** mode (pure black for battery saving).
- **Localization:** Full support for English and Russian languages (including system auto-detection).
- **Built-in Explorer:** Open save folders directly from the app with one click.
- **Smart Web Server:** The web version automatically shuts down 1 second after you close the browser tab, leaving no hanging processes in memory (RAM).

### 💾 How to download the ready-to-use version
Go to the **[Releases](../../releases)** section on this GitHub and download the pre-compiled `.exe` files for Windows. (For macOS and Linux, run from source).

### 🛠️ How to run (from source)
1. Install [Python](https://www.python.org/downloads/) (version 3.8+).
2. Download `gd_save_manager.py` and/or `gd_save_manager_web.py`.
3. Run the desired version:
   - Desktop version: `python gd_save_manager.py`
   - Web version: `python gd_save_manager_web.py`

*(No third-party libraries like `pip install ...` are required, it runs entirely on the Python standard library!)*

### 💻 How to compile to a single .exe (Windows)
If you want to get a single executable file without installing Python:
1. Install PyInstaller: `pip install pyinstaller`
2. Run the command in the folder with the files:
   - For the desktop version: `python -m PyInstaller --noconsole --onefile gd_save_manager.py`
   - For the web version: `python -m PyInstaller --noconsole --onefile gd_save_manager_web.py`
3. The ready `.exe` files will appear in the `dist` folder.

---
*Created for the Geometry Dash community. Play safe! / Создано для комьюнити Geometry Dash. Играйте безопасно!* 💙
