use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use clap::Args;
use walkdir::WalkDir;

use crate::replace::project_replacements;
use crate::template::{resolve_template_source, TemplateSource};

#[derive(Debug, Args)]
pub struct NewArgs {
    /// 新项目名称（kebab-case，如 my-app）
    pub name: String,

    /// 目标目录（默认：当前目录下的 `<name>`）
    #[arg(short, long)]
    pub path: Option<PathBuf>,

    /// 本地模板目录（复制文件，不 git clone）
    #[arg(long = "template-dir", value_name = "DIR")]
    pub template_path: Option<PathBuf>,

    /// 跳过 git init
    #[arg(long)]
    pub no_git: bool,
}

pub fn run(args: NewArgs) -> Result<()> {
    validate_project_name(&args.name)?;

    let dest = match &args.path {
        Some(p) => p.clone(),
        None => std::env::current_dir()
            .context("无法读取当前目录")?
            .join(&args.name),
    };

    if dest.exists() {
        bail!(
            "目标路径已存在，拒绝覆盖: {}",
            dest.display()
        );
    }

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("无法创建父目录 {}", parent.display()))?;
    }

    let source = resolve_template_source(args.template_path.clone())?;

    println!("正在从模板创建项目…");
    match &source {
        TemplateSource::LocalDir(path) => {
            println!("  模式: 本地目录复制（不走 git clone）");
            println!("  模板: {}", path.display());
        }
        TemplateSource::Git { url, branch } => {
            println!("  模式: git clone");
            println!("  仓库: {url}");
            println!("  分支: {branch}");
        }
    }
    println!("  目录: {}", dest.display());

    materialize_template(&source, &dest)?;
    remove_template_artifacts(&dest)?;
    apply_renames(&dest, &args.name)?;
    init_local_files(&dest)?;
    if !args.no_git {
        git_init(&dest)?;
    }

    let display_dest = fs::canonicalize(&dest).unwrap_or(dest);
    print_next_steps(&display_dest, &args.name);
    Ok(())
}

fn validate_project_name(name: &str) -> Result<()> {
    let ok = !name.is_empty()
        && name.len() <= 64
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !name.starts_with('-')
        && !name.ends_with('-')
        && !name.contains("--");

    if !ok {
        bail!(
            "项目名须为小写 kebab-case（字母、数字、单连字符），例如: my-app"
        );
    }
    Ok(())
}

fn materialize_template(source: &TemplateSource, dest: &Path) -> Result<()> {
    match source {
        TemplateSource::LocalDir(src) => copy_local_template(src, dest),
        TemplateSource::Git { url, branch } => git_clone_template(url, branch, dest),
    }
}

fn copy_local_template(src: &Path, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest).context("创建目标目录失败")?;

    for entry in WalkDir::new(src)
        .into_iter()
        .filter_entry(|e| !should_skip_path(e.path(), src))
    {
        let entry = entry?;
        let rel = entry
            .path()
            .strip_prefix(src)
            .context("模板路径前缀异常")?;
        if rel.as_os_str().is_empty() {
            continue;
        }

        let target = dest.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)
                .with_context(|| format!("创建目录 {} 失败", target.display()))?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("创建目录 {} 失败", parent.display()))?;
            }
            fs::copy(entry.path(), &target).with_context(|| {
                format!(
                    "复制 {} → {} 失败",
                    entry.path().display(),
                    target.display()
                )
            })?;
        }
    }
    Ok(())
}

fn git_clone_template(url: &str, branch: &str, dest: &Path) -> Result<()> {
    let status = Command::new("git")
        .args(["clone", "--depth", "1", "--branch", branch, url])
        .arg(dest)
        .status()
        .context("执行 git clone 失败（请确认已安装 git 且网络可访问远程仓库）")?;

    if !status.success() {
        bail!("git clone 失败（退出码 {:?}）", status.code());
    }
    Ok(())
}

fn remove_template_artifacts(dest: &Path) -> Result<()> {
    let git_dir = dest.join(".git");
    if git_dir.exists() {
        fs::remove_dir_all(&git_dir).context("删除 .git 失败")?;
    }

    // lock 文件含模板路径，由用户自行 npm install 生成
    let lock = dest.join("package-lock.json");
    if lock.exists() {
        fs::remove_file(&lock).context("删除 package-lock.json 失败")?;
    }

    for dir in ["target", "node_modules"] {
        let p = dest.join(dir);
        if p.exists() {
            fs::remove_dir_all(&p).with_context(|| format!("删除 {} 失败", dir))?;
        }
    }

    Ok(())
}

fn apply_renames(dest: &Path, project_name: &str) -> Result<()> {
    let replacements = project_replacements(project_name);

    for entry in WalkDir::new(dest)
        .into_iter()
        .filter_entry(|e| !should_skip_path(e.path(), dest))
    {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if !is_probably_text(path) {
            continue;
        }

        let content = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let mut next = content.clone();
        for (from, to) in &replacements {
            next = next.replace(from, to);
        }

        if next != content {
            fs::write(path, next).with_context(|| format!("写入 {} 失败", path.display()))?;
        }
    }

    Ok(())
}

fn init_local_files(dest: &Path) -> Result<()> {
    copy_if_example(dest, ".env.example", ".env")?;
    copy_if_example(
        dest,
        "docker-compose.example.yml",
        "docker-compose.yml",
    )?;
    Ok(())
}

fn copy_if_example(dest: &Path, example: &str, target: &str) -> Result<()> {
    let from = dest.join(example);
    let to = dest.join(target);
    if from.exists() && !to.exists() {
        fs::copy(&from, &to).with_context(|| {
            format!("复制 {} → {}", from.display(), to.display())
        })?;
    }
    Ok(())
}

fn git_init(dest: &Path) -> Result<()> {
    let status = Command::new("git")
        .args(["init"])
        .current_dir(dest)
        .status()
        .context("执行 git init 失败")?;
    if !status.success() {
        bail!("git init 失败");
    }
    Ok(())
}

fn print_next_steps(dest: &Path, name: &str) {
    let dir = dest.display();
    println!();
    println!("✓ 项目 {name} 已创建: {dir}");
    println!();
    println!("下一步:");
    println!("  cd {dir}");
    println!("  npm install");
    println!("  cp .env.example .env   # 若尚未生成 .env");
    println!("  just dev-db");
    println!("  just migrate && just seed");
    println!("  just dev-api");
    println!("  just dev-admin   # 管理后台 :5174");
    println!("  just dev-web     # ToC :5173");
}

fn should_skip_path(path: &Path, root: &Path) -> bool {
    let rel = path.strip_prefix(root).unwrap_or(path);
    let s = rel.to_string_lossy();
    s.starts_with(".git")
        || s.contains("/.git/")
        || s.starts_with("target")
        || s.contains("/target/")
        || s.starts_with("node_modules")
        || s.contains("/node_modules/")
        || s.ends_with(".png")
        || s.ends_with(".jpg")
        || s.ends_with(".ico")
        || s.ends_with(".woff")
        || s.ends_with(".woff2")
}

fn is_probably_text(path: &Path) -> bool {
    const TEXT_EXT: &[&str] = &[
        "rs", "toml", "json", "md", "yml", "yaml", "env", "example", "sql", "sh",
        "ts", "tsx", "js", "jsx", "css", "html", "justfile", "gitignore", "txt",
    ];
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| TEXT_EXT.contains(&e))
        .unwrap_or(false)
        || path.file_name().is_some_and(|n| {
            let n = n.to_string_lossy();
            n == "justfile" || n == ".env.example" || n.starts_with('.') && n.contains("env")
        })
}
