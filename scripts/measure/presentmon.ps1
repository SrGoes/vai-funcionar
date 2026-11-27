# Per-frame times of the title that is ALREADY OPEN, taken with Intel PresentMon, and their summary.
#   scripts\measure\presentmon.ps1 -PresentMon <path of PresentMon-x64.exe> [-Seconds 60] [-Out <folder>]
#   scripts\measure\presentmon.ps1 -Csv <a PresentMon csv>       only the summary of a capture made before
# -PresentMon  the PresentMon console executable (github.com/GameTechDev/PresentMon releases); not
#              shipped here. Capturing needs an elevated PowerShell or the "Performance Log Users" group.
# -Seconds     how long to capture (60 by default)
# -Out         where frames.csv and summary.txt go (a new folder under %TEMP% by default; never the
#              repository: captures and logs of the title stay out of it)
# -Process     the process to follow (victory.exe by default)
# The title is not started, closed or touched here, and it never runs elevated: open it first with
# Play-VIctory.cmd or scripts\play.ps1, reach the world, then run this in another window.
param(
    [string]$PresentMon,
    [int]$Seconds = 60,
    [string]$Out,
    [string]$Csv,
    [string]$Process = 'victory.exe'
)
$ErrorActionPreference = 'Stop'
$inv = [Globalization.CultureInfo]::InvariantCulture

function Read-Column([object[]]$rows, [string]$name) {
    $values = New-Object System.Collections.Generic.List[double]
    foreach ($row in $rows) {
        $text = $row.$name
        $number = 0.0
        if ($text -and $text -ne 'NA' -and [double]::TryParse($text, [Globalization.NumberStyles]::Float, $inv, [ref]$number)) { $values.Add($number) }
    }
    return ,$values.ToArray()
}

# Nearest rank on the sorted values: the smallest value with at least p percent of them at or below it.
function Get-Percentile([double[]]$sorted, [double]$p) {
    if ($sorted.Length -eq 0) { return [double]::NaN }
    $rank = [Math]::Ceiling($p / 100.0 * $sorted.Length)
    return $sorted[[Math]::Max(1, $rank) - 1]
}

function Get-Mean([double[]]$values) {
    if ($values.Length -eq 0) { return [double]::NaN }
    return ($values | Measure-Object -Average).Average
}

function Write-Summary([string]$path) {
    $rows = @(Import-Csv -LiteralPath $path)
    if ($rows.Count -eq 0) { throw "no frames in $path (was the title open and presenting?)" }
    $between = Read-Column $rows 'MsBetweenPresents'
    if ($between.Length -eq 0) { throw "no MsBetweenPresents column in $path (PresentMon 2.x is expected)" }
    $sorted = $between.Clone(); [Array]::Sort($sorted)
    $total = ($between | Measure-Object -Sum).Sum
    $fmt = { param($v) $v.ToString('0.0', $inv) }
    $lines = @()
    $lines += "frames            $($between.Length) in $((& $fmt ($total / 1000.0))) s"
    $lines += "fps (average)     $((1000.0 * $between.Length / $total).ToString('0.00', $inv))"
    $lines += "ms between frames median $(& $fmt (Get-Percentile $sorted 50)), p95 $(& $fmt (Get-Percentile $sorted 95)), p99 $(& $fmt (Get-Percentile $sorted 99)), longest $(& $fmt $sorted[-1])"
    $lines += "1% low            $((1000.0 / (Get-Percentile $sorted 99)).ToString('0.0', $inv)) fps (1000 / p99, nearest rank)"
    $lines += "frames over 150 ms $(@($between | Where-Object { $_ -gt 150 }).Count)"
    foreach ($name in 'MsCPUBusy', 'MsCPUWait', 'MsGPUBusy', 'MsGPUWait') {
        $values = Read-Column $rows $name
        if ($values.Length -gt 0) { $lines += ('{0,-17} {1} ms a frame (mean)' -f $name, (& $fmt (Get-Mean $values))) }
    }
    $sync = @($rows | Group-Object SyncInterval | ForEach-Object { "$($_.Name): $($_.Count)" }) -join ', '
    $mode = @($rows | Group-Object PresentMode | Sort-Object Count -Descending | ForEach-Object { "$($_.Name): $($_.Count)" }) -join ', '
    $lines += "sync interval     $sync"
    $lines += "present mode      $mode"
    return $lines
}

if ($Csv) {
    Write-Summary $Csv | ForEach-Object { Write-Host $_ }
    exit 0
}

if (-not $PresentMon -or -not (Test-Path -LiteralPath $PresentMon)) {
    Write-Host 'Give the PresentMon executable: -PresentMon <path of PresentMon-x64.exe> (or -Csv <file> to summarise a capture).' -ForegroundColor Yellow
    exit 2
}
$identity = [Security.Principal.WindowsIdentity]::GetCurrent()
$elevated = (New-Object Security.Principal.WindowsPrincipal($identity)).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $elevated) { Write-Host 'This window is not elevated: PresentMon answers "access denied" unless this user is in "Performance Log Users".' -ForegroundColor Yellow }
$name = [IO.Path]::GetFileNameWithoutExtension($Process)
$title = Get-Process $name -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowHandle -ne 0 } | Select-Object -First 1
if (-not $title) { Write-Host "No window of ${Process}: open the title, reach the world and run this again." -ForegroundColor Yellow; exit 2 }
if (-not $Out) { $Out = Join-Path $env:TEMP ('victory-presentmon-' + (Get-Date -Format 'yyyyMMdd-HHmmss')) }
New-Item -ItemType Directory -Force $Out | Out-Null
$frames = Join-Path $Out 'frames.csv'
Write-Host "PresentMon: $Seconds seconds of $Process (window title: $($title.MainWindowTitle))" -ForegroundColor Cyan
& $PresentMon --process_name $Process --timed $Seconds --terminate_after_timed --no_console_stats --stop_existing_session --session_name VictoryMeasure --output_file $frames
if ($LASTEXITCODE -ne 0 -or -not (Test-Path -LiteralPath $frames)) { Write-Host "PresentMon ended with $LASTEXITCODE and wrote no frames." -ForegroundColor Yellow; exit 1 }
$summary = Write-Summary $frames
$summary | Set-Content (Join-Path $Out 'summary.txt')
$summary | ForEach-Object { Write-Host $_ }
Write-Host "Files in $Out" -ForegroundColor Green
