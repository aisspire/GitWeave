# Everything Git 扫描测试说明

## 自动化测试

运行：

```text
cargo test
```

覆盖点：

- Everything `.git` 输出解析为仓库根目录。
- 重复仓库去重并保持发现顺序。
- 非 `.git` 路径跳过。
- 提交次数输出解析。
- 终端输出格式。
- `config.yaml` 中的 `everything.path` 配置优先于默认候选路径。

## 手工验证

前置条件：

- 本机已安装 Everything CLI (`es.exe`)。
- 本机已安装 Git CLI。
- 如果 `es.exe` 不在 PATH 或默认安装目录，先在 `config.yaml` 设置 `everything.path`。

运行：

```text
cargo run
```

预期：

- 终端打印仓库路径和提交次数。
- 如果找不到 `es.exe`，错误信息列出已检查路径。
- 如果单个仓库无法统计提交次数，终端打印 warning 并继续处理其它仓库。
