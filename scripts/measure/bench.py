r"""One unattended measuring run: start the title through scripts/play.ps1, wait for the world, sample the
window title for SAMPLE seconds, read the log of the sample window, close.

    python scripts/measure/bench.py --name N --route nvidia [--build diag] [--console series-s] [--safe]
                                    [--vars "VICTORY_A=1;VICTORY_B=2"] [--victory PATH] [--settle 20] [--sample 75]
    python scripts/measure/bench.py --parse RUN_DIR START_UTC END_UTC     (read a finished run's log again)

The title is VICTORY_GAME_DIR\<VICTORY_GAME_EXE, default game_scarlett_master.exe>, as Play-VIctory.cmd starts it.
A run is valid only if the title lived through the whole sample; a close by hand (close button, Alt+F4) or an early
end is reported as the reason, never as a result. The window's close command is greyed during the run.
Input: one Enter at 10 s (the account picker) and one at 14 s (falls on the intro); nothing in the world, where Enter
is the A button and moves the player. The world arrives about 37 s after the first Enter.
Output: <out>/victory-run-<time>-<name>/ with out.log, err.log, cap/ (captures) and result.json.
Needs Windows, Python 3.9+, a build with diagnostics for the log figures (release builds give the title's only).
"""
import argparse, ctypes, json, math, os, re, statistics, subprocess, sys, tempfile, time
from ctypes import wintypes
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
KEYS = os.environ.get("VICTORY_BENCH_KEYS", "10:Enter,14:Enter")
WORLD_ALSO = os.environ.get("VICTORY_BENCH_WORLD_ALSO", "264,304,360,400")  # frames for ab.py
ANSI = re.compile(r"\x1b\[[\d;]*m")
STAMP = re.compile(r"^(\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d\.\d+)Z ")
SHAPE = re.compile(r"([\d.]+) fps ([\d.]+) ms/frame (\d+) draws/frame")
user32 = ctypes.windll.user32
user32.GetSystemMenu.restype = wintypes.HMENU
user32.GetSystemMenu.argtypes = [wintypes.HWND, wintypes.BOOL]
user32.EnableMenuItem.argtypes = [wintypes.HMENU, wintypes.UINT, wintypes.UINT]
SC_CLOSE, MF_GRAYED = 0xF060, 0x1


def windows():
    """(pids of victory.exe, [(hwnd, title)] of its windows whose title carries the figures)."""
    out = subprocess.run('tasklist /FI "IMAGENAME eq victory.exe" /NH /FO CSV', capture_output=True, text=True, shell=True).stdout
    pids = {int(l.split('","')[1]) for l in out.splitlines() if l.lower().startswith('"victory.exe"')}
    found = []
    def each(hwnd, _):
        pid = wintypes.DWORD()
        user32.GetWindowThreadProcessId(hwnd, ctypes.byref(pid))
        if pid.value in pids:
            buf = ctypes.create_unicode_buffer(300)
            user32.GetWindowTextW(hwnd, buf, 300)
            if "fps" in buf.value:
                found.append((hwnd, buf.value))
        return True
    user32.EnumWindows(ctypes.WINFUNCTYPE(ctypes.c_bool, wintypes.HWND, wintypes.LPARAM)(each), 0)
    return pids, found


def read(lock_close=True):
    pids, found = windows()
    if found and lock_close:
        user32.EnableMenuItem(user32.GetSystemMenu(found[0][0], False), SC_CLOSE, MF_GRAYED)
    m = SHAPE.search(found[0][1]) if found else None
    return pids, (float(m[1]), float(m[2]), int(m[3])) if m else None


def busy(names=("cargo.exe", "rustc.exe")):
    """Builds running now: a build takes processor and memory from the title, so nothing is measured beside one."""
    out = subprocess.run("tasklist /NH /FO CSV", capture_output=True, text=True, shell=True).stdout.lower()
    return [n for n in names if f'"{n}"' in out]


def p95(values):
    s = sorted(values)
    return s[max(0, math.ceil(len(s) * 0.95) - 1)]


def launch(run, route, build, console, safe, vars_, victory=None):
    """Starts scripts/play.ps1 with its output in run/out.log and run/err.log."""
    game = Path(os.environ["VICTORY_GAME_DIR"]) / os.environ.get("VICTORY_GAME_EXE", "game_scarlett_master.exe")
    args = ["powershell.exe", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", str(ROOT / "scripts" / "play.ps1"),
            "-Exe", str(game), "-Route", route, "-Build", build, "-Console", console, "-Vars", vars_]
    if safe: args.append("-Safe")
    if victory: args += ["-Victory", victory]
    (run / "env.txt").write_text(" ".join(args[5:]))
    with open(run / "out.log", "wb") as o, open(run / "err.log", "wb") as e:
        subprocess.Popen(args, cwd=ROOT, stdout=o, stderr=e, stdin=subprocess.DEVNULL, creationflags=subprocess.CREATE_NEW_PROCESS_GROUP)


