# AI 功能变更流程

## 开发前

1. 读取 `AGENTS.md` 和 `docs/index.md`。
2. 根据任务类型读取最小相关文档。
3. 明确配置、测试和文档更新边界。

## 开发中

1. 对行为变更先写失败测试。
2. 实现最小代码让测试通过。
3. 遇到测试失败时先定位根因，再修复。

## 开发后

1. 运行 `cargo fmt`。
2. 运行 `cargo test`。
3. 需要本机工具时运行 `cargo run` 做 smoke test。
4. 更新相关 `docs/` 文档和 `docs/index.md`。
