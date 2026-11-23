@echo off
setlocal EnableDelayedExpansion
title THE TANK
cd /d "%~dp0"

rem --- startup checks: explain problems instead of failing silently ---
echo "%~dp0" | find /i "\Temp\" >nul && goto inzip
if not exist "%~dp0tank.exe" goto noexe
"%~dp0tank.exe" version >nul 2>&1
if errorlevel 1 goto broken
if not exist "%~dp0data\climate\chicago_daily.csv" goto nodata
if not exist runs mkdir runs

:menu
cls
echo.
echo   ==========================================
echo     THE TANK  -  Phase 1 test build
echo   ==========================================
echo.
echo     1. Watch a new tank   (random seed, 12 cycles, opens viewer)
echo     2. Watch a specific seed
echo     3. Batch test         (8 tanks x 40 cycles, opens report)
echo     4. Scenario test      (famine, plague, stranger... on 3 seeds)
echo     5. Determinism check  (compare with the reference hash)
echo     6. Open the runs folder
echo     Q. Quit
echo.
set "choice="
set /p "choice=  Choose: "
if /i "%choice%"=="1" goto random
if /i "%choice%"=="2" goto seed
if /i "%choice%"=="3" goto batch
if /i "%choice%"=="4" goto scen
if /i "%choice%"=="5" goto det
if /i "%choice%"=="6" ( start "" explorer "%~dp0runs" & goto menu )
if /i "%choice%"=="q" exit /b 0
goto menu

:random
set /a SEED=%RANDOM% %% 9999 + 1
set CYC=12
goto watch

:seed
set "SEED="
set /p "SEED=  Seed number: "
if "%SEED%"=="" goto menu
set "CYC="
set /p "CYC=  How many cycles? [12]: "
if "%CYC%"=="" set CYC=12
goto watch

:watch
echo.
echo   Running tank %SEED% for %CYC% cycles...
echo.
tank.exe run --seed %SEED% --cycles %CYC% --viewer --viewer-cycles 0..%CYC% --out runs\seed-%SEED%
if errorlevel 1 goto failed
echo.
echo   Opening the viewer. Press Space to play; click a dot to read their thoughts.
start "" "%~dp0runs\seed-%SEED%\view.html"
echo   Files are in runs\seed-%SEED%  (chronicle.txt is a readable history).
pause
goto menu

:batch
echo.
echo   Running 8 tanks x 40 cycles (uses all CPU cores; a few minutes)...
tank.exe batch --seeds 1..8 --cycles 40 --out runs\batch > runs\batch.log 2>&1
if errorlevel 1 goto failed
start "" notepad "%~dp0runs\batch\report.md"
goto menu

:scen
echo.
echo   Running every scenario on 3 seeds (a few minutes)...
tank.exe scenarios --seeds 1..3 --cycles 30 --at 8 --out runs\scenarios > runs\scenarios.log 2>&1
if errorlevel 1 goto failed
start "" notepad "%~dp0runs\scenarios\scenarios.md"
goto menu

:det
echo.
tank.exe determinism --seed 3 --cycles 8
echo.
echo   Reference (Linux build): 1a12f34314e5220b
echo   If all three lines above show that number with MATCH, results are identical on Windows.
pause
goto menu

:failed
echo.
echo   Something went wrong (see the messages above). Please send them to Claude.
pause
goto menu

:inzip
echo.
echo   It looks like Tank.bat was opened from inside the zip file.
echo   Windows only unpacked this one file, so the program next to it is missing.
echo.
echo   Fix: close this window. Right-click TheTank-windows.zip, choose
echo   "Extract All...", then open the extracted folder and double-click Tank.bat.
echo.
pause
exit /b 1

:noexe
echo.
echo   tank.exe is missing from this folder:
echo   %~dp0
echo.
echo   Either the zip was not fully extracted, or Windows Defender removed it.
echo   To check: Start menu, "Windows Security", "Virus and threat protection",
echo   "Protection history". If tank.exe is listed there, choose "Allow",
echo   then extract the zip again.
echo.
pause
exit /b 1

:nodata
echo.
echo   The climate data file is missing: data\climate\chicago_daily.csv
echo   Please extract the whole zip again, keeping its folders.
echo.
pause
exit /b 1

:broken
echo.
echo   tank.exe is here but will not start. Its own error message follows:
echo   ----------------------------------------------------------------
"%~dp0tank.exe" version
echo   ----------------------------------------------------------------
echo   Please send Claude a screenshot of this window.
echo.
pause
exit /b 1
