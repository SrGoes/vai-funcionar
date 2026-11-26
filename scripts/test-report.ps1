# Sums up a run of scripts\test-session.ps1 (or any folder with a log.txt and cap\ from play.ps1)
# into files beside it, outside the repository:
#   frames.csv    the runtime's frame line every ~5 s: fps, ms a frame, draws, drops, waiting textures
#   memory.csv    its memory lines (diagnostics builds): private, resident, free RAM, commit left,
#                 video memory budget and use, PC memory shared with the card
#   summary.md    environment, result, frame times in the world, memory and GPU peaks, drops by
#                 reason, warnings and errors grouped, shaders that do not translate, crashes
#   sheet-NN.jpg  presented frames of the world, 60 to a sheet, to look through
#   scripts\test-report.ps1 <run folder>
# Frame times come from the runtime's own line, one value per interval of about 5 s: the
# statistics are of those intervals, not of single frames (no 1% low without per-frame times).
param([Parameter(Mandatory)][string]$Run, [int]$PerSheet = 60, [int]$SheetWidth = 1920)
$ErrorActionPreference = 'Stop'
$log = Join-Path $Run 'log.txt'
if (-not (Test-Path $log)) { throw "No $log" }
$logLines = Get-Content $log
$culture = [System.Globalization.CultureInfo]::InvariantCulture
# Numbers in the CSV files with a decimal point whatever the PC's locale.
[System.Threading.Thread]::CurrentThread.CurrentCulture = $culture
function Stat([double[]]$v) {
    if (-not $v -or $v.Count -eq 0) { return $null }
    $s = $v | Sort-Object
    $at = { param($q) $s[[math]::Min($s.Count - 1, [int][math]::Floor($q * $s.Count))] }
    [pscustomobject]@{ n = $s.Count; min = $s[0]; median = (& $at 0.5); mean = ($s | Measure-Object -Average).Average; p90 = (& $at 0.9); p99 = (& $at 0.99); max = $s[-1] }
}
function F($x, $d = 1) { if ($null -eq $x) { 'N/D' } else { ([double]$x).ToString("N$d", $culture) } }
# The time a log line begins with, to the second ('' for the lines a multi-line record goes on in).
function T([string]$s) { if ($s -match '^\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d') { $matches[0] } else { '' } }

# --- When the world began ----------------------------------------------------------------------
$worldLine = $logLines | Select-String 'first frame of 500 draws' | Select-Object -First 1
$worldTime = if ($worldLine) { T $worldLine.Line } else { $null }
$worldFrame = if ($worldLine -and $worldLine.Line -match 'frame=(\d+)') { [int]$matches[1] } else { $null }

# --- Frame lines ---------------------------------------------------------------------------------
$frames = foreach ($m in ($logLines | Select-String 'fps, [\d.]+ ms/frame')) {
    $l = $m.Line
    if ($l -match '^(\S+)Z .* ([\d.]+) fps, ([\d.]+) ms/frame') {
        $row = [ordered]@{ time = $matches[1]; fps = [double]::Parse($matches[2], $culture); ms = [double]::Parse($matches[3], $culture) }
        $row.draws = if ($l -match '([\d.]+) draws/frame') { [double]::Parse($matches[1], $culture) } else { $null }
        $row.dropped = if ($l -match 'dropped ([\d.]+) without a host pipeline') { [double]::Parse($matches[1], $culture) } else { $null }
        $row.making = if ($l -match '\((\d+) being made\)') { [int]$matches[1] } else { $null }
        $row.indirect_dropped = if ($l -match 'and ([\d.]+) indirect;') { [double]::Parse($matches[1], $culture) } else { $null }
        $row.textures_waiting = if ($l -match '(\d+) waiting, \d+ passed over') { [int]$matches[1] } else { $null }
        $row.world = [bool]($worldTime -and (T $l) -ge $worldTime)
        [pscustomobject]$row
    }
}
$frames | Export-Csv -NoTypeInformation -Encoding ascii (Join-Path $Run 'frames.csv')
$inWorld = @($frames | Where-Object world | Select-Object -Skip 2)   # the first lines mix in the loading

