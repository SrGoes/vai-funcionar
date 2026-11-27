@echo off
setlocal EnableExtensions DisableDelayedExpansion
rem Double-click shortcut for a GeForce in a PC of 16 GB of RAM: scripts\play.ps1 -Route nvidia
rem -SmallRam, which holds the kit of settings (and prints it). Other arguments go to play.ps1:
rem   Play-VIctory-16GB.cmd -DryRun
rem   Play-VIctory-16GB.cmd -Vars "VICTORY_LOG=info"
rem Settings set in this window do not reach the title: play.ps1 starts from a clean kit.
if not defined VICTORY_GAME_DIR (
  echo Set VICTORY_GAME_DIR to the folder containing your title executable. 1>&2
  exit /b 2
)
if not exist "%VICTORY_GAME_DIR%\game_scarlett_master.exe" (
  echo game_scarlett_master.exe not found in VICTORY_GAME_DIR: "%VICTORY_GAME_DIR%" 1>&2
  exit /b 2
)
set "build=release"
if not exist "%~dp0target\release\victory.exe" set "build=diag"
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\play.ps1" -Exe "%VICTORY_GAME_DIR%\game_scarlett_master.exe" -Route nvidia -SmallRam -Console series-s -Build %build% %*
exit /b %errorlevel%
