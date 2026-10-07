#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod art;
mod auth;
mod player;
mod ui;
mod ytm;

use eframe::egui;
use std::sync::OnceLock;

pub const APP_NAME: &str = "Tubefast";
const APP_ID: &str = "tubefast";
const LEGACY_ID: &str = "plak";
static PROFILE: OnceLock<String> = OnceLock::new();

pub fn app_id() -> &'static str {
    PROFILE.get().map_or(APP_ID, String::as_str)
}

fn take_profile(arguments: &mut Vec<String>) {
    let Some(at) = arguments.iter().position(|argument| argument == "--profile") else {
        return;
    };
    let name = arguments.drain(at..(at + 2).min(arguments.len())).nth(1).unwrap_or_default();
    if !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        let _ = PROFILE.set(format!("{APP_ID}-{name}"));
    }
}

#[cfg(windows)]
fn attach_console() {
    use windows_sys::Win32::System::Console::{ATTACH_PARENT_PROCESS, AttachConsole};
    unsafe { AttachConsole(ATTACH_PARENT_PROCESS) };
}

#[cfg(not(windows))]
fn attach_console() {}

fn adopt_legacy_data() {
    if PROFILE.get().is_some() {
        return;
    }
    let (Some(old), Some(new)) = (eframe::storage_dir(LEGACY_ID), eframe::storage_dir(APP_ID)) else {
        return;
    };
    if !old.exists() || new.exists() {
        return;
    }
    let moved = new.parent().is_some_and(|folder| std::fs::create_dir_all(folder).is_ok()) && std::fs::rename(&old, &new).is_ok();
    if let Some(folder) = old.parent().filter(|folder| moved && folder.ends_with(LEGACY_ID)) {
        let _ = std::fs::remove_dir(folder);
    }
}

fn main() -> eframe::Result {
    let mut arguments: Vec<String> = std::env::args().skip(1).collect();
    take_profile(&mut arguments);
    let mut arguments = arguments.into_iter();
    let (flag, value) = (arguments.next(), arguments.next());
    if flag.as_deref() == Some("--selftest") {
        attach_console();
        match player::selftest(value.as_deref().unwrap_or("daft punk get lucky")) {
            Ok(report) => println!("{report}"),
            Err(error) => {
                eprintln!("selftest failed: {error}");
                std::process::exit(1);
            }
        }
        return Ok(());
    }
    let (query, autoplay) = match (flag, value) {
        (Some(flag), Some(query)) if flag == "--play" => (Some(query), true),
        (Some(query), None) if !query.starts_with("--") => (Some(query), false),
        _ => (None, false),
    };
    adopt_legacy_data();
    let viewport = egui::ViewportBuilder::default()
        .with_title(APP_NAME)
        .with_app_id(app_id())
        .with_inner_size(app::WINDOW_SIZE)
        .with_min_inner_size(app::WINDOW_MIN)
        .with_icon(ui::app_icon());
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        APP_NAME,
        options,
        Box::new(move |cc| Ok(Box::new(app::App::new(cc, query, autoplay)))),
    )
}
