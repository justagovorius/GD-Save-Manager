import os
import sys
import json
import shutil
import subprocess
import tkinter as tk
from tkinter import filedialog, messagebox
from datetime import datetime
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

# ------------------------------------------------------------ Переводы
TRANSLATIONS = {
    "ru": {
        "title": "GD Save Manager", "subtitle": "Локальное резервное копирование сохранений GD и GDPS",
        "theme": "Тема:", "light": "Светлая", "dark": "Тёмная", "system": "Системная",
        "lang": "Язык:", "ru": "Русский", "en": "English", "lang_sys": "Системный",
        "src_title": "📁  Папка с сохранениями игры", "src_sub": "по умолчанию: %LOCALAPPDATA%\\GeometryDash",
        "bk_title": "💾  Папка GD_Backups", "bk_sub": "место, где будут храниться резервные копии",
        "btn_pick": "Выбрать", "btn_open": "Обзор", "btn_save": "💾   СОХРАНИТЬ  (игра → копия)", "btn_load": "📂   ЗАГРУЗИТЬ  (копия → игра)",
        "files_title": "📄  Какие файлы задействовать", "files_sub": "поставь галочку у файлов, которые нужно копировать",
        "btn_all": "Выбрать все", "btn_none": "Снять все", "log_title": "📝  Журнал операций",
        "f_cc1": "Основной прогресс игры", "f_cc2": "Дубликат прогресса", "f_cl1": "Локальные уровни",
        "f_cl2": "Дубликат уровней", "f_sfx": "Библиотека звуков", "f_mus": "Библиотека музыки", "f_plat": "Сохранение платформера",
        "log_src": "Папка игры:   ", "log_dst": "Папка бэкапов:", "log_open": "📁 Открыта папка: ",
        "log_copy_src": "   Откуда: ", "log_copy_dst": "   Куда:   ", "log_success": "\n✅ Успешно скопировано ({n}):",
        "log_skip": "\n⚠️  Пропущено ({n}):", "log_fail": "\n❌ Ошибки копирования ({n}):",
        "log_save_t": "СОХРАНЕНИЕ (игра → копия)", "log_load_t": "ЗАГРУЗКА (копия → игра)",
        "msg_warn": "Внимание", "msg_paths": "Укажите обе папки: с сохранениями игры и для резервных копий.",
        "msg_nofiles": "Не выбрано ни одного файла для операции.", "msg_err": "Ошибка",
        "err_src_no": "Папка-источник не существует:", "err_dst_no": "Папка с резервными копиями не существует:",
        "err_create": "Не удалось создать папку:", "msg_done": "Готово",
        "msg_done_all": "Все выбранные файлы успешно скопированы ({n}).", "msg_done_part": "Готово (частично)",
        "msg_err_copy": "Завершено с ошибками", "msg_nothing": "Ничего не скопировано", "details": "\n\nПодробности — в журнале.",
        "sum_ok": "Успешно: ", "sum_skip": "\nПропущено: ", "sum_err": "\nОшибок: ",
        "skip_reason": "файл отсутствует в папке-источнике", "err_permission": "нет прав на запись (возможно, файл занят игрой): ",
        "err_io": "ошибка ввода-вывода: ", "err_unknown": "неизвестная ошибка: ",
        "err_no_dir_src": "Папка не существует: ", "err_cant_open": "Не удалось открыть папку: "
    },
    "en": {
        "title": "GD Save Manager", "subtitle": "Local backup tool for GD and GDPS saves",
        "theme": "Theme:", "light": "Light", "dark": "Dark", "system": "System",
        "lang": "Language:", "ru": "Русский", "en": "English", "lang_sys": "System",
        "src_title": "📁  Game Save Folder", "src_sub": "default: %LOCALAPPDATA%\\GeometryDash",
        "bk_title": "💾  GD_Backups Folder", "bk_sub": "where backup files will be stored",
        "btn_pick": "Select", "btn_open": "Browse", "btn_save": "💾   SAVE  (game → backup)", "btn_load": "📂   LOAD  (backup → game)",
        "files_title": "📄  Files to include", "files_sub": "check the files you want to copy",
        "btn_all": "Select All", "btn_none": "Deselect All", "log_title": "📝  Operation Log",
        "f_cc1": "Main game progress", "f_cc2": "Progress backup", "f_cl1": "Local levels",
        "f_cl2": "Levels backup", "f_sfx": "Sound library", "f_mus": "Music library", "f_plat": "Platformer save",
        "log_src": "Game folder:   ", "log_dst": "Backup folder: ", "log_open": "📁 Opened folder: ",
        "log_copy_src": "   From: ", "log_copy_dst": "   To:   ", "log_success": "\n✅ Successfully copied ({n}):",
        "log_skip": "\n⚠️  Skipped ({n}):", "log_fail": "\n❌ Copy errors ({n}):",
        "log_save_t": "SAVE (game → backup)", "log_load_t": "LOAD (backup → game)",
        "msg_warn": "Warning", "msg_paths": "Please specify both folders: game saves and backups.",
        "msg_nofiles": "No files selected for the operation.", "msg_err": "Error",
        "err_src_no": "Source folder does not exist:", "err_dst_no": "Backup folder does not exist:",
        "err_create": "Failed to create folder:", "msg_done": "Done",
        "msg_done_all": "All selected files successfully copied ({n}).", "msg_done_part": "Done (partially)",
        "msg_err_copy": "Completed with errors", "msg_nothing": "Nothing copied", "details": "\n\nCheck the log for details.",
        "sum_ok": "Success: ", "sum_skip": "\nSkipped: ", "sum_err": "\nErrors: ",
        "skip_reason": "file not found in source folder", "err_permission": "permission denied (file might be in use): ",
        "err_io": "I/O error: ", "err_unknown": "unknown error: ",
        "err_no_dir_src": "Folder does not exist: ", "err_cant_open": "Failed to open folder: "
    }
}

