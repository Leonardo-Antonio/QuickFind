use gtk::gdk;
use gtk::prelude::*;
use std::sync::Arc;
use std::cell::RefCell;
use std::rc::Rc;

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
    pub fn new(app: &gtk::Application, config: Arc<Config>, launcher: Arc<AppLauncher>) -> Self {
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
            .max_content_height(360)
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

        let results_box_clone = results_box.clone();
        let scrolled_clone = scrolled.clone();
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
            let new_results = if text.is_empty() {
                Vec::new()
            } else {
                launcher_clone.search(&text, config_clone.max_results)
                    .into_iter()
                    .map(|(entry, _)| entry)
                    .collect()
            };

            *results_clone.borrow_mut() = new_results;
            *selected_clone.borrow_mut() = 0;
            Self::update_results_ui(
                &results_box_clone,
                &scrolled_clone,
                &results_clone.borrow(),
                text,
                &rows_clone,
                &web_row_clone,
                *selected_clone.borrow(),
                config_clone.show_icons,
                config_clone.icon_size,
            );
        });

        let win_key = window.clone();
        let search_key = search_entry.clone();
        let results_key = results.clone();
        let selected_key = selected_index.clone();
        let rows_key = result_rows.clone();
        let launcher_key = launcher.clone();
        let query_key = current_query.clone();

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
                    Self::update_selection(&rows_key, *idx);
                    glib::Propagation::Stop
                }
                gdk::Key::Down => {
                    let mut idx = selected_key.borrow_mut();
                    let app_count = results_key.borrow().len();
                    let max_idx = app_count.saturating_sub(1);
                    if *idx < max_idx {
                        *idx += 1;
                    }
                    Self::update_selection(&rows_key, *idx);
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

    #[cfg(feature = "layer-shell")]
    fn setup_window(window: &gtk::ApplicationWindow) {
        use gtk4_layer_shell::{Edge, Layer, LayerShell};
        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive);

        // Anclado arriba y centrado horizontalmente (estilo Spotlight).
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Left, false);
        window.set_anchor(Edge::Right, false);
        window.set_anchor(Edge::Bottom, false);
        window.set_margin(Edge::Top, 180);
        // No reservamos zona exclusiva: es un overlay flotante.
        window.set_exclusive_zone(-1);
    }

    #[cfg(not(feature = "layer-shell"))]
    fn setup_window(window: &gtk::ApplicationWindow) {
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

    fn update_selection(rows: &Rc<RefCell<Vec<gtk::Box>>>, selected: usize) {
        for (i, row) in rows.borrow().iter().enumerate() {
            if i == selected {
                row.add_css_class("selected");
            } else {
                row.remove_css_class("selected");
            }
        }
    }
}
