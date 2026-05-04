#!/bin/bash
# Smainer Desktop Application Wrapper
# A simple desktop launcher for the Smainer provider node

INSTALL_DIR="/opt/smainer-desktop"
PROVIDER_BINARY="$INSTALL_DIR/smainer-provider"
WEB_DIR="$INSTALL_DIR/web-dist"
PORT=8080

# Function to start the provider daemon
start_provider() {
    echo "🚀 Starting Smainer Provider..."
    if [[ -x "$PROVIDER_BINARY" ]]; then
        "$PROVIDER_BINARY" &
        PROVIDER_PID=$!
        echo "Provider started with PID: $PROVIDER_PID"
    else
        echo "❌ Provider binary not found at: $PROVIDER_BINARY"
        exit 1
    fi
}

# Function to start simple web server
start_web_server() {
    echo "🌐 Starting Web Interface..."
    if [[ -d "$WEB_DIR" ]]; then
        cd "$WEB_DIR"
        python3 -m http.server $PORT &
        WEB_PID=$!
        echo "Web interface started on http://localhost:$PORT"
    else
        echo "❌ Web assets not found at: $WEB_DIR"
        exit 1
    fi
}

# Function to open browser
open_browser() {
    sleep 2
    if command -v xdg-open &> /dev/null; then
        xdg-open "http://localhost:$PORT" &
    elif command -v gnome-open &> /dev/null; then
        gnome-open "http://localhost:$PORT" &
    elif command -v firefox &> /dev/null; then
        firefox "http://localhost:$PORT" &
    else
        echo "ℹ️  Open http://localhost:$PORT in your browser"
    fi
}

# Cleanup function
cleanup() {
    echo "🛑 Shutting down..."
    if [[ -n "$WEB_PID" ]]; then
        kill $WEB_PID 2>/dev/null || true
    fi
    if [[ -n "$PROVIDER_PID" ]]; then
        kill $PROVIDER_PID 2>/dev/null || true
    fi
    exit 0
}

# Set up signal handlers
trap cleanup SIGINT SIGTERM

echo "🎯 Smainer Desktop Application"
echo "================================"

# Check if running from correct location
if [[ "$INSTALL_DIR" != "$(dirname "$(realpath "$0")")" ]] && [[ ! -d "$INSTALL_DIR" ]]; then
    # Running from package directory for testing
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    PROVIDER_BINARY="$SCRIPT_DIR/smainer-provider"
    WEB_DIR="$SCRIPT_DIR/web-dist"
fi

start_provider
start_web_server
open_browser

echo ""
echo "✅ Smainer Desktop is running!"
echo "   Web Interface: http://localhost:$PORT"
echo "   Provider PID: $PROVIDER_PID"
echo ""
echo "Press Ctrl+C to stop the application"

# Wait for signals
wait