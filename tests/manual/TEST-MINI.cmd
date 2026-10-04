@echo off
setlocal
cd /d "%~dp0"
echo VeekPanel Mini 1.0 hardware test - experimental, no audio changes.
echo Close ALL other PCPanel apps, connect the Mini, then press any key.
echo No COM port, Device Manager, Rust, or administrator account is needed.
pause
veek-probe.exe test-mini
set "mini_result=%ERRORLEVEL%"
echo.
if not "%mini_result%"=="0" echo The test reported an error. Keep the capture folder and note the console error.
echo Open captures, then the newest mini folder. Read SUMMARY.txt and fill in RESULTS.txt.
echo Zip that entire mini folder and return it. Nothing uploads automatically.
pause
exit /b %mini_result%
