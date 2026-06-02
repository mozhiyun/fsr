use std::process::Command;

use anyhow::{bail, Context, Result};
use clap::Args;

/// 默认 fsr CLI 源码仓库（`cargo install --git`）
pub const DEFAULT_FSR_INSTALL_REPO: &str = "https://github.com/mozhiyun/fsr";

#[derive(Args)]
pub struct UpgradeArgs {
    /// 安装的 Git 分支
    #[arg(long, default_value = "main")]
    pub branch: String,

    /// fsr 源码仓库 URL（默认见 `FSR_INSTALL_REPO` 或 GitHub mozhiyun/fsr）
    #[arg(long)]
    pub repo: Option<String>,
}

pub fn run(args: UpgradeArgs) -> Result<()> {
    let repo = args
        .repo
        .or_else(|| std::env::var("FSR_INSTALL_REPO").ok())
        .unwrap_or_else(|| DEFAULT_FSR_INSTALL_REPO.to_string());

    println!("正在安装最新 fsr…");
    println!("  仓库: {repo}");
    println!("  分支: {}", args.branch);
    println!("  命令: cargo install --git … --force");
    println!();

    let status = Command::new("cargo")
        .args([
            "install",
            "--git",
            &repo,
            "--branch",
            &args.branch,
            "--force",
        ])
        .status()
        .context(
            "执行 cargo install 失败（请确认已安装 Rust/cargo，且 cargo 在 PATH 中）",
        )?;

    if !status.success() {
        bail!("升级失败（退出码 {:?}）", status.code());
    }

    println!();
    println!("✓ fsr 已更新到 {repo}@{branch}", branch = args.branch);
    println!("  若命令未生效，请确认 ~/.cargo/bin 在 PATH 中");
    println!("  验证: fsr --version");

    Ok(())
}
