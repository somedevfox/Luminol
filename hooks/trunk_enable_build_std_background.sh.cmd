@echo off
setlocal

:: Wait until Trunk errors out or builds successfully, then restore the old Cargo config
:loop
timeout /t 1 /nobreak >nul
if not exist %TRUNK_STAGING_DIR%\ goto end
if exist %TRUNK_STAGING_DIR%\luminol.js goto end
tasklist | find trunk.exe >nul
if errorlevel 1 goto end
goto loop
:end
move .cargo\config.toml.bak .cargo\config.toml
