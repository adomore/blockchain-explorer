# Blockchain Explorer

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/github/actions/workflow/status/YOUR_USERNAME/blockchain-explorer/ci.yml?branch=main)](https://github.com/YOUR_USERNAME/blockchain-explorer/actions)
[![Crates.io](https://img.shields.io/crates/v/blockchain_explorer.svg)](https://crates.io/crates/blockchain_explorer)
[![docs.rs](https://img.shields.io/docsrs/blockchain_explorer)](https://docs.rs/blockchain_explorer)

> 一个用 Rust 编写的命令行工具，用于查询区块链（ETH、BTC）地址数据。

**功能特性：**
- 🔗 查询 ETH/BTC 地址的余额和交易明细
- 📦 支持批量查询地址列表
- 📊 支持导出 CSV 格式
- ⚡ 异步并行查询，高效处理
- 🔒 开源 MIT 许可证

## 安装

### 前置要求

- **Rust 工具链** (stable) - 支持 Windows 11, macOS, Linux
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

# 导出到文件
blockchain-explorer query 0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21 -o result.json
```

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
Balance:             1.50000000
Total Transactions:  10

--------------------------------------------------------------------------------
Recent Transactions:
--------------------------------------------------------------------------------
Transaction #1
  Hash:        0xabc123...
  From:        0xfrom...
  To:          0xto...
  Value:       0.50000000
  Timestamp:   1234567890
  Status:      Success
================================================================================
```

### CSV 输出

```csv
Address,Blockchain,Balance,Total_Transactions,First_Tx_Hash,...
0x742d35Cc6634C0532925a3b844Bc9e7595f8fE21,Ethereum,1.5,10,0xabc123,...
```

## 项目结构

```
blockchain-explorer/
├── Cargo.toml
├── README.md
├── src/
│   ├── main.rs          # CLI 入口
│   ├── lib.rs           # 库文件
│   ├── cli.rs           # CLI 参数解析
│   ├── models.rs        # 数据模型
│   ├── blockchain.rs    # 区块链 trait
│   ├── ethereum.rs      # Ethereum 提供者
│   ├── bitcoin.rs       # Bitcoin 提供者
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
```

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

- **Etherscan API V2**：自 2025 年 8 月 15 日起，原 V1 API 已弃用。本工具已更新为使用 V2 接口。
- **API Key**：强烈建议申请 Etherscan API Key 以避免速率限制
- **Bitcoin**：使用公开的 Blockstream API，无需 API Key

## License

MIT License
