@echo off
setlocal EnableExtensions DisableDelayedExpansion
rem Double-click shortcut for GeForce cards: scripts\play.ps1 -Route nvidia, which holds
rem the kit of settings (and prints it). Other arguments go to play.ps1, for example
rem   Play-VIctory.cmd -DryRun
rem   Play-VIctory.cmd -Vars "VICTORY_THIN=1;VICTORY_LOG=info"
rem Settings set in this window do not reach the title: play.ps1 starts from a clean kit.
set "first=%~1"
set "rest=%*"
if not defined first goto checked
if /i "%first%"=="pareto-residency" (
  call set "rest=%%rest:*pareto-residency=%%"
  goto checked
)
if "%first:~0,1%"=="-" goto checked
echo Usage: Play-VIctory.cmd [pareto-residency] [play.ps1 arguments] 1>&2
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
rem Series X, not S: on an 8 GB card series-s drops the head/skin meshes (faces
rem render as eyes and teeth only); series-x renders them. Verified 2026-11-26.
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\play.ps1" -Exe "%VICTORY_GAME_DIR%\game_scarlett_master.exe" -Route nvidia -Console series-x -Victory "%runner%" %rest%
exit /b %errorlevel%
