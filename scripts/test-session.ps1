# A test session: the title through scripts\play.ps1 with everything recorded outside the
# repository, under %LOCALAPPDATA%\victory\runs\<date>-<name> (or -Root):
#   env.txt       the PC, the driver, the build, the branch, the settings the session ran with
#   log.txt       the runtime's whole log (VICTORY_LOG=info unless -Log says otherwise)
#   cap\          presented frames (VICTORY_CAPTURE, every 8th frame of the world)
#   metrics.csv   every 2 s: the process's memory and CPU, the PC's memory, the GPU's use,
#                 memory, power, temperature and clocks (nvidia-smi, read only)
#   settings.xml  the title's own settings file as it was before the session
#   crashes.log   what the runtime added to its crash log during the session
#   result.txt    exit code, duration, when the world began
# Then scripts\test-report.ps1 <run folder> sums it up.
#   scripts\test-session.ps1 -Exe <your copy's executable> -Name prison-intro -Minutes 10
# -Minutes  stop the title that many minutes after its first world frame (else: when you close it)
# -Safe, -RayTracing, -UploadAhead, -Console, -Build, -More: passed to play.ps1
param(
    [Parameter(Mandatory)][string]$Exe,
    [string]$Name = 'session',
    [double]$Minutes = 0,
    [string]$Log = 'info',
    [string]$Root = (Join-Path $env:LOCALAPPDATA 'victory\runs'),
    [switch]$Safe,
    [switch]$RayTracing,
    [switch]$UploadAhead,
    [ValidateSet('series-x', 'series-s', 'auto')][string]$Console = 'auto',
    [ValidateSet('diag', 'release')][string]$Build = 'diag',
    [hashtable]$More = @{}
)
$ErrorActionPreference = 'Stop'
$repo = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$run = Join-Path $Root ((Get-Date -Format 'yyyyMMdd-HHmm') + "-$Name")
if ([System.IO.Path]::GetFullPath($run).StartsWith($repo, [System.StringComparison]::OrdinalIgnoreCase)) {
    throw "The run folder $run is inside the repository; game-derived data stays outside it (docs/asset-policy.md)."
}
New-Item -ItemType Directory -Force (Join-Path $run 'cap') | Out-Null
$victory = Join-Path $repo "target\$Build\victory.exe"
if (-not (Test-Path $victory)) { throw "No $victory. Build it first." }

# --- The environment ---------------------------------------------------------------------
$data = if ($env:VICTORY_DATA_DIR) { $env:VICTORY_DATA_DIR } else { Join-Path $env:LOCALAPPDATA 'victory\data' }
$os = Get-CimInstance Win32_OperatingSystem
$cs = Get-CimInstance Win32_ComputerSystem
$env_lines = @(
    "date        $(Get-Date -Format o)",
    "os          $($os.Caption) $($os.Version) build $($os.BuildNumber)",
    "cpu         $((Get-CimInstance Win32_Processor | Select-Object -First 1).Name)",
    ("ram         {0:N1} GiB; commit limit {1:N1} GiB" -f ($cs.TotalPhysicalMemory / 1GB), ($os.TotalVirtualMemorySize / 1MB)),
    "pagefile    $((Get-CimInstance Win32_PageFileUsage | ForEach-Object { "$($_.Name) $($_.AllocatedBaseSize) MB" }) -join '; ')",
    "gpu         $((& nvidia-smi --query-gpu=name,pci.device_id,driver_version,memory.total,power.limit --format=csv,noheader 2>$null) -join '; ')",
    "branch      $(git -C $repo rev-parse --abbrev-ref HEAD) at $(git -C $repo rev-parse --short HEAD)$(if (git -C $repo status --porcelain) { ' (with local changes)' })",
    "build       $Build $victory",
    "exe sha256  $((Get-FileHash $victory -Algorithm SHA256).Hash.ToLower())",
    "title exe   $Exe",
    "options     Safe=$Safe RayTracing=$RayTracing UploadAhead=$UploadAhead Console=$Console Log=$Log Minutes=$Minutes More=$(($More.GetEnumerator() | ForEach-Object { "$($_.Key)=$($_.Value)" }) -join ' ')"
)
$env_lines | Out-File -Encoding utf8 (Join-Path $run 'env.txt')
$settings = Get-ChildItem $data -Recurse -Filter settings.xml -ErrorAction SilentlyContinue | Select-Object -First 1
if ($settings) { Copy-Item $settings.FullName (Join-Path $run 'settings.xml') }
$crashes = Get-ChildItem $data -Recurse -Filter crashes.log -ErrorAction SilentlyContinue | Select-Object -First 1
$crash_lines = if ($crashes) { (Get-Content $crashes.FullName).Count } else { 0 }

