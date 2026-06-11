mod calc;
mod clipboard;
mod config;
mod launcher;
mod timestamp;
mod ui;

use gtk::prelude::*;
use gtk::{glib, Application};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

const APP_ID: &str = "com.leonardo.QuickFind";

/// Lee un valor de consulta inicial de `-q`/`--query` (acepta `-q="texto"`).
fn parse_initial_query() -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        if arg == "-q" || arg == "--query" {
            return args.get(i + 1).cloned();
        }
        if let Some(rest) = arg.strip_prefix("-q=").or_else(|| arg.strip_prefix("--query=")) {
            return Some(rest.to_string());
        }
        i += 1;
    }
    None
}

fn main() -> glib::ExitCode {
    gtk::init().expect("Failed to initialize GTK");
    config::Config::ensure_default();

    let initial_query = parse_initial_query();

    let app = Application::builder().application_id(APP_ID).build();
    let current_window: Rc<RefCell<Option<ui::QuickFindWindow>>> = Rc::new(RefCell::new(None));

    {
        let current_window = current_window.clone();
        let initial_query = initial_query.clone();
        app.connect_activate(move |app| {
            let mut win_ref = current_window.borrow_mut();
            if let Some(ref window) = *win_ref {
                window.present();
                window.focus_search();
                return;
            }

            let config = Arc::new(config::Config::load());
            let launcher = Arc::new(launcher::AppLauncher::new());

            let window = ui::QuickFindWindow::new(app, config, launcher, initial_query.clone());
            window.present();
            window.focus_search();
            *win_ref = Some(window);
        });
    }

    // No dejamos que GTK intente parsear nuestros argumentos (-q, etc.).
    let prog = std::env::args().next().unwrap_or_default();
    app.run_with_args(&[prog])
}
