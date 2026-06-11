mod config;
mod launcher;
mod ui;

use gtk::prelude::*;
use gtk::{glib, Application};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

const APP_ID: &str = "com.leonardo.QuickFind";

fn main() -> glib::ExitCode {
    gtk::init().expect("Failed to initialize GTK");
    config::Config::ensure_default();

    let app = Application::builder().application_id(APP_ID).build();
    let current_window: Rc<RefCell<Option<ui::QuickFindWindow>>> = Rc::new(RefCell::new(None));

    {
        let current_window = current_window.clone();
        app.connect_activate(move |app| {
            let mut win_ref = current_window.borrow_mut();
            if let Some(ref window) = *win_ref {
                window.present();
                window.focus_search();
                return;
            }

            let config = Arc::new(config::Config::load());
            let launcher = Arc::new(launcher::AppLauncher::new());
            
            let window = ui::QuickFindWindow::new(app, config, launcher);
            window.present();
            window.focus_search();
            *win_ref = Some(window);
        });
    }

    app.run()
}
