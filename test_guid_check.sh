#!/bin/bash
# Launch Ryujinx config update to see what GUID is generated
cd ui
npm run tauri dev 2>&1 | grep "Controller" | head -5 &
PID=$!
sleep 5
kill $PID 2>/dev/null
