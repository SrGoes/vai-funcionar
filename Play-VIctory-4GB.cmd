@echo off
setlocal EnableExtensions DisableDelayedExpansion
rem Double-click shortcut for 4 GB cards: scripts\play.ps1 -Route nvidia -LowMemory, which
rem holds the kit of settings (and prints it). Other arguments go to play.ps1, for example
rem   Play-VIctory-4GB.cmd -DryRun
rem   Play-VIctory-4GB.cmd -Vars "VICTORY_THIN=1;VICTORY_LOG=info"
rem Settings set in this window do not reach the title: play.ps1 starts from a clean kit.
rem The route and the console are fixed here; for another console run play.ps1 itself.
set "rest=%*"
if /i "%~1"=="pareto-residency" call set "rest=%%rest:*pareto-residency=%%"
if not defined VICTORY_GAME_DIR (
  echo Set VICTORY_GAME_DIR to the folder containing your title executable. 1>&2
  exit /b 2
)
if not exist "%VICTORY_GAME_DIR%\game_scarlett_master.exe" (
  echo game_scarlett_master.exe not found in VICTORY_GAME_DIR: "%VICTORY_GAME_DIR%" 1>&2
  exit /b 2
)
set "runner=%~dp0target\release\victory.exe"
if not exist "%runner%" set "runner=%~dp0victory-3d-rtx-pareto-residency.exe"
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\play.ps1" -Exe "%VICTORY_GAME_DIR%\game_scarlett_master.exe" -Route nvidia -LowMemory -Console series-s -Victory "%runner%" %rest%
exit /b %errorlevel%
