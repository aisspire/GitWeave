# 架构概览

## 当前形态

GitWeave 是一个 Rust CLI。`src/main.rs` 只负责进程入口和退出码处理，核心逻辑位于 `src/lib.rs`。

## 模块边界

- `AppConfig`：承载运行配置，目前包含 `everything_path`。
- `parse_git_repositories`：把 Everything 输出的 `.git` 路径解析为仓库根目录，并保持发现顺序去重。
- `parse_commit_count`：解析 `git rev-list --count HEAD` 的输出。
- `run_with_config`：执行完整扫描流程。
- `run`：从环境变量构造默认配置，并调用 `run_with_config`。

## 配置

`es.exe` 路径通过项目根目录的 `config.yaml` 配置：

```text
everything:
  path: E:\code\GitWeave\docs\resource\es.exe
```

查找优先级：

1. `config.yaml` 中的 `everything.path`
2. PATH 中的 `es.exe`
3. `C:\Program Files\Everything\es.exe`
4. `C:\Program Files (x86)\Everything\es.exe`

## 外部依赖

- Everything CLI：查找 `.git` 目录。
- Git CLI：统计仓库提交次数。

## 错误处理

找不到 `es.exe` 时，程序返回非零退出码并打印已检查的候选路径。单个仓库无法统计提交次数时，程序打印 warning 并继续处理其它仓库。
