//! 模板仓库配置。

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

/// 默认模板 Git 地址（可通过环境变量 `FSR_TEMPLATE_REPO` 覆盖）
pub const DEFAULT_TEMPLATE_REPO: &str =
    "https://github.com/mozhiyun/fullstack-rust-react-starter.git";

/// kebab-case 项目 slug（根 package.json、Docker 容器名等）
pub const TEMPLATE_SLUG: &str = "fullstack-rust-react-starter";

/// snake_case（Postgres 库名、Docker volume 名等）
pub const TEMPLATE_SLUG_SNAKE: &str = "fullstack_rust_react_starter";

/// 人类可读标题（页面标题、侧栏、OpenAPI 描述等）
pub const TEMPLATE_TITLE: &str = "Fullstack Rust React Starter";

/// OpenAPI `info.title`（须先于 TEMPLATE_TITLE 替换，避免误伤）
pub const TEMPLATE_API_TITLE: &str = "Fullstack Rust React Starter API";

/// 模板来源：本地目录复制，或远程 git clone。
pub enum TemplateSource {
    LocalDir(PathBuf),
    Git { url: String, branch: String },
}

pub fn template_repo_url() -> String {
    std::env::var("FSR_TEMPLATE_REPO").unwrap_or_else(|_| DEFAULT_TEMPLATE_REPO.to_string())
}

pub fn template_branch() -> String {
    std::env::var("FSR_TEMPLATE_BRANCH").unwrap_or_else(|_| "main".to_string())
}

/// 解析模板来源：`--template-dir` > `FSR_TEMPLATE_REPO` > 默认 GitHub。
pub fn resolve_template_source(cli_template: Option<PathBuf>) -> Result<TemplateSource> {
    if let Some(path) = cli_template {
        return Ok(TemplateSource::LocalDir(
            canonicalize_template_dir(&path).with_context(|| {
                format!("--template 路径无效: {}", path.display())
            })?,
        ));
    }

    let spec = template_repo_url();
    if let Some(path) = parse_local_spec(&spec)? {
        return Ok(TemplateSource::LocalDir(path));
    }

    if is_git_remote(&spec) {
        return Ok(TemplateSource::Git {
            url: spec,
            branch: template_branch(),
        });
    }

    bail!(
        "无法识别模板来源: {spec}\n\
         远程请用 https:// 或 git@ 地址；本地请用目录路径或 file:///path"
    );
}

fn parse_local_spec(spec: &str) -> Result<Option<PathBuf>> {
    let path = if let Some(rest) = spec.strip_prefix("file://") {
        PathBuf::from(rest)
    } else if spec.starts_with("https://")
        || spec.starts_with("http://")
        || spec.starts_with("git@")
    {
        return Ok(None);
    } else {
        PathBuf::from(spec)
    };

    if !path.exists() {
        bail!(
            "本地模板路径不存在: {}\n\
             提示: 环境变量须与 fsr 在同一行，例如:\n\
               FSR_TEMPLATE_REPO=/home/zhangyi/fullstack-rust-react-starter fsr new demo-app",
            path.display()
        );
    }

    Ok(Some(canonicalize_template_dir(&path)?))
}

fn canonicalize_template_dir(path: &Path) -> Result<PathBuf> {
    let meta = fs::metadata(path)
        .with_context(|| format!("无法访问 {}", path.display()))?;
    if !meta.is_dir() {
        bail!("模板路径不是目录: {}", path.display());
    }
    fs::canonicalize(path).context("无法解析模板目录绝对路径")
}

fn is_git_remote(spec: &str) -> bool {
    spec.starts_with("https://") || spec.starts_with("http://") || spec.starts_with("git@")
}
