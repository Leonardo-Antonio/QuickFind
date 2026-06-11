use gtk::gdk;
use gtk::prelude::*;
use std::sync::Arc;
use std::cell::RefCell;
use std::rc::Rc;

use crate::clipboard::ClipItem;
use crate::config::Config;
use crate::launcher::{AppLauncher, DesktopEntry};

const CSS: &str = r#"
window.quickfind-window {
    background: transparent;
}

.main-box {
    background: rgba(28, 28, 30, 0.97);
    border: 1px solid rgba(255, 255, 255, 0.07);
    border-radius: 20px;
    margin: 2px;
    padding: 0;
}

.search-entry,
.search-entry:focus,
.search-entry:focus-within,
.search-entry:hover,
.search-entry > text {
    background: transparent;
    border: none;
    border-width: 0;
    border-radius: 0;
    box-shadow: none;
    outline: none;
    outline-width: 0;
    min-height: 0;
}

.search-entry {
    padding: 20px 24px;
    color: #ffffff;
    font-size: 30px;
    font-weight: 300;
    caret-color: #4a9d9d;
    margin: 0;
}

.search-entry selection {
    background: rgba(74, 157, 157, 0.35);
}

.search-entry image:first-child {
    color: rgba(255, 255, 255, 0.4);
    margin-right: 14px;
    -gtk-icon-size: 26px;
}

.results-area {
    background: transparent;
    border-top: 1px solid rgba(255, 255, 255, 0.08);
    padding: 8px 10px 10px 10px;
}

.result-row {
    background: transparent;
    border-radius: 12px;
    padding: 10px 14px;
    margin: 1px 0;
    transition: all 60ms ease;
}

.result-row:hover {
    background: rgba(255, 255, 255, 0.05);
}

.result-row.selected {
    background: rgba(74, 157, 157, 0.85);
}

.result-row.selected .result-name {
    color: #ffffff;
    font-weight: 600;
}

.result-row.selected .result-desc {
    color: rgba(255, 255, 255, 0.8);
}

.result-row.selected .result-shortcut {
    color: rgba(255, 255, 255, 0.7);
}

.result-icon {
    margin-right: 16px;
    margin-left: 2px;
}

.result-name {
    color: rgba(255, 255, 255, 0.92);
    font-size: 16px;
    font-weight: 450;
    letter-spacing: 0.15px;
}

.result-desc {
    color: rgba(255, 255, 255, 0.45);
    font-size: 13px;
    font-weight: 400;
    margin-top: 2px;
}

.result-shortcut {
    color: rgba(255, 255, 255, 0.28);
    font-size: 14px;
    font-weight: 500;
    min-width: 44px;
}

.scrolled-window {
    background: transparent;
    border: none;
}

.scrolled-window scrollbar {
    background: transparent;
    border: none;
}

.scrolled-window scrollbar slider {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 99px;
    min-width: 3px;
    min-height: 28px;
}

.empty-label {
    color: rgba(255, 255, 255, 0.22);
    font-size: 14px;
    padding: 32px 8px;
}

.ts-hint {
    color: rgba(255, 255, 255, 0.35);
    font-size: 12px;
    font-weight: 450;
    padding: 4px 14px 6px 14px;
}

.ts-tag {
    color: #4a9d9d;
    font-size: 13px;
    font-weight: 600;
    letter-spacing: 0.4px;
}

.result-row.selected .ts-tag {
    color: rgba(255, 255, 255, 0.85);
}

.ts-value {
    color: rgba(255, 255, 255, 0.92);
    font-size: 17px;
    font-weight: 400;
}

.calc-row {
    background: transparent;
    border-radius: 12px;
    padding: 12px 14px;
    margin: 1px 0;
}

.calc-result {
    color: #ffffff;
    font-size: 26px;
    font-weight: 300;
    letter-spacing: 0.3px;
}

.calc-hint {
    color: rgba(255, 255, 255, 0.3);
    font-size: 13px;
    font-weight: 450;
}

.web-search-row {
    background: transparent;
    border-radius: 10px;
    padding: 10px 12px;
    margin: 6px 0 2px 0;
    border-top: 1px solid rgba(255, 255, 255, 0.05);
    transition: all 60ms ease;
}

