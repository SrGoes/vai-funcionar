r"""A matrix: several configurations measured at the same place, REPS valid runs each, in rotated order.

    python scripts/measure/matrix.py CONFIG.json [OUT_DIR]

CONFIG.json (see matrix-example.json):
  route, build, console, safe, common, reps (3), retries (2), settle_s (20), sample_s (75), warmup (true), between_s (10)
  configs: [{name, vars, route?, build?, console?, safe?, victory?, warm_key?}]
One discarded warm-up run per distinct (route, console, build, victory, warm_key) first, so the caches are warm; give a
config that changes the translation (VICTORY_WAVE_WIDE, for one) a warm_key of its own. An invalid run (closed by hand, the
title ended, no world) is repeated at most "retries" times per cell. Stops if a build is running or a victory.exe
it did not start is open. Writes runs.csv (one line per run) and summary.csv (median of the valid runs per cell).
"""
import csv, json, statistics, sys, tempfile, time
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))
import bench

cfg = json.loads(Path(sys.argv[1]).read_text(encoding="utf8"))
out = Path(sys.argv[2]) if len(sys.argv) > 2 else Path(tempfile.gettempdir()) / f"victory-matrix-{time.strftime('%Y%m%d-%H%M%S')}"
out.mkdir(parents=True, exist_ok=True)
reps, retries = cfg.get("reps", 3), cfg.get("retries", 2)
COLS = ["config", "rep", "kind", "valid", "why", "world_s", "samples", "fps_median", "fps_from_ms_median", "ms_median",
        "ms_p95_of_title_samples", "draws_median", "split_preparing_ms", "split_textures_ms", "split_gpu_ms", "split_copyback_ms",
        "card_busy_pct_median", "shared_buffers_per_s", "dropped_no_pipeline_median", "dropped_by_reason_sum_of_lines",
        "private_mb_peak", "ram_free_mb_min", "host_memory_pressure_lines", "vram_dedicated_in_use_mb_peak",
        "vram_shared_in_use_mb_peak", "shader_cache_hits", "shader_cache_misses", "async_submit_on", "async_submit_refused",
        "card_buffers_min_kb", "dynamic_tables_asked_median", "dynamic_tables_refused_max", "pipelines_still_compiling_in_sample",
        "views_asked_per_frame", "views_copies_in_new_buffers_per_frame", "fps_min", "fps_max", "device_removed",
        "panics", "views_line", "vars", "run"]
SUMMED = ["fps_from_ms_median", "ms_median", "ms_p95_of_title_samples", "draws_median", "split_preparing_ms", "split_textures_ms",
          "split_gpu_ms", "split_copyback_ms", "card_busy_pct_median", "shared_buffers_per_s", "dropped_no_pipeline_median",
          "private_mb_peak", "ram_free_mb_min", "vram_dedicated_in_use_mb_peak", "vram_shared_in_use_mb_peak",
          "views_asked_per_frame", "views_copies_in_new_buffers_per_frame", "dynamic_tables_asked_median"]

rows = []
def save():
    with open(out / "runs.csv", "w", newline="", encoding="utf8") as f:
        w = csv.DictWriter(f, COLS, extrasaction="ignore"); w.writeheader(); w.writerows(rows)
    with open(out / "summary.csv", "w", newline="", encoding="utf8") as f:
        w = csv.writer(f); w.writerow(["config", "valid_runs", "invalid_runs"] + SUMMED)
        for c in cfg["configs"]:
            mine = [r for r in rows if r["config"] == c["name"] and r["kind"] == "measure"]
            good = [r for r in mine if r["valid"]]
            w.writerow([c["name"], len(good), len(mine) - len(good)] +
                       [statistics.median(v) if (v := [r[k] for r in good if isinstance(r.get(k), (int, float))]) else "" for k in SUMMED])

def setting(c, key, default):
    return c.get(key, cfg.get(key, default))

def go(c, rep, kind):
    if bench.busy(): raise SystemExit(f"stopped: {bench.busy()} is running (nothing is measured beside a build)")
    extra = ";".join(v for v in (cfg.get("common", ""), c.get("vars", "")) if v)
    r = bench.one(f"{c['name']}-{kind}{rep}", setting(c, "route", "nvidia"), setting(c, "build", "diag"), setting(c, "console", "series-s"),
                  setting(c, "safe", False), extra, c.get("victory"), settle=cfg.get("settle_s", 20), sample=cfg.get("sample_s", 75), out=out)
    r |= {"config": c["name"], "rep": rep, "kind": kind}
    rows.append(r); save()
    print(f"== {c['name']} {kind} {rep}: valid={r['valid']} {r['why']} fps={r.get('fps_from_ms_median')} ms={r.get('ms_median')}", flush=True)
    if r["why"] == "victory.exe already running": raise SystemExit("stopped: a victory.exe this script did not start is open")
    time.sleep(cfg.get("between_s", 10))
    return r

try:
    seen = set()
    for c in cfg["configs"]:
        key = (setting(c, "route", "nvidia"), setting(c, "console", "series-s"), setting(c, "build", "diag"), c.get("victory"), c.get("warm_key"))
        if key not in seen and cfg.get("warmup", True):
            seen.add(key); go(c, 0, "warmup")
    order = list(cfg["configs"])
    for rep in range(1, reps + 1):
        for c in order:
            for _ in range(1 + retries):
                if go(c, rep, "measure")["valid"]: break
        order = order[1:] + order[:1]
finally:
    bench.close_ours()   # only a title this matrix started (an interrupted run); never one that was open before
    save()
print("done", out)
