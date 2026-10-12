use std::path::Path;

use log::LevelFilter;
use tauri::Manager;
use tauri_plugin_log::{Target, TargetKind, TimezoneStrategy};

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presentation;
pub mod test_support;
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn load_env() {
    dotenvy::dotenv().ok();
}

fn get_log_level() -> LevelFilter {
    if cfg!(test) {
        LevelFilter::Trace
    } else if cfg!(debug_assertions) {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    }
}

async fn create_pool(database_path: &Path) -> sqlx::Result<sqlx::SqlitePool> {
    let option = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(database_path)
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
        .busy_timeout(std::time::Duration::from_secs(5))
        .foreign_keys(true);

    sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(option)
        .await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let log_level = get_log_level();
    let log_output_target = match log_level {
        LevelFilter::Info => vec![
            Target::new(TargetKind::Stdout),
            Target::new(TargetKind::LogDir { file_name: None }),
        ],
        _ => vec![
            Target::new(TargetKind::Stdout),
            Target::new(TargetKind::Webview),
        ],
    };

    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets(log_output_target)
                .level(log_level)
                .timezone_strategy(TimezoneStrategy::UseLocal)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .setup(|app| {
            #[cfg(debug_assertions)]
            load_env();

            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let database_path = data_dir.join("pundarika.sqlite3");

            let pool = tauri::async_runtime::block_on(async {
                let pool = create_pool(&database_path).await?;
                sqlx::migrate!("./migrations").run(&pool).await?;
                Ok::<_, sqlx::Error>(pool)
            })?;

            app.manage(pool);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
