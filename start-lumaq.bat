@echo off
setlocal EnableExtensions
cd /d "%~dp0"

set "APP=%~dp0lumaq.exe"
set "WEBVIEW=%~dp0WebView2Loader.dll"

echo ====================================================
echo Starting Lumaq Liquid Glass Desktop Widget...
echo ====================================================

if not exist "%APP%" (
    echo [ERROR] lumaq.exe was not found next to this script.
    echo         Build it first with: npm run tauri build
    echo.
    pause
    exit /b 1
)

if not exist "%WEBVIEW%" (
    echo [WARN] WebView2Loader.dll is missing next to lumaq.exe.
    echo        The app may fail to start unless the WebView2 runtime is present.
    echo.
)

tasklist /FI "IMAGENAME eq lumaq.exe" 2>nul | find /I "lumaq.exe" >nul
if not errorlevel 1 (
    echo Closing existing Lumaq instance...
    taskkill /F /IM lumaq.exe >nul 2>&1
    if errorlevel 1 (
        echo [ERROR] Could not close the running Lumaq instance.
        echo.
        pause
        exit /b 1
    )
)

start "" "%APP%"

rem `start` does not reliably set ERRORLEVEL, so verify the process is running.
ping -n 3 127.0.0.1 >nul 2>&1
tasklist /FI "IMAGENAME eq lumaq.exe" 2>nul | find /I "lumaq.exe" >nul
if errorlevel 1 (
    echo [ERROR] Lumaq did not start.
    echo         If a message mentions "Device Guard policy" or "Smart App Control",
    echo         Windows is blocking this unsigned build. Sign the executable or
    echo         disable Smart App Control, then try again.
    echo         Run lumaq.exe directly for more details.
    echo.
    pause
    exit /b 1
)

echo Lumaq started successfully.
exit /b 0
