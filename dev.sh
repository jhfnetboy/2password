#!/bin/bash
# 2Password 开发模式启动脚本

set -e

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${PURPLE}⚡ 2Password 开发模式启动脚本${NC}"
echo "=================================================="

# 检查依赖
echo -e "${BLUE}📋 检查开发依赖...${NC}"
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Cargo 未安装${NC}"
    exit 1
fi

if ! command -v npm &> /dev/null; then
    echo -e "${RED}❌ npm 未安装${NC}"
    exit 1
fi

echo -e "${GREEN}✅ 开发依赖检查通过${NC}"
echo ""

# 检查端口占用
echo -e "${BLUE}🔍 检查端口使用情况...${NC}"
if lsof -Pi :3000 -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo -e "${YELLOW}⚠️  端口 3000 已被占用${NC}"
    echo -e "${CYAN}正在终止占用端口 3000 的进程...${NC}"
    lsof -ti:3000 | xargs kill -9 2>/dev/null || true
    sleep 2
fi

if lsof -Pi :1430 -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo -e "${YELLOW}⚠️  Tauri 默认端口 1430 已被占用${NC}"
    echo -e "${CYAN}正在终止占用端口 1430 的进程...${NC}"
    lsof -ti:1430 | xargs kill -9 2>/dev/null || true
    sleep 2
fi

echo -e "${GREEN}✅ 端口清理完成${NC}"
echo ""

# 检查和安装依赖
echo -e "${BLUE}📦 检查项目依赖...${NC}"
if [ ! -d "node_modules" ] || [ package.json -nt node_modules ]; then
    echo -e "${YELLOW}📥 安装前端依赖...${NC}"
    npm install
fi
echo -e "${GREEN}✅ 前端依赖就绪${NC}"
echo ""

# 创建日志目录
LOG_DIR="logs"
if [ ! -d "$LOG_DIR" ]; then
    mkdir -p "$LOG_DIR"
fi

# 定义日志文件
FRONTEND_LOG="$LOG_DIR/frontend-dev.log"
TAURI_LOG="$LOG_DIR/tauri-dev.log"

echo -e "${BLUE}📝 日志文件位置:${NC}"
echo -e "  • 前端日志: ${CYAN}$FRONTEND_LOG${NC}"
echo -e "  • Tauri日志: ${CYAN}$TAURI_LOG${NC}"
echo ""

# 清理旧日志
> "$FRONTEND_LOG"
> "$TAURI_LOG"

# 启动前端开发服务器
echo -e "${BLUE}🚀 启动前端开发服务器 (端口 3000)...${NC}"
echo "=================================================="

npm run dev > "$FRONTEND_LOG" 2>&1 &
FRONTEND_PID=$!

echo -e "${GREEN}✅ 前端服务器已启动 (PID: $FRONTEND_PID)${NC}"
echo -e "${CYAN}   访问地址: http://localhost:3000${NC}"

# 等待前端服务器启动
echo -e "${YELLOW}⏳ 等待前端服务器完全启动...${NC}"
for i in {1..30}; do
    if curl -s http://localhost:3000 >/dev/null 2>&1; then
        echo -e "${GREEN}✅ 前端服务器已就绪！${NC}"
        break
    fi
    sleep 1
    if [ $i -eq 30 ]; then
        echo -e "${RED}❌ 前端服务器启动超时${NC}"
        kill $FRONTEND_PID 2>/dev/null || true
        exit 1
    fi
done

echo ""

# 启动 Tauri 开发模式（Debug）
echo -e "${BLUE}🦀 启动 Tauri 调试模式...${NC}"
echo "=================================================="

# 设置 Rust 调试环境变量
export RUST_LOG=debug
export RUST_BACKTRACE=1

npm run tauri dev > "$TAURI_LOG" 2>&1 &
TAURI_PID=$!

echo -e "${GREEN}✅ Tauri 调试模式已启动 (PID: $TAURI_PID)${NC}"
echo -e "${CYAN}   调试信息: 查看 $TAURI_LOG${NC}"

echo ""
echo -e "${GREEN}🎉 开发环境已完全启动！${NC}"
echo "=================================================="
echo -e "${BLUE}📊 运行状态:${NC}"
echo -e "  • ${GREEN}前端服务器${NC}: http://localhost:3000 (PID: $FRONTEND_PID)"
echo -e "  • ${GREEN}Tauri 应用${NC}: Debug 模式 (PID: $TAURI_PID)"
echo -e "  • ${GREEN}热重载${NC}: 前端和后端都支持"
echo -e "  • ${GREEN}调试模式${NC}: RUST_LOG=debug, RUST_BACKTRACE=1"
echo ""
echo -e "${YELLOW}💡 开发提示:${NC}"
echo -e "  • ${CYAN}实时查看前端日志${NC}: tail -f $FRONTEND_LOG"
echo -e "  • ${CYAN}实时查看Tauri日志${NC}: tail -f $TAURI_LOG"
echo -e "  • ${CYAN}停止开发服务器${NC}: Ctrl+C 或运行 kill $FRONTEND_PID $TAURI_PID"
echo ""

# 创建停止脚本
STOP_SCRIPT="stop-dev.sh"
cat > "$STOP_SCRIPT" << EOF
#!/bin/bash
# 停止开发服务器

echo "🛑 正在停止 2Password 开发服务器..."

# 终止进程
kill $FRONTEND_PID $TAURI_PID 2>/dev/null || true

# 清理端口
lsof -ti:3000 | xargs kill -9 2>/dev/null || true
lsof -ti:1430 | xargs kill -9 2>/dev/null || true

echo "✅ 开发服务器已停止"
rm -f "$STOP_SCRIPT"
EOF

chmod +x "$STOP_SCRIPT"

echo -e "${PURPLE}🛑 快速停止命令: ${CYAN}./$STOP_SCRIPT${NC}"
echo ""

# 实时监控
echo -e "${BLUE}📡 实时监控模式 (Ctrl+C 退出)${NC}"
echo "=================================================="

# 监控函数
monitor_services() {
    while true; do
        sleep 5
        
        # 检查前端服务器
        if ! kill -0 $FRONTEND_PID 2>/dev/null; then
            echo -e "${RED}❌ 前端服务器已停止${NC}"
            break
        fi
        
        # 检查 Tauri
        if ! kill -0 $TAURI_PID 2>/dev/null; then
            echo -e "${RED}❌ Tauri 应用已停止${NC}"
            break
        fi
        
        # 显示状态
        echo -e "${GREEN}✅ $(date '+%H:%M:%S') - 所有服务运行正常${NC}"
    done
}

# 设置陷阱以清理进程
cleanup() {
    echo ""
    echo -e "${YELLOW}🛑 正在停止开发服务器...${NC}"
    
    kill $FRONTEND_PID $TAURI_PID 2>/dev/null || true
    
    # 等待进程结束
    sleep 2
    
    # 强制清理端口
    lsof -ti:3000 | xargs kill -9 2>/dev/null || true
    lsof -ti:1430 | xargs kill -9 2>/dev/null || true
    
    echo -e "${GREEN}✅ 开发环境已清理完毕${NC}"
    rm -f "$STOP_SCRIPT"
    exit 0
}

# 注册信号处理
trap cleanup SIGINT SIGTERM

# 开始监控
monitor_services