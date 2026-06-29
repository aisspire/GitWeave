# 项目背景

## 项目目标

GitWeave 当前目标是提供一个本地 Rust CLI，用 Everything 快速发现本机 Git 仓库，并聚合仓库信息。

## 使用者与场景

使用者是在 Windows 本机维护多个 Git 仓库的开发者。基础场景是从终端运行 CLI，快速查看本机仓库及其提交次数。

## 范围

### 包含

- 使用 Rust 开发 CLI。
- 调用本地 Everything CLI (`es.exe`) 查找 `.git` 目录。
- 调用本地 `git` 获取每个仓库的提交次数。
- 支持通过 `config.yaml` 指定 `es.exe` 路径。

### 不包含

- 前端界面。
- 仓库提交详情、作者统计和协作者聚合。
- Everything HTTP 服务或 SDK 集成。

## 技术栈

- Rust 2021 edition
- Cargo
- Everything Command-line Interface (`es.exe`)
- Git CLI

## 当前阶段

基础 CLI 已实现，后续需求记录在 `docs/BACKLOG.md`。

## 核心约束

- 遵循 `AGENTS.md`：使用 Rust 开发，中文注释标注清晰，尽可能抽出配置。
- 优先通过 `config.yaml` 的 `everything.path` 配置 `es.exe` 路径。
- 如果未配置 `everything.path`，再尝试 PATH 和常见 Everything 安装目录。

## 文档维护规则

- 开发前先读 `docs/index.md`。
- 只读取本次任务相关的最小文档集合。
- 代码与文档冲突时，以代码核对结果为准，并记录 drift。
- 修改后更新相关文档和索引。
