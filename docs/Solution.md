# 2Password - 基于 Passkey 的密码管理器技术方案

## 项目概述

2Password 是一个使用 Rust 开发的个人密码管理工具，旨在替代不安全的 iCloud 明文密码存储。通过结合简单易记的主密码、Passkey 生物识别技术和 iCloud 云备份，提供军用级安全性和消费级易用性。

## 核心设计理念

### 安全性原则
- **零知识架构**：所有加密操作在本地进行，云端无法访问明文数据
- **多层防护**：AES-256-GCM + Argon2 + Passkey 三重安全保障
- **容错设计**：任意单点故障不影响密码访问

### 易用性原则  
- **简单记忆**：只需记住类似 "123456" 的简单主密码
- **无缝体验**：Touch ID/Face ID 一触即开
- **跨设备同步**：基于 Apple/Google 生态自动同步

## 技术架构

### 1. 加密系统设计

#### 密钥派生方案
```
主密钥 = Argon2id(
    简单主密码 + 
    Passkey 认证令牌 + 
    用户 iCloud ID 哈希值 +
    随机盐值
)
```

#### 加密规范
- **对称加密**：AES-256-GCM（提供认证和完整性保护）
- **密钥派生**：Argon2id（内存 64MB，迭代 3 次，并行度 4）
- **随机数生成**：基于硬件的加密安全随机数生成器

#### 数据存储格式
```
密码库文件结构：
├── 文件头部 (版本、算法标识符)
├── 盐值 (32 字节随机)
├── Argon2 配置参数
├── 恢复分片元数据
└── 加密数据块
    ├── 初始向量 IV (12 字节)
    ├── 加密的密码条目数据
    └── GCM 认证标签 (16 字节)
```

### 2. Passkey 集成架构

#### WebAuthn 实现
- **标准遵循**：WebAuthn Level 3 + CTAP2 协议
- **生物识别**：Touch ID / Face ID / 指纹识别
- **跨平台支持**：Apple iCloud Keychain + Google Password Manager

#### Passkey 工作流程
1. **注册阶段**：生成 Passkey 并存储到系统钥匙串
2. **认证阶段**：生物识别验证 + 密钥派生
3. **同步阶段**：通过云服务商自动跨设备同步

### 3. 多因子恢复系统

#### Shamir 秘密分享方案 (2-of-3)
```
恢复因子：
Factor 1: 简单主密码派生分片
Factor 2: Passkey 生物识别分片  
Factor 3: iCloud 备份文件分片

任意 2 个因子 → 完整恢复主密钥
```

#### 灾难恢复场景
- **设备损坏**：简单密码 + iCloud 备份 → 新设备恢复
- **忘记密码**：Passkey + iCloud 备份 → 重设新密码
- **iCloud 问题**：简单密码 + Passkey → 本地恢复

## 安全性分析

### 攻击面评估

| 攻击类型 | 防护措施 | 安全等级 |
|---------|---------|----------|
| 暴力破解 | Argon2 + 设备绑定 | 🔒🔒🔒🔒🔒 |
| 社会工程 | 生物识别 + 多因子 | 🔒🔒🔒🔒 |
| 恶意软件 | 本地加密 + 权限隔离 | 🔒🔒🔒🔒 |
| 物理访问 | 生物识别锁定 | 🔒🔒🔒🔒 |
| 云端泄露 | 零知识架构 | 🔒🔒🔒🔒🔒 |

### 合规性考虑
- 符合 NIST 密码学标准
- 遵循 GDPR 数据保护原则
- 满足企业级安全要求

## 技术栈选型

### 核心依赖库
```toml
[dependencies]
# 密码学核心
argon2 = "0.5"              # 密钥派生函数
aes-gcm = "0.10"            # AES-256-GCM 加密
ring = "0.17"               # 高性能加密原语
zeroize = "1.5"             # 内存安全清零

# Passkey 支持
webauthn-rs = "0.5"         # WebAuthn 协议实现
passkey = "0.2"             # 1Password 开源 Passkey 库
localauthentication-rs = "0.1" # macOS Touch ID 集成

# Apple 平台集成
security-framework = "2.9"   # macOS Keychain Services
core-foundation = "0.9"      # macOS 系统框架

# 数据处理
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"          # JSON 序列化
bincode = "1.3"             # 二进制序列化

# CLI 和用户界面
clap = { version = "4.0", features = ["derive"] }
rpassword = "7.0"           # 安全密码输入
console = "0.15"            # 终端 UI 控制

# 系统集成
dirs = "5.0"                # 系统目录获取
chrono = "0.4"              # 时间处理
anyhow = "1.0"              # 错误处理
```

