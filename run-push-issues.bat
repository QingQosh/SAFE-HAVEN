@echo off
echo.
echo ============================================
echo   Pushing All Remaining Issues to GitHub
echo ============================================
echo.
echo This will create issues 3-125...
echo Please wait, this takes about 5 minutes.
echo.

cd /d "%~dp0"
powershell.exe -ExecutionPolicy Bypass -NoProfile -File "%~dp0push-all-remaining-issues.ps1"

echo.
echo Done! Check https://github.com/shortheartone/SAFE-HAVEN/issues
echo.
pause
