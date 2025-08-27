# 🚀 2Password 脚本使用指南

## 📋 脚本概览

### 1. `test.sh` - 功能测试脚本
**用途**: 运行所有功能测试，验证应用稳定性
```bash
./test.sh
```

**测试内容**:
- ✅ 安全基础设施测试
- ✅ 密码重用检测测试  
- ✅ 风险等级评估测试
- ✅ 审计日志功能测试
- ✅ 编译检查
- ✅ TypeScript 类型检查

### 2. `build-and-run.sh` - 构建和运行脚本
**用途**: 构建生产版本的桌面应用并运行
```bash
./build-and-run.sh
```

**功能**:
- 🧹 清理旧构建
- 📦 安装依赖
- ⚡ 构建前端 (React + TypeScript)
- 🦀 构建 Tauri 桌面应用
- 🚀 运行构建的应用

## 🎯 使用场景

### 开发时测试
```bash
# 在提交代码前运行测试
./test.sh

# 如果测试通过，再构建发布版本
./build-and-run.sh
```

### 发布构建
```bash
# 构建生产版本
./build-and-run.sh

# 构建产物会保存在:
# macOS: src-tauri/target/release/bundle/macos/2Password.app
# Linux: src-tauri/target/release/bundle/deb/*.deb
# Windows: src-tauri/target/release/bundle/msi/*.msi
```

## 📁 构建产物

### macOS
- **应用包**: `2Password.app` (可直接运行)
- **二进制**: `twopassword-gui`

### Linux  
- **Debian 包**: `twopassword-gui_1.0.0_amd64.deb`
- **AppImage**: `twopassword-gui_1.0.0_amd64.AppImage`
- **二进制**: `twopassword-gui`

### Windows
- **安装包**: `2Password_1.0.0_x64.msi`  
- **二进制**: `twopassword-gui.exe`

## ⚡ 快速开始

```bash
# 1. 克隆仓库并进入目录
git clone <repository-url>
cd 2password

# 2. 运行测试确保一切正常
./test.sh

# 3. 构建和运行应用
./build-and-run.sh

# 4. 享受你的密码管理器！🎉
```

## 🔧 系统要求

**必需**:
- Node.js (v16+)
- Rust (latest stable)
- npm 或 yarn

**平台特定**:
- **macOS**: Xcode Command Line Tools
- **Linux**: build-essential, webkit2gtk (Ubuntu: `sudo apt install build-essential webkit2gtk-4.0-dev`)
- **Windows**: Microsoft C++ Build Tools

## 🐛 故障排除

### 常见问题

**Q: 脚本无法执行**
```bash
chmod +x test.sh build-and-run.sh
```

**Q: 构建失败**
```bash
# 清理并重试
rm -rf node_modules src-tauri/target
npm install
./build-and-run.sh
```

**Q: 测试失败**
```bash
# 单独运行特定测试
cargo test test_security_infrastructure_creation -- --nocapture
```

## 📊 脚本特性

### `test.sh` 特性
- 🎨 彩色输出
- 📋 详细的测试报告
- 🔍 环境依赖检查
- ⚠️ 智能错误处理

### `build-and-run.sh` 特性  
- 🧹 自动清理
- 📦 智能依赖管理
- 🎯 平台自适应构建
- 🚀 一键运行
- 📏 文件大小显示

---

**💡 提示**: 这些脚本设计为开发和发布工作流的一部分，确保代码质量和构建一致性。