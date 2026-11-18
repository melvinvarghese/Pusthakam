//! Closing the main window hides it; the explicit tray Quit action ends the process/session.
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Runtime, Window, WindowEvent,
};

struct TrayState<R: Runtime> {
    _icon: TrayIcon<R>,
    quitting: AtomicBool,
}

fn show<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        if let Err(error) = window
            .show()
            .and_then(|_| window.unminimize())
            .and_then(|_| window.set_focus())
        {
            tracing::warn!(%error, "Could not restore the application window");
        }
    }
}

pub fn install<R: Runtime>(app: &AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
    let open = MenuItem::with_id(app, "open-pusthakam", "Open Pusthakam", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit-pusthakam", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &quit])?;
    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or("Application tray icon is unavailable")?;
    let tray = TrayIconBuilder::with_id("pusthakam-tray")
        .icon(icon)
        .tooltip("Pusthakam — click to open; right-click for Quit")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open-pusthakam" => show(app),
            "quit-pusthakam" => {
                if let Some(state) = app.try_state::<TrayState<R>>() {
                    state.quitting.store(true, Ordering::SeqCst);
                }
                app.exit(0);
            }
            _ => (),
        })
        .on_tray_icon_event(|tray, event| {
            if matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
            ) {
                show(tray.app_handle());
            }
        })
        .build(app)?;
    app.manage(TrayState {
        _icon: tray,
        quitting: AtomicBool::new(false),
    });
    Ok(())
}

pub fn on_window_event<R: Runtime>(window: &Window<R>, event: &WindowEvent) {
    if window.label() != "main" {
        return;
    }
    if let WindowEvent::CloseRequested { api, .. } = event {
        // Without a working tray, keep ordinary close behavior so the window is never stranded.
        if let Some(state) = window.app_handle().try_state::<TrayState<R>>() {
            if !state.quitting.load(Ordering::SeqCst) {
                match window.hide() {
                    Ok(()) => api.prevent_close(),
                    Err(error) => tracing::warn!(%error, "Could not hide the application window"),
                }
            }
        }
    }
}
