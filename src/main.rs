mod new;
mod replace;
mod template;
mod upgrade;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "fsr", about = "Scaffold Fullstack Rust + React monorepos", version)]
struct Top {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// 从模板创建新项目
    New {
        /// 新项目名称（kebab-case，如 my-app）
        name: String,

        /// 本地模板目录（复制文件，不 git clone）
        #[arg(long = "template-dir", value_name = "DIR")]
        template_dir: Option<PathBuf>,

        /// 目标目录（默认：当前目录下的 `<name>`）
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// 跳过 git init
        #[arg(long)]
        no_git: bool,
    },
    /// 从 GitHub 拉取最新 fsr 并覆盖安装（cargo install --force）
    Upgrade(#[command(flatten)] upgrade::UpgradeArgs),
}

fn main() -> Result<()> {
    match Top::parse().cmd {
        Cmd::New {
            name,
            template_dir,
            path,
            no_git,
        } => new::run(new::NewArgs {
            name,
            path,
            template_path: template_dir,
            no_git,
        }),
        Cmd::Upgrade(args) => upgrade::run(args),
    }
}
