---
AIGC:
    ContentProducer: Minimax Agent AI
    ContentPropagator: Minimax Agent AI
    Label: AIGC
    ProduceID: "00000000000000000000000000000000"
    PropagateID: "00000000000000000000000000000000"
    ReservedCode1: 30450221008686f01d613a60436dd2bf46e1eebfc71fd9040daab3756194060213ded238bf02205ba7492ca921dbc82a0f47d54e70601074ea2490118a61c486eb532e8314a2fe
    ReservedCode2: 3045022100a47cfefb60be1cf6995ec19702e0a701a5ff48061b1e70c5b840565713f2d08502207f2ab52e00853c94205a151b6149eebefcc794bb199e2e1803d88b8cb0a5a385
---

# 为 Blockchain Explorer 做贡献

**简体中文** | [English](CONTRIBUTING.en.md)

感谢你有兴趣参与本项目！

## 如何参与

### 报告缺陷

1. 先检索已有的 [issues](https://github.com/adomore/blockchain-explorer/issues)
2. 新建一个 issue，并说明：
   - 清晰的标题与描述
   - 复现步骤
   - 期望行为与实际行为的差异
   - Rust 版本与操作系统信息

### 提议新功能

1. 新建一个 issue，打上 "enhancement" 标签
2. 描述这个功能以及它的使用场景
3. 说明它为什么有价值

### 提交 Pull Request

1. Fork 本仓库
2. 新建一个分支：`git checkout -b feature/your-feature-name`
3. 完成你的修改
4. 运行测试：`cargo test`
5. 提交修改：`git commit -m "Add feature: description"`
6. 推送分支：`git push origin feature/your-feature-name`
7. 发起 Pull Request

### 代码风格

- 遵循 Rust 的惯用写法与约定
- 提交前运行 `cargo fmt`
- 运行 `cargo clippy` 检查常见问题——CI 会把警告当作错误
- 为新增功能补充测试

### 文档

每份文档都有两个版本：中文原本与英文镜像，命名为
`README.md` 与 `README.en.md`，变更日志和本指南同理。
两侧必须保持 lockstep 一致——相同的标题、相同的顺序、相同的
代码示例、相同的表格、相同的链接、相同的数字。只有正文可以翻译；
代码示例只翻译其中的注释，别的一律不动。

请在同一个提交里同时更新两侧，并在推送前自查：

```bash
python3 scripts/check_lockstep.py
```

## 开发环境准备

```bash
# 克隆你 fork 的仓库
git clone https://github.com/adomore/blockchain-explorer.git
cd blockchain-explorer

# 安装依赖
cargo build

# 运行测试
cargo test

# 带日志运行
RUST_LOG=debug cargo run -- query <ADDRESS>
```

## 许可证

提交贡献即表示你同意你的贡献以 MIT 许可证授权。