# --- The sampler ---------------------------------------------------------------------------
$sampler = Start-Job -ArgumentList (Join-Path $run 'metrics.csv') -ScriptBlock {
    param($out)
    'time,victory_private_mb,victory_ws_mb,victory_cpu_pct,free_phys_mb,commit_used_mb,commit_limit_mb,gpu_util_pct,gpu_mem_used_mb,gpu_mem_total_mb,gpu_power_w,gpu_temp_c,gpu_clock_mhz,gpu_mem_clock_mhz' | Out-File -Encoding ascii $out
    $last = $null
    $cores = [Environment]::ProcessorCount
    while ($true) {
        $now = Get-Date
        $p = Get-Process victory -ErrorAction SilentlyContinue | Select-Object -First 1
        $cpu = ''
        if ($p -and $last -and $last.Id -eq $p.Id) {
            # (With a point whatever the PC's locale: a comma would split the column.)
            $cpu = ([math]::Round(100 * ($p.TotalProcessorTime - $last.Cpu).TotalSeconds / (($now - $last.At).TotalSeconds * $cores), 1)).ToString([cultureinfo]::InvariantCulture)
        }
        if ($p) { $last = @{ Id = $p.Id; Cpu = $p.TotalProcessorTime; At = $now } }
        $os = Get-CimInstance Win32_OperatingSystem
        $g = (& nvidia-smi --query-gpu=utilization.gpu,memory.used,memory.total,power.draw,temperature.gpu,clocks.gr,clocks.mem --format=csv,noheader,nounits 2>$null) -replace ' ', ''
        $line = @(
            $now.ToString('o'),
            $(if ($p) { [math]::Round($p.PrivateMemorySize64 / 1MB) } else { '' }),
            $(if ($p) { [math]::Round($p.WorkingSet64 / 1MB) } else { '' }),
            $cpu,
            [math]::Round($os.FreePhysicalMemory / 1KB),
            [math]::Round(($os.TotalVirtualMemorySize - $os.FreeVirtualMemory) / 1KB),
            [math]::Round($os.TotalVirtualMemorySize / 1KB),
            $g
        ) -join ','
        $line | Out-File -Encoding ascii -Append $out
        Start-Sleep -Seconds 2
    }
}

# --- The stop after -Minutes of world --------------------------------------------------------
$logfile = Join-Path $run 'log.txt'
$stopper = $null
if ($Minutes -gt 0) {
    $stopper = Start-Job -ArgumentList $logfile, $Minutes -ScriptBlock {
        param($log, $minutes)
        while (-not ((Test-Path $log) -and (Select-String -Path $log -Pattern 'first frame of 500 draws' -Quiet))) { Start-Sleep -Seconds 3 }
        Start-Sleep -Seconds ([int]($minutes * 60))
        Stop-Process -Name victory -Force -ErrorAction SilentlyContinue
        'stopped by -Minutes'
    }
}

# --- The session ---------------------------------------------------------------------------
# (PowerShell names do not tell case apart: this one must not be called $more.)
$settingsForPlay = @{ VICTORY_LOG = $Log; VICTORY_CAPTURE = (Join-Path $run 'cap'); VICTORY_CAPTURE_WORLD = '1' }
foreach ($k in @($More.Keys)) { $settingsForPlay[$k] = $More[$k] }
$play = @{ Exe = $Exe; Console = $Console; Build = $Build; More = $settingsForPlay }
if ($Safe) { $play.Safe = $true }
if ($RayTracing) { $play.RayTracing = $true }
if ($UploadAhead) { $play.UploadAhead = $true }
$began = Get-Date
Write-Host "Recording into $run"
# The runtime logs to stderr: under 'Stop' its first line would end the session.
$ErrorActionPreference = 'Continue'
& (Join-Path $PSScriptRoot 'play.ps1') @play 2>&1 | ForEach-Object { "$_" } | Out-File -Encoding utf8 $logfile
$code = $LASTEXITCODE
$ended = Get-Date

Stop-Job $sampler -ErrorAction SilentlyContinue; Remove-Job $sampler -Force -ErrorAction SilentlyContinue
$stopped = ''
if ($stopper) { $stopped = (Receive-Job $stopper -ErrorAction SilentlyContinue) -join ''; Stop-Job $stopper -ErrorAction SilentlyContinue; Remove-Job $stopper -Force -ErrorAction SilentlyContinue }
if ($crashes -and (Test-Path $crashes.FullName)) {
    Get-Content $crashes.FullName | Select-Object -Skip $crash_lines | Out-File -Encoding utf8 (Join-Path $run 'crashes.log')
}
$world = Select-String -Path $logfile -Pattern 'first frame of 500 draws' | Select-Object -First 1
@(
    "exit code   $code (0x$('{0:X8}' -f $code))$(if ($stopped) { " - $stopped" })",
    "began       $($began.ToString('o'))",
    "ended       $($ended.ToString('o'))",
    "duration    $([math]::Round(($ended - $began).TotalMinutes, 1)) min",
    "world       $(if ($world) { $world.Line.Substring(0, [math]::Min(140, $world.Line.Length)) } else { 'not reached' })"
) | Out-File -Encoding utf8 (Join-Path $run 'result.txt')
Write-Host "Done: $run (exit $code). Next: scripts\test-report.ps1 '$run'"