def log_lines(run):
    text = ""
    for name in ("err.log", "out.log"):
        p = Path(run) / name
        if p.exists(): text += p.read_text(encoding="utf8", errors="replace")
    return ANSI.sub("", text).splitlines()


def parse_log(run, start, end):
    """Figures of the log; those per frame or cumulative are taken from lines stamped in [start, end]."""
    log = log_lines(run)
    def stamped(key, inside=True):
        hits = []
        for l in log:
            if key in l:
                m = STAMP.match(l)
                t = m and datetime.fromisoformat(m[1]).replace(tzinfo=timezone.utc)
                if not inside or (t and start <= t <= end):
                    hits.append((t, l))
        return hits
    def nums(line, shape):
        m = re.search(shape, line)
        return m.groups() if m else None
    r = {}
    made = [(t, int(n[0])) for t, l in stamped("made so far:") if (n := nums(l, r"(\d+) shared buffers"))]
    if len(made) >= 2:
        span = (made[-1][0] - made[0][0]).total_seconds()
        r["shared_buffers_made"] = made[-1][1] - made[0][1]
        r["shared_buffers_per_s"] = round(r["shared_buffers_made"] / span, 1) if span else None
    card = [int(n[0]) for _, l in stamped("the card's clock per frame") if (n := nums(l, r"the card about (\d+)% busy"))]
    if card: r["card_busy_pct_median"] = statistics.median(card)
    split = [n for _, l in stamped(" fps, ") if (n := nums(l, r"\(([\d.]+) preparing.*?textures ([\d.]+) ms.*?; ([\d.]+) on the GPU; ([\d.]+) copying back"))]
    if split:
        for i, k in enumerate(("split_preparing_ms", "split_textures_ms", "split_gpu_ms", "split_copyback_ms")):
            r[k] = statistics.median(float(s[i]) for s in split)
    frames = [n for _, l in stamped(" fps, ") if (n := nums(l, r"([\d.]+) fps, ([\d.]+) ms/frame"))]
    if frames:
        r["log_fps_median"] = statistics.median(float(n[0]) for n in frames)
        r["log_ms_median"] = statistics.median(float(n[1]) for n in frames)
    dropped = [int(n[0]) for _, l in stamped(" fps, ") if (n := nums(l, r"dropped (\d+) without a host pipeline"))]
    if dropped: r["dropped_no_pipeline_median"] = statistics.median(dropped)
    reasons = {}
    for _, l in stamped("/frame without a host pipeline: "):
        m = re.search(r"dropped ([\d.]+)/frame without a host pipeline: (.+?)(?: [0-9a-f]{16}|\s*\[|,|$)", l)
        if m: reasons[m[2]] = round(reasons.get(m[2], 0) + float(m[1]), 1)
    r["dropped_by_reason_sum_of_lines"] = reasons
    views = stamped("buffer views off their copy's elements")
    if views: r["views_line"] = views[-1][1].split(": ", 1)[-1]
    vram = [n for _, l in stamped("video memory as the system counts it") if (n := nums(l, r"the card's own: budget \d+ MB, in use (\d+) MB.*?shared with the card: budget \d+ MB, in use (\d+) MB"))]
    if vram:
        r["vram_dedicated_in_use_mb_peak"] = max(int(v[0]) for v in vram)
        r["vram_shared_in_use_mb_peak"] = max(int(v[1]) for v in vram)
    mem = [n for _, l in stamped("copystat: memory: ") if (n := nums(l, r"memory: (\d+) MB private.*?the PC has (\d+) MB free"))]
    if mem:
        r["private_mb_peak"] = max(int(m[0]) for m in mem)
        r["ram_free_mb_min"] = min(int(m[1]) for m in mem)
    r["host_memory_pressure_lines"] = len(stamped("host memory pressure"))
    hm = [n for _, l in stamped("shaders from the cache and translated so far", inside=False) if (n := nums(l, r"hits=(\d+).*?misses=(\d+)"))]
    if hm: r["shader_cache_hits"], r["shader_cache_misses"] = int(hm[-1][0]), int(hm[-1][1])
    r["async_submit_on"] = any("submissions are prepared and sent by a thread of ours" in l for l in log)
    r["device_removed"] = any("device removed" in l.lower() or "DXGI_ERROR_DEVICE" in l for l in log)
    r["panics"] = sum("panicked at" in l for l in log)
    r["window_closed"] = any("window closed: suspending the title" in l for l in log)
    return r


