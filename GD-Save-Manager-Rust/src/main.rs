#![windows_subsystem = "windows"]

use eframe::egui::{self, Color32, RichText, Stroke};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const BACKUP_FOLDER_NAME: &str = "GD_Backups";

const SAVE_FILES: &[(&str, &str)] = &[
    ("CCGameManager.dat", "f_cc1"),
    ("CCGameManager2.dat", "f_cc2"),
    ("CCLocalLevels.dat", "f_cl1"),
    ("CCLocalLevels2.dat", "f_cl2"),
    ("sfxlibrary.dat", "f_sfx"),
    ("musiclibrary.dat", "f_mus"),
    ("CCPlatformerSave.dat", "f_plat"),
];

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
        let mut selected = HashMap::new();
        for (name, _) in SAVE_FILES {
            selected.insert(name.to_string(), true);
        }
        Config {
            theme_mode: "system".into(),
            amoled: false,
            language: "system".into(),
            source_dir: default_source_dir(),
            backup_dir: default_backup_dir(),
            selected_files: selected,
        }
    }
}

fn config_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("GD_SaveManager")
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

fn default_source_dir() -> String {
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        return PathBuf::from(local).join("GeometryDash").to_string_lossy().to_string();
    }
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join("AppData").join("Local").join("GeometryDash")
        .to_string_lossy().to_string()
}

fn default_backup_dir() -> String {
    let docs = dirs::document_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")));
    docs.join(BACKUP_FOLDER_NAME).to_string_lossy().to_string()
}

// ============================================================ SYSTEM DETECTION
#[cfg(target_os = "windows")]
fn get_system_theme() -> String {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(key) = hkcu.open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize") {
        if let Ok(value) = key.get_value::<u32, _>("AppsUseLightTheme") {
            return if value == 1 { "light".into() } else { "dark".into() };
        }
    }
    "light".into()
}

#[cfg(target_os = "macos")]
fn get_system_theme() -> String {
    if let Ok(out) = Command::new("defaults").args(["read", "-g", "AppleInterfaceStyle"]).output() {
        if String::from_utf8_lossy(&out.stdout).to_lowercase().contains("dark") {
            return "dark".into();
        }
    }
    "light".into()
}

#[cfg(all(unix, not(target_os = "macos")))]
fn get_system_theme() -> String {
    if let Ok(out) = Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "color-scheme"])
        .output()
    {
        if String::from_utf8_lossy(&out.stdout).to_lowercase().contains("dark") {
            return "dark".into();
        }
    }
    "light".into()
}

fn get_system_language() -> String {
    let loc = sys_locale::get_locale().unwrap_or_default();
    if loc.to_lowercase().starts_with("ru") { "ru".into() } else { "en".into() }
}

fn open_folder(path: &Path) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        Command::new("explorer")
            .raw_arg(format!("/n,\"{}\"", path.display()))
            .spawn()?;
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

// ============================================================ FONTS
fn load_system_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    #[cfg(target_os = "windows")]
    let paths: [&str; 2] = ["C:\\Windows\\Fonts\\segoeui.ttf", "C:\\Windows\\Fonts\\arial.ttf"];
    #[cfg(target_os = "macos")]
    let paths: [&str; 2] = ["/Library/Fonts/Arial.ttf", "/System/Library/Fonts/Supplemental/Arial.ttf"];
    #[cfg(all(unix, not(target_os = "macos")))]
    let paths: [&str; 2] = ["/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", "/usr/share/fonts/TTF/DejaVuSans.ttf"];

    for path in paths {
        if let Ok(bytes) = fs::read(path) {
            fonts.font_data.insert(
                "sys_font".to_owned(),
                egui::FontData::from_owned(bytes),
            );
            if let Some(f) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                f.insert(0, "sys_font".to_owned());
            }
            if let Some(f) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                f.push("sys_font".to_owned());
            }
            ctx.set_fonts(fonts);
            return;
        }
    }
}

