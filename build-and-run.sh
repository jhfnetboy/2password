#!/bin/bash
# 2Password 构建和运行脚本

set -e

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

echo -e "${PURPLE}🚀 2Password 构建和运行脚本${NC}"
echo "=================================================="

# 检查依赖
echo -e "${BLUE}📋 检查构建依赖...${NC}"
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Cargo 未安装${NC}"
    exit 1
fi

if ! command -v npm &> /dev/null; then
    echo -e "${RED}❌ npm 未安装${NC}"
    exit 1
fi

echo -e "${GREEN}✅ 构建依赖检查通过${NC}"
echo ""

# 清理之前的构建
echo -e "${BLUE}🧹 清理之前的构建...${NC}"
if [ -d "dist" ]; then
    rm -rf dist
    echo -e "${YELLOW}  • 删除 dist 目录${NC}"
fi

if [ -d "src-tauri/target/release" ]; then
    rm -rf src-tauri/target/release
    echo -e "${YELLOW}  • 删除 Rust release 目录${NC}"
fi

echo -e "${GREEN}✅ 清理完成${NC}"
echo ""

# 安装依赖
echo -e "${BLUE}📦 检查和安装依赖...${NC}"
if [ ! -d "node_modules" ] || [ package.json -nt node_modules ]; then
    echo -e "${YELLOW}📥 安装前端依赖...${NC}"
    npm install
fi
echo -e "${GREEN}✅ 依赖安装完成${NC}"
echo ""

# 构建前端
echo -e "${BLUE}⚡ 构建前端应用...${NC}"
echo "----------------------------------"
npm run build
echo -e "${GREEN}✅ 前端构建完成${NC}"
echo ""

# 构建 Tauri 应用
echo -e "${BLUE}🦀 构建 Tauri 应用 (Release 模式)...${NC}"
echo "----------------------------------"
echo -e "${YELLOW}⏳ 这可能需要几分钟时间...${NC}"

# 使用 Tauri CLI 构建
npm run tauri build

echo -e "${GREEN}✅ Tauri 应用构建完成${NC}"
echo ""

# 查找构建的二进制文件
echo -e "${BLUE}🔍 查找构建的应用...${NC}"
echo "----------------------------------"

if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    APP_PATH="src-tauri/target/release/bundle/macos/2Password.app"
    BINARY_PATH="src-tauri/target/release/twopassword-gui"
    
    if [ -d "$APP_PATH" ]; then
        echo -e "${GREEN}📱 macOS 应用包: $APP_PATH${NC}"
        APP_SIZE=$(du -sh "$APP_PATH" | cut -f1)
        echo -e "${BLUE}   文件大小: $APP_SIZE${NC}"
    fi
    
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    # Linux
    BINARY_PATH="src-tauri/target/release/twopassword-gui" 
    DEB_PATH="src-tauri/target/release/bundle/deb/twopassword-gui_1.0.0_amd64.deb"
    APPIMAGE_PATH="src-tauri/target/release/bundle/appimage/twopassword-gui_1.0.0_amd64.AppImage"
    
    if [ -f "$DEB_PATH" ]; then
        echo -e "${GREEN}📦 Debian 包: $DEB_PATH${NC}"
        DEB_SIZE=$(du -sh "$DEB_PATH" | cut -f1)
        echo -e "${BLUE}   文件大小: $DEB_SIZE${NC}"
    fi
    
    if [ -f "$APPIMAGE_PATH" ]; then
        echo -e "${GREEN}📦 AppImage: $APPIMAGE_PATH${NC}"
        APPIMAGE_SIZE=$(du -sh "$APPIMAGE_PATH" | cut -f1)
        echo -e "${BLUE}   文件大小: $APPIMAGE_SIZE${NC}"
    fi
    
elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
    # Windows
    BINARY_PATH="src-tauri/target/release/twopassword-gui.exe"
    MSI_PATH="src-tauri/target/release/bundle/msi/2Password_1.0.0_x64.msi"
    
    if [ -f "$MSI_PATH" ]; then
        echo -e "${GREEN}📦 Windows 安装包: $MSI_PATH${NC}"
        MSI_SIZE=$(du -sh "$MSI_PATH" | cut -f1)
        echo -e "${BLUE}   文件大小: $MSI_SIZE${NC}"
    fi
fi

if [ -f "$BINARY_PATH" ]; then
    echo -e "${GREEN}⚡ 可执行文件: $BINARY_PATH${NC}"
    BINARY_SIZE=$(du -sh "$BINARY_PATH" | cut -f1)
    echo -e "${BLUE}   文件大小: $BINARY_SIZE${NC}"
else
    echo -e "${RED}❌ 未找到可执行文件${NC}"
    exit 1
fi

echo ""

# 运行应用
echo -e "${BLUE}🎯 是否要运行构建的应用? (y/n)${NC}"
read -r response

if [[ "$response" =~ ^([yY][eE][sS]|[yY])$ ]]; then
    echo -e "${PURPLE}🚀 启动 2Password...${NC}"
    echo "----------------------------------"
    
    if [[ "$OSTYPE" == "darwin"* ]] && [ -d "$APP_PATH" ]; then
        # macOS: 使用 open 命令启动 .app
        echo -e "${YELLOW}在 macOS 上启动应用包...${NC}"
        open "$APP_PATH"
    else
        # 其他平台: 直接运行二进制文件
        echo -e "${YELLOW}启动可执行文件...${NC}"
        if [ -f "$BINARY_PATH" ]; then
            chmod +x "$BINARY_PATH"
            "$BINARY_PATH" &
            APP_PID=$!
            echo -e "${GREEN}✅ 应用已启动 (PID: $APP_PID)${NC}"
        else
            echo -e "${RED}❌ 可执行文件不存在${NC}"
            exit 1
        fi
    fi
    
    echo -e "${GREEN}🎉 2Password 应用已成功启动！${NC}"
else
    echo -e "${YELLOW}⏸️  跳过运行步骤${NC}"
fi

echo ""
echo "=================================================="
echo -e "${GREEN}🎊 构建完成！${NC}"
echo -e "${BLUE}📁 构建产物位置:${NC}"

if [[ "$OSTYPE" == "darwin"* ]]; then
    echo -e "${YELLOW}  • macOS 应用包: src-tauri/target/release/bundle/macos/${NC}"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo -e "${YELLOW}  • Debian 包: src-tauri/target/release/bundle/deb/${NC}"
    echo -e "${YELLOW}  • AppImage: src-tauri/target/release/bundle/appimage/${NC}"
elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
    echo -e "${YELLOW}  • Windows 安装包: src-tauri/target/release/bundle/msi/${NC}"
fi

echo -e "${YELLOW}  • 可执行文件: src-tauri/target/release/${NC}"
echo ""
echo -e "${GREEN}现在你有了一个完整的 2Password 桌面应用！${NC}"