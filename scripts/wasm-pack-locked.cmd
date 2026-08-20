@echo off
setlocal EnableExtensions
powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File "%~dp0wasm-pack-locked.ps1" %*
exit /b %ERRORLEVEL%