// ============================================================ TRANSLATIONS
struct Strings {
    title: &'static str,
    subtitle: &'static str,
    lang_label: &'static str,
    lang_ru: &'static str,
    lang_en: &'static str,
    lang_sys: &'static str,
    theme_label: &'static str,
    theme_light: &'static str,
    theme_dark: &'static str,
    theme_sys: &'static str,
    src_title: &'static str,
    src_sub: &'static str,
    bk_title: &'static str,
    bk_sub: &'static str,
    btn_pick: &'static str,
    btn_open: &'static str,
    btn_save: &'static str,
    btn_load: &'static str,
    files_title: &'static str,
    files_sub: &'static str,
    btn_all: &'static str,
    btn_none: &'static str,
    log_title: &'static str,
    f_cc1: &'static str,
    f_cc2: &'static str,
    f_cl1: &'static str,
    f_cl2: &'static str,
    f_sfx: &'static str,
    f_mus: &'static str,
    f_plat: &'static str,
    log_src: &'static str,
    log_dst: &'static str,
    log_open: &'static str,
    log_copy_src: &'static str,
    log_copy_dst: &'static str,
    log_success: &'static str,
    log_skip: &'static str,
    log_fail: &'static str,
    log_save_t: &'static str,
    log_load_t: &'static str,
    msg_err: &'static str,
    msg_warn: &'static str,
    msg_done: &'static str,
    msg_paths: &'static str,
    msg_nofiles: &'static str,
    err_src_no: &'static str,
    err_dst_no: &'static str,
    err_create: &'static str,
    err_no_dir: &'static str,
    err_cant_open: &'static str,
    skip_reason: &'static str,
    err_permission: &'static str,
    err_io: &'static str,
}

const STRINGS_RU: Strings = Strings {
    title: "GD Save Manager",
    subtitle: "Локальное резервное копирование сохранений GD и GDPS",
    lang_label: "Язык:",
    lang_ru: "Русский",
    lang_en: "English",
    lang_sys: "Системный",
    theme_label: "Тема:",
    theme_light: "Светлая",
    theme_dark: "Тёмная",
    theme_sys: "Системная",
    src_title: "📁  Папка с сохранениями игры",
    src_sub: "по умолчанию: %LOCALAPPDATA%\\GeometryDash",
    bk_title: "💾  Папка GD_Backups",
    bk_sub: "место, где будут храниться резервные копии",
    btn_pick: "Выбрать",
    btn_open: "Обзор",
    btn_save: "💾   СОХРАНИТЬ  (игра → копия)",
    btn_load: "📂   ЗАГРУЗИТЬ  (копия → игра)",
    files_title: "📄  Какие файлы задействовать",
    files_sub: "поставь галочку у файлов, которые нужно копировать",
    btn_all: "Выбрать все",
    btn_none: "Снять все",
    log_title: "📝  Журнал операций",
    f_cc1: "Основной прогресс игры",
    f_cc2: "Дубликат прогресса",
    f_cl1: "Локальные уровни",
    f_cl2: "Дубликат уровней",
    f_sfx: "Библиотека звуков",
    f_mus: "Библиотека музыки",
    f_plat: "Сохранение платформера",
    log_src: "Папка игры:   ",
    log_dst: "Папка бэкапов:",
    log_open: "📁 Открыта папка: ",
    log_copy_src: "   Откуда: ",
    log_copy_dst: "   Куда:   ",
    log_success: "✅ Успешно скопировано ({n}):",
    log_skip: "⚠️  Пропущено ({n}):",
    log_fail: "❌ Ошибки копирования ({n}):",
    log_save_t: "СОХРАНЕНИЕ (игра → копия)",
    log_load_t: "ЗАГРУЗКА (копия → игра)",
    msg_err: "Ошибка",
    msg_warn: "Внимание",
    msg_done: "Готово",
    msg_paths: "Укажите обе папки: с сохранениями игры и для резервных копий.",
    msg_nofiles: "Не выбрано ни одного файла для операции.",
    err_src_no: "Папка-источник не существует:",
    err_dst_no: "Папка с резервными копиями не существует:",
    err_create: "Не удалось создать папку:",
    err_no_dir: "Папка не существует: ",
    err_cant_open: "Не удалось открыть папку: ",
    skip_reason: "файл отсутствует в папке-источнике",
    err_permission: "нет прав на запись (возможно, файл занят игрой): ",
    err_io: "ошибка ввода-вывода: ",
};

