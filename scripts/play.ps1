# Starts the title with the settings this fork is played with (README.md, "How to Run").
#   scripts\play.ps1 -Exe <path of your own copy's executable> -Route nvidia|amd
# -Route        which kit of settings: amd (the default, what this script always set) or nvidia
#               (the kit of Play-VIctory.cmd, the route tested on GeForce cards). Measure and play
#               on the route of your card: the two kits give different pictures and frame rates.
# -LowMemory    with -Route nvidia: the kit of Play-VIctory-4GB.cmd (4 GB cards, 16 GB of RAM)
# -SmallRam     with -Route nvidia: for a PC of 16 GB of RAM (Play-VIctory-16GB.cmd). Windows
#               gives the card half the RAM as shared memory (7.4 GB there); the port's windows
#               of title memory (~5 GB) and the staging of a loading world went past it, upload
#               buffers failed and the driver crashed closing a list (RTX 5060, 27/11, three runs
#               at 7-11 min). This keeps it under: 1280x720 output with the title's own upscaler
#               (FidelityFX divides by zero below 1080p on Series S), textures at most 1024
#               texels a side, windows in the card's memory while it has room, hair on, the
#               defrag (#46) left out until its picture is checked, and the windows that lie in
#               private memory read in place (VICTORY_INPLACE=1). Reached the open world
#               (boat chase) at 7-10 fps with shared memory at 5.6-5.8 GB; with VICTORY_INPLACE=1,
#               4.7 GB (median, 11 min, one run).
# -Safe         without the newer, faster ways (the first thing to try when something is wrong)
# -RayTracing   the title's own ray-traced indirect light: lit interiors, about a third slower
# -Build        target\diag (the default, the build all measurements are of) or target\release
# -Victory      another victory.exe to run instead of target\<Build>\victory.exe
# -Hair         VICTORY_HAIR=1: the hair programs (and a few others) the upstream skips by default
# -More         other settings: -More @{VICTORY_PANEL='1'}
# -Vars         other settings as text, for callers that cannot pass a hashtable (a .cmd):
#               -Vars "VICTORY_PANEL=1;VICTORY_LOG=info"
# -Console      the console the title sees: auto (the default: series-s on a PC with less
#               than 48 GiB of RAM or 12 GiB of video memory, which series-x runs out of;
#               the console line of the output says what was chosen and why), series-s,
#               or series-x
# -UploadAhead  textures' first uploads on a copy queue of their own (VICTORY_UPLOAD_AHEAD),
#               on by default before: on an RTX 5060 with 16 GB it left decals, posters and
#               tattoos black in the prison corridor and frames were longer with it (median
#               170 against 159 ms, p90 424 against 206; runs 05 and 07, docs/bug-journal.md).
#               The nvidia kit has it on already.
# -DryRun       prints the settings and the command, and starts nothing
# Every VICTORY_* setting left over in this window goes first (VICTORY_CONFIG, VICTORY_GAME_DIR,
# VICTORY_CACHE_DIR and VICTORY_DATA_DIR stay): the kit printed is the whole kit the title runs with.
# Run it from a normal window, not as administrator: the driver then keeps its shader cache elsewhere.
param(
    [Parameter(Mandatory)][string]$Exe,
    [ValidateSet('amd', 'nvidia')][string]$Route = 'amd',
    [switch]$LowMemory,
    [switch]$SmallRam,
    [switch]$Safe,
    [switch]$RayTracing,
    [switch]$UploadAhead,
    [switch]$Hair,
    [ValidateSet('diag', 'release')][string]$Build = 'diag',
    [string]$Victory = '',
    [hashtable]$More = @{},
    [string]$Vars = '',
    [ValidateSet('series-x', 'series-s', 'auto')][string]$Console = 'auto',
    [switch]$DryRun
)
if ($LowMemory -and $Route -ne 'nvidia') { Write-Host '-LowMemory is a variant of -Route nvidia.'; exit 2 }
if ($SmallRam -and $Route -ne 'nvidia') { Write-Host '-SmallRam is a variant of -Route nvidia.'; exit 2 }