# --- Memory lines (diagnostics builds) ----------------------------------------------------------------
$memory = foreach ($m in ($logLines | Select-String 'MB private \(|with the memory line: the card.s own: budget')) {
    $l = $m.Line
    # Every row with every column: Export-Csv takes the columns of the first row alone.
    $row = [ordered]@{ time = (T $l); kind = ''; private_mb = $null; resident_mb = $null; free_mb = $null; commit_left_mb = $null
        vram_budget_mb = $null; vram_used_mb = $null; shared_budget_mb = $null; shared_used_mb = $null }
    if ($l -match '(\d+) MB private \((\d+) MB of it in memory; the PC has (\d+) MB free and (\d+) MB left to commit') {
        $row.kind = 'process'; $row.private_mb = [int]$matches[1]; $row.resident_mb = [int]$matches[2]; $row.free_mb = [int]$matches[3]; $row.commit_left_mb = [int]$matches[4]
    } elseif ($l -match "card.s own: budget (\d+) MB, in use (\d+) MB.*shared with the card: budget (\d+) MB, in use (\d+) MB") {
        $row.kind = 'video'; $row.vram_budget_mb = [int]$matches[1]; $row.vram_used_mb = [int]$matches[2]; $row.shared_budget_mb = [int]$matches[3]; $row.shared_used_mb = [int]$matches[4]
    } else { continue }
    [pscustomobject]$row
}
$memory | Export-Csv -NoTypeInformation -Encoding ascii (Join-Path $Run 'memory.csv')

# --- Sampler ---------------------------------------------------------------------------------------
$metrics = @()
$metricsFile = Join-Path $Run 'metrics.csv'
if (Test-Path $metricsFile) {
    $raw = Get-Content $metricsFile
    $header = $raw[0] -split ','
    # A first version of the sampler wrote the CPU share with the PC's decimal comma: one
    # field too many on those lines, joined back here.
    $metrics = foreach ($line in ($raw | Select-Object -Skip 1)) {
        $f = $line -split ','
        if ($f.Count -eq $header.Count + 1) { $f = @($f[0..2]) + @("$($f[3]).$($f[4])") + @($f[5..($f.Count - 1)]) }
        if ($f.Count -ne $header.Count) { continue }
        $o = [ordered]@{}; for ($i = 0; $i -lt $header.Count; $i++) { $o[$header[$i]] = $f[$i] }
        [pscustomobject]$o
    }
}
function Col($name) { @($metrics | ForEach-Object { $_.$name } | Where-Object { $_ -ne '' -and $null -ne $_ } | ForEach-Object { [double]::Parse($_, $culture) }) }

# --- Drops, warnings, errors, shaders, crashes ---------------------------------------------------------
$drops = $logLines | Select-String 'dropped ([\d.]+)/frame without a host pipeline: (.*)$' | Where-Object { -not $worldTime -or (T $_.Line) -ge $worldTime } |
    ForEach-Object { [pscustomobject]@{ why = $_.Matches[0].Groups[2].Value; n = [double]::Parse($_.Matches[0].Groups[1].Value, $culture) } } |
    Group-Object why | ForEach-Object { [pscustomobject]@{ why = $_.Name; mean = ($_.Group | Measure-Object n -Average).Average; intervals = $_.Count } } | Sort-Object mean -Descending