.web-search-row:hover {
    background: rgba(255, 255, 255, 0.04);
}

.web-search-name {
    color: rgba(255, 255, 255, 0.45);
    font-size: 13px;
    font-weight: 450;
}
"#;

pub struct QuickFindWindow {
    window: gtk::ApplicationWindow,
    search_entry: gtk::Entry,
}

impl QuickFindWindow {
    pub fn new(
        app: &gtk::Application,
        config: Arc<Config>,
        launcher: Arc<AppLauncher>,
        initial_query: Option<String>,
    ) -> Self {
        let provider = gtk::CssProvider::new();
        provider.load_from_string(CSS);

        if let Some(display) = gdk::Display::default() {
            gtk::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }

        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title("QuickFind")
            .default_width(config.window_width)
            .css_classes(vec!["quickfind-window".to_string()])
            .build();
        // No fijamos altura: la ventana crece con el contenido (estilo Spotlight).

        Self::setup_window(&window);

        let main_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(0)
            .css_classes(vec!["main-box".to_string()])
            .build();

        let search_entry = gtk::Entry::builder()
            .placeholder_text("Spotlight Search")
            .css_classes(vec!["search-entry".to_string()])
            .build();
        search_entry.set_icon_from_icon_name(gtk::EntryIconPosition::Primary, Some("system-search-symbolic"));

        let scrolled = gtk::ScrolledWindow::builder()
            .vexpand(false)
            .css_classes(vec!["scrolled-window".to_string(), "results-area".to_string()])
            .propagate_natural_height(true)
            .max_content_height(420)
            .build();
        // Oculto al inicio: la ventana es solo la barra de búsqueda.
        scrolled.set_visible(false);

        let results_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(0)
            .build();

        scrolled.set_child(Some(&results_box));

        main_box.append(&search_entry);
        main_box.append(&scrolled);
        window.set_child(Some(&main_box));

        let results: Rc<RefCell<Vec<DesktopEntry>>> = Rc::new(RefCell::new(Vec::new()));
        let selected_index: Rc<RefCell<usize>> = Rc::new(RefCell::new(0));
        let result_rows: Rc<RefCell<Vec<gtk::Box>>> = Rc::new(RefCell::new(Vec::new()));
        let web_row: Rc<RefCell<Option<gtk::Box>>> = Rc::new(RefCell::new(None));
        let current_query: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));
        let current_calc: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
        let clip_items: Rc<RefCell<Vec<ClipItem>>> = Rc::new(RefCell::new(Vec::new()));
        // Valores copiables de las filas de timestamp (UTC, local).
        let ts_values: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));

        let results_box_clone = results_box.clone();
        let scrolled_clone = scrolled.clone();
        let calc_clone = current_calc.clone();
        let clip_clone = clip_items.clone();
        let ts_clone = ts_values.clone();
        let results_clone = results.clone();
        let selected_clone = selected_index.clone();
        let rows_clone = result_rows.clone();
        let launcher_clone = launcher.clone();
        let config_clone = config.clone();
        let web_row_clone = web_row.clone();
        let query_clone = current_query.clone();

        search_entry.connect_changed(move |entry| {
            let text = entry.text().to_string();
            *query_clone.borrow_mut() = text.clone();

            // Palabras clave que activan el historial del portapapeles.
            const CLIP_KEYWORDS: [&str; 3] = [":clipboard", ":cbhistory", ":cb"];
            let trimmed = text.trim();
            // Los comandos empiezan con ":" → altura máxima fija (sin medir).
            let is_command = trimmed.starts_with(':');
            let is_empty = trimmed.is_empty();
            let lower = trimmed.to_lowercase();
            let clip_keyword = CLIP_KEYWORDS.iter().find(|kw| {
                lower == **kw || lower.starts_with(&format!("{} ", kw))
            });

            // Modo historial del portapapeles: ":cb [filtro]".
            let clips = if let Some(kw) = clip_keyword {
                let filter = trimmed[kw.len()..].trim();
                Some(crate::clipboard::history(25, filter))
            } else {
                None
            };
            let clip_mode = clips.is_some();

            // Conversión de timestamp: ":timestamp <n>" o ":tsp <n>".
            const TS_KEYWORDS: [&str; 2] = [":timestamp", ":tsp"];
            let ts_keyword = if clip_mode {
                None
            } else {
                TS_KEYWORDS
                    .iter()
                    .find(|kw| lower == **kw || lower.starts_with(&format!("{} ", kw)))
            };
            let ts_active = ts_keyword.is_some();
            let ts = ts_keyword.and_then(|kw| crate::timestamp::convert(trimmed[kw.len()..].trim()));

            // ¿Es una operación matemática? Si lo es, mostramos el resultado.
            let calc = if clip_mode || ts_active {
                None
            } else {
                crate::calc::evaluate(&text).map(crate::calc::format_result)
            };
            *calc_clone.borrow_mut() = calc.clone();

            let new_results = if text.is_empty() || calc.is_some() || clip_mode || ts_active {
                Vec::new()
            } else {
                launcher_clone.search(&text, config_clone.max_results)
                    .into_iter()
                    .map(|(entry, _)| entry)
                    .collect()
            };

            *results_clone.borrow_mut() = new_results;
            *clip_clone.borrow_mut() = clips.clone().unwrap_or_default();
            *ts_clone.borrow_mut() = ts
                .as_ref()
                .map(|r| vec![r.utc.clone(), r.local.clone()])
                .unwrap_or_default();
            *selected_clone.borrow_mut() = 0;
            Self::update_results_ui(
                &results_box_clone,
                &scrolled_clone,
                &results_clone.borrow(),
                text,
                calc.as_deref(),
                clips.as_deref(),
                ts.as_ref(),
                ts_active,
                &rows_clone,
                &web_row_clone,
                *selected_clone.borrow(),
                config_clone.show_icons,
                config_clone.icon_size,
            );
            // Ajuste de altura del área de resultados.
            if is_empty {
                // Input vacío: volvemos al estado inicial (solo la barra).
                scrolled_clone.set_min_content_height(0);
                scrolled_clone.set_max_content_height(420);
            } else if is_command {
                // Comandos (":cb", ":tsp", ...): altura máxima fija, sin medir.
                scrolled_clone.set_min_content_height(420);
                scrolled_clone.set_max_content_height(420);
            } else if scrolled_clone.is_visible() {
                // Búsqueda normal: la altura sigue al contenido. Diferido a un
                // idle para medir cuando el layout ya está estable.
                let s = scrolled_clone.clone();
                let c = results_box_clone.clone();
                glib::idle_add_local_once(move || {
                    Self::fit_scrolled(&s, &c);
                });
            }
        });

        let win_key = window.clone();
        let search_key = search_entry.clone();
        let results_key = results.clone();
        let selected_key = selected_index.clone();
        let rows_key = result_rows.clone();
        let launcher_key = launcher.clone();
        let query_key = current_query.clone();
        let calc_key = current_calc.clone();
        let clip_key = clip_items.clone();
        let ts_key = ts_values.clone();
        let win_calc = window.clone();
        let scrolled_key = scrolled.clone();

        let event_controller = gtk::EventControllerKey::new();
        // Capture: interceptamos teclas (Enter, flechas) antes de que el
        // Entry las consuma, para que un simple Enter abra el seleccionado.
        event_controller.set_propagation_phase(gtk::PropagationPhase::Capture);
        event_controller.connect_key_pressed(move |_, keyval, _keycode, state| {
            let ctrl = state.contains(gdk::ModifierType::CONTROL_MASK);

            match keyval {
                gdk::Key::Escape => {
                    win_key.close();
                    glib::Propagation::Stop
                }
                gdk::Key::Return | gdk::Key::KP_Enter if ctrl => {
                    let query = query_key.borrow().clone();
                    if !query.is_empty() {
                        let url = format!("https://duckduckgo.com/?q={}", urlencoding::encode(&query));
                        let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
                        win_key.close();
                    }
                    glib::Propagation::Stop
                }
                gdk::Key::Return | gdk::Key::KP_Enter => {
                    // Si hay un resultado de cálculo, Enter lo copia al portapapeles.
                    if let Some(result) = calc_key.borrow().clone() {
                        if let Some(display) = gdk::Display::default() {
                            display.clipboard().set_text(&result);
                        }
                        win_calc.close();
                        return glib::Propagation::Stop;
                    }
                    // Modo timestamp: Enter copia la fecha seleccionada (UTC/local).
                    {
                        let values = ts_key.borrow();
                        if !values.is_empty() {
                            let idx = *selected_key.borrow();
                            if let Some(value) = values.get(idx) {
                                if let Some(display) = gdk::Display::default() {
                                    display.clipboard().set_text(value);
                                }
                                win_calc.close();
                            }
                            return glib::Propagation::Stop;
                        }
                    }
                    // Modo clipboard: Enter copia el registro seleccionado.
                    {
                        let clips = clip_key.borrow();
                        if !clips.is_empty() {
                            let idx = *selected_key.borrow();
                            if let Some(item) = clips.get(idx) {
                                let _ = crate::clipboard::copy_to_clipboard(item);
                                win_calc.close();
                            }
                            return glib::Propagation::Stop;
                        }
                    }
                    let idx = *selected_key.borrow();
                    let results = results_key.borrow();
                    if let Some(entry) = results.get(idx) {
                        if let Err(e) = AppLauncher::launch(entry) {
                            eprintln!("Error launching app: {}", e);
                        }
                        win_key.close();
                    }
                    glib::Propagation::Stop
                }
                gdk::Key::Up => {
                    let mut idx = selected_key.borrow_mut();
                    if *idx > 0 {
                        *idx -= 1;
                    }
                    Self::update_selection(&rows_key, *idx, &scrolled_key);
                    glib::Propagation::Stop
                }
                gdk::Key::Down => {
                    let mut idx = selected_key.borrow_mut();
                    // Las filas navegables (apps o clipboard) están en rows_key.
                    let max_idx = rows_key.borrow().len().saturating_sub(1);
                    if *idx < max_idx {
                        *idx += 1;
                    }
                    Self::update_selection(&rows_key, *idx, &scrolled_key);
                    glib::Propagation::Stop
                }
                gdk::Key::r if ctrl => {
                    launcher_key.refresh_cache();
                    let text = search_key.text().to_string();
                    search_key.set_text(&text);
                    glib::Propagation::Stop
                }
                gdk::Key::_1 | gdk::Key::_2 | gdk::Key::_3 | gdk::Key::_4 | gdk::Key::_5 |
                gdk::Key::_6 | gdk::Key::_7 | gdk::Key::_8 | gdk::Key::_9 if ctrl => {
                    let digit = match keyval {
                        gdk::Key::_1 => 0,
                        gdk::Key::_2 => 1,
                        gdk::Key::_3 => 2,
                        gdk::Key::_4 => 3,
                        gdk::Key::_5 => 4,
                        gdk::Key::_6 => 5,
                        gdk::Key::_7 => 6,
                        gdk::Key::_8 => 7,
                        gdk::Key::_9 => 8,
                        _ => 0,
                    };
                    // En modo timestamp, Ctrl+N copia la fecha N (UTC/local).
                    {
                        let values = ts_key.borrow();
                        if !values.is_empty() {
                            if let Some(value) = values.get(digit) {
                                if let Some(display) = gdk::Display::default() {
                                    display.clipboard().set_text(value);
                                }
                                win_key.close();
                            }
                            return glib::Propagation::Stop;
                        }
                    }
                    // En modo clipboard, Ctrl+N copia el registro N.
                    {
                        let clips = clip_key.borrow();
                        if !clips.is_empty() {
                            if let Some(item) = clips.get(digit) {
                                let _ = crate::clipboard::copy_to_clipboard(item);
                                win_key.close();
                            }
                            return glib::Propagation::Stop;
                        }
                    }
                    let results = results_key.borrow();
                    if let Some(entry) = results.get(digit) {
                        if let Err(e) = AppLauncher::launch(entry) {
                            eprintln!("Error launching app: {}", e);
                        }
                        win_key.close();
                    }
                    glib::Propagation::Stop
                }
                _ => glib::Propagation::Proceed,
            }
        });

        window.add_controller(event_controller);

        // Consulta inicial desde la línea de comandos (-q): rellena y filtra.
        if let Some(query) = initial_query.filter(|q| !q.is_empty()) {
            search_entry.set_text(&query);
            search_entry.set_position(-1); // cursor al final
        }

        search_entry.grab_focus();

        if let Some(display) = gdk::Display::default() {
            if let Some(surface) = window.surface() {
                let monitor = display.monitor_at_surface(&surface)
                    .or_else(|| display.monitors().item(0).and_downcast::<gdk::Monitor>());
                if monitor.is_some() {
                    window.set_default_size(config.window_width, config.window_height);
                }
            }
        }

        QuickFindWindow { window, search_entry }
    }

    pub fn focus_search(&self) {
        self.search_entry.grab_focus();
    }

    // Con el feature compilado, usamos layer-shell SOLO si el compositor lo
    // soporta (niri, Hyprland, Sway...). En GNOME/KDE/X11 no existe el
    // protocolo, así que caemos a una ventana normal en vez de crashear.
    #[cfg(feature = "layer-shell")]
    fn setup_window(window: &gtk::ApplicationWindow) {
        if gtk4_layer_shell::is_supported() {
            use gtk4_layer_shell::{Edge, Layer, LayerShell};
            window.init_layer_shell();
            window.set_layer(Layer::Overlay);
            window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive);
            // Sin esto la ventana conserva su tamaño máximo y no encoge al vaciar.
            window.set_resizable(false);

            // Anclado arriba y centrado horizontalmente (estilo Spotlight).
            window.set_anchor(Edge::Top, true);
            window.set_anchor(Edge::Left, false);
            window.set_anchor(Edge::Right, false);
            window.set_anchor(Edge::Bottom, false);
            window.set_margin(Edge::Top, 180);
            // No reservamos zona exclusiva: es un overlay flotante.
            window.set_exclusive_zone(-1);
        } else {
            Self::setup_normal_window(window);
        }
    }

    #[cfg(not(feature = "layer-shell"))]
    fn setup_window(window: &gtk::ApplicationWindow) {
        Self::setup_normal_window(window);
    }

    /// Ventana sin decoraciones, tamaño según contenido. El compositor la
    /// posiciona (normalmente centrada). Usada fuera de wlroots/layer-shell.
    fn setup_normal_window(window: &gtk::ApplicationWindow) {
        window.set_decorated(false);
        window.set_resizable(false);

        if let Some(surface) = window.surface().and_then(|s| s.downcast::<gdk::Toplevel>().ok()) {
            surface.set_startup_id("com.leonardo.QuickFind");
        }
    }

    pub fn present(&self) {
        self.window.present();
    }

    fn update_results_ui(
        container: &gtk::Box,
        scrolled: &gtk::ScrolledWindow,
        entries: &[DesktopEntry],
        query: String,
        calc: Option<&str>,
        clips: Option<&[ClipItem]>,
        ts: Option<&crate::timestamp::TsResult>,
        ts_active: bool,
        rows: &Rc<RefCell<Vec<gtk::Box>>>,
        web_row_ref: &Rc<RefCell<Option<gtk::Box>>>,
        selected: usize,
        show_icons: bool,
        icon_size: i32,
    ) {
        while let Some(child) = container.first_child() {
            container.remove(&child);
        }
        rows.borrow_mut().clear();
        *web_row_ref.borrow_mut() = None;

        // Sin texto: colapsamos a solo la barra de búsqueda (estilo Spotlight).
        if query.is_empty() {
            scrolled.set_visible(false);
            return;
        }

        // Hay algo escrito: mostramos el área de resultados que va creciendo.
        scrolled.set_visible(true);

        // Modo historial del portapapeles.
        if let Some(clips) = clips {
            if clips.is_empty() {
                let msg = if crate::clipboard::is_available() {
                    "Historial del portapapeles vacío"
                } else {
                    "cliphist no está instalado o no está corriendo"
                };
                let label = gtk::Label::builder()
                    .label(msg)
                    .css_classes(vec!["empty-label".to_string()])
                    .build();
                container.append(&label);
                return;
            }
            for (i, item) in clips.iter().enumerate() {
                let row = Self::build_clip_row(item, i, selected);
                container.append(&row);
                rows.borrow_mut().push(row);
            }
            return;
        }

        // Modo conversión de timestamp: dos filas (UTC y local).
        if ts_active {
            match ts {
                Some(r) => {
                    let unit = if r.unit == "ms" { "milisegundos" } else { "segundos" };
                    let hint = gtk::Label::builder()
                        .label(&format!("Interpretado como {} · Enter para copiar", unit))
                        .halign(gtk::Align::Start)
                        .css_classes(vec!["ts-hint".to_string()])
                        .build();
                    container.append(&hint);

                    let utc = Self::build_ts_row("UTC", &r.utc, 0, selected);
                    let local = Self::build_ts_row("Local", &r.local, 1, selected);
                    container.append(&utc);
                    container.append(&local);
                    rows.borrow_mut().push(utc);
                    rows.borrow_mut().push(local);
                }
                None => {
                    let label = gtk::Label::builder()
                        .label("Timestamp inválido (epoch en segundos o ms)")
                        .css_classes(vec!["empty-label".to_string()])
                        .build();
                    container.append(&label);
                }
            }
            return;
        }

        // Resultado de una operación matemática.
        if let Some(result) = calc {
            container.append(&Self::build_calc_row(result));
            return;
        }

        for (i, entry) in entries.iter().enumerate() {
            let row = Self::build_app_row(entry, i, selected, show_icons, icon_size);
            container.append(&row);
            rows.borrow_mut().push(row);
        }

        if !query.is_empty() {
            let web_row = Self::build_web_row(&query);
            container.append(&web_row);
            *web_row_ref.borrow_mut() = Some(web_row);
        }
    }

    fn build_app_row(
        entry: &DesktopEntry,
        index: usize,
        selected: usize,
        show_icons: bool,
        icon_size: i32,
    ) -> gtk::Box {
        let row = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(12)
            .css_classes(vec!["result-row".to_string()])
            .build();

        if show_icons && !entry.icon.is_empty() {
            let icon = gtk::Image::builder()
                .pixel_size(icon_size)
                .css_classes(vec!["result-icon".to_string()])
                .build();

            if entry.icon.contains('/') || entry.icon.ends_with(".png") || entry.icon.ends_with(".svg") {
                icon.set_from_file(Some(&std::path::PathBuf::from(&entry.icon)));
            } else {
                icon.set_icon_name(Some(&entry.icon));
            }
            row.append(&icon);
        }

        let text_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .hexpand(true)
            .build();

        let name_label = gtk::Label::builder()
            .label(&entry.name)
            .halign(gtk::Align::Start)
            .css_classes(vec!["result-name".to_string()])
            .build();
        text_box.append(&name_label);

        if let Some(ref desc) = entry.comment {
            let desc_label = gtk::Label::builder()
                .label(desc)
                .halign(gtk::Align::Start)
                .css_classes(vec!["result-desc".to_string()])
                .build();
            text_box.append(&desc_label);
        }

        row.append(&text_box);

        if index < 9 {
            let shortcut = gtk::Label::builder()
                .label(&format!("⌘{}", index + 1))
                .css_classes(vec!["result-shortcut".to_string()])
                .valign(gtk::Align::Center)
                .build();
            row.append(&shortcut);
        }

        if index == selected {
            row.add_css_class("selected");
        }

        row
    }

    fn build_web_row(query: &str) -> gtk::Box {
        let row = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(12)
            .css_classes(vec!["web-search-row".to_string()])
            .build();

        let icon = gtk::Image::builder()
            .pixel_size(20)
            .css_classes(vec!["result-icon".to_string()])
            .icon_name("web-browser")
            .build();
        row.append(&icon);

        let label = gtk::Label::builder()
            .label(&format!("Buscar '{}' en la web  (Ctrl+Enter)", query))
            .halign(gtk::Align::Start)
            .css_classes(vec!["web-search-name".to_string()])
            .hexpand(true)
            .build();
        row.append(&label);

        row
    }

    fn build_clip_row(item: &ClipItem, index: usize, selected: usize) -> gtk::Box {
        let row = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(12)
            .css_classes(vec!["result-row".to_string()])
            .build();

        let icon = gtk::Image::builder()
            .pixel_size(22)
            .css_classes(vec!["result-icon".to_string()])
            .icon_name("edit-paste-symbolic")
            .valign(gtk::Align::Center)
            .build();
        row.append(&icon);

        // Vista previa en una sola línea, recortada con elipsis.
        let label = gtk::Label::builder()
            .label(&item.preview)
            .halign(gtk::Align::Start)
            .css_classes(vec!["result-name".to_string()])
            .hexpand(true)
            .ellipsize(gtk::pango::EllipsizeMode::End)
            .single_line_mode(true)
            .max_width_chars(60)
            .build();
        row.append(&label);

        if index < 9 {
            let shortcut = gtk::Label::builder()
                .label(&format!("⌘{}", index + 1))
                .css_classes(vec!["result-shortcut".to_string()])
                .valign(gtk::Align::Center)
                .build();
            row.append(&shortcut);
        }

        if index == selected {
            row.add_css_class("selected");
        }

        row
    }

    fn build_ts_row(tag: &str, value: &str, index: usize, selected: usize) -> gtk::Box {
        let row = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(12)
            .css_classes(vec!["result-row".to_string()])
            .build();

        let tag_label = gtk::Label::builder()
            .label(tag)
            .css_classes(vec!["ts-tag".to_string()])
            .valign(gtk::Align::Center)
            .width_request(60)
            .xalign(0.0)
            .build();
        row.append(&tag_label);

        let value_label = gtk::Label::builder()
            .label(value)
            .halign(gtk::Align::Start)
            .css_classes(vec!["ts-value".to_string()])
            .hexpand(true)
            .build();
        row.append(&value_label);

        if index < 9 {
            let shortcut = gtk::Label::builder()
                .label(&format!("⌘{}", index + 1))
                .css_classes(vec!["result-shortcut".to_string()])
                .valign(gtk::Align::Center)
                .build();
            row.append(&shortcut);
        }

        if index == selected {
            row.add_css_class("selected");
        }

        row
    }

    fn build_calc_row(result: &str) -> gtk::Box {
        let row = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(12)
            .css_classes(vec!["calc-row".to_string()])
            .build();

        let icon = gtk::Image::builder()
            .pixel_size(28)
            .css_classes(vec!["result-icon".to_string()])
            .icon_name("accessories-calculator-symbolic")
            .valign(gtk::Align::Center)
            .build();
        row.append(&icon);

        let value = gtk::Label::builder()
            .label(&format!("= {}", result))
            .halign(gtk::Align::Start)
            .css_classes(vec!["calc-result".to_string()])
            .hexpand(true)
            .build();
        row.append(&value);

        let hint = gtk::Label::builder()
            .label("Enter para copiar")
            .css_classes(vec!["calc-hint".to_string()])
            .valign(gtk::Align::Center)
            .build();
        row.append(&hint);

        row
    }

    /// Fuerza la altura del ScrolledWindow al alto natural del contenido
    /// (con tope de 420px). Evita que se quede "pegado" al tamaño de una
    /// búsqueda anterior cuando esta tenía 0 o 1 resultados.
    fn fit_scrolled(scrolled: &gtk::ScrolledWindow, container: &gtk::Box) {
        if !scrolled.is_visible() {
            return;
        }
        const MAX_H: i32 = 420;
        let (_, natural, _, _) = container.measure(gtk::Orientation::Vertical, -1);
        let h = natural.clamp(0, MAX_H);
        scrolled.set_min_content_height(h);
        scrolled.set_max_content_height(h);
    }

    fn update_selection(
        rows: &Rc<RefCell<Vec<gtk::Box>>>,
        selected: usize,
        scrolled: &gtk::ScrolledWindow,
    ) {
        let rows = rows.borrow();
        for (i, row) in rows.iter().enumerate() {
            if i == selected {
                row.add_css_class("selected");
            } else {
                row.remove_css_class("selected");
            }
        }

        // Desplazamos el viewport para que la fila seleccionada quede visible.
        if let Some(row) = rows.get(selected) {
            if let Some(parent) = row.parent() {
                if let Some(bounds) = row.compute_bounds(&parent) {
                    let adj = scrolled.vadjustment();
                    let top = bounds.y() as f64;
                    let bottom = top + bounds.height() as f64;
                    let page = adj.page_size();
                    let value = adj.value();
                    if top < value {
                        adj.set_value(top);
                    } else if bottom > value + page {
                        adj.set_value(bottom - page);
                    }
                }
            }
        }
    }
}
