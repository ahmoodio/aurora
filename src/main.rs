mod app;
mod core;
mod ui;

fn main() {
    let app = app::AuroraApp::new();
    app.run();
}