$norm = { param($s) (($s -replace '0x[0-9a-fA-F]+', '0x#') -replace '\b[0-9a-f]{16}\b', '<hash>' -replace '\d+(\.\d+)?', 'N') }
$warnings = $logLines | Select-String ' (WARN|ERROR) ' | ForEach-Object {
    $t = $_.Line -replace '^\S+\s+', ''
    $t = & $norm $t
    $t.Substring(0, [math]::Min(220, $t.Length))
} | Group-Object | Sort-Object Count -Descending
$untranslated = $logLines | Select-String 'shader does not translate.*stage="(\w+)" hash=(\w+) error=(.*)$' | ForEach-Object { "$($_.Matches[0].Groups[1].Value) $($_.Matches[0].Groups[2].Value): $($_.Matches[0].Groups[3].Value)" } | Sort-Object -Unique
$failures = $logLines | Select-String 'host GPU failure|unhandled exception|device removed|DEVICE_REMOVED|panicked' | ForEach-Object { $_.Line.Substring(0, [math]::Min(300, $_.Line.Length)) }
$console = $logLines | Select-String '^console ' | Select-Object -First 1

# --- Sheets of presented frames ---------------------------------------------------------------------------
Add-Type -AssemblyName System.Drawing
$caps = Get-ChildItem (Join-Path $Run 'cap') -Filter 'frame-*.png' -ErrorAction SilentlyContinue |
    Where-Object { $_.BaseName -match '^frame-(\d+)$' -and (-not $worldFrame -or [int]$matches[1] -ge $worldFrame) } |
    Sort-Object { [int]($_.BaseName -replace 'frame-', '') }
Get-ChildItem $Run -Filter 'sheet-*.jpg' -ErrorAction SilentlyContinue | Remove-Item
$sheets = 0
for ($i = 0; $i -lt $caps.Count; $i += $PerSheet) {
    $group = $caps[$i..([math]::Min($caps.Count, $i + $PerSheet) - 1)]
    $cols = 10; $w = [int]($SheetWidth / $cols); $h = [int]($w * 9 / 16)
    $rows = [math]::Ceiling($group.Count / $cols)
    $bmp = New-Object System.Drawing.Bitmap ($cols * $w), ($rows * ($h + 14))
    $g = [System.Drawing.Graphics]::FromImage($bmp); $g.Clear([System.Drawing.Color]::Gray)
    $font = New-Object System.Drawing.Font 'Arial', 8
    for ($k = 0; $k -lt $group.Count; $k++) {
        $img = [System.Drawing.Image]::FromFile($group[$k].FullName)
        $x = ($k % $cols) * $w; $y = [math]::Floor($k / $cols) * ($h + 14)
        $g.DrawImage($img, $x, $y + 14, $w, $h); $g.DrawString($group[$k].BaseName, $font, [System.Drawing.Brushes]::Yellow, $x + 1, $y)
        $img.Dispose()
    }
    $sheets++
    $bmp.Save((Join-Path $Run ('sheet-{0:D2}.jpg' -f $sheets)), [System.Drawing.Imaging.ImageFormat]::Jpeg)
    $g.Dispose(); $bmp.Dispose()
}

