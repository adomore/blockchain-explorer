# Blockchain Explorer

**简体中文** | [English](README.en.md)

[![CI](https://github.com/adomore/blockchain-explorer/actions/workflows/ci.yml/badge.svg)](https://github.com/adomore/blockchain-explorer/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/Rust-1.86+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

> 一个用 Rust 编写的命令行工具，用于查询区块链（ETH、BTC）地址数据。

**功能特性：**
- 🔗 查询 ETH/BTC 地址的余额和交易明细
- 📦 支持批量查询地址列表
- 🔍 从任意文本文件中匹配 BTC/ETH 地址（Base58Check / Bech32 校验和验证）
- 📊 支持导出 CSV 格式
- ⚡ 异步并行查询，高效处理
- 🔒 开源 MIT 许可证

## 安装

### 下载预编译二进制

只想用的话不必安装 Rust 工具链。[Releases 页面](https://github.com/adomore/blockchain-explorer/releases/latest)为每个平台提供了预编译压缩包：

| 平台 | 文件 |
|------|------|
| Linux x86_64（glibc） | `blockchain-explorer-<版本>-x86_64-unknown-linux-gnu.tar.gz` |
| Linux x86_64（静态链接，musl） | `blockchain-explorer-<版本>-x86_64-unknown-linux-musl.tar.gz` |
| Linux aarch64 | `blockchain-explorer-<版本>-aarch64-unknown-linux-gnu.tar.gz` |
| macOS Apple Silicon | `blockchain-explorer-<版本>-aarch64-apple-darwin.tar.gz` |
| macOS Intel | `blockchain-explorer-<版本>-x86_64-apple-darwin.tar.gz` |
| Windows x86_64 | `blockchain-explorer-<版本>-x86_64-pc-windows-msvc.zip` |
| Windows ARM64 | `blockchain-explorer-<版本>-aarch64-pc-windows-msvc.zip` |

每个压缩包内含可执行文件、两份 README、两份变更日志与许可证。musl 版本是静态链接的，因此可以在比构建机更老的发行版上运行。

校验并解压：

```bash
# 校验完整性（SHA256SUMS 与压缩包一同放在 Release 页面）
sha256sum -c SHA256SUMS --ignore-missing

tar -xzf blockchain-explorer-*-x86_64-unknown-linux-gnu.tar.gz
./blockchain-explorer-*/blockchain-explorer --help
```

### 前置要求

- **Rust 工具链** 1.86 或更高（stable）- 支持 Windows 11, macOS, Linux
- **Git** - 用于克隆代码仓库

### 从源码编译

```bash
# 克隆项目
git clone https://github.com/adomore/blockchain-explorer.git
cd blockchain-explorer

# 编译（Debug 模式）
cargo build

# 编译（Release 模式，推荐用于生产环境）
cargo build --release

# 运行
./target/release/blockchain-explorer --help
```

---

## 各操作系统编译步骤

### Windows 11

#### 方式 1：使用 PowerShell

```powershell
# 1. 安装 Rust（如果尚未安装）
# 下载并运行 rustup-init.exe：https://rustup.rs

# 2. 打开 PowerShell 并克隆项目
git clone https://github.com/adomore/blockchain-explorer.git
cd blockchain-explorer

# 3. 编译项目
cargo build --release

# 4. 运行程序
.\target\release\blockchain-explorer.exe --help

# 5. 设置 Etherscan API Key（可选）
$env:ETHERSCAN_API_KEY = "YOUR_API_KEY"
.\target\release\blockchain-explorer.exe query 0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21
```

#### 方式 2：使用命令提示符 (CMD)

```cmd
:: 克隆项目
git clone https://github.com/adomore/blockchain-explorer.git
cd blockchain-explorer

:: 编译项目
cargo build --release

:: 运行程序
target\release\blockchain-explorer.exe --help
```

---

### macOS 15.7.5

#### 使用终端

```bash
# 1. 安装 Homebrew（如果尚未安装）
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# 2. 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 3. 激活 Rust 环境
source ~/.cargo/env

# 4. 克隆项目
git clone https://github.com/adomore/blockchain-explorer.git
cd blockchain-explorer

# 5. 编译项目
cargo build --release

# 6. 运行程序
./target/release/blockchain-explorer --help

# 7. 设置 Etherscan API Key（可选）
export ETHERSCAN_API_KEY="YOUR_API_KEY"
./target/release/blockchain-explorer query 0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21
```

---

### Ubuntu 24.04 LTS

#### 使用终端

```bash
# 1. 安装依赖
sudo apt update
sudo apt install -y build-essential curl git

# 2. 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 3. 激活 Rust 环境
source ~/.cargo/env

# 4. 克隆项目
git clone https://github.com/adomore/blockchain-explorer.git
cd blockchain-explorer

# 5. 编译项目
cargo build --release

# 6. 运行程序
./target/release/blockchain-explorer --help

# 7. 设置 Etherscan API Key（可选）
export ETHERSCAN_API_KEY="YOUR_API_KEY"
./target/release/blockchain-explorer query 0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21
```

---

## 获取 API Key

### Etherscan API Key（用于 Ethereum）

1. 访问 https://etherscan.io/apidashboard
2. 注册/登录账户
3. 创建一个新的 API Key（免费）

### 使用 API Key

```bash
# 方式 1：环境变量
export ETHERSCAN_API_KEY="YOUR_API_KEY"

# 方式 2：命令行参数
blockchain-explorer query <ADDRESS> --etherscan-api-key YOUR_API_KEY
```

> **注意**：如果不提供 API Key，程序将使用默认的占位符密钥，可能受到速率限制。

## 使用方法

### 全局选项

以下选项对所有子命令都有效：

| 选项 | 说明 | 默认值 |
|------|------|--------|
| `-v, --verbose` | 输出 DEBUG 级别日志（API 请求 URL 中的 Key 会被脱敏） | 关闭 |
| `--no-color` | 关闭日志输出的 ANSI 颜色，便于重定向到文件 | 关闭 |
| `--etherscan-api-key <KEY>` | Etherscan API Key，等价于 `ETHERSCAN_API_KEY` 环境变量 | 无 |

### 查询单个地址

```bash
# 自动检测区块链类型
blockchain-explorer query 0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21

# 指定以太坊
blockchain-explorer query 0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21 --blockchain ethereum

# 指定比特币
blockchain-explorer query 1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2 --blockchain bitcoin

# 输出交易明细
blockchain-explorer query 0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21 --include-txs

# 输出为 JSON 格式
blockchain-explorer query 0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21 --format json

# 输出为 CSV 单行（便于管道处理）
blockchain-explorer query 0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21 --format csv

# 导出到文件（默认 JSON）
blockchain-explorer query 0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21 -o result.json

# 导出到文件并指定格式（json / csv）
blockchain-explorer query 0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21 -o result.csv --export-format csv
```

`--format` 控制打印到终端的内容，`--export-format` 控制写入 `-o` 文件的内容，两者相互独立。

### 批量查询

```bash
# 创建地址列表文件（每行一个地址）
cat > addresses.txt << EOF
0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21
0xabcdefabcdefabcdefabcdefabcdefabcdefabcd
1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2
bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq
EOF

# 执行批量查询
blockchain-explorer batch addresses.txt -o results.csv

# 指定并行数
blockchain-explorer batch addresses.txt -o results.csv --parallel 10
```

### 从文本文件中匹配地址

与 `batch`（每行一个地址）不同，`match` 子命令会扫描**任意文本文件**——日志、
聊天记录、笔记、JSON、HTML 都可以——把其中的 BTC/ETH 地址找出来，并将结果写入文件。

```bash
# 扫描文本文件，结果写入默认输出文件 matches.txt
blockchain-explorer match notes.txt

# 指定输出文件；输出格式按扩展名自动推断（.csv / .json，其余为表格）
blockchain-explorer match notes.txt -o result.csv
blockchain-explorer match notes.txt -o result.json

# 显式指定输出格式（table / csv / json）
blockchain-explorer match notes.txt -o result.out --format json

# 只匹配某一条链
blockchain-explorer match notes.txt -o btc.csv --blockchain bitcoin

# 只按格式匹配，不验证比特币校验和（示例地址、伪造地址也会命中）
blockchain-explorer match notes.txt -o all.csv --no-checksum

# 列出每一次出现，而不是按地址去重
blockchain-explorer match notes.txt -o all.csv --all-occurrences

# 匹配之后顺带查询余额与交易数（需要联网）
blockchain-explorer match notes.txt -o result.csv --query --parallel 10
```

**匹配规则：**

| 地址类型 | 格式 | 校验方式 |
|----------|------|----------|
| Ethereum | `0x` + 40 位十六进制字符 | 仅格式校验（不验证 EIP-55 大小写校验和） |
| Bitcoin P2PKH / P2SH | `1...` / `3...`，26-35 位 Base58 字符 | Base58Check 校验和 |
| Bitcoin SegWit | `bc1...`（P2WPKH / P2WSH / P2TR） | Bech32 / Bech32m 校验和（BIP-173 / BIP-350） |

- 地址两侧必须是非字母数字字符；粘在其它文字里的地址不会被匹配（如 `abc0x742d...`）。
- 交易哈希（`0x` + 64 位十六进制）等不会被误判成地址。
- 默认按地址去重，并记录首次出现的行号、列号与出现次数。
- 文件以 UTF-8 宽松方式解码，非法 UTF-8 的日志文件同样可以扫描。
- 使用 `--no-checksum` 时只做格式匹配，误报会明显增多。

**输出示例（默认表格格式）：**

```
=========================================================================================
BTC / ETH Address Match Report
=========================================================================================
Source File:           notes.txt
Scanned Lines:         9
Matched Occurrences:   6
Unique Addresses:      5
  Ethereum:            1
  Bitcoin:             4
Checksum Verified:     yes
Balance Queried:       no
-----------------------------------------------------------------------------------------
#    Chain    Type    Checksum    Line   Col  Hits Address
-----------------------------------------------------------------------------------------
1    Ethereum ETH     N/A            2    14     2 0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21
2    Bitcoin  P2PKH   Verified       3    12     1 1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa
3    Bitcoin  P2SH    Verified       3    58     1 3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy
4    Bitcoin  P2WPKH  Verified       4    36     1 bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq
```

CSV 输出包含 `Index,Address,Blockchain,Address_Type,Checksum,Line,Column,Occurrences,Balance,Total_Transactions,Query_Error,Context`；
JSON 输出为完整的匹配报告，可直接被程序读取。

### 地址类型检测

```bash
# 检测地址类型
blockchain-explorer detect 0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21
blockchain-explorer detect 1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2
```

### 比较多个地址

```bash
# 比较多个地址
blockchain-explorer compare \
    0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21 \
    0xabcdefabcdefabcdefabcdefabcdefabcdefabcd \
    -o comparison.csv
```

### 导出已有结果

```bash
# 将 JSON 格式导出为 CSV
blockchain-explorer export data.json -o output.csv
```

## 输出格式示例

### 表格输出

```
================================================================================
Address Information
================================================================================
Address:             0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21
Blockchain:          Ethereum
Balance:             1.5
Total Transactions:  10

--------------------------------------------------------------------------------
Recent Transactions:
--------------------------------------------------------------------------------
Transaction #1
  Hash:        0xabc123...
  From:        0xfrom...
  To:          0xto...
  Value:       0.5
  Timestamp:   1234567890
  Status:      Success
================================================================================
```

### CSV 输出

```csv
Address,Blockchain,Balance,Total_Transactions,First_Tx_Hash,...
0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21,Ethereum,1.5,10,0xabc123,...
```

### 余额格式

ETH 与 BTC 的余额使用同一套规则渲染：按整数除法换算（ETH 为 10^18 wei，
BTC 为 10^8 satoshi），去掉小数末尾多余的零。1 ETH 显示为 `1`，0.5 BTC
显示为 `0.5`，1 satoshi 显示为 `0.00000001`。换算全程使用 `u128`，不经过
浮点数，因此大额余额不会丢失精度。

> `Balance_USD` 列目前恒为 `N/A`——本工具没有接入任何汇率数据源。

## 项目结构

```
blockchain-explorer/
├── Cargo.toml            # 包定义（含 rust-version = 1.86）
├── README.md             # 中文原本
├── README.en.md          # 英文镜像
├── CHANGELOG.md          # 版本变更记录（英文镜像见 .en.md）
├── CONTRIBUTING.md       # 贡献指南（英文镜像见 .en.md）
├── LICENSE               # MIT
├── .github/
│   └── workflows/
│       ├── ci.yml        # 三平台测试 + fmt/clippy + MSRV + lockstep 校验
│       └── release.yml   # 打 tag 时构建各平台二进制包
├── scripts/
│   └── check_lockstep.py # EN/ZH 文档结构一致性闸门
├── src/
│   ├── main.rs          # CLI 入口
│   ├── lib.rs           # 库文件
│   ├── cli.rs           # CLI 参数解析
│   ├── models.rs        # 数据模型
│   ├── blockchain.rs    # 区块链 trait
│   ├── ethereum.rs      # Ethereum 提供者
│   ├── bitcoin.rs       # Bitcoin 提供者
│   ├── address_match.rs # 文本地址匹配与报告输出
│   ├── checksum.rs      # Base58Check / Bech32 校验和
│   └── csv_export.rs    # CSV 导出
└── tests/
    └── integration_tests.rs  # 集成测试
```

## 测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_ethereum
cargo test test_bitcoin

# 运行带日志的测试
RUST_LOG=debug cargo test

# CI 上执行的完整检查
cargo fmt --all --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

测试全部使用 Mock 提供者，不访问网络，因此离线也能运行。

## 技术栈

- **Rust** - 系统编程语言
- **Tokio** - 异步运行时
- **Reqwest** - HTTP 客户端
- **Clap** - CLI 参数解析
- **Serde** - 序列化/反序列化
- **CSV** - CSV 文件处理
- **Tracing** - 日志记录

## API 说明

本工具使用以下公开 API：

- **Ethereum**: Etherscan API V2 (https://api.etherscan.io/v2/api)
- **Bitcoin**: Blockstream API (https://blockstream.info/api)

### 重要说明

- **Etherscan API V2**：原 V1 API 已于 2025-08-15 弃用。本工具已更新为使用 V2 接口。
- **API Key**：强烈建议申请 Etherscan API Key 以避免速率限制
- **Bitcoin**：使用公开的 Blockstream API，无需 API Key

## License

MIT License
