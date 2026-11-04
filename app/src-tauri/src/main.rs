// Stops a console window from popping up next to the app on Windows
// release builds. Debug builds keep it so you can see logs.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    larder_app_lib::run();
}
