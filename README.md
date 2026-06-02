# fsr — Fullstack Rust 脚手架 CLI

从 [fullstack-rust-react-starter](https://github.com/mozhiyun/fullstack-rust-react-starter) 模板拉取并初始化新项目。

本仓库 **仅包含 CLI**；业务模板在独立仓库，clone 模板时不会带上本工具。

## 安装

```bash
cargo install --git https://github.com/mozhiyun/fsr
# 或克隆后本地安装:
# git clone git@github.com:mozhiyun/fsr.git && cd fsr && cargo install --path .
```

## 用法

```bash
fsr new my-app
```

- 在当前目录创建 `./my-app/`（目录**必须不存在**，拒绝覆盖）
- 默认 **git clone** 远程模板 → 删除 `.git` → 替换项目名 → 复制 `.env` / `docker-compose.yml` → `git init`

### 本地模板（开发时，不走 git clone）

```bash
fsr new demo-app --template-dir /home/zhangyi/fullstack-rust-react-starter
# 或（环境变量必须与 fsr 写在同一行，单独一行无效）：
FSR_TEMPLATE_REPO=/home/zhangyi/fullstack-rust-react-starter fsr new demo-app
```

编译后请用 `target/debug/fsr`；若环境里设置了 `CARGO_TARGET_DIR`，需指向本仓库 `target` 或直接用该路径下的二进制。

```bash
fsr new my-app --path /tmp/my-app   # 指定路径
fsr new my-app --no-git             # 不执行 git init
```

## 配置

| 环境变量 | 说明 |
|----------|------|
| `FSR_TEMPLATE_REPO` | 模板 Git URL（默认见 `src/template.rs`） |
| `FSR_TEMPLATE_BRANCH` | 分支，默认 `main` |

## 项目名规则

小写 **kebab-case**：`my-app`、`acme2024`（勿用 `--`、勿大写）。

### 替换规则（按占位符长度从长到短）

| 模板占位 | 新项目 `my-app` |
|----------|-----------------|
| `Fullstack Rust React Starter API` | `My App API` |
| `fullstack-rust-react-starter` | `my-app` |
| `fullstack_rust_react_starter` | `my_app` |
| `Fullstack Rust React Starter` | `My App` |

覆盖：根 `package.json`、`@…/api-client`、`.env.example`、`docker-compose.example.yml`、OpenAPI 标题、前端页面标题等文本文件。

**不替换**：`web` / `admin` / `backend` 等工作区包名与 `just` 脚本。

## 相关仓库

| 仓库 | 说明 |
|------|------|
| [mozhiyun/fsr](https://github.com/mozhiyun/fsr) | 本 CLI（本仓库） |
| [mozhiyun/fullstack-rust-react-starter](https://github.com/mozhiyun/fullstack-rust-react-starter) | 应用模板（无 CLI 源码） |