def one(name, route, build="diag", console="series-s", safe=False, extra="", victory=None,
        world_timeout=420, settle=20, sample=75, out=None, keep=False):
    run = Path(out or tempfile.gettempdir()) / f"victory-run-{time.strftime('%Y%m%d-%H%M%S')}-{name}"
    (run / "cap").mkdir(parents=True)
    # VICTORY_LOG=info: the nvidia route sets warn, which hides the figures read below.
    vars_ = ";".join(v for v in ("VICTORY_LOG=info", extra, f"VICTORY_KEYS={KEYS}", f"VICTORY_CAPTURE={run / 'cap'}",
                                 "VICTORY_CAPTURE_WORLD=1", f"VICTORY_CAPTURE_WORLD_ALSO={WORLD_ALSO}" if WORLD_ALSO else "") if v)
    result = {"name": name, "run": str(run), "route": route, "build": build, "console": console, "safe": safe,
              "vars": extra, "valid": False, "why": ""}
    def done(why=""):
        result["why"], result["valid"] = why, not why
        (run / "result.json").write_text(json.dumps(result, indent=1))
        if not keep:
            subprocess.run("taskkill /IM victory.exe /F", shell=True, capture_output=True)
            for _ in range(20):
                if not windows()[0]: break
                time.sleep(0.5)
        return result
    if windows()[0]: return {**result, "why": "victory.exe already running"}
    if busy(): return {**result, "why": f"a build is running: {busy()}"}
    launch(run, route, build, console, safe, vars_, victory)
    print("run", run, flush=True)
    started, world_at, last = time.time(), None, None
    while time.time() - started < world_timeout:
        time.sleep(3)
        pids, now = read()
        if not pids and time.time() - started > 15: return done("the title ended before the world")
        if now:
            last = now
            if now[2] >= 1500: world_at = time.time(); break
    if not world_at: return done(f"no world in {time.time() - started:.0f}s; last title {last}")
    result["world_s"] = round(world_at - started)
    time.sleep(settle)
    rows, ended = [], False
    t0 = datetime.now(timezone.utc); begin = time.time()
    while time.time() - begin < sample:
        pids, now = read()
        if not pids: ended = True; break
        if now: rows.append(now)
        time.sleep(2)
    t1 = datetime.now(timezone.utc)
    alive = bool(windows()[0])
    result |= {"sample_start_utc": t0.isoformat(), "sample_end_utc": t1.isoformat(), "samples": len(rows)}
    if rows:
        ms = [r[1] for r in rows]
        result |= {"fps_median": statistics.median(r[0] for r in rows), "fps_from_ms_median": round(1000 / statistics.median(ms), 2),
                   "ms_median": statistics.median(ms), "ms_p95_of_title_samples": p95(ms),
                   "draws_median": statistics.median(r[2] for r in rows)}
    time.sleep(1)
    result |= parse_log(run, t0, t1)
    why = ""
    if result["window_closed"]: why = "window closed by hand during the run"
    elif ended or not alive: why = "the title ended while sampling"
    elif len(rows) < sample / 2 * 0.8: why = f"only {len(rows)} title samples in {sample}s"
    elif min(r[2] for r in rows) < 1500: why = "left the world while sampling (a frame under 1500 draws)"
    return done(why)


if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "--parse":
        iso = lambda s: datetime.fromisoformat(s.replace("Z", "+00:00"))
        print(json.dumps(parse_log(Path(sys.argv[2]), iso(sys.argv[3]), iso(sys.argv[4])), indent=1)); sys.exit(0)
    a = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    a.add_argument("--name", required=True); a.add_argument("--route", required=True, choices=["nvidia", "amd"])
    a.add_argument("--build", default="diag", choices=["diag", "release"]); a.add_argument("--console", default="series-s")
    a.add_argument("--safe", action="store_true"); a.add_argument("--vars", default=""); a.add_argument("--victory")
    a.add_argument("--settle", type=int, default=20); a.add_argument("--sample", type=int, default=75)
    a.add_argument("--out"); a.add_argument("--keep", action="store_true")
    o = a.parse_args()
    r = one(o.name, o.route, o.build, o.console, o.safe, o.vars, o.victory, settle=o.settle, sample=o.sample, out=o.out, keep=o.keep)
    print(json.dumps(r, indent=1)); sys.exit(0 if r["valid"] else 2)