const STRINGS_EN: Strings = Strings {
    title: "GD Save Manager",
    subtitle: "Local backup tool for GD and GDPS saves",
    lang_label: "Language:",
    lang_ru: "Русский",
    lang_en: "English",
    lang_sys: "System",
    theme_label: "Theme:",
    theme_light: "Light",
    theme_dark: "Dark",
    theme_sys: "System",
    src_title: "📁  Game Save Folder",
    src_sub: "default: %LOCALAPPDATA%\\GeometryDash",
    bk_title: "💾  GD_Backups Folder",
    bk_sub: "where backup files will be stored",
    btn_pick: "Select",
    btn_open: "Browse",
    btn_save: "💾   SAVE  (game → backup)",
    btn_load: "📂   LOAD  (backup → game)",
    files_title: "📄  Files to include",
    files_sub: "check the files you want to copy",
    btn_all: "Select All",
    btn_none: "Deselect All",
    log_title: "📝  Operation Log",
    f_cc1: "Main game progress",
    f_cc2: "Progress backup",
    f_cl1: "Local levels",
    f_cl2: "Levels backup",
    f_sfx: "Sound library",
    f_mus: "Music library",
    f_plat: "Platformer save",
    log_src: "Game folder:   ",
    log_dst: "Backup folder: ",
    log_open: "📁 Opened folder: ",
    log_copy_src: "   From: ",
    log_copy_dst: "   To:   ",
    log_success: "✅ Successfully copied ({n}):",
    log_skip: "⚠️  Skipped ({n}):",
    log_fail: "❌ Copy errors ({n}):",
    log_save_t: "SAVE (game → backup)",
    log_load_t: "LOAD (backup → game)",
    msg_err: "Error",
    msg_warn: "Warning",
    msg_done: "Done",
    msg_paths: "Please specify both folders: game saves and backups.",
    msg_nofiles: "No files selected for the operation.",
    err_src_no: "Source folder does not exist:",
    err_dst_no: "Backup folder does not exist:",
    err_create: "Failed to create folder:",
    err_no_dir: "Folder does not exist: ",
    err_cant_open: "Failed to open folder: ",
    skip_reason: "file not found in source folder",
    err_permission: "permission denied (file might be in use): ",
    err_io: "I/O error: ",
};

fn get_strings(lang: &str) -> &'static Strings {
    match lang {
        "ru" => &STRINGS_RU,
        _ => &STRINGS_EN,
    }
}

// ============================================================ LOG
#[derive(Clone, Copy, PartialEq)]
enum LogLevel { Success, Error, Warning, Header, Muted, Normal }

struct LogEntry {
    text: String,
    level: LogLevel,
}

// ============================================================ PALETTE
#[derive(Clone, Copy)]
struct Palette {
    bg: Color32, card: Color32, border: Color32, text: Color32, muted: Color32,
    primary: Color32, success: Color32, danger: Color32, warning: Color32,
    input_bg: Color32, chip: Color32,
}

fn palette(name: &str) -> Palette {
    match name {
        "light" => Palette {
            bg: Color32::from_rgb(0xee, 0xf1, 0xf5),
            card: Color32::from_rgb(0xff, 0xff, 0xff),
            border: Color32::from_rgb(0xd8, 0xde, 0xe6),
            text: Color32::from_rgb(0x1f, 0x29, 0x37),
            muted: Color32::from_rgb(0x6b, 0x72, 0x80),
            primary: Color32::from_rgb(0x25, 0x63, 0xeb),
            success: Color32::from_rgb(0x16, 0xa3, 0x4a),
            danger: Color32::from_rgb(0xdc, 0x26, 0x26),
            warning: Color32::from_rgb(0xd9, 0x77, 0x06),
            input_bg: Color32::from_rgb(0xf9, 0xfa, 0xfb),
            chip: Color32::from_rgb(0xe5, 0xe7, 0xeb),
        },
        "dark" => Palette {
            bg: Color32::from_rgb(0x1a, 0x1d, 0x24),
            card: Color32::from_rgb(0x25, 0x29, 0x32),
            border: Color32::from_rgb(0x35, 0x3a, 0x45),
            text: Color32::from_rgb(0xe5, 0xe7, 0xeb),
            muted: Color32::from_rgb(0x9c, 0xa3, 0xaf),
            primary: Color32::from_rgb(0x3b, 0x82, 0xf6),
            success: Color32::from_rgb(0x22, 0xc5, 0x5e),
            danger: Color32::from_rgb(0xef, 0x44, 0x44),
            warning: Color32::from_rgb(0xf5, 0x9e, 0x0b),
            input_bg: Color32::from_rgb(0x1e, 0x22, 0x2a),
            chip: Color32::from_rgb(0x35, 0x3a, 0x45),
        },
        _ => Palette { // amoled
            bg: Color32::BLACK,
            card: Color32::from_rgb(0x0a, 0x0a, 0x0a),
            border: Color32::from_rgb(0x1c, 0x1c, 0x1c),
            text: Color32::from_rgb(0xe5, 0xe7, 0xeb),
            muted: Color32::from_rgb(0x9c, 0xa3, 0xaf),
            primary: Color32::from_rgb(0x3b, 0x82, 0xf6),
            success: Color32::from_rgb(0x22, 0xc5, 0x5e),
            danger: Color32::from_rgb(0xef, 0x44, 0x44),
            warning: Color32::from_rgb(0xf5, 0x9e, 0x0b),
            input_bg: Color32::BLACK,
            chip: Color32::from_rgb(0x1a, 0x1a, 0x1a),
        },
    }
}