if ($Route -eq 'amd') {
    $settings = @{
        VICTORY_TEXTURE_NO_WAIT = '1'; VICTORY_RELAY = '2'; VICTORY_WIDE_BACK = '1'; VICTORY_CARD_REHASH_RUNS = '1'
        VICTORY_JOIN_COPIES = '1'; VICTORY_INDIRECT_WINDOW = '1'; VICTORY_RELAY_COPY = '1'; VICTORY_CARD_STEPS = '2'
        VICTORY_INDIRECT_BATCH = '2'; VICTORY_MAP_THREADS = '1'; VICTORY_TITLE_PRIORITIES = '0'; VICTORY_CARD_AWAKE = '1'
        VICTORY_SHOW_AFTER = '1'; VICTORY_AMBIENT = '0.8'; VICTORY_D3D12_DRED = 'light'; VICTORY_SKIP_PIPELINES = 'none'
    }
    if ($Safe) {
        $settings.VICTORY_UNDER_WIDE = '0'; $settings.VICTORY_LAND_ECHO = '0'
    } else {
        $settings.VICTORY_THIN = '1'; $settings.VICTORY_JOIN_PARTS = '8'
    }
} else {
    # The kit of Play-VIctory.cmd (the route tested on GeForce cards).
    $settings = @{
        VICTORY_AMBIENT_AIR = '1.5'; VICTORY_TEXTURE_NO_WAIT = '1'; VICTORY_RELAY = '2'; VICTORY_WIDE_BACK = '1'
        VICTORY_CARD_REHASH_RUNS = '1'; VICTORY_JOIN_COPIES = '1'; VICTORY_INDIRECT_WINDOW = '1'; VICTORY_RELAY_COPY = '1'
        VICTORY_CARD_STEPS = '2'; VICTORY_INDIRECT_BATCH = '2'; VICTORY_MAP_THREADS = '1'; VICTORY_CARD_AWAKE = '1'
        VICTORY_SHOW_AFTER = '1'
        # A constant, not the frame estimate: walking into a bright doorway raised
        # the estimate and washed the whole frame out. constant:1.0 is even light.
        VICTORY_AMBIENT = 'constant:0.8'
        # The write guard misses some titles' writes: skin on a guard's arms and the
        # phone's frame read garbage (colour noise) until every check hashed the
        # texture again. Costs some CPU; the frame rate held at 5-7 fps here.
        VICTORY_TEXTURE_GUARD = '0'
        # The asynchronous submission paths stay off on NVIDIA: with THIN the head
        # skin of people in play is not drawn (eyes and teeth are), and JOIN_PARTS
        # repeats frames of the opening video. -Vars "VICTORY_THIN=1" to try instead.
        VICTORY_THIN = '0'; VICTORY_JOIN_PARTS = '0'
        # Renderer ports 1 & 5 stay opt-in: on 8 GB NVIDIA cards PORT_CBV=1 can drop
        # road surfaces and PORT_CONSTANTS+PORT_CBV can raise a C++ exception (#15).
        VICTORY_PORT_CONSTANTS = '0'; VICTORY_PORT_CBV = '0'
        VICTORY_LIBRARY_STORE_LATER = '1'; VICTORY_PIPELINE_WAIT_MS = '0'; VICTORY_UPLOAD_AHEAD = '1'
        VICTORY_LEVELS_ON_DEMAND = '1'; VICTORY_CARD_BUFFERS_MIN_KB = '64'
        # 27/11 night, RTX 4080 / 9800X3D, fixed scene by bench.py: 9.3 -> 12-13 fps
        # (p50 96 -> 58 ms; 1% low did not improve). EXACT_LAZY is most of it: the
        # recording threads queued on the EXACT lock while it swept ~8,700 copies at
        # every miss. The others each gave 1.5-4.5 ms in A/B runs (VICTORY_AB).
        # VICTORY_TEXTURE_ONCE_A_FRAME stays opt-in: with the guard off it can show a
        # texture the title rewrote mid-frame a frame late, and no picture A/B was made.
        VICTORY_EXACT_LAZY = '1'; VICTORY_COPY_BY_RANGE = '1'; VICTORY_FLUSH_GROUPED = '1'
        VICTORY_CARD_STEPS_JOIN = '1'
        VICTORY_PORT_GUARD = '1'; VICTORY_PORT_GUARD_PLAN = '1'; VICTORY_ASK_NORESET = '1'
        # The EXACT map's hits under a shared lock, and a miss's buffer made with
        # the map unlocked: with EXACT_LAZY the lock still had 70% of the waiting
        # inside the recording threads' long list calls (RTX 5060, timeline, 27/11).
        VICTORY_EXACT_SPLIT = '1'
    }
    if ($LowMemory) {
        # Play-VIctory-4GB.cmd: the estimate (not the constant), the guard at its
        # default, and fewer, larger card buffers: reduce card-buffer thrashing and
        # the driver's background recompile threads on 4 GB cards.
        $settings.VICTORY_AMBIENT = '0.8'
        $settings.Remove('VICTORY_TEXTURE_GUARD')
        $settings.VICTORY_CARD_BUFFERS_MIN_KB = '4096'; $settings.VICTORY_CARD_BUFFERS_MB = '256'
        $settings.VICTORY_D3D12_BACKGROUND = '0'
    }
    if ($SmallRam) {
        # See -SmallRam above.
        $settings.VICTORY_OUTPUT = '1280x720'; $settings.VICTORY_UPSCALER = '6'
        $settings.VICTORY_TEXTURE_MAX = '1024'; $settings.VICTORY_WINDOWS_BAR_MB = '1536'
        $settings.VICTORY_HAIR = '1'; $settings.VICTORY_SKIP_PIPELINES = '02384a60fda6f2d0'
        # Windows of title memory in private memory committed whole are read in
        # place, no copy (~80 windows, ~0.4 GB on an RTX 5060 with 16 GB, 27/11):
        # 11 min, no crash, no upload failure, picture checked on captures.
        $settings.VICTORY_INPLACE = '1'
    }
    if ($Safe) { $settings.VICTORY_UNDER_WIDE = '0'; $settings.VICTORY_LAND_ECHO = '0' }
}
if ($UploadAhead) { $settings.VICTORY_UPLOAD_AHEAD = '1' }
if ($RayTracing) { $settings.VICTORY_BVH_BUILDER = '1' }
if ($Hair) { $settings.VICTORY_HAIR = '1' }
foreach ($k in $More.Keys) { $settings[$k] = [string]$More[$k] }
foreach ($pair in $Vars.Split(';', [StringSplitOptions]::RemoveEmptyEntries)) {
    $name, $value = $pair.Split('=', 2)
    if ($null -eq $value) { Write-Host "-Vars: '$pair' is not NAME=value."; exit 2 }
    $settings[$name.Trim()] = $value
}
if ($Route -eq 'nvidia' -and -not $settings.ContainsKey('VICTORY_LOG')) {
    # As the .cmd shortcuts did: warnings only, unless a level was asked for
    # (Play-VIctory.cmd also leaves RUST_LOG alone; the 4 GB one did not look).
    if ($LowMemory -or -not $env:RUST_LOG) { $settings.VICTORY_LOG = 'warn' }
}

