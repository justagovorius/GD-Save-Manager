#![windows_subsystem = "windows"]

use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::Notify;

const BACKUP_FOLDER_NAME: &str = "GD_Backups";

// ============================================================ CONFIG
#[derive(Serialize, Deserialize, Clone)]
struct Config {
    #[serde(default = "default_theme")]
    theme_mode: String,
    #[serde(default)]
    amoled: bool,
    #[serde(default = "default_lang")]
    language: String,
    #[serde(default)]
    source_dir: String,
    #[serde(default)]
    backup_dir: String,
    #[serde(default)]
    selected_files: HashMap<String, bool>,
}

fn default_theme() -> String { "system".into() }
fn default_lang() -> String { "system".into() }

impl Default for Config {
    fn default() -> Self {
        Config {
            theme_mode: "system".into(),
            amoled: false,
            language: "system".into(),
            source_dir: String::new(),
            backup_dir: String::new(),
            selected_files: HashMap::new(),
        }
    }
}

fn config_dir() -> PathBuf {
    dirs::data_local_dir().unwrap_or_else(|| PathBuf::from(".")).join("GD_SaveManager")
}

fn config_path() -> PathBuf { config_dir().join("config.json") }

fn load_config() -> Config {
    fs::read_to_string(config_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_config(cfg: &Config) {
    let _ = fs::create_dir_all(config_dir());
    if let Ok(s) = serde_json::to_string_pretty(cfg) {
        let _ = fs::write(config_path(), s);
    }
}

fn open_folder_sync(path: &Path) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        Command::new("explorer").raw_arg(format!("/n,\"{}\"", path.display())).spawn()?;
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg("-n").arg(path).spawn()?;
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Command::new("xdg-open").arg(path).spawn()?;
    }
    Ok(())
}

// ============================================================ STATE
#[derive(Clone)]
struct AppState {
    last_ping: Arc<Mutex<Instant>>,
    start_time: Instant,
    shutdown_notify: Arc<Notify>,
}

