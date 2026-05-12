mod config;
use config::WindowConfig;

slint::include_modules!();

use arboard::Clipboard;
use i_slint_backend_winit::winit::platform::wayland::WindowAttributesExtWayland;
use i_slint_backend_winit::winit::platform::x11::WindowAttributesExtX11;
use rfd::FileDialog;
use std::sync::{Arc, Mutex};
use std::{env, fs, path::PathBuf, thread};

struct EditorState {
    current_path: Option<PathBuf>,
    last_saved_content: String,
}

fn update_line_numbers(ui: &AppWindow, text: &str) {
    let line_count = text.split('\n').count().max(1);
    let mut lines_string = String::new();
    for i in 1..=line_count {
        lines_string.push_str(&format!("{}\n", i));
    }
    ui.set_line_numbers(lines_string.into());
}

fn update_title(ui: &AppWindow, state: &EditorState, current_content: &str) {
    let filename = match &state.current_path {
        Some(path) => path.file_name().unwrap().to_string_lossy().to_string(),
        None => "New Document".to_string(),
    };
    let is_modified = current_content != state.last_saved_content;
    ui.set_window_title(format!("{}{}", if is_modified { "*" } else { "" }, filename).into());
}

fn perform_save(
    ui_weak: slint::Weak<AppWindow>,
    state: Arc<Mutex<EditorState>>,
    close_after: bool,
) {
    let ui = ui_weak.upgrade().unwrap();
    let content = ui.get_text().to_string();
    let mut s = state.lock().unwrap();

    if let Some(path) = &s.current_path {
        let _ = fs::write(path, &content);
        s.last_saved_content = content;

        if close_after {
            let logical_size = ui.window().size().to_logical(ui.window().scale_factor());
            WindowConfig::save(logical_size.width, logical_size.height);
            std::process::exit(0);
        }
    } else {
        let state_inner = state.clone();
        let content_to_thread = content.clone();
        let ui_handle = ui_weak.clone();

        thread::spawn(move || {
            if let Some(path) = FileDialog::new()
                .set_file_name("new_document.txt")
                .save_file()
            {
                let _ = fs::write(&path, &content_to_thread);

                slint::invoke_from_event_loop(move || {
                    if let Some(ui_up) = ui_handle.upgrade() {
                        let mut s = state_inner.lock().unwrap();
                        s.current_path = Some(path);
                        s.last_saved_content = content_to_thread;

                        if close_after {
                            let logical_size = ui_up
                                .window()
                                .size()
                                .to_logical(ui_up.window().scale_factor());
                            WindowConfig::save(logical_size.width, logical_size.height);
                            std::process::exit(0);
                        }
                    }
                })
                .unwrap();
            }
        });
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let mut backend = i_slint_backend_winit::Backend::new().map_err(|e| e.to_string())?;

    backend.window_attributes_hook = Some(Box::new(|builder| {
        let builder = WindowAttributesExtX11::with_name(
            builder,
            "org.sunaipa.justext",
            "org.sunaipa.justext",
        );
        let builder = WindowAttributesExtWayland::with_name(
            builder,
            "org.sunaipa.justext",
            "org.sunaipa.justext",
        );

        builder
    }));

    slint::platform::set_platform(Box::new(backend)).map_err(|e| e.to_string())?;

    let cfg = WindowConfig::load();
    let ui = AppWindow::new()?;
    let args: Vec<String> = env::args().collect();

    ui.window()
        .set_size(slint::LogicalSize::new(cfg.width, cfg.height));

    let initial_state = if args.len() > 1 && PathBuf::from(&args[1]).exists() {
        let path = PathBuf::from(&args[1]);
        let content = fs::read_to_string(&path).unwrap_or_default();
        ui.set_text(content.clone().into());
        EditorState {
            current_path: Some(path),
            last_saved_content: content,
        }
    } else {
        EditorState {
            current_path: None,
            last_saved_content: String::new(),
        }
    };

    // Thread-safe state yönetimi
    let state = Arc::new(Mutex::new(initial_state));

    ui.on_open_url(|url| {
        let url_str = url.as_str();
        if let Err(e) = webbrowser::open(url_str) {
            eprintln!("Failed to open URL: {}", e);
        }
    });

    // -- OPEN FILE ---
    let ui_handle = ui.as_weak();
    let state_open = state.clone();

    ui.on_open_requested(move || {
        // thread::spawn için handle'ı klonla
        let ui_handle_for_thread = ui_handle.clone();
        let state_inner = state_open.clone();

        thread::spawn(move || {
            if let Some(path) = rfd::FileDialog::new().set_title("Open File").pick_file() {
                if let Ok(content) = fs::read_to_string(&path) {
                    // Event loop için handle'ı tekrar klonla (içeride move edileceği için)
                    let ui_handle_for_loop = ui_handle_for_thread.clone();

                    slint::invoke_from_event_loop(move || {
                        if let Some(ui_up) = ui_handle_for_loop.upgrade() {
                            let mut s = state_inner.lock().unwrap();

                            ui_up.set_text(content.clone().into());
                            s.current_path = Some(path);
                            s.last_saved_content = content.clone();

                            update_title(&ui_up, &s, &content);
                            ui_up.set_is_dirty(false);
                        }
                    })
                    .unwrap();
                }
            }
        });
    });

    // --- SAVE LOGIC ---
    let state_save = state.clone();
    let ui_weak_save = ui.as_weak();
    ui.on_save_requested(move || {
        perform_save(ui_weak_save.clone(), state_save.clone(), false);
    });

    // --- EDIT CALLBACKS ---
    let ui_handle = ui.as_weak();
    ui.on_copy_all_requested(move || {
        let ui = ui_handle.upgrade().unwrap();
        if let Ok(mut cb) = Clipboard::new() {
            let _ = cb.set_text(ui.get_text().to_string());
        }
    });

    let ui_handle = ui.as_weak();
    ui.on_cut_all_requested(move || {
        let ui = ui_handle.upgrade().unwrap();
        if let Ok(mut cb) = Clipboard::new() {
            let text = ui.get_text().to_string();
            let _ = cb.set_text(text);
            ui.set_text("".into());
        }
    });

    let ui_handle = ui.as_weak();
    ui.on_delete_all_requested(move || {
        ui_handle.upgrade().unwrap().set_text("".into());
    });

    // -- BEFORE-APP-CLOSE --
    let ui_handle = ui.as_weak();
    let state_close = state.clone();
    ui.window().on_close_requested(move || {
        if let Some(ui) = ui_handle.upgrade() {
            let s = state_close.lock().unwrap();
            let current_content = ui.get_text().to_string();

            if current_content != s.last_saved_content {
                ui.set_show_close_modal(true);
                return slint::CloseRequestResponse::KeepWindowShown;
            }

            let logical_size = ui.window().size().to_logical(ui.window().scale_factor());
            WindowConfig::save(logical_size.width, logical_size.height);
        }
        slint::CloseRequestResponse::HideWindow
    });

    // --- MODAL: SAVE AND CLOSE ---
    let state_modal_save = state.clone();
    let ui_weak_modal = ui.as_weak();
    ui.on_save_and_close(move || {
        perform_save(ui_weak_modal.clone(), state_modal_save.clone(), true);
    });

    // --- MODAL: CLOSE ANYWAY ---
    let ui_handle_exit = ui.as_weak();
    ui.on_close_anyway(move || {
        if let Some(ui) = ui_handle_exit.upgrade() {
            let logical_size = ui.window().size().to_logical(ui.window().scale_factor());
            WindowConfig::save(logical_size.width, logical_size.height);
        }
        std::process::exit(0);
    });

    // --- AUTO-SAVE & REFRESH ---
    let ui_weak_timer = ui.as_weak();
    let state_timer = state.clone();
    let mut last_focus = true;

    let timer = slint::Timer::default();
    timer.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_millis(100),
        move || {
            if let Some(ui) = ui_weak_timer.upgrade() {
                let current_text = ui.get_text().to_string();
                let current_focus = ui.get_window_focused();

                if last_focus && !current_focus {
                    let mut s = state_timer.lock().unwrap();
                    if let Some(path) = &s.current_path {
                        let _ = fs::write(path, &current_text);
                        s.last_saved_content = current_text.clone();
                    }
                }
                last_focus = current_focus;

                update_line_numbers(&ui, &current_text);
                update_title(&ui, &state_timer.lock().unwrap(), &current_text);
            }
        },
    );

    ui.run()
}
