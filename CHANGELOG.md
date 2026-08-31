---
AIGC:
    ContentProducer: Minimax Agent AI
    ContentPropagator: Minimax Agent AI
    Label: AIGC
    ProduceID: "00000000000000000000000000000000"
    PropagateID: "00000000000000000000000000000000"
    ReservedCode1: 3044022077be2ae431d114b03fcaa4a6bb9d13c7359621cc9ae6d0dd19d7a44e45151234022056cc87f847861acfe5818f47d4971d56894d60d7bd22a58ce94635c7de8c72a0
    ReservedCode2: 30440220191a32a0668d3667aa0440b12ebdee8829670c3b01586c70c45d491909ea2fa102203459121942c2ecbe98d23ab339e67156373bd1a9a57a223bf0959fbded39c653
---

# 变更日志

**简体中文** | [English](CHANGELOG.en.md)

本文件记录本项目所有值得留意的变更。

格式参考 [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)，
版本号遵循 [语义化版本](https://semver.org/spec/v2.0.0.html)。

0.3.0 的日期取自本仓库的初始提交——该提交的 `Cargo.toml` 已经声明为 0.3.0。
0.1.0 与 0.2.0 早于这里的任何记录：仓库既没有 tag 也没有更早的提交，因此宁可
不标日期，也不填一个编造出来的日期。

## [未发布]

### 新增
- `match` 子命令：扫描任意文本文件中的 BTC/ETH 地址并将结果写入文件
  （表格、CSV 或 JSON，按输出文件扩展名推断）
- 比特币校验和验证：`1.../3...` 使用 Base58Check，`bc1...` 使用
  Bech32/Bech32m（BIP-173 / BIP-350），并识别地址类型
  （P2PKH、P2SH、P2WPKH、P2WSH、P2TR）
- `match` 子命令的 `--no-checksum`、`--all-occurrences`、`--blockchain`
  与 `--query` 选项
- 报告条目记录每个地址的行号、列号与出现次数
- GitHub Actions CI：在 Linux、macOS 与 Windows 上运行测试，另有 `cargo fmt`、
  `cargo clippy -D warnings` 以及一个固定在所声明 MSRV 上的任务
- `Cargo.toml` 中的 `rust-version = "1.85"`，与锁定依赖中的最高要求一致
- `blockchain::utils` 与 `models` 的单元测试，此前两者都没有测试
- 每份文档现在都有中文原本与英文镜像，由 CI 中的
  `scripts/check_lockstep.py` 校验

### 变更
- ETH 与 BTC 余额现在渲染方式完全一致：整数除法，去掉末尾多余的零。
  1 BTC 显示为 `1`，而不是 `1.00000000`
- `--no-color` 现在真的会关闭日志输出中的 ANSI 颜色

### 移除
- `batch` 子命令的 `--continue-on-error` 与 `--show-progress`。两者从未被读取过：
  批量查询本来就会跳过失败继续执行，也不存在任何可以关闭的
  进度输出

### 修复
- 缺失的公开 re-export（`BlockchainType`、`Transaction`、`CsvRecord`），
  它导致集成测试套件无法编译
- 测试写入硬编码的 `/tmp` 路径，在 Windows 上必然失败
- `wei_to_eth` 经由 `f64` 换算，会静默地对高于
  2^53 wei 的余额四舍五入。换算现在全程留在 `u128` 中

## [0.3.0] - 2026-04-12

### 新增
- 支持 Etherscan API V2（V1 已弃用）
- 通过环境变量或命令行参数配置 API Key
- API 请求的调试日志
- 更完善的错误处理与提示信息

### 修复
- 以太坊余额显示格式（wei 到 ETH 的换算）
- 比特币余额计算（现在显示可用余额，而非累计收款）

### 变更
- 更新以太坊 API 的基础 URL
- 改进余额格式化（去掉末尾多余的零）

## [0.2.0] - 日期不详

### 新增
- 通过 Blockstream API 支持比特币地址
- CSV 导出功能
- 批量查询支持
- 交易明细展示

## [0.1.0] - 日期不详

### 新增
- 首个版本
- 通过 Etherscan API 查询以太坊地址
- 基础命令行界面
- 表格与 JSON 输出格式