### 项目结构
```
2password/
├── Cargo.toml
├── src/
│   ├── main.rs                 # CLI 入口点
│   ├── lib.rs                  # 库入口点
│   ├── crypto/                 # 加密模块
│   │   ├── mod.rs
│   │   ├── kdf.rs              # Argon2 密钥派生
│   │   ├── cipher.rs           # AES-GCM 加密实现
│   │   ├── random.rs           # 安全随机数生成
│   │   └── recovery.rs         # 多因子恢复系统
│   ├── passkey/                # Passkey 集成
│   │   ├── mod.rs
│   │   ├── webauthn.rs         # WebAuthn 协议处理
│   │   ├── platform.rs         # 平台特定实现
│   │   └── sync.rs             # 云端同步逻辑
│   ├── storage/                # 存储管理
│   │   ├── mod.rs
│   │   ├── vault.rs            # 密码库管理
│   │   ├── entry.rs            # 密码条目定义
│   │   └── backup.rs           # 备份和同步
│   ├── cli/                    # 命令行界面
│   │   ├── mod.rs
│   │   ├── commands.rs         # CLI 命令实现
│   │   └── ui.rs               # 用户交互
│   └── web/                    # Web 界面（Phase 2）
│       ├── mod.rs
│       ├── server.rs           # 本地 HTTP 服务器
│       └── static/             # 静态资源
├── tests/                      # 单元测试和集成测试
├── docs/                       # 项目文档
└── examples/                   # 示例代码
```

## 部署和分发

### macOS 集成
- **安装方式**：Homebrew 或直接二进制分发
- **系统权限**：Keychain Access、Touch ID 使用权限
- **自动启动**：可选的系统服务模式

### 跨平台考虑
- **主要平台**：macOS（优先支持）
- **扩展平台**：iOS（Phase 3）、Linux/Windows（可选）

## 性能指标

### 预期性能
- **启动时间**：< 100ms (冷启动)
- **解锁时间**：< 200ms (Passkey 验证)
- **搜索响应**：< 50ms (1000+ 条目)
- **内存占用**：< 10MB (运行时)
- **存储空间**：< 1MB (1000 条密码)

### 安全性能
- **Argon2 计算时间**：~500ms (安全与体验平衡)
- **AES 加密性能**：> 100MB/s (硬件加速)
- **密钥派生强度**：2^20 计算复杂度

## 风险评估和缓解

### 主要风险
1. **Apple 生态依赖**：缓解方案是支持 Google Password Manager
2. **Passkey 标准变更**：采用稳定的 WebAuthn 标准
3. **硬件兼容性**：提供软件回退方案

### 应急预案
- **完全离线模式**：支持纯本地使用
- **数据导出**：支持标准格式导出
- **开源透明性**：代码完全开源，用户可审计

## 开发里程碑

### Phase 1 (MVP) - 预计 4-6 周
- 核心加密功能实现
- 基础 Passkey 集成
- CLI 基础命令
- 本地存储和恢复

### Phase 2 - 预计 4-6 周  
- Web UI 界面
- iCloud 同步集成
- 高级搜索和管理
- 导入导出功能

### Phase 3 - 预计 6-8 周
- iOS 移动端支持
- 浏览器扩展集成
- 高级安全功能
- 企业特性支持

## 结论

2Password 通过创新的 Passkey + 简单密码组合，实现了前所未有的安全性与易用性平衡。基于 Rust 的高性能实现和 Apple 生态的深度集成，为个人用户提供了银行级安全保护的同时，保持了消费级产品的简洁体验。

多因子恢复系统确保了极高的容错性，而零知识架构则提供了隐私保护的最高标准。该方案代表了 2025 年密码管理技术的最先进实践。