import os
import sys
import json
import time
import shutil
import subprocess
import threading
import webbrowser
import http.server
import socketserver
import tkinter as tk
from tkinter import filedialog
import locale

# ------------------------------------------------------------ Скрытие консоли
if os.name == "nt":
    try:
        import ctypes
        _hwnd = ctypes.windll.kernel32.GetConsoleWindow()
        if _hwnd:
            ctypes.windll.user32.ShowWindow(_hwnd, 0)
    except Exception:
        pass

BACKUP_FOLDER_NAME = "GD_Backups"
LAST_PING = time.time()
SERVER_START_TIME = time.time()

def _config_dir():
    if os.name == "nt":
        base = os.environ.get("LOCALAPPDATA") or os.path.expanduser("~")
    else:
        base = os.path.join(os.path.expanduser("~"), ".config")
    return os.path.join(base, "GD_SaveManager")

def _config_path(): return os.path.join(_config_dir(), "config.json")

def load_config():
    try:
        with open(_config_path(), "r", encoding="utf-8") as f: return json.load(f)
    except Exception: return {}

def save_config(cfg):
    try:
        os.makedirs(_config_dir(), exist_ok=True)
        with open(_config_path(), "w", encoding="utf-8") as f: json.dump(cfg, f, indent=2, ensure_ascii=False)
    except Exception: pass