# --- summary.md -------------------------------------------------------------------------------------------
$ms = Stat ($inWorld | ForEach-Object ms)
$fps = Stat ($inWorld | ForEach-Object fps)
$proc = @($memory | Where-Object kind -eq 'process'); $video = @($memory | Where-Object kind -eq 'video')
$out = New-Object System.Collections.Generic.List[string]
$out.Add("# Test run $(Split-Path $Run -Leaf)"); $out.Add('')
foreach ($f in 'env.txt', 'result.txt') { if (Test-Path (Join-Path $Run $f)) { $out.Add('```'); (Get-Content (Join-Path $Run $f)) | ForEach-Object { $out.Add($_) }; $out.Add('```') } }
$out.Add("- $(if ($console) { $console.Line.Trim() } else { 'console line not found' })")
$out.Add("- world from frame $(if ($worldFrame) { $worldFrame } else { 'N/D' }) at $(if ($worldTime) { "$worldTime UTC" } else { 'N/D' }); $($caps.Count) captured world frames in $sheets sheets")
$out.Add(''); $out.Add('## Frame time in the world (one value per ~5 s interval, first two left out)'); $out.Add('')
$out.Add('| | n | min | median | mean | p90 | p99 | max |'); $out.Add('|---|---|---|---|---|---|---|---|')
if ($ms) { $out.Add("| ms/frame | $($ms.n) | $(F $ms.min) | $(F $ms.median) | $(F $ms.mean) | $(F $ms.p90) | $(F $ms.p99) | $(F $ms.max) |") }
if ($fps) { $out.Add("| fps | $($fps.n) | $(F $fps.min) | $(F $fps.median) | $(F $fps.mean) | $(F $fps.p90) | $(F $fps.p99) | $(F $fps.max) |") }
$d = Stat ($inWorld | ForEach-Object draws); $dr = Stat ($inWorld | Where-Object { $null -ne $_.dropped } | ForEach-Object dropped); $di = Stat ($inWorld | Where-Object { $null -ne $_.indirect_dropped } | ForEach-Object indirect_dropped); $tw = Stat ($inWorld | Where-Object { $null -ne $_.textures_waiting } | ForEach-Object textures_waiting)
$out.Add(''); $out.Add("- draws/frame median $(F $d.median 0); dropped without a host pipeline median $(F $dr.median 0), indirect dropped median $(F $di.median 0); textures picked by index waiting median $(F $tw.median 0), max $(F $tw.max 0)")
$out.Add(''); $out.Add('## Memory and GPU'); $out.Add('')
$out.Add('| | median | max |'); $out.Add('|---|---|---|')
foreach ($c in 'victory_private_mb', 'victory_ws_mb', 'victory_cpu_pct', 'free_phys_mb', 'commit_used_mb', 'gpu_util_pct', 'gpu_mem_used_mb', 'gpu_power_w', 'gpu_temp_c', 'gpu_clock_mhz') {
    $s = Stat (Col $c); if ($s) { $out.Add("| sampler $c | $(F $s.median 0) | $(F $s.max 0) |") }
}
foreach ($c in 'private_mb', 'resident_mb', 'free_mb', 'commit_left_mb') { $s = Stat ($proc | ForEach-Object $c); if ($s) { $out.Add("| runtime $c | $(F $s.median 0) | $(F $s.max 0) (min $(F $s.min 0)) |") } }
foreach ($c in 'vram_budget_mb', 'vram_used_mb', 'shared_used_mb') { $s = Stat ($video | ForEach-Object $c); if ($s) { $out.Add("| runtime $c | $(F $s.median 0) | $(F $s.max 0) |") } }
$out.Add(''); $out.Add('## Draws and dispatches dropped in the world, by reason (mean a frame over the intervals)'); $out.Add('')
foreach ($x in $drops) { $out.Add("- $(F $x.mean 1)/frame over $($x.intervals) intervals: $($x.why)") }
$out.Add(''); $out.Add('## Crashes and GPU failures'); $out.Add('')
if ($failures) { $failures | ForEach-Object { $out.Add("- $_") } } else { $out.Add('- none in the log') }
if ((Test-Path (Join-Path $Run 'crashes.log')) -and (Get-Content (Join-Path $Run 'crashes.log'))) { $out.Add(''); $out.Add('```'); Get-Content (Join-Path $Run 'crashes.log') | Select-Object -First 40 | ForEach-Object { $out.Add($_) }; $out.Add('```') }
$out.Add(''); $out.Add('## Shaders that do not translate'); $out.Add('')
if ($untranslated) { $untranslated | ForEach-Object { $out.Add("- $_") } } else { $out.Add('- none') }
$out.Add(''); $out.Add('## Warnings and errors, grouped (numbers and addresses made N, 0x#, <hash>)'); $out.Add('')
foreach ($x in ($warnings | Select-Object -First 40)) { $out.Add("- $($x.Count) x ``$($x.Name)``") }
$out | Out-File -Encoding utf8 (Join-Path $Run 'summary.md')
Write-Host "Wrote $(Join-Path $Run 'summary.md'), frames.csv, memory.csv and $sheets sheets"
