// #![windows_subsystem = "windows"]
// 在debug模式下终端显示print，发行版不显示终端窗口
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod dialog;
pub mod myapp;
pub mod pyenv;
pub mod resources;
pub mod status;
pub mod steps;
pub mod style;
pub mod utils;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let mut app = myapp::MyApp::new();
    app.run();

    Ok(())
}

// pub async fn cmd_main() -> Result<()> {
//     use pyenv::Installer;
//     let target_dir = std::env::current_dir()?;

//     use pyenv::{create_winlnk, fix_patches};
//     use pyenv::{ensure_python_venv, install_requirements};

//     let mut installer = Installer::new(target_dir)?;

//     ensure_python_venv(&mut installer).await?;
//     install_requirements(&installer).await?;
//     create_winlnk(&installer, &installer.target_dir())?;
//     fix_patches(&installer)?;

//     Ok(())
// }
