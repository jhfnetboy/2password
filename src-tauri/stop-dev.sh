#!/bin/bash
# 停止开发服务器

echo "🛑 正在停止 2Password 开发服务器..."

# 终止进程
kill 17692 17721 2>/dev/null || true

# 清理端口
lsof -ti:3000 | xargs kill -9 2>/dev/null || true
lsof -ti:1430 | xargs kill -9 2>/dev/null || true

echo "✅ 开发服务器已停止"
rm -f "stop-dev.sh"