# ------------------------------------------------------------ HTML/CSS/JS
HTML_CONTENT = """
<!DOCTYPE html>
<html lang="ru">
<head>
    <meta charset="UTF-8">
    <title>GD Save Manager</title>
    <style>
        :root { --bg: #eef1f5; --card: #ffffff; --border: #d8dee6; --text: #1f2937; --muted: #6b7280; --primary: #2563eb; --success: #16a34a; --danger: #dc2626; --warning: #d97706; --input-bg: #f9fafb; --chip: #e5e7eb; --secondary: #6b7280; }
        body.dark { --bg: #1a1d24; --card: #252932; --border: #353a45; --text: #e5e7eb; --muted: #9ca3af; --primary: #3b82f6; --success: #22c55e; --danger: #ef4444; --warning: #f59e0b; --input-bg: #1e222a; --chip: #353a45; --secondary: #6b7280; }
        body.amoled { --bg: #000000; --card: #0a0a0a; --border: #1c1c1c; --text: #e5e7eb; --muted: #9ca3af; --primary: #3b82f6; --success: #22c55e; --danger: #ef4444; --warning: #f59e0b; --input-bg: #000000; --chip: #1a1a1a; --secondary: #4b5563; }
        * { box-sizing: border-box; margin: 0; padding: 0; font-family: 'Segoe UI', -apple-system, sans-serif; }
        body { background: var(--bg); color: var(--text); transition: 0.3s; padding: 24px; }
        .header { display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 20px; }
        .title h1 { font-size: 24px; margin-bottom: 4px; }
        .title p { color: var(--muted); font-size: 14px; }
        .settings-bar { display: flex; align-items: center; gap: 8px; font-size: 14px; flex-wrap: wrap; justify-content: flex-end;}
        .pill { background: var(--chip); color: var(--text); border: none; padding: 6px 12px; border-radius: 20px; cursor: pointer; font-size: 13px; transition: 0.2s; }
        .pill:hover { filter: brightness(0.9); }
        .pill.active { background: var(--primary); color: #fff; font-weight: bold; }
        .checkbox-container { display: flex; align-items: center; cursor: pointer; user-select: none; margin-left: 5px;}
        .custom-cb { width: 18px; height: 18px; border: 2px solid var(--border); border-radius: 3px; background: var(--input-bg); display: flex; justify-content: center; align-items: center; margin-right: 8px; transition: 0.1s; }
        .custom-cb svg { display: none; width: 12px; height: 12px; stroke: #fff; stroke-width: 3; stroke-linecap: round; stroke-linejoin: round; }
        input[type="checkbox"] { display: none; }
        input[type="checkbox"]:checked + .custom-cb { background: var(--primary); border-color: var(--primary); }
        input[type="checkbox"]:checked + .custom-cb svg { display: block; }
        input[type="checkbox"]:disabled + .custom-cb { opacity: 0.5; }
        input[type="checkbox"]:disabled ~ span { opacity: 0.5; }
        .card { background: var(--card); border: 1px solid var(--border); border-radius: 8px; padding: 16px; margin-bottom: 16px; box-shadow: 0 1px 3px rgba(0,0,0,0.05); }
        .card-title { font-size: 15px; font-weight: bold; margin-bottom: 4px; }
        .card-sub { font-size: 13px; color: var(--muted); margin-bottom: 12px; }
        .row { display: flex; gap: 8px; margin-bottom: 12px; }
        .row input[type="text"] { flex: 1; padding: 10px; border: 1px solid var(--border); border-radius: 6px; background: var(--input-bg); color: var(--text); outline: none; font-size: 14px; }
        .row input[type="text"]:focus { border-color: var(--primary); }
        button.btn { border: none; padding: 10px 16px; border-radius: 6px; cursor: pointer; font-size: 14px; font-weight: bold; color: white; transition: 0.2s; }
        button.btn:hover { filter: brightness(0.9); }
        .btn-primary { background: var(--primary); } .btn-secondary { background: var(--secondary); } .btn-success { background: var(--success); }
        .btn-chip { background: var(--chip); color: var(--text); font-weight: normal; font-size: 12px; padding: 6px 12px; }
        .file-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; margin-top: 12px; }
        .file-item { display: flex; align-items: center; }
        .file-name { font-weight: bold; font-size: 14px; margin-left: 4px; cursor: pointer; }
        .file-desc { color: var(--muted); font-size: 12px; margin-left: 8px; cursor: pointer; }
        .action-row { display: flex; gap: 12px; margin-bottom: 20px; }
        .action-row button { flex: 1; padding: 14px; font-size: 16px; }
        .log-area { background: var(--input-bg); border: 1px solid var(--border); border-radius: 6px; padding: 12px; height: 180px; overflow-y: auto; font-family: monospace; font-size: 13px; white-space: pre-wrap; line-height: 1.4; }
        .log-success { color: var(--success); } .log-error { color: var(--danger); } .log-warning { color: var(--warning); } .log-header { font-weight: bold; color: var(--text); } .log-muted { color: var(--muted); }
        select { padding: 5px 10px; border-radius: 6px; border: 1px solid var(--border); background: var(--input-bg); color: var(--text); outline: none; font-size: 13px; cursor: pointer;}
    </style>
</head>
<body>
    <div class="header">
        <div class="title">
            <h1 data-i18n="app_title">GD Save Manager</h1>
            <p data-i18n="app_sub">Локальное резервное копирование сохранений GD и GDPS</p>
        </div>
        <div class="settings-bar">
            <span style="color: var(--muted); font-size: 13px;" data-i18n="lang">Язык:</span>
            <select id="lang-select" onchange="setLang(this.value)">
                <option value="ru">Русский</option>
                <option value="en">English</option>
                <option value="system" data-i18n="lang_sys">Системный</option>
            </select>
            <span style="color: var(--muted); font-size: 13px; margin-left: 10px;" data-i18n="theme">Тема:</span>
            <button class="pill" id="btn-light" onclick="setThemeMode('light')" data-i18n="theme_light">Светлая</button>
            <button class="pill" id="btn-dark" onclick="setThemeMode('dark')" data-i18n="theme_dark">Тёмная</button>
            <button class="pill" id="btn-system" onclick="setThemeMode('system')" data-i18n="theme_sys">Системная</button>
            <label class="checkbox-container">
                <input type="checkbox" id="cb-amoled" onchange="toggleAmoled()">
                <div class="custom-cb"><svg viewBox="0 0 24 24"><polyline points="20 6 9 17 4 12"></polyline></svg></div>
                <span style="font-size: 13px;">AMOLED</span>
            </label>
            <button class="btn btn-chip" style="margin-left: 10px; background: transparent; border: 1px solid var(--border);" onclick="exitApp()" data-i18n="btn_exit">Выход</button>
        </div>
    </div>

    <div class="card">
        <div class="card-title" data-i18n="src_title">📁 Папка с сохранениями игры</div>
        <div class="card-sub" data-i18n="src_sub">по умолчанию: %LOCALAPPDATA%\\GeometryDash</div>
        <div class="row">
            <input type="text" id="source-path">
            <button class="btn btn-primary" onclick="pickFolder('source')" data-i18n="btn_pick">Выбрать</button>
            <button class="btn btn-secondary" onclick="openFolder('source')" data-i18n="btn_open">Обзор</button>
        </div>
    </div>

    <div class="card">
        <div class="card-title" data-i18n="bk_title">💾 Папка GD_Backups</div>
        <div class="card-sub" data-i18n="bk_sub">место, где будут храниться резервные копии</div>
        <div class="row">
            <input type="text" id="backup-path">
            <button class="btn btn-primary" onclick="pickFolder('backup')" data-i18n="btn_pick">Выбрать</button>
            <button class="btn btn-secondary" onclick="openFolder('backup')" data-i18n="btn_open">Обзор</button>
        </div>
    </div>

    <div class="card">
        <div class="card-title" data-i18n="files_title">📄 Какие файлы задействовать</div>
        <div class="card-sub" data-i18n="files_sub">поставь галочку у файлов, которые нужно копировать</div>
        <div style="display: flex; gap: 10px;">
            <button class="btn btn-chip" onclick="setAllCheckboxes(true)" data-i18n="btn_all">Выбрать все</button>
            <button class="btn btn-chip" onclick="setAllCheckboxes(false)" data-i18n="btn_none">Снять все</button>
        </div>
        <div class="file-grid" id="file-list"></div>
    </div>

    <div class="action-row">
        <button class="btn btn-primary" onclick="doAction('save')" data-i18n="btn_save">💾 СОХРАНИТЬ (игра → копия)</button>
        <button class="btn btn-success" onclick="doAction('load')" data-i18n="btn_load">📂 ЗАГРУЗИТЬ (копия → игра)</button>
    </div>

    <div class="card" style="margin-bottom: 0;">
        <div class="card-title" data-i18n="log_title">📝 Журнал операций</div>
        <div class="log-area" id="log"></div>
    </div>

    <script>
        const translations = {
            "ru": {
                app_title: "GD Save Manager", app_sub: "Локальное резервное копирование сохранений GD и GDPS",
                lang: "Язык:", lang_sys: "Системный", theme: "Тема:", theme_light: "Светлая", theme_dark: "Тёмная", theme_sys: "Системная", btn_exit: "Выход",
                src_title: "📁 Папка с сохранениями игры", src_sub: "по умолчанию: %LOCALAPPDATA%\\GeometryDash", btn_pick: "Выбрать", btn_open: "Обзор",
                bk_title: "💾 Папка GD_Backups", bk_sub: "место, где будут храниться резервные копии",
                files_title: "📄 Какие файлы задействовать", files_sub: "поставь галочку у файлов, которые нужно копировать", btn_all: "Выбрать все", btn_none: "Снять все",
                btn_save: "💾 СОХРАНИТЬ (игра → копия)", btn_load: "📂 ЗАГРУЗИТЬ (копия → игра)", log_title: "📝 Журнал операций",
                f_cc1: "Основной прогресс игры", f_cc2: "Дубликат прогресса", f_cl1: "Локальные уровни", f_cl2: "Дубликат уровней",
                f_sfx: "Библиотека звуков", f_mus: "Библиотека музыки", f_plat: "Сохранение платформера",
                log_src: "Папка игры:   ", log_dst: "Папка бэкапов:", log_open: "📁 Открыта папка: ",
                log_copy_src: "   Откуда: ", log_copy_dst: "   Куда:   ", log_save_t: "СОХРАНЕНИЕ (игра → копия)", log_load_t: "ЗАГРУЗКА (копия → игра)",
                log_success: "\\n✅ Успешно скопировано ({n}):", log_skip: "\\n⚠️  Пропущено ({n}):", log_fail: "\\n❌ Ошибки копирования ({n}):",
                msg_err_pick: "❌ Сначала укажите папку.", msg_err_paths: "❌ Укажите обе папки."
            },
            "en": {
                app_title: "GD Save Manager", app_sub: "Local backup tool for GD and GDPS saves",
                lang: "Language:", lang_sys: "System", theme: "Theme:", theme_light: "Light", theme_dark: "Dark", theme_sys: "System", btn_exit: "Exit",
                src_title: "📁 Game Save Folder", src_sub: "default: %LOCALAPPDATA%\\GeometryDash", btn_pick: "Select", btn_open: "Browse",
                bk_title: "💾 GD_Backups Folder", bk_sub: "where backup files will be stored",
                files_title: "📄 Files to include", files_sub: "check the files you want to copy", btn_all: "Select All", btn_none: "Deselect All",
                btn_save: "💾 SAVE (game → backup)", btn_load: "📂 LOAD (backup → game)", log_title: "📝 Operation Log",
                f_cc1: "Main game progress", f_cc2: "Progress backup", f_cl1: "Local levels", f_cl2: "Levels backup",
                f_sfx: "Sound library", f_mus: "Music library", f_plat: "Platformer save",
                log_src: "Game folder:   ", log_dst: "Backup folder: ", log_open: "📁 Opened folder: ",
                log_copy_src: "   From: ", log_copy_dst: "   To:   ", log_save_t: "SAVE (game → backup)", log_load_t: "LOAD (backup → game)",
                log_success: "\\n✅ Successfully copied ({n}):", log_skip: "\\n⚠️  Skipped ({n}):", log_fail: "\\n❌ Copy errors ({n}):",
                msg_err_pick: "❌ Please select a folder first.", msg_err_paths: "❌ Please specify both folders."
            }
        };

        const fileDescriptions = {
            "CCGameManager.dat": "f_cc1", "CCGameManager2.dat": "f_cc2", "CCLocalLevels.dat": "f_cl1", "CCLocalLevels2.dat": "f_cl2",
            "sfxlibrary.dat": "f_sfx", "musiclibrary.dat": "f_mus", "CCPlatformerSave.dat": "f_plat"
        };

        // Считываем системные настройки ОДИН РАЗ при запуске
        const SYSTEM_LANG = navigator.language.startsWith('ru') ? 'ru' : 'en';
        const SYSTEM_THEME = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';

        let config = {};
        let selectedFiles = {};
        let currentLang = "ru";

        setInterval(() => { fetch('/api/ping', { method: 'POST' }).catch(e => {}); }, 100);

        async function init() {
            const res = await fetch('/api/config');
            config = await res.json();
            
            document.getElementById('source-path').value = config.source_dir || "";
            document.getElementById('backup-path').value = config.backup_dir || "";
            document.getElementById('lang-select').value = config.language || "system";
            
            selectedFiles = config.selected_files || {};
            if (!config.selected_files) Object.keys(fileDescriptions).forEach(k => selectedFiles[k] = true);

            detectLanguage();
            renderFiles();
            setThemeMode(config.theme_mode || 'system', true);
            document.getElementById('cb-amoled').checked = config.amoled || false;
            applyTheme();
            applyLanguage();
            
            log(t('log_src') + document.getElementById('source-path').value, 'muted');
            log(t('log_dst') + document.getElementById('backup-path').value, 'muted');
        }

        function detectLanguage() {
            const mode = config.language || 'system';
            currentLang = mode === 'system' ? SYSTEM_LANG : mode;
        }

        function t(key, args) {
            let str = translations[currentLang][key] || key;
            if (args) for (const [k, v] of Object.entries(args)) str = str.replace(`{${k}}`, v);
            return str;
        }

        function applyLanguage() {
            document.querySelectorAll('[data-i18n]').forEach(el => {
                el.innerText = t(el.getAttribute('data-i18n'));
            });
            renderFiles();
        }

        function setLang(val) {
            config.language = val;
            saveConfig();
            detectLanguage();
            applyLanguage();
        }

        function renderFiles() {
            const grid = document.getElementById('file-list');
            grid.innerHTML = '';
            for (const [filename, descKey] of Object.entries(fileDescriptions)) {
                const isChecked = selectedFiles[filename] !== false;
                const div = document.createElement('div');
                div.className = 'file-item';
                div.innerHTML = `
                    <label class="checkbox-container" style="margin-right: 8px;">
                        <input type="checkbox" id="cb-${filename}" ${isChecked ? 'checked' : ''} onchange="toggleFile('${filename}')">
                        <div class="custom-cb"><svg viewBox="0 0 24 24"><polyline points="20 6 9 17 4 12"></polyline></svg></div>
                    </label>
                    <span class="file-name" onclick="toggleFile('${filename}')">${filename}</span>
                    <span class="file-desc" onclick="toggleFile('${filename}')">${t(descKey)}</span>
                `;
                grid.appendChild(div);
            }
        }

        function toggleFile(filename) {
            selectedFiles[filename] = !selectedFiles[filename];
            document.getElementById(`cb-${filename}`).checked = selectedFiles[filename];
            saveConfig();
        }

        function setAllCheckboxes(val) {
            Object.keys(fileDescriptions).forEach(k => selectedFiles[k] = val);
            renderFiles();
            saveConfig();
        }

        function setThemeMode(mode, silent=false) {
            config.theme_mode = mode;
            if (!silent) saveConfig();
            applyTheme();
        }

        function toggleAmoled() {
            config.amoled = document.getElementById('cb-amoled').checked;
            saveConfig();
            applyTheme();
        }

        function applyTheme() {
            let mode = config.theme_mode || 'system';
            if (mode === 'system') mode = SYSTEM_THEME; // Используем зафиксированную системную тему
            
            document.body.className = '';
            if (mode === 'dark' && config.amoled) document.body.className = 'amoled';
            else if (mode === 'dark') document.body.className = 'dark';
            
            document.getElementById('btn-light').classList.toggle('active', config.theme_mode === 'light');
            document.getElementById('btn-dark').classList.toggle('active', config.theme_mode === 'dark');
            document.getElementById('btn-system').classList.toggle('active', config.theme_mode === 'system');
            document.getElementById('cb-amoled').disabled = (config.theme_mode === 'light');
        }

        async function saveConfig() {
            config.source_dir = document.getElementById('source-path').value;
            config.backup_dir = document.getElementById('backup-path').value;
            config.selected_files = selectedFiles;
            await fetch('/api/config', {
                method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(config)
            });
        }

        document.getElementById('source-path').addEventListener('change', saveConfig);
        document.getElementById('backup-path').addEventListener('change', saveConfig);

        async function pickFolder(type) {
            const res = await fetch('/api/pick_folder', { method: 'POST' });
            const data = await res.json();
            if (data.path) {
                if (type === 'source') document.getElementById('source-path').value = data.path;
                if (type === 'backup') document.getElementById('backup-path').value = data.path;
                saveConfig();
            }
        }

        async function openFolder(type) {
            let path = type === 'source' ? document.getElementById('source-path').value : document.getElementById('backup-path').value;
            if (!path) return log(t('msg_err_pick'), "error");
            const res = await fetch('/api/open_folder', {
                method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ path: path, type: type })
            });
            const data = await res.json();
            if (data.error) log("❌ " + data.error, "error");
            else log(t('log_open') + data.path, "muted");
        }

        async function doAction(direction) {
            const source = document.getElementById('source-path').value;
            const backup = document.getElementById('backup-path').value;
            if (!source || !backup) return log(t('msg_err_paths'), "error");

            log("─".repeat(60), "muted");
            const title = direction === 'save' ? t('log_save_t') : t('log_load_t');
            log(`[${new Date().toLocaleString()}]  ${title}`, "header");

            const res = await fetch('/api/action', {
                method: 'POST', headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ direction: direction, source: source, backup: backup, files: Object.keys(selectedFiles).filter(k => selectedFiles[k]) })
            });
            const data = await res.json();
            if (data.error) return log("❌ " + data.error, "error");

            log(`${t('log_copy_src')} ${data.src_dir}`, "muted");
            log(`${t('log_copy_dst')} ${data.dst_dir}`, "muted");

            if (data.succeeded.length) {
                log(t('log_success', {n: data.succeeded.length}), "success");
                data.succeeded.forEach(f => log(`   • ${f}`, "success"));
            }
            if (data.skipped.length) {
                log(t('log_skip', {n: data.skipped.length}), "warning");
                data.skipped.forEach(item => log(`   • ${item[0]} — ${item[1]}`, "warning"));
            }
            if (data.failed.length) {
                log(t('log_fail', {n: data.failed.length}), "error");
                data.failed.forEach(item => log(`   • ${item[0]} — ${item[1]}`, "error"));
            }
        }

        async function exitApp() {
            if(confirm(t('btn_exit') + "?")) {
                await fetch('/api/exit');
                document.body.innerHTML = "<div style='text-align:center; padding:50px; font-family:sans-serif;'><h1>Server stopped</h1><p>You can close this tab.</p></div>";
            }
        }

        function log(msg, type='') {
            const area = document.getElementById('log');
            const span = document.createElement('span');
            span.className = 'log-' + type;
            span.innerText = msg + "\\n";
            area.appendChild(span);
            area.scrollTop = area.scrollHeight;
        }

        init();
    </script>
</body>
</html>
"""

