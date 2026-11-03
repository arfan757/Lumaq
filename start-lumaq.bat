@echo off
cd /d "%~dp0"
echo ====================================================
echo Starting Lumaq Liquid Glass Desktop Widget...
echo ====================================================
taskkill /F /IM lumaq.exe >nul 2>&1
start "" "%~dp0lumaq.exe"