// ============================================================ HTML
const HTML_CONTENT: &str = r#"<!DOCTYPE html>
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
                lang: "Язык:", lang_sys: "Системный", theme: "Тема:", theme_light: "Светлая", theme_dark: "Тёмная", theme_sys: "Системная",
                src_title: "📁 Папка с сохранениями игры", src_sub: "по умолчанию: %LOCALAPPDATA%\\GeometryDash", btn_pick: "Выбрать", btn_open: "Обзор",
                bk_title: "💾 Папка GD_Backups", bk_sub: "место, где будут храниться резервные копии",
                files_title: "📄 Какие файлы задействовать", files_sub: "поставь галочку у файлов, которые нужно копировать", btn_all: "Выбрать все", btn_none: "Снять все",
                btn_save: "💾 СОХРАНИТЬ (игра → копия)", btn_load: "📂 ЗАГРУЗИТЬ (копия → игра)", log_title: "📝 Журнал операций",
                f_cc1: "Основной прогресс игры", f_cc2: "Дубликат прогресса", f_cl1: "Локальные уровни", f_cl2: "Дубликат уровней",
                f_sfx: "Библиотека звуков", f_mus: "Библиотека музыки", f_plat: "Сохранение платформера",
                log_src: "Папка игры:   ", log_dst: "Папка бэкапов:", log_open: "📁 Открыта папка: ",
                log_copy_src: "   Откуда: ", log_copy_dst: "   Куда:   ", log_save_t: "СОХРАНЕНИЕ (игра → копия)", log_load_t: "ЗАГРУЗКА (копия → игра)",
                log_success: "\n✅ Успешно скопировано ({n}):", log_skip: "\n⚠️  Пропущено ({n}):", log_fail: "\n❌ Ошибки копирования ({n}):",
                msg_err_pick: "❌ Сначала укажите папку.", msg_err_paths: "❌ Укажите обе папки."
            },
            "en": {
                app_title: "GD Save Manager", app_sub: "Local backup tool for GD and GDPS saves",
                lang: "Language:", lang_sys: "System", theme: "Theme:", theme_light: "Light", theme_dark: "Dark", theme_sys: "System",
                src_title: "📁 Game Save Folder", src_sub: "default: %LOCALAPPDATA%\\GeometryDash", btn_pick: "Select", btn_open: "Browse",
                bk_title: "💾 GD_Backups Folder", bk_sub: "where backup files will be stored",
                files_title: "📄 Files to include", files_sub: "check the files you want to copy", btn_all: "Select All", btn_none: "Deselect All",
                btn_save: "💾 SAVE (game → backup)", btn_load: "📂 LOAD (backup → game)", log_title: "📝 Operation Log",
                f_cc1: "Main game progress", f_cc2: "Progress backup", f_cl1: "Local levels", f_cl2: "Levels backup",
                f_sfx: "Sound library", f_mus: "Music library", f_plat: "Platformer save",
                log_src: "Game folder:   ", log_dst: "Backup folder: ", log_open: "📁 Opened folder: ",
                log_copy_src: "   From: ", log_copy_dst: "   To:   ", log_save_t: "SAVE (game → backup)", log_load_t: "LOAD (backup → game)",
                log_success: "\n✅ Successfully copied ({n}):", log_skip: "\n⚠️  Skipped ({n}):", log_fail: "\n❌ Copy errors ({n}):",
                msg_err_pick: "❌ Please select a folder first.", msg_err_paths: "❌ Please specify both folders."
            }
        };

        const fileDescriptions = {
            "CCGameManager.dat": "f_cc1", "CCGameManager2.dat": "f_cc2", "CCLocalLevels.dat": "f_cl1", "CCLocalLevels2.dat": "f_cl2",
            "sfxlibrary.dat": "f_sfx", "musiclibrary.dat": "f_mus", "CCPlatformerSave.dat": "f_plat"
        };

        const SYSTEM_LANG = navigator.language.startsWith('ru') ? 'ru' : 'en';
        const SYSTEM_THEME = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';

        let config = {};
        let selectedFiles = {};
        let currentLang = "ru";

        setInterval(() => { fetch('/api/ping', { method: 'POST' }).catch(e => {}); }, 100);

        async function init() {
            try {
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
            } catch (e) {
                log("❌ Ошибка загрузки конфигурации: " + e.message, "error");
            }
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
            if (mode === 'system') mode = SYSTEM_THEME;
            
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
            try {
                await fetch('/api/config', {
                    method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(config)
                });
            } catch (e) {}
        }

        document.getElementById('source-path').addEventListener('change', saveConfig);
        document.getElementById('backup-path').addEventListener('change', saveConfig);

        async function pickFolder(type) {
            try {
                const res = await fetch('/api/pick_folder', { method: 'POST' });
                const data = await res.json();
                if (data.path) {
                    if (type === 'source') document.getElementById('source-path').value = data.path;
                    if (type === 'backup') document.getElementById('backup-path').value = data.path;
                    saveConfig();
                }
            } catch (e) {
                log("❌ Ошибка выбора папки: " + e.message, "error");
            }
        }

        async function openFolder(type) {
            let path = type === 'source' ? document.getElementById('source-path').value : document.getElementById('backup-path').value;
            if (!path) return log(t('msg_err_pick'), "error");
            try {
                const res = await fetch('/api/open_folder', {
                    method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ path: path, type: type })
                });
                const data = await res.json();
                if (data.error) log("❌ " + data.error, "error");
                else log(t('log_open') + data.path, "muted");
            } catch (e) {
                log("❌ Ошибка: " + e.message, "error");
            }
        }

        async function doAction(direction) {
            const source = document.getElementById('source-path').value;
            const backup = document.getElementById('backup-path').value;
            if (!source || !backup) return log(t('msg_err_paths'), "error");

            log("─".repeat(60), "muted");
            const title = direction === 'save' ? t('log_save_t') : t('log_load_t');
            log(`[${new Date().toLocaleString()}]  ${title}`, "header");

            try {
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
            } catch (e) {
                log("❌ Ошибка: " + e.message, "error");
            }
        }

        function log(msg, type='') {
            const area = document.getElementById('log');
            const span = document.createElement('span');
            span.className = 'log-' + type;
            span.innerText = msg + "\n";
            area.appendChild(span);
            area.scrollTop = area.scrollHeight;
        }

        init();
    </script>
</body>
</html>
"#;

// ============================================================ HANDLERS
async fn serve_html() -> impl IntoResponse {
    Html(HTML_CONTENT)
}

async fn get_config() -> impl IntoResponse {
    Json(load_config())
}

async fn post_config(Json(cfg): Json<Config>) -> impl IntoResponse {
    save_config(&cfg);
    Json(serde_json::json!({ "status": "ok" }))
}

async fn ping(State(state): State<AppState>) -> impl IntoResponse {
    *state.last_ping.lock().unwrap() = Instant::now();
    Json(serde_json::json!({ "status": "ok" }))
}

async fn pick_folder() -> impl IntoResponse {
    let res = tokio::task::spawn_blocking(|| {
        rfd::FileDialog::new().pick_folder()
    }).await.unwrap();

    if let Some(p) = res {
        Json(serde_json::json!({ "path": p.to_string_lossy().to_string() }))
    } else {
        Json(serde_json::json!({ "path": null }))
    }
}

#[derive(Deserialize)]
struct OpenFolderReq {
    path: String,
    #[serde(rename = "type")]
    ftype: String,
}

