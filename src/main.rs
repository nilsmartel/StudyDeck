mod app;
mod loader;
mod question;

use std::path::PathBuf;

use app::App;

fn main() -> iced::Result {
    // Directory of scenario JSON files: first CLI argument, or the default.
    let dir = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "example-questions".to_string()),
    );

    iced::application(
        move || App::new(loader::read_scenarios(&dir)),
        App::update,
        App::view,
    )
    .title("Rust Answering Machine")
    .theme(theme)
    .run()
}

fn theme(_state: &App) -> iced::Theme {
    iced::Theme::TokyoNight
}