SAVE_FILES = [
    ("CCGameManager.dat", "f_cc1"), ("CCGameManager2.dat", "f_cc2"),
    ("CCLocalLevels.dat", "f_cl1"), ("CCLocalLevels2.dat", "f_cl2"),
    ("sfxlibrary.dat", "f_sfx"), ("musiclibrary.dat", "f_mus"),
    ("CCPlatformerSave.dat", "f_plat")
]

# ------------------------------------------------------------ Системные функции
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

def _default_source_dir():
    local = os.environ.get("LOCALAPPDATA")
    return os.path.join(local, "GeometryDash") if local else os.path.join(os.path.expanduser("~"), "AppData", "Local", "GeometryDash")

def _default_backup_dir():
    home = os.path.expanduser("~"); docs = os.path.join(home, "Documents")
    base = docs if os.path.isdir(docs) else home
    return os.path.join(base, BACKUP_FOLDER_NAME)

def get_system_language():
    try:
        if os.name == "nt":
            import ctypes
            lang_id = ctypes.windll.kernel32.GetUserDefaultUILanguage()
            if lang_id == 1049: return "ru"
        else:
            lang, _ = locale.getdefaultlocale()
            if lang and lang.startswith("ru"): return "ru"
    except Exception: pass
    return "en"

def get_system_theme():
    if os.name == "nt":
        try:
            import winreg
            key = winreg.OpenKey(winreg.HKEY_CURRENT_USER, r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize")
            value, _ = winreg.QueryValueEx(key, "AppsUseLightTheme"); winreg.CloseKey(key)
            return "light" if value == 1 else "dark"
        except Exception: return "light"
    return "light"

def _font(size=10, weight="normal"):
    family = "Segoe UI" if os.name == "nt" else ("SF Pro Text" if sys.platform == "darwin" else "Helvetica")
    return (family, size, weight)

def _hover_color(bg):
    h = bg.lstrip("#"); r, g, b = int(h[0:2], 16), int(h[2:4], 16), int(h[4:6], 16)
    lum = (0.299 * r + 0.587 * g + 0.114 * b) / 255
    factor = 1.15 if lum < 0.5 else 0.88
    return f"#{min(255, int(r*factor)):02x}{min(255, int(g*factor)):02x}{min(255, int(b*factor)):02x}"

THEMES = {
    "light": {"bg": "#eef1f5", "card": "#ffffff", "border": "#d8dee6", "text": "#1f2937", "muted": "#6b7280", "primary": "#2563eb", "success": "#16a34a", "danger": "#dc2626", "warning": "#d97706", "input_bg": "#f9fafb", "chip": "#e5e7eb", "secondary": "#6b7280"},
    "dark": {"bg": "#1a1d24", "card": "#252932", "border": "#353a45", "text": "#e5e7eb", "muted": "#9ca3af", "primary": "#3b82f6", "success": "#22c55e", "danger": "#ef4444", "warning": "#f59e0b", "input_bg": "#1e222a", "chip": "#353a45", "secondary": "#6b7280"},
    "amoled": {"bg": "#000000", "card": "#0a0a0a", "border": "#1c1c1c", "text": "#e5e7eb", "muted": "#9ca3af", "primary": "#3b82f6", "success": "#22c55e", "danger": "#ef4444", "warning": "#f59e0b", "input_bg": "#000000", "chip": "#1a1a1a", "secondary": "#4b5563"}
}

class ThemedCheckbox(tk.Frame):
    def __init__(self, parent, variable, theme, size=18, bg=None, disabled=False, command=None):
        bg = bg if bg is not None else theme["card"]
        super().__init__(parent, bg=bg)
        self.variable, self.theme, self.disabled, self.command = variable, theme, disabled, command
        self.canvas = tk.Canvas(self, width=size, height=size, bg=bg, highlightthickness=0, bd=0, cursor="arrow" if disabled else "hand2")
        self.canvas.pack()
        self.rect = self.canvas.create_rectangle(1.5, 1.5, size-1.5, size-1.5, outline=theme["border"], fill=theme["input_bg"], width=1.5)
        self.check = self.canvas.create_line(size*0.22, size*0.50, size*0.42, size*0.70, size*0.78, size*0.30, fill="white", width=max(2.0, size*0.13), capstyle="round", joinstyle="round", state="hidden")
        if not disabled: self.canvas.bind("<Button-1>", self._toggle)
        self.variable.trace_add("write", self._refresh)
        self._refresh()
    def toggle(self): self._toggle()
    def _toggle(self, event=None):
        if self.disabled: return
        self.variable.set(not self.variable.get())
        if self.command: self.command()
    def _refresh(self, *args):
        t = self.theme
        if self.variable.get():
            fill = t["primary"] if not self.disabled else t["muted"]
            self.canvas.itemconfig(self.rect, fill=fill, outline=fill)
            self.canvas.itemconfig(self.check, state="normal")
        else:
            outline = t["border"] if not self.disabled else t["muted"]
            self.canvas.itemconfig(self.rect, fill=t["input_bg"], outline=outline)
            self.canvas.itemconfig(self.check, state="hidden")

class GD_SaveManager:
    def __init__(self, root):
        self.root = root
        self.root.title("GD Save Manager")
        self.root.minsize(920, 680)

        sw, sh = self.root.winfo_screenwidth(), self.root.winfo_screenheight()
        w, h = min(1040, sw - 120), min(880, sh - 100)
        self.root.geometry(f"{w}x{h}+{(sw - w) // 2}+{(sh - h) // 3}")

        # Считываем системные настройки ОДИН РАЗ при запуске
        self.sys_lang = get_system_language()
        self.sys_theme = get_system_theme()

        cfg = load_config()
        self.theme_mode_var = tk.StringVar(value=cfg.get("theme_mode", "system"))
        self.lang_mode_var = tk.StringVar(value=cfg.get("language", "system"))
        self.amoled_var = tk.BooleanVar(value=bool(cfg.get("amoled", False)))
        self.source_var = tk.StringVar(value=cfg.get("source_dir") or _default_source_dir())
        self.backup_var = tk.StringVar(value=cfg.get("backup_dir") or _default_backup_dir())
        
        saved_sel = cfg.get("selected_files", {})
        self.checkbox_vars = {name: tk.BooleanVar(value=saved_sel.get(name, True)) for name, _ in SAVE_FILES}

        self.log_lines = []
        self.log_text = None
        self._current_theme = THEMES["light"]
        self.T = TRANSLATIONS["ru"]

        self._build_ui()
        self.root.protocol("WM_DELETE_WINDOW", self._on_close)

    def _get_lang(self):
        mode = self.lang_mode_var.get()
        if mode == "system": return self.sys_lang
        return mode

    def _t(self, key, **kwargs):
        txt = self.T.get(key, key)
        if kwargs: txt = txt.format(**kwargs)
        return txt

    def _save_config(self):
        save_config({
            "theme_mode": self.theme_mode_var.get(), "language": self.lang_mode_var.get(),
            "amoled": self.amoled_var.get(), "source_dir": self.source_var.get(),
            "backup_dir": self.backup_var.get(), "selected_files": {n: v.get() for n, v in self.checkbox_vars.items()}
        })

    def _resolve_palette_name(self):
        mode = self.theme_mode_var.get()
        if mode == "system": mode = self.sys_theme
        if mode == "dark" and self.amoled_var.get(): return "amoled"
        return mode

    def _change_setting(self, var, val):
        if var.get() != val:
            var.set(val)
            self._save_config()
            self._build_ui()

    def _button(self, parent, theme, text, command, bg_key="primary", fg="white", font_size=10, pad=8, bold=True):
        bg = theme[bg_key]
        btn = tk.Button(parent, text=text, command=command, bg=bg, fg=fg, font=_font(font_size, "bold" if bold else "normal"), relief="flat", bd=0, cursor="hand2", activebackground=bg, activeforeground=fg, padx=16, pady=pad)
        hover = _hover_color(bg)
        btn.bind("<Enter>", lambda e: btn.configure(bg=hover, activebackground=hover))
        btn.bind("<Leave>", lambda e: btn.configure(bg=bg, activebackground=bg))
        return btn

    def _pill(self, parent, theme, text, active, command):
        bg = theme["primary"] if active else theme["chip"]
        btn = tk.Button(parent, text=text, command=command, bg=bg, fg="white" if active else theme["text"], font=_font(9, "bold" if active else "normal"), relief="flat", bd=0, cursor="hand2", activebackground=bg, padx=12, pady=5)
        hover = _hover_color(bg)
        btn.bind("<Enter>", lambda e: btn.configure(bg=hover, activebackground=hover))
        btn.bind("<Leave>", lambda e: btn.configure(bg=bg, activebackground=bg))
        return btn

    def _entry(self, parent, theme, variable):
        return tk.Entry(parent, textvariable=variable, font=_font(10), relief="flat", bg=theme["input_bg"], fg=theme["text"], insertbackground=theme["text"], highlightbackground=theme["border"], highlightcolor=theme["primary"], highlightthickness=1)

    def _card(self, parent, theme, title=None, subtitle=None):
        outer = tk.Frame(parent, bg=theme["card"], highlightbackground=theme["border"], highlightthickness=1)
        if title:
            head = tk.Frame(outer, bg=theme["card"]); head.pack(fill="x", padx=18, pady=(14, 0))
            tk.Label(head, text=title, font=_font(11, "bold"), bg=theme["card"], fg=theme["text"], anchor="w").pack(fill="x")
            if subtitle: tk.Label(head, text=subtitle, font=_font(9), bg=theme["card"], fg=theme["muted"], anchor="w").pack(fill="x")
        body = tk.Frame(outer, bg=theme["card"]); body.pack(fill="both", expand=True, padx=18, pady=(12, 16))
        return outer, body

    def _build_ui(self):
        self.T = TRANSLATIONS[self._get_lang()]
        self.log_text = None
        for w in self.root.winfo_children(): w.destroy()

        palette_name = self._resolve_palette_name()
        theme = THEMES[palette_name]
        self.root.configure(bg=theme["bg"])

        # Header
        header = tk.Frame(self.root, bg=theme["bg"]); header.pack(fill="x", padx=24, pady=(20, 6))
        title_block = tk.Frame(header, bg=theme["bg"]); title_block.pack(side="left", fill="x", expand=True)
        tk.Label(title_block, text=self._t("title"), font=_font(20, "bold"), bg=theme["bg"], fg=theme["text"], anchor="w").pack(fill="x")
        tk.Label(title_block, text=self._t("subtitle"), font=_font(10), bg=theme["bg"], fg=theme["muted"], anchor="w").pack(fill="x")

        settings_bar = tk.Frame(header, bg=theme["bg"]); settings_bar.pack(side="right", anchor="ne")
        
        # Language
        tk.Label(settings_bar, text=self._t("lang"), font=_font(9), bg=theme["bg"], fg=theme["muted"]).pack(side="left", padx=(0, 6))
        for val, key in [("ru", "ru"), ("en", "en"), ("system", "lang_sys")]:
            self._pill(settings_bar, theme, self._t(key), self.lang_mode_var.get() == val, lambda v=val: self._change_setting(self.lang_mode_var, v)).pack(side="left", padx=(0, 4))
            
        tk.Frame(settings_bar, bg=theme["bg"], width=15).pack(side="left")

        # Theme
        tk.Label(settings_bar, text=self._t("theme"), font=_font(9), bg=theme["bg"], fg=theme["muted"]).pack(side="left", padx=(0, 6))
        for val, key in [("light", "light"), ("dark", "dark"), ("system", "system")]:
            self._pill(settings_bar, theme, self._t(key), self.theme_mode_var.get() == val, lambda v=val: self._change_setting(self.theme_mode_var, v)).pack(side="left", padx=(0, 4))

        amoled_enabled = self.theme_mode_var.get() != "light"
        amoled_container = tk.Frame(settings_bar, bg=theme["bg"]); amoled_container.pack(side="left", padx=(8, 0))
        amoled_cb = ThemedCheckbox(amoled_container, self.amoled_var, theme, size=16, bg=theme["bg"], disabled=not amoled_enabled, command=lambda: self.root.after_idle(lambda: self._change_setting(self.amoled_var, self.amoled_var.get())))
        amoled_cb.pack(side="left")
        amoled_lbl = tk.Label(amoled_container, text="AMOLED", bg=theme["bg"], fg=theme["text"] if amoled_enabled else theme["muted"], font=_font(9), cursor="hand2" if amoled_enabled else "arrow")
        amoled_lbl.pack(side="left", padx=(4, 0))
        if amoled_enabled: amoled_lbl.bind("<Button-1>", lambda e: amoled_cb.toggle())

        # Paths
        paths = tk.Frame(self.root, bg=theme["bg"]); paths.pack(fill="x", padx=24, pady=(10, 8))
        
        src_card, src_body = self._card(paths, theme, title=self._t("src_title"), subtitle=self._t("src_sub"))
        src_card.pack(fill="x", pady=(0, 10))
        src_row = tk.Frame(src_body, bg=theme["card"]); src_row.pack(fill="x")
        self._entry(src_row, theme, self.source_var).pack(side="left", fill="x", expand=True, ipady=7, ipadx=8)
        self._button(src_row, theme, self._t("btn_pick"), self._select_source_dir, "primary", pad=7).pack(side="left", padx=(8, 4))
        self._button(src_row, theme, self._t("btn_open"), lambda: self._open_folder_in_explorer(self.source_var.get().strip(), False, "src"), "secondary", pad=7).pack(side="left")

        bk_card, bk_body = self._card(paths, theme, title=self._t("bk_title"), subtitle=self._t("bk_sub"))
        bk_card.pack(fill="x")
        bk_row = tk.Frame(bk_body, bg=theme["card"]); bk_row.pack(fill="x")
        self._entry(bk_row, theme, self.backup_var).pack(side="left", fill="x", expand=True, ipady=7, ipadx=8)
        self._button(bk_row, theme, self._t("btn_pick"), self._select_backup_dir, "primary", pad=7).pack(side="left", padx=(8, 4))
        self._button(bk_row, theme, self._t("btn_open"), lambda: self._open_folder_in_explorer(self.backup_var.get().strip(), True, "bk"), "secondary", pad=7).pack(side="left")

        # Files
        files_card, files_body = self._card(self.root, theme, title=self._t("files_title"), subtitle=self._t("files_sub"))
        files_card.pack(fill="x", padx=24, pady=(0, 10))
        quick = tk.Frame(files_body, bg=theme["card"]); quick.pack(fill="x", pady=(0, 10))
        self._button(quick, theme, self._t("btn_all"), lambda: self._set_all(True), "chip", fg=theme["text"], font_size=9, pad=4, bold=False).pack(side="left", padx=(0, 6))
        self._button(quick, theme, self._t("btn_none"), lambda: self._set_all(False), "chip", fg=theme["text"], font_size=9, pad=4, bold=False).pack(side="left")

        grid = tk.Frame(files_body, bg=theme["card"]); grid.pack(fill="x")
        for i, (filename, desc_key) in enumerate(SAVE_FILES):
            cell = tk.Frame(grid, bg=theme["card"]); cell.grid(row=i // 2, column=i % 2, sticky="w", padx=(0, 24), pady=3)
            var = self.checkbox_vars[filename]
            cb = ThemedCheckbox(cell, var, theme); cb.pack(side="left", padx=(0, 6))
            lbl = tk.Label(cell, text=filename, font=_font(10, "bold"), bg=theme["card"], fg=theme["text"], cursor="hand2"); lbl.pack(side="left")
            lbl.bind("<Button-1>", lambda e, v=var: v.set(not v.get()))
            desc = tk.Label(cell, text=self._t(desc_key), font=_font(9), bg=theme["card"], fg=theme["muted"], cursor="hand2"); desc.pack(side="left", padx=(8, 0))
            desc.bind("<Button-1>", lambda e, v=var: v.set(not v.get()))
        grid.columnconfigure(0, weight=1); grid.columnconfigure(1, weight=1)

        # Actions
        actions = tk.Frame(self.root, bg=theme["bg"]); actions.pack(fill="x", padx=24, pady=(4, 10))
        self._button(actions, theme, self._t("btn_save"), lambda: self._run_copy("save"), "primary", font_size=12, pad=12).pack(side="left", expand=True, fill="x", padx=(0, 6))
        self._button(actions, theme, self._t("btn_load"), lambda: self._run_copy("load"), "success", font_size=12, pad=12).pack(side="left", expand=True, fill="x", padx=(6, 0))

        # Log
        log_card, log_body = self._card(self.root, theme, title=self._t("log_title"))
        log_card.pack(fill="both", expand=True, padx=24, pady=(0, 20))
        log_wrap = tk.Frame(log_body, bg=theme["card"]); log_wrap.pack(fill="both", expand=True)
        self.log_text = tk.Text(log_wrap, height=10, wrap="word", state="disabled", bg=theme["input_bg"], fg=theme["text"], font=_font(10), relief="flat", highlightbackground=theme["border"], highlightthickness=1, padx=10, pady=8)
        self.log_text.pack(side="left", fill="both", expand=True)
        sb = tk.Scrollbar(log_wrap, command=self.log_text.yview); sb.pack(side="right", fill="y"); self.log_text.config(yscrollcommand=sb.set)
        for tag, color in [("success", "success"), ("error", "danger"), ("warning", "warning"), ("header", "text"), ("muted", "muted")]:
            self.log_text.tag_configure(tag, foreground=theme[color], font=_font(10, "bold") if tag == "header" else _font(10))
        self._render_log()

    def _render_log(self):
        if not self.log_text: return
        self.log_text.config(state="normal"); self.log_text.delete("1.0", tk.END)
        for text, tag in self.log_lines:
            self.log_text.insert(tk.END, text + "\n", tag if tag else ())
        self.log_text.see(tk.END); self.log_text.config(state="disabled")

    def _log(self, message, tag=None):
        self.log_lines.append((message, tag))
        if self.log_text and self.log_text.winfo_exists():
            self.log_text.config(state="normal")
            self.log_text.insert(tk.END, message + "\n", tag if tag else ())
            self.log_text.see(tk.END); self.log_text.config(state="disabled")

    def _select_source_dir(self):
        curr = self.source_var.get().strip()
        d = filedialog.askdirectory(initialdir=curr if os.path.isdir(curr) else None)
        if d: self.source_var.set(os.path.normpath(d)); self._save_config()

    def _select_backup_dir(self):
        curr = self.backup_var.get().strip().rstrip("\\/")
        init = os.path.dirname(os.path.normpath(curr)) if os.path.basename(os.path.normpath(curr)) == BACKUP_FOLDER_NAME else curr
        d = filedialog.askdirectory(initialdir=init if os.path.isdir(init) else None)
        if d:
            norm = os.path.normpath(d)
            if os.path.basename(norm) != BACKUP_FOLDER_NAME: norm = os.path.join(norm, BACKUP_FOLDER_NAME)
            self.backup_var.set(norm); self._save_config()

    def _set_all(self, value):
        for v in self.checkbox_vars.values(): v.set(value)

    def _open_folder_in_explorer(self, path, create, ftype):
        if not path: return
        path = os.path.normpath(path)
        if ftype == "bk" and os.path.basename(path) != BACKUP_FOLDER_NAME: path = os.path.join(path, BACKUP_FOLDER_NAME)
        if not os.path.isdir(path):
            if create:
                try: os.makedirs(path, exist_ok=True)
                except Exception as e:
                    messagebox.showerror(self._t("msg_err"), f"{self._t('err_create')}\n{path}\n\n{e}")
                    self._log(f"❌ {self._t('err_create')} {path} — {e}", "error"); return
            else:
                messagebox.showerror(self._t("msg_err"), f"{self._t('err_no_dir_src')}\n{path}")
                self._log(f"❌ {self._t('err_no_dir_src')} {path}", "error"); return
        try:
            if os.name == "nt": subprocess.Popen(["explorer.exe", "/n," + path])
            elif sys.platform == "darwin": subprocess.Popen(["open", "-n", path])
            else: subprocess.Popen(["xdg-open", path])
            self._log(f"{self._t('log_open')} {path}", "muted")
        except Exception as e:
            messagebox.showerror(self._t("msg_err"), f"{self._t('err_cant_open')}\n{path}\n\n{e}")
            self._log(f"❌ {self._t('err_cant_open')} {path} — {e}", "error")

    def _run_copy(self, direction):
        src, bk = self.source_var.get().strip(), self.backup_var.get().strip()
        if os.path.basename(os.path.normpath(bk)) != BACKUP_FOLDER_NAME: bk = os.path.join(bk, BACKUP_FOLDER_NAME)
        self.backup_var.set(bk)

        if not src or not bk:
            messagebox.showwarning(self._t("msg_warn"), self._t("msg_paths")); return
        selected = [n for n, v in self.checkbox_vars.items() if v.get()]
        if not selected:
            messagebox.showwarning(self._t("msg_warn"), self._t("msg_nofiles")); return

        src_dir, dst_dir = (src, bk) if direction == "save" else (bk, src)
        title = self._t("log_save_t") if direction == "save" else self._t("log_load_t")

        if not os.path.isdir(src_dir):
            messagebox.showerror(self._t("msg_err"), f"{self._t('err_src_no')}\n{src_dir}")
            self._log(f"❌ {title}: {self._t('err_src_no')} {src_dir}", "error"); return
        if not os.path.isdir(dst_dir):
            if direction == "save":
                try: os.makedirs(dst_dir, exist_ok=True)
                except Exception as e:
                    messagebox.showerror(self._t("msg_err"), f"{self._t('err_create')}\n{dst_dir}\n\n{e}")
                    self._log(f"❌ {title}: {self._t('err_create')} {e}", "error"); return
            else:
                messagebox.showerror(self._t("msg_err"), f"{self._t('err_dst_no')}\n{dst_dir}")
                self._log(f"❌ {title}: {self._t('err_dst_no')} {dst_dir}", "error"); return

        ok, skip, fail = [], [], []
        for f in selected:
            s, d = os.path.join(src_dir, f), os.path.join(dst_dir, f)
            if not os.path.isfile(s): skip.append((f, self._t("skip_reason"))); continue
            try: shutil.copy2(s, d); ok.append(f)
            except PermissionError as e: fail.append((f, self._t("err_permission") + str(e)))
            except OSError as e: fail.append((f, self._t("err_io") + str(e)))
            except Exception as e: fail.append((f, self._t("err_unknown") + str(e)))

        self._log("─" * 60, "muted")
        self._log(f"[{datetime.now():%Y-%m-%d %H:%M:%S}]  {title}", "header")
        self._log(f"{self._t('log_copy_src')} {src_dir}", "muted")
        self._log(f"{self._t('log_copy_dst')} {dst_dir}", "muted")
        
        if ok:
            self._log(self._t("log_success", n=len(ok)), "success")
            for f in ok: self._log(f"   • {f}", "success")
        if skip:
            self._log(self._t("log_skip", n=len(skip)), "warning")
            for f, r in skip: self._log(f"   • {f} — {r}", "warning")
        if fail:
            self._log(self._t("log_fail", n=len(fail)), "error")
            for f, r in fail: self._log(f"   • {f} — {r}", "error")

        self._save_config()
        summary = f"{self._t('sum_ok')}{len(ok)}{self._t('sum_skip')}{len(skip)}{self._t('sum_err')}{len(fail)}"
        
        if fail: messagebox.showwarning(self._t("msg_err_copy"), summary + self._t("details"))
        elif skip and not ok: messagebox.showwarning(self._t("msg_nothing"), summary + self._t("details"))
        elif skip: messagebox.showinfo(self._t("msg_done_part"), summary + self._t("details"))
        else: messagebox.showinfo(self._t("msg_done"), self._t("msg_done_all", n=len(ok)))

    def _on_close(self):
        self._save_config(); self.root.destroy()

if __name__ == "__main__":
    root = tk.Tk()
    app = GD_SaveManager(root)
    root.mainloop()