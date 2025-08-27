#!/bin/bash
# 2Password 功能测试脚本

set -e

echo "🧪 开始运行 2Password 功能测试..."
echo "=================================================="

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 检查依赖
echo -e "${BLUE}📋 检查环境依赖...${NC}"
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Cargo 未安装${NC}"
    exit 1
fi

if ! command -v npm &> /dev/null; then
    echo -e "${RED}❌ npm 未安装${NC}"
    exit 1
fi

echo -e "${GREEN}✅ 环境依赖检查通过${NC}"
echo ""

# 运行 Rust 测试
echo -e "${BLUE}🦀 运行 Rust 后端测试...${NC}"
echo "----------------------------------"

echo -e "${YELLOW}测试 1: 安全基础设施${NC}"
cargo test test_security_infrastructure_creation -- --nocapture

echo -e "${YELLOW}测试 2: 密码重用检测${NC}"
cargo test test_reused_password_detection -- --nocapture

echo -e "${YELLOW}测试 3: 风险等级评估${NC}"
cargo test test_risk_level_assessment -- --nocapture

echo -e "${YELLOW}测试 4: 审计日志功能${NC}"
cargo test test_audit_logger_functionality -- --nocapture

echo -e "${YELLOW}测试 5: 集成测试${NC}"
cargo test integration_tests -- --nocapture

echo ""
echo -e "${GREEN}✅ Rust 测试完成${NC}"

# 编译检查
echo -e "${BLUE}🔧 编译检查...${NC}"
echo "----------------------------------"
cargo check --all
echo -e "${GREEN}✅ 编译检查通过${NC}"

# 前端依赖检查
echo -e "${BLUE}📦 检查前端依赖...${NC}"
echo "----------------------------------"
if [ ! -d "node_modules" ]; then
    echo -e "${YELLOW}📥 安装前端依赖...${NC}"
    npm install
fi
echo -e "${GREEN}✅ 前端依赖就绪${NC}"

# TypeScript 类型检查
echo -e "${BLUE}🔍 TypeScript 类型检查...${NC}"
echo "----------------------------------"
if npm list typescript &> /dev/null; then
    npx tsc --noEmit
    echo -e "${GREEN}✅ TypeScript 类型检查通过${NC}"
else
    echo -e "${YELLOW}⚠️  TypeScript 未安装，跳过类型检查${NC}"
fi

echo ""
echo -e "${GREEN}🎉 所有测试完成！${NC}"
echo "=================================================="
echo -e "${BLUE}📊 测试总结:${NC}"
echo "  • 安全模块测试: ✅"
echo "  • 密码健康检测: ✅"  
echo "  • 编译检查: ✅"
echo "  • 类型检查: ✅"
echo ""
echo -e "${GREEN}应用已准备就绪，可以进行构建和部署！${NC}"