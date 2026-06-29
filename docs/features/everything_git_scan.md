# Everything Git 扫描 CLI

## 功能目标

通过本地 Everything CLI 查询所有 Git 仓库，并打印每个仓库的提交次数。

## 用户可见行为

运行：

```text
cargo run
```

输出：

```text
E:\code\GitWeave	2
D:\work\app	128
```

每行包含仓库路径和提交次数，中间使用 tab 分隔。

## 配置

如果 `es.exe` 不在 PATH 或默认安装目录，可以在项目根目录配置 `config.yaml`：

```text
everything:
  path: E:\code\GitWeave\docs\resource\es.exe
```

配置路径优先级高于 PATH 和默认安装目录。

## 业务规则

- Everything 查询结果中只有文件名为 `.git` 的路径会被视为 Git 仓库标记。
- `.git` 路径的父目录是仓库根目录。
- 重复仓库只输出一次，并保持 Everything 发现顺序。
- 单个仓库计数失败时跳过该仓库，不中断整个扫描。

## 不确定项

- 是否需要支持 Everything HTTP 服务。
- 是否需要支持命令行参数覆盖环境变量。
- 是否需要输出 JSON 或表格格式。
