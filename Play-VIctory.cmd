@echo off
setlocal EnableExtensions DisableDelayedExpansion
rem Double-click shortcut for GeForce cards: scripts\play.ps1 -Route nvidia, which holds
rem the kit of settings (and prints it).
rem   Play-VIctory.cmd                    the console is chosen by this PC's memory (auto)
rem   Play-VIctory.cmd series-s           or series-x: the console the title sees
rem   Play-VIctory.cmd series-x -DryRun   other arguments go to play.ps1
rem   Play-VIctory.cmd -Vars "VICTORY_THIN=1;VICTORY_LOG=info"
rem Settings set in this window do not reach the title: play.ps1 starts from a clean kit.
rem The route is fixed here. Name the console with the word, not with -Console.
set "console=auto"
set "rest=%*"
:lead
set "first=%~1"
if not defined first goto checked
if /i "%first%"=="pareto-residency" (
  call set "rest=%%rest:*pareto-residency=%%"
  shift /1
  goto lead
)
if /i "%first%"=="series-s" (
  set "console=series-s"
  call set "rest=%%rest:*series-s=%%"
  shift /1
  goto lead
)
if /i "%first%"=="series-x" (
  set "console=series-x"
  call set "rest=%%rest:*series-x=%%"
  shift /1
  goto lead
)
if /i "%first%"=="auto" (
  set "console=auto"
  call set "rest=%%rest:*auto=%%"
  shift /1
  goto lead
)
if "%first:~0,1%"=="-" goto checked
echo Usage: Play-VIctory.cmd [pareto-residency] [series-s^|series-x^|auto] [play.ps1 arguments] 1>&2
exit /b 2
:checked
if not defined VICTORY_GAME_DIR (
  echo Set VICTORY_GAME_DIR to the folder containing your title executable. 1>&2
  exit /b 2
)
if not exist "%VICTORY_GAME_DIR%\game_scarlett_master.exe" (
  echo game_scarlett_master.exe not found in VICTORY_GAME_DIR: "%VICTORY_GAME_DIR%" 1>&2
  exit /b 2
)
set "runner=%~dp0victory-3d-rtx-pareto-residency.exe"
if not exist "%runner%" set "runner=%~dp0target\release\victory.exe"
if not exist "%runner%" (
  echo Extract the Windows release or run cargo build --release --locked -p victory. 1>&2
  exit /b 2
)
rem auto: series-s on a PC with less than 48 GiB of RAM or 12 GiB of video memory, which
rem series-x runs out of (play.ps1 prints what was chosen and why). On an 8 GB card
rem series-s dropped the head/skin meshes (faces rendered as eyes and teeth only, seen
rem 2026-11-26) and series-x rendered them: say "series-x" to ask for it.
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\play.ps1" -Exe "%VICTORY_GAME_DIR%\game_scarlett_master.exe" -Route nvidia -Console %console% -Victory "%runner%" %rest%
exit /b %errorlevel%
