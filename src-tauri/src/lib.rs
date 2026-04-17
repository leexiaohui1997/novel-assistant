pub mod config;

use tauri::Builder;

pub fn run() {
    Builder::default()
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用时出错");
}
