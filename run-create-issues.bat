@echo off
echo.
echo ========================================================
echo   Creating 125 GitHub Issues for SAFE-HAVEN
echo ========================================================
echo.

powershell -ExecutionPolicy Bypass -Command "& { Set-Location '%~dp0'; . '.\Create-AllIssues.ps1' }"

echo.
echo Done!
pause