$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
if (-not $Victory) { $Victory = Join-Path $root "target\$Build\victory.exe" }
$paths = 'VICTORY_CONFIG', 'VICTORY_GAME_DIR', 'VICTORY_CACHE_DIR', 'VICTORY_DATA_DIR'
# The nvidia kit keeps a translation cache of its own, as the .cmd shortcuts did
# (Play-VIctory.cmd only when none is set; the 4 GB one always).
$cache = $null
if ($Route -eq 'nvidia' -and ($LowMemory -or -not $env:VICTORY_CACHE_DIR)) {
    $cache = Join-Path $env:LOCALAPPDATA 'VIctory\cache-pareto'
}

$kit = foreach ($k in $settings.Keys) { $k + '=' + $settings[$k] }
$kept = foreach ($k in $paths) {
    if ($k -eq 'VICTORY_CACHE_DIR' -and $cache) { $k + '=' + $cache }
    elseif (Test-Path "Env:$k") { $k + '=' + (Get-Item "Env:$k").Value }
}
$line = '"' + $Victory + '" run "' + $Exe + '" --console ' + $Console + ' --execute'
$variant = ''
if ($LowMemory) { $variant = ' (low memory)' }
Write-Host ('route ' + $Route + $variant + ', console ' + $Console)
Write-Host 'settings:'
@($kit) + @($kept) | Sort-Object | ForEach-Object { Write-Host ('  ' + $_) }
Write-Host ('command (in ' + $root + '): ' + $line)
if ($DryRun) { exit 0 }

if (-not (Test-Path $Victory)) { Write-Host "No $Victory. Build it first (README.md)."; exit 2 }
if (-not (Test-Path $Exe)) { Write-Host "No $Exe."; exit 2 }
Get-ChildItem Env:VICTORY_* | Where-Object { $_.Name -notin $paths } | ForEach-Object { Remove-Item "Env:$($_.Name)" }
foreach ($k in $settings.Keys) { Set-Item "Env:$k" $settings[$k] }
if ($cache) { $env:VICTORY_CACHE_DIR = $cache }
$Exe = (Resolve-Path $Exe).Path
Push-Location $root   # victory.toml is looked for here
try { & $Victory run $Exe --console $Console --execute } finally { Pop-Location }
exit $LASTEXITCODE
