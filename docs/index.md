# 文档索引

## 项目入口

- `project.md`：项目目标、范围、技术栈和核心约束。
- `architecture/overview.md`：当前 CLI 架构、配置入口和外部命令边界。
- `flows/ai_feature_change_flow.md`：AI 协作开发流程。
- `BACKLOG.md`：后续功能想法列表。

## 模块索引

| 模块 | 功能文档 | API 文档 | 数据文档 | 测试文档 | 流程文档 | 状态 |
| --- | --- | --- | --- | --- | --- | --- |
| Everything Git 扫描 CLI | `features/everything_git_scan.md` | 无 | 无 | `tests/everything_git_scan.md` | 无 | 基础版已实现 |

## 任务路由

| 任务类型 | 优先读取 |
| --- | --- |
| 新功能 | `features/`、`tests/`、相关 `flows/` |
| CLI 行为调整 | `features/everything_git_scan.md`、`architecture/overview.md`、`tests/everything_git_scan.md` |
| 配置调整 | `architecture/overview.md`、相关功能文档 |
| 规则维护 | `ai/rule_candidates.md`、`ai/agents_patch_suggestions.md` |

## 权威来源

- 代码：`src/lib.rs`、`src/main.rs`
- 配置：`config.yaml` 和 `src/lib.rs` 中的 `AppConfig`
- 测试：`cargo test`
- 计划：`docs/plans/`

## 已知 Drift

| 文档 | 问题 | 发现时间 | 处理状态 |
| --- | --- | --- | --- |
| 无 | 无 | 2026-06-30 | 无 |