class WebHandler(http.server.BaseHTTPRequestHandler):
    allow_reuse_address = True
    def log_message(self, format, *args): pass

    def do_GET(self):
        if self.path == '/':
            self.send_response(200); self.send_header('Content-Type', 'text/html; charset=utf-8'); self.end_headers()
            self.wfile.write(HTML_CONTENT.encode('utf-8'))
        elif self.path == '/api/config':
            self._json_response(load_config())
        else: self.send_error(404)

    def do_POST(self):
        global LAST_PING
        length = int(self.headers.get('Content-Length', 0))
        post_data = self.rfile.read(length) if length else b'{}'
        if self.path == '/api/ping':
            LAST_PING = time.time(); self._json_response({"status": "ok"}); return
        try: data = json.loads(post_data.decode('utf-8'))
        except: data = {}

        if self.path == '/api/config':
            save_config(data); self._json_response({"status": "ok"})
        elif self.path == '/api/pick_folder':
            root = tk.Tk(); root.withdraw(); root.attributes('-topmost', 1)
            path = filedialog.askdirectory(); root.destroy()
            self._json_response({"path": path})
        elif self.path == '/api/open_folder':
            path = data.get('path', ''); ftype = data.get('type', '')
            if not path: return self._json_response({"error": "Path not specified."})
            path = os.path.normpath(path)
            if ftype == 'backup' and os.path.basename(path) != BACKUP_FOLDER_NAME: path = os.path.join(path, BACKUP_FOLDER_NAME)
            if not os.path.isdir(path):
                if ftype == 'backup':
                    try: os.makedirs(path, exist_ok=True)
                    except Exception as e: return self._json_response({"error": f"Failed to create folder: {e}"})
                else: return self._json_response({"error": "Folder does not exist."})
            try:
                if os.name == "nt": subprocess.Popen(["explorer.exe", "/n," + path])
                elif sys.platform == "darwin": subprocess.Popen(["open", "-n", path])
                else: subprocess.Popen(["xdg-open", path])
                self._json_response({"path": path})
            except Exception as e: self._json_response({"error": str(e)})
        elif self.path == '/api/action':
            direction = data.get('direction'); source = data.get('source', '').strip()
            backup = data.get('backup', '').strip(); files = data.get('files', [])
            if not backup.endswith(BACKUP_FOLDER_NAME): backup = os.path.join(backup, BACKUP_FOLDER_NAME)
            src_dir, dst_dir = (source, backup) if direction == 'save' else (backup, source)
            if not os.path.isdir(src_dir): return self._json_response({"error": f"Source folder does not exist:\\n{src_dir}"})
            if not os.path.isdir(dst_dir):
                if direction == 'save':
                    try: os.makedirs(dst_dir, exist_ok=True)
                    except Exception as e: return self._json_response({"error": f"Failed to create folder:\\n{dst_dir}\\n\\n{e}"})
                else: return self._json_response({"error": f"Backup folder does not exist:\\n{dst_dir}"})
            succeeded, skipped, failed = [], [], []
            for filename in files:
                src_path, dst_path = os.path.join(src_dir, filename), os.path.join(dst_dir, filename)
                if not os.path.isfile(src_path): skipped.append((filename, "file not found in source")); continue
                try: shutil.copy2(src_path, dst_path); succeeded.append(filename)
                except PermissionError as e: failed.append((filename, f"permission denied (file in use?): {e}"))
                except Exception as e: failed.append((filename, f"error: {e}"))
            self._json_response({"succeeded": succeeded, "skipped": skipped, "failed": failed, "src_dir": src_dir, "dst_dir": dst_dir})
        elif self.path == '/api/exit':
            self._json_response({"status": "ok"}); threading.Thread(target=self.server.shutdown, daemon=True).start()
        else: self.send_error(404)

    def _json_response(self, obj):
        self.send_response(200); self.send_header('Content-Type', 'application/json; charset=utf-8'); self.send_header('Access-Control-Allow-Origin', '*'); self.end_headers()
        self.wfile.write(json.dumps(obj, ensure_ascii=False).encode('utf-8'))

def run_server():
    global LAST_PING, SERVER_START_TIME
    httpd = None
    port_to_use = 8042
    for port in range(8042, 8100):
        try:
            httpd = socketserver.TCPServer(('127.0.0.1', port), WebHandler); port_to_use = port; break
        except OSError: continue
    if not httpd: raise RuntimeError("No free ports.")

    def watchdog():
        while True:
            time.sleep(0.1)
            if time.time() - LAST_PING > 1.0 and time.time() - SERVER_START_TIME > 5.0:
                httpd.shutdown(); break

    threading.Thread(target=watchdog, daemon=True).start()
    LAST_PING = time.time(); SERVER_START_TIME = time.time()
    threading.Timer(0.5, lambda: webbrowser.open(f'http://127.0.0.1:{port_to_use}')).start()
    try: httpd.serve_forever()
    except KeyboardInterrupt: pass
    finally: httpd.server_close()

if __name__ == "__main__":
    run_server()