async fn open_folder(Json(req): Json<OpenFolderReq>) -> impl IntoResponse {
    if req.path.is_empty() {
        return Json(serde_json::json!({ "error": "Path not specified." }));
    }
    let mut p = PathBuf::from(req.path);
    if req.ftype == "backup" && p.file_name().and_then(|n| n.to_str()) != Some(BACKUP_FOLDER_NAME) {
        p = p.join(BACKUP_FOLDER_NAME);
    }
    if !p.is_dir() {
        if req.ftype == "backup" {
            if let Err(e) = fs::create_dir_all(&p) {
                return Json(serde_json::json!({ "error": format!("Failed to create folder: {}", e) }));
            }
        } else {
            return Json(serde_json::json!({ "error": "Folder does not exist." }));
        }
    }
    let p_clone = p.clone();
    let res = tokio::task::spawn_blocking(move || open_folder_sync(&p_clone)).await.unwrap();
    match res {
        Ok(_) => Json(serde_json::json!({ "path": p.to_string_lossy().to_string() })),
        Err(e) => Json(serde_json::json!({ "error": format!("{}", e) })),
    }
}

#[derive(Deserialize)]
struct ActionReq {
    direction: String,
    source: String,
    backup: String,
    files: Vec<String>,
}

async fn do_action(Json(req): Json<ActionReq>) -> impl IntoResponse {
    let mut backup = req.backup.trim().to_string();
    let is_backup = Path::new(&backup).file_name().and_then(|n| n.to_str()) == Some(BACKUP_FOLDER_NAME);
    if !is_backup {
        backup = Path::new(&backup).join(BACKUP_FOLDER_NAME).to_string_lossy().to_string();
    }
    let source = req.source.trim().to_string();

    let (src_dir, dst_dir) = if req.direction == "save" {
        (source, backup)
    } else {
        (backup, source)
    };

    if !Path::new(&src_dir).is_dir() {
        return Json(serde_json::json!({ "error": format!("Source folder does not exist:\n{}", src_dir) }));
    }
    if !Path::new(&dst_dir).is_dir() {
        if req.direction == "save" {
            if let Err(e) = fs::create_dir_all(&dst_dir) {
                return Json(serde_json::json!({ "error": format!("Failed to create folder:\n{}\n\n{}", dst_dir, e) }));
            }
        } else {
            return Json(serde_json::json!({ "error": format!("Backup folder does not exist:\n{}", dst_dir) }));
        }
    }

    let (src_dir, dst_dir) = (PathBuf::from(src_dir), PathBuf::from(dst_dir));
    let src_dir_str = src_dir.to_string_lossy().to_string();
    let dst_dir_str = dst_dir.to_string_lossy().to_string();

    let res = tokio::task::spawn_blocking(move || {
        let mut ok = Vec::new();
        let mut skip = Vec::new();
        let mut fail = Vec::new();
        for f in req.files {
            let s = src_dir.join(&f);
            let d = dst_dir.join(&f);
            if !s.is_file() {
                skip.push((f, "file not found in source".to_string()));
                continue;
            }
            match fs::copy(&s, &d) {
                Ok(_) => ok.push(f),
                Err(e) => {
                    let msg = if e.kind() == std::io::ErrorKind::PermissionDenied {
                        format!("permission denied (file in use?): {}", e)
                    } else {
                        format!("error: {}", e)
                    };
                    fail.push((f, msg));
                }
            }
        }
        (ok, skip, fail)
    }).await.unwrap();

    Json(serde_json::json!({
        "succeeded": res.0,
        "skipped": res.1,
        "failed": res.2,
        "src_dir": src_dir_str,
        "dst_dir": dst_dir_str
    }))
}

// ============================================================ MAIN
#[tokio::main]
async fn main() {
    let state = AppState {
        last_ping: Arc::new(Mutex::new(Instant::now())),
        start_time: Instant::now(),
        shutdown_notify: Arc::new(Notify::new()),
    };

    let app = Router::new()
        .route("/", get(serve_html))
        .route("/api/config", get(get_config).post(post_config))
        .route("/api/pick_folder", post(pick_folder))
        .route("/api/open_folder", post(open_folder))
        .route("/api/action", post(do_action))
        .route("/api/ping", post(ping))
        .with_state(state.clone());

    // Ищем свободный порт
    let mut listener = None;
    let mut port = 8042;
    for p in 8042..8100 {
        if let Ok(l) = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", p)).await {
            listener = Some(l);
            port = p;
            break;
        }
    }
    let listener = listener.expect("Не удалось найти свободный порт.");

    // Сторож (watchdog)
    let state_watchdog = state.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(100)).await;
            let last = *state_watchdog.last_ping.lock().unwrap();
            if last.elapsed() > Duration::from_secs(1) && state_watchdog.start_time.elapsed() > Duration::from_secs(5) {
                state_watchdog.shutdown_notify.notify_waiters();
                break;
            }
        }
    });

    // Открываем браузер
    let url = format!("http://127.0.0.1:{}", port);
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(500)).await;
        let _ = webbrowser::open(&url);
    });

    // Запускаем сервер
    let graceful = async move {
        state.shutdown_notify.notified().await;
    };

    axum::serve(listener, app)
        .with_graceful_shutdown(graceful)
        .await
        .unwrap();
}