fn visuals_for(theme: &str) -> egui::Visuals {
    let p = palette(theme);
    let mut v = if theme == "light" { egui::Visuals::light() } else { egui::Visuals::dark() };
    v.panel_fill = p.bg;
    v.window_fill = p.card;
    v.extreme_bg_color = p.input_bg;
    v.faint_bg_color = p.chip;
    v.override_text_color = Some(p.text);
    v.widgets.noninteractive.bg_fill = p.card;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, p.border);
    v.widgets.inactive.bg_fill = p.input_bg;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, p.border);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, p.text);
    v.widgets.hovered.bg_fill = p.chip;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, p.primary);
    v.widgets.hovered.fg_stroke = Stroke::new(1.5_f32, p.text);
    v.widgets.active.bg_fill = p.primary;
    v.widgets.active.bg_stroke = Stroke::new(1.0_f32, p.primary);
    v.widgets.active.fg_stroke = Stroke::new(1.5_f32, Color32::WHITE);
    v.selection.bg_fill = p.primary;
    v.selection.stroke = Stroke::new(1.0_f32, p.primary);
    v.hyperlink_color = p.primary;
    v
}

// ============================================================ APP
struct App {
    config: Config,
    sys_theme: String,
    sys_lang: String,
    logs: Vec<LogEntry>,
    applied_theme: Option<String>,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        load_system_fonts(&cc.egui_ctx);
        let config = load_config();
        let sys_theme = get_system_theme();
        let sys_lang = get_system_language();
        let strings = get_strings(if config.language == "system" { &sys_lang } else { &config.language });
        let logs = vec![
            LogEntry { text: format!("{} {}", strings.log_src, config.source_dir), level: LogLevel::Muted },
            LogEntry { text: format!("{} {}", strings.log_dst, config.backup_dir), level: LogLevel::Muted },
        ];
        Self { config, sys_theme, sys_lang, logs, applied_theme: None }
    }

    fn resolved_lang(&self) -> &str {
        if self.config.language == "system" { &self.sys_lang } else { &self.config.language }
    }

    fn resolved_theme(&self) -> &'static str {
        let mode = self.config.theme_mode.as_str();
        let r: &str = if mode == "system" { self.sys_theme.as_str() } else { mode };
        if r == "dark" && self.config.amoled { "amoled" }
        else {
            match r {
                "dark" => "dark",
                "amoled" => "amoled",
                _ => "light",
            }
        }
    }

    fn strings(&self) -> &'static Strings { get_strings(self.resolved_lang()) }
    
    fn persist_config(&self) {
        save_config(&self.config);
    }

    fn log(&mut self, text: impl Into<String>, level: LogLevel) {
        self.logs.push(LogEntry { text: text.into(), level });
    }

    fn normalize_backup(&mut self) -> String {
        let mut b = self.config.backup_dir.trim().to_string();
        let is_backup = Path::new(&b).file_name()
            .and_then(|n| n.to_str()).map(|n| n == BACKUP_FOLDER_NAME).unwrap_or(false);
        if !is_backup {
            b = Path::new(&b).join(BACKUP_FOLDER_NAME).to_string_lossy().to_string();
        }
        self.config.backup_dir = b.clone();
        b
    }

    fn open_source(&mut self) {
        let s = self.strings();
        let path = self.config.source_dir.trim().to_string();
        if path.is_empty() { return; }
        let p = Path::new(&path);
        if !p.is_dir() {
            self.log(format!("❌ {}{}", s.err_no_dir, path), LogLevel::Error);
            rfd::MessageDialog::new()
                .set_level(rfd::MessageLevel::Error)
                .set_title(s.msg_err)
                .set_description(format!("{}\n{}", s.err_no_dir, path))
                .show();
            return;
        }
        if let Err(e) = open_folder(p) {
            self.log(format!("❌ {}{}: {}", s.err_cant_open, path, e), LogLevel::Error);
        } else {
            self.log(format!("{}{}", s.log_open, path), LogLevel::Muted);
        }
    }

    fn open_backup(&mut self) {
        let s = self.strings();
        let path = self.normalize_backup();
        let p = PathBuf::from(&path);
        if !p.is_dir() {
            if let Err(e) = fs::create_dir_all(&p) {
                self.log(format!("❌ {}{}: {}", s.err_create, path, e), LogLevel::Error);
                return;
            }
        }
        if let Err(e) = open_folder(&p) {
            self.log(format!("❌ {}{}: {}", s.err_cant_open, path, e), LogLevel::Error);
        } else {
            self.log(format!("{}{}", s.log_open, path), LogLevel::Muted);
        }
    }

    fn pick_source(&mut self) {
        if let Some(p) = rfd::FileDialog::new().pick_folder() {
            self.config.source_dir = p.to_string_lossy().to_string();
            self.persist_config();
        }
    }

    fn pick_backup(&mut self) {
        if let Some(p) = rfd::FileDialog::new().pick_folder() {
            let mut path = p;
            let is_backup = path.file_name().and_then(|n| n.to_str())
                .map(|n| n == BACKUP_FOLDER_NAME).unwrap_or(false);
            if !is_backup {
                path = path.join(BACKUP_FOLDER_NAME);
            }
            self.config.backup_dir = path.to_string_lossy().to_string();
            self.persist_config();
        }
    }

    fn do_copy(&mut self, direction: &str) {
        let s = self.strings();
        let source = self.config.source_dir.trim().to_string();
        let backup = self.normalize_backup();

        if source.is_empty() || backup.is_empty() {
            rfd::MessageDialog::new().set_level(rfd::MessageLevel::Warning)
                .set_title(s.msg_warn).set_description(s.msg_paths).show();
            return;
        }

        let selected: Vec<String> = SAVE_FILES.iter()
            .filter(|(name, _)| self.config.selected_files.get(*name).copied().unwrap_or(true))
            .map(|(name, _)| name.to_string())
            .collect();

        if selected.is_empty() {
            rfd::MessageDialog::new().set_level(rfd::MessageLevel::Warning)
                .set_title(s.msg_warn).set_description(s.msg_nofiles).show();
            return;
        }

        let (src_dir, dst_dir, title) = if direction == "save" {
            (source.clone(), backup.clone(), s.log_save_t)
        } else {
            (backup.clone(), source.clone(), s.log_load_t)
        };

        if !Path::new(&src_dir).is_dir() {
            self.log(format!("❌ {}: {}", s.err_src_no, src_dir), LogLevel::Error);
            rfd::MessageDialog::new().set_level(rfd::MessageLevel::Error)
                .set_title(s.msg_err)
                .set_description(format!("{}\n{}", s.err_src_no, src_dir)).show();
            return;
        }

        if !Path::new(&dst_dir).is_dir() {
            if direction == "save" {
                if let Err(e) = fs::create_dir_all(&dst_dir) {
                    self.log(format!("❌ {} {}: {}", s.err_create, dst_dir, e), LogLevel::Error);
                    rfd::MessageDialog::new().set_level(rfd::MessageLevel::Error)
                        .set_title(s.msg_err)
                        .set_description(format!("{}\n{}\n{}", s.err_create, dst_dir, e)).show();
                    return;
                }
            } else {
                self.log(format!("❌ {}: {}", s.err_dst_no, dst_dir), LogLevel::Error);
                rfd::MessageDialog::new().set_level(rfd::MessageLevel::Error)
                    .set_title(s.msg_err)
                    .set_description(format!("{}\n{}", s.err_dst_no, dst_dir)).show();
                return;
            }
        }

        let mut ok = Vec::new();
        let mut skip = Vec::new();
        let mut fail = Vec::new();

        for f in &selected {
            let src = Path::new(&src_dir).join(f);
            let dst = Path::new(&dst_dir).join(f);
            if !src.is_file() {
                skip.push((f.clone(), s.skip_reason.to_string()));
                continue;
            }
            match fs::copy(&src, &dst) {
                Ok(_) => ok.push(f.clone()),
                Err(e) => {
                    let msg = if e.kind() == std::io::ErrorKind::PermissionDenied {
                        format!("{}{}", s.err_permission, e)
                    } else {
                        format!("{}{}", s.err_io, e)
                    };
                    fail.push((f.clone(), msg));
                }
            }
        }

        self.log("─".repeat(60), LogLevel::Muted);
        self.log(format!("[{}]  {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), title), LogLevel::Header);
        self.log(format!("{} {}", s.log_copy_src, src_dir), LogLevel::Muted);
        self.log(format!("{} {}", s.log_copy_dst, dst_dir), LogLevel::Muted);

        if !ok.is_empty() {
            self.log(s.log_success.replace("{n}", &ok.len().to_string()), LogLevel::Success);
            for f in &ok { self.log(format!("   • {}", f), LogLevel::Success); }
        }
        if !skip.is_empty() {
            self.log(s.log_skip.replace("{n}", &skip.len().to_string()), LogLevel::Warning);
            for (f, r) in &skip { self.log(format!("   • {} — {}", f, r), LogLevel::Warning); }
        }
        if !fail.is_empty() {
            self.log(s.log_fail.replace("{n}", &fail.len().to_string()), LogLevel::Error);
            for (f, r) in &fail { self.log(format!("   • {} — {}", f, r), LogLevel::Error); }
        }

        self.persist_config();

        let summary = format!("{}: {}\n{}: {}\n{}: {}",
            if direction == "save" { "OK" } else { "OK" }, ok.len(),
            "Skipped", skip.len(), "Errors", fail.len());
        if !fail.is_empty() {
            rfd::MessageDialog::new().set_level(rfd::MessageLevel::Warning)
                .set_title(s.msg_err).set_description(&summary).show();
        } else if ok.is_empty() && !skip.is_empty() {
            rfd::MessageDialog::new().set_level(rfd::MessageLevel::Warning)
                .set_title(s.msg_warn).set_description(&summary).show();
        } else {
            rfd::MessageDialog::new().set_level(rfd::MessageLevel::Info)
                .set_title(s.msg_done).set_description(&summary).show();
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let theme = self.resolved_theme();
        if self.applied_theme.as_deref() != Some(theme) {
            ctx.set_visuals(visuals_for(theme));
            self.applied_theme = Some(theme.to_string());
        }

        let p = palette(theme);
        let s = self.strings();

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(p.bg).inner_margin(egui::Margin::same(20.0_f32)))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    // ---------------- HEADER ----------------
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new(s.title).size(24.0_f32).strong().color(p.text));
                            ui.label(RichText::new(s.subtitle).size(13.0_f32).color(p.muted));
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                            ui.vertical(|ui| {
                                // language row
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(s.lang_label).size(12.0_f32).color(p.muted));
                                    for (val, label) in [("ru", s.lang_ru), ("en", s.lang_en), ("system", s.lang_sys)] {
                                        if ui.selectable_label(self.config.language == val, label).clicked() {
                                            self.config.language = val.to_string();
                                            self.persist_config();
                                        }
                                    }
                                });
                                // theme row
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(s.theme_label).size(12.0_f32).color(p.muted));
                                    for (val, label) in [("light", s.theme_light), ("dark", s.theme_dark), ("system", s.theme_sys)] {
                                        if ui.selectable_label(self.config.theme_mode == val, label).clicked() {
                                            self.config.theme_mode = val.to_string();
                                            self.persist_config();
                                        }
                                    }
                                    let amoled_enabled = self.resolved_theme() != "light";
                                    let mut amoled = self.config.amoled;
                                    if ui.add_enabled(amoled_enabled, egui::Checkbox::new(&mut amoled, "AMOLED")).changed() {
                                        self.config.amoled = amoled;
                                        self.persist_config();
                                    }
                                });
                            });
                        });
                    });

                    ui.add_space(15.0_f32);

                    // ---------------- SOURCE CARD ----------------
                    egui::Frame::none().fill(p.card).stroke(Stroke::new(1.0_f32, p.border))
                        .rounding(egui::Rounding::same(8.0_f32))
                        .inner_margin(egui::Margin::symmetric(18.0_f32, 14.0_f32))
                        .show(ui, |ui| {
                            ui.label(RichText::new(s.src_title).size(14.0_f32).strong().color(p.text));
                            ui.label(RichText::new(s.src_sub).size(12.0_f32).color(p.muted));
                            ui.add_space(8.0_f32);
                            ui.horizontal(|ui| {
                                ui.add_sized([ui.available_width() - 220.0_f32, 32.0_f32],
                                    egui::TextEdit::singleline(&mut self.config.source_dir));
                                let pick_btn = egui::Button::new(RichText::new(s.btn_pick).color(Color32::WHITE))
                                    .fill(p.primary).rounding(6.0_f32).min_size(egui::vec2(90.0_f32, 32.0_f32));
                                if ui.add(pick_btn).clicked() { self.pick_source(); }
                                let open_btn = egui::Button::new(RichText::new(s.btn_open).color(Color32::WHITE))
                                    .fill(p.muted).rounding(6.0_f32).min_size(egui::vec2(90.0_f32, 32.0_f32));
                                if ui.add(open_btn).clicked() { self.open_source(); }
                            });
                        });
                    ui.add_space(10.0_f32);

                    // ---------------- BACKUP CARD ----------------
                    egui::Frame::none().fill(p.card).stroke(Stroke::new(1.0_f32, p.border))
                        .rounding(egui::Rounding::same(8.0_f32))
                        .inner_margin(egui::Margin::symmetric(18.0_f32, 14.0_f32))
                        .show(ui, |ui| {
                            ui.label(RichText::new(s.bk_title).size(14.0_f32).strong().color(p.text));
                            ui.label(RichText::new(s.bk_sub).size(12.0_f32).color(p.muted));
                            ui.add_space(8.0_f32);
                            ui.horizontal(|ui| {
                                ui.add_sized([ui.available_width() - 220.0_f32, 32.0_f32],
                                    egui::TextEdit::singleline(&mut self.config.backup_dir));
                                let pick_btn = egui::Button::new(RichText::new(s.btn_pick).color(Color32::WHITE))
                                    .fill(p.primary).rounding(6.0_f32).min_size(egui::vec2(90.0_f32, 32.0_f32));
                                if ui.add(pick_btn).clicked() { self.pick_backup(); }
                                let open_btn = egui::Button::new(RichText::new(s.btn_open).color(Color32::WHITE))
                                    .fill(p.muted).rounding(6.0_f32).min_size(egui::vec2(90.0_f32, 32.0_f32));
                                if ui.add(open_btn).clicked() { self.open_backup(); }
                            });
                        });
                    ui.add_space(10.0_f32);

                    // ---------------- FILES CARD ----------------
                    egui::Frame::none().fill(p.card).stroke(Stroke::new(1.0_f32, p.border))
                        .rounding(egui::Rounding::same(8.0_f32))
                        .inner_margin(egui::Margin::symmetric(18.0_f32, 14.0_f32))
                        .show(ui, |ui| {
                            ui.label(RichText::new(s.files_title).size(14.0_f32).strong().color(p.text));
                            ui.label(RichText::new(s.files_sub).size(12.0_f32).color(p.muted));
                            ui.add_space(8.0_f32);
                            ui.horizontal(|ui| {
                                let all_btn = egui::Button::new(RichText::new(s.btn_all).color(p.text).size(12.0_f32))
                                    .fill(p.chip).rounding(6.0_f32);
                                if ui.add(all_btn).clicked() {
                                    for (n, _) in SAVE_FILES { self.config.selected_files.insert(n.to_string(), true); }
                                    self.persist_config();
                                }
                                let none_btn = egui::Button::new(RichText::new(s.btn_none).color(p.text).size(12.0_f32))
                                    .fill(p.chip).rounding(6.0_f32);
                                if ui.add(none_btn).clicked() {
                                    for (n, _) in SAVE_FILES { self.config.selected_files.insert(n.to_string(), false); }
                                    self.persist_config();
                                }
                            });
                            ui.add_space(10.0_f32);
                            ui.columns(2, |cols| {
                                for (i, (name, desc_key)) in SAVE_FILES.iter().enumerate() {
                                    let col = if i % 2 == 0 { &mut cols[0] } else { &mut cols[1] };
                                    let desc = match *desc_key {
                                        "f_cc1" => s.f_cc1, "f_cc2" => s.f_cc2,
                                        "f_cl1" => s.f_cl1, "f_cl2" => s.f_cl2,
                                        "f_sfx" => s.f_sfx, "f_mus" => s.f_mus,
                                        "f_plat" => s.f_plat, _ => "",
                                    };
                                    let entry = self.config.selected_files.entry(name.to_string()).or_insert(true);
                                    col.horizontal(|ui| {
                                        if ui.checkbox(entry, "").changed() { /* saved below */ }
                                        ui.label(RichText::new(*name).strong().color(p.text));
                                        ui.label(RichText::new(desc).color(p.muted).size(12.0_f32));
                                    });
                                }
                            });
                            self.persist_config();
                        });
                    ui.add_space(15.0_f32);

                    // ---------------- ACTION BUTTONS ----------------
                    ui.horizontal(|ui| {
                        let save_btn = egui::Button::new(
                            RichText::new(s.btn_save).color(Color32::WHITE).size(16.0_f32).strong()
                        ).fill(p.primary).rounding(8.0_f32).min_size(egui::vec2(ui.available_width() / 2.0_f32 - 8.0_f32, 55.0_f32));
                        if ui.add(save_btn).clicked() { self.do_copy("save"); }

                        let load_btn = egui::Button::new(
                            RichText::new(s.btn_load).color(Color32::WHITE).size(16.0_f32).strong()
                        ).fill(p.success).rounding(8.0_f32).min_size(egui::vec2(ui.available_width() - 4.0_f32, 55.0_f32));
                        if ui.add(load_btn).clicked() { self.do_copy("load"); }
                    });
                    ui.add_space(15.0_f32);

                    // ---------------- LOG CARD ----------------
                    egui::Frame::none().fill(p.card).stroke(Stroke::new(1.0_f32, p.border))
                        .rounding(egui::Rounding::same(8.0_f32))
                        .inner_margin(egui::Margin::symmetric(18.0_f32, 14.0_f32))
                        .show(ui, |ui| {
                            ui.label(RichText::new(s.log_title).size(14.0_f32).strong().color(p.text));
                            ui.add_space(8.0_f32);
                            egui::Frame::none().fill(p.input_bg)
                                .stroke(Stroke::new(1.0_f32, p.border))
                                .rounding(egui::Rounding::same(6.0_f32))
                                .inner_margin(egui::Margin::same(10.0_f32))
                                .show(ui, |ui| {
                                    ui.set_min_height(180.0_f32);
                                    egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                                        for entry in &self.logs {
                                            let color = match entry.level {
                                                LogLevel::Success => p.success,
                                                LogLevel::Error => p.danger,
                                                LogLevel::Warning => p.warning,
                                                LogLevel::Muted => p.muted,
                                                LogLevel::Header | LogLevel::Normal => p.text,
                                            };
                                            let text = RichText::new(&entry.text).color(color).size(12.0_f32);
                                            ui.label(match entry.level {
                                                LogLevel::Header => text.strong(),
                                                _ => text,
                                            });
                                        }
                                    });
                                });
                        });
                });
            });
    }
}

// ============================================================ MAIN
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("GD Save Manager")
            .with_inner_size([1040.0_f32, 880.0_f32])
            .with_min_inner_size([820.0_f32, 640.0_f32]),
        ..Default::default()
    };

    eframe::run_native(
        "GD Save Manager",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}