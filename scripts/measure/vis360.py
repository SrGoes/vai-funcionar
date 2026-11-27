r"""Roteiro de imagem: mundo + 20 s parado, rajada parada, giro de 360 graus, rajada parada.

run:      vis360.py run NAME ROUTE BUILD VARS [END_S_AFTER_WORLD=60]   (VIS_VICTORY=<exe> para outro victory.exe)
          Abre pelo scripts\play.ps1 na ROUTE (nvidia|amd), Series S, injeta só o Enter aos 10 s mais o que VARS trouxer
          (o giro e as capturas vêm em VARS: VICTORY_KEYS extra, VICTORY_CAPTURE_EVERY_MS, rajadas), espera
          END_S_AFTER_WORLD segundos depois do primeiro quadro do mundo e fecha. O Fechar da janela fica cinza.
compare:  vis360.py compare RUN_BASE RUN_CAND OUT [--giro A-B] [--mask x0,y0,x1,y1]
          Casa as capturas pelo TEMPO desde o primeiro quadro do mundo (linhas "frame saved" do log), não pelo
          número do quadro: em builds com FPS diferente o mesmo instante do giro cai no mesmo ângulo.
          Saída: OUT/contato-NN.jpg (base | candidato | diferença) por ângulo, OUT/flicker-*.png, OUT/resumo.json
          com o pisca de cada rajada (quadros de números seguidos), a diferença por ângulo e os draws descartados
          por motivo de cada rodada. Nada sai da máquina.
"""
import json, os, re, sys, tempfile, time
from datetime import datetime
from pathlib import Path
import numpy as np
from PIL import Image
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import bench

ANSI = re.compile(r"\x1b\[[\d;]*m")
STAMP = re.compile(r"^(\d{4}-\d\d-\d\dT\d\d:\d\d:\d\d\.\d+)Z ")


def world_time(lines):
    for l in lines:
        if "the first frame of 500 draws or more" in l:
            return datetime.fromisoformat(STAMP.match(l)[1]), int(re.search(r"frame=(\d+)", l)[1])
    return None, None

def captures(run):
    """[(seconds after world, frame number, path)] in order."""
    lines = bench.log_lines(run); t0, f0 = world_time(lines)
    out = []
    for l in lines:
        m = re.search(r"frame saved path=(\S*?frame-(\d+)\.png)", l)
        if m and t0 and int(m[2]) > f0:
            w = re.search(r"world_ms=(?:Some\()?(\d+)", l)   # the capture commit's own clock, when present
            t = int(w[1]) / 1000 if w else (datetime.fromisoformat(STAMP.match(l)[1]) - t0).total_seconds()
            out.append((t, int(m[2]), m[1].strip()))
    return out

def dropped(run):
    return bench.dropped_by_reason(bench.log_lines(run))

def bursts(caps, least=6):
    """Runs of consecutive frame numbers (a burst taken without touching the camera)."""
    runs, cur = [], []
    for c in caps:
        if cur and c[1] == cur[-1][1] + 1: cur.append(c)
        else:
            if len(cur) >= least: runs.append(cur)
            cur = [c]
    if len(cur) >= least: runs.append(cur)
    return runs

def flicker(burst, B=16, T=24.0, R=0.35):
    """Blocks that change from both neighbours while the neighbours agree: a thing shown in one frame only.
    B: block side in pixels of the half-size capture; T: least mean change (of 255) from each neighbour;
    R: the neighbours must differ from each other by less than R times that change."""
    bl = []
    for _, _, p in burst:
        a = np.asarray(Image.open(p).convert("RGB"), np.float32); h, w, _ = a.shape
        bl.append(a[:h//B*B, :w//B*B].reshape(h//B, B, w//B, B, 3).mean(axis=(1, 3)))
    n, frames, worst = 0, 0, []
    for k in range(1, len(bl) - 1):
        dp = np.abs(bl[k]-bl[k-1]).mean(2); dn = np.abs(bl[k]-bl[k+1]).mean(2); ds = np.abs(bl[k+1]-bl[k-1]).mean(2)
        m = (dp > T) & (dn > T) & (ds < R*np.minimum(dp, dn)); c = int(m.sum()); n += c; frames += bool(c)
        if c: ys, xs = np.nonzero(m); worst.append((c, burst[k][1], int(xs.min()*B), int(ys.min()*B), int((xs.max()+1)*B), int((ys.max()+1)*B)))
    return {"frames": len(burst), "frames_with_flicker": frames, "flicker_blocks": n, "worst": sorted(worst, reverse=True)[:3]}

def run_cmd(name, route, build, extra, end_s=60):
    out = Path(tempfile.gettempdir()) / f"victory-vis360-{time.strftime('%Y%m%d-%H%M%S')}-{name}"
    (out / "cap").mkdir(parents=True)
    keys = "10:Enter"
    m = re.search(r"VICTORY_KEYS=([^;]*)", extra)
    if m:                                  # the turn's keys join the Enter
        keys += "," + m[1]
        extra = ";".join(p for p in extra.split(";") if p and not p.startswith("VICTORY_KEYS="))
    vars_ = ";".join(v for v in ("VICTORY_LOG=info", extra, f"VICTORY_KEYS={keys}", f"VICTORY_CAPTURE={out / 'cap'}", "VICTORY_CAPTURE_WORLD=1") if v)
    if bench.windows()[0]: raise SystemExit("victory.exe already running")
    if bench.busy(): raise SystemExit(f"a build is running: {bench.busy()}")
    bench.launch(out, route, build, "series-s", False, vars_, os.environ.get("VIS_VICTORY"))
    start, why, t_world = time.time(), "timeout", None
    while time.time() - start < 420 + end_s:
        time.sleep(2); pids, _ = bench.read(True)
        if not pids and time.time() - start > 15: why = "ended"; break
        if t_world is None and world_time(bench.log_lines(out))[0]: t_world = time.time()
        if t_world and time.time() - t_world >= end_s: why = "done"; break
    bench.close_ours()
    r = {"name": name, "run": str(out), "why": why, "captures": len(captures(out)), "vars": vars_}
    (out / "vis360.json").write_text(json.dumps(r, indent=1)); print(json.dumps(r))

def compare(base, cand, out, giro=None, masks=()):
    os.makedirs(out, exist_ok=True)
    cb, cc = captures(base), captures(cand)
    res = {"base": base, "cand": cand, "dropped_base": dropped(base), "dropped_cand": dropped(cand)}
    for tag, caps in (("base", cb), ("cand", cc)):
        res[f"bursts_{tag}"] = [dict(start_s=round(b[0][0], 1), **flicker(b)) for b in bursts(caps)]
    lo, hi = (float(x) for x in giro.split("-")) if giro else (0, 1e9)
    gb = [c for c in cb if lo <= c[0] <= hi]; angles = []
    for i, (t, f, p) in enumerate(gb):
        near = min(cc, key=lambda c: abs(c[0] - t)) if cc else None
        if not near or abs(near[0] - t) > 0.6: continue
        A = Image.open(p).convert("RGB"); Bm = Image.open(near[2]).convert("RGB").resize(A.size)
        d = np.abs(np.asarray(A, float) - np.asarray(Bm, float)).max(2)
        for x0, y0, x1, y1 in masks: d[y0:y1, x0:x1] = 0
        heat = Image.fromarray(np.clip(d*4, 0, 255).astype(np.uint8)).convert("RGB")
        W, H = A.size; sheet = Image.new("RGB", (W*3, H))
        for j, im in enumerate((A, Bm, heat)): sheet.paste(im, (j*W, 0))
        sheet.save(f"{out}/contato-{i:02d}.jpg", quality=85)
        angles.append({"i": i, "t_base": round(t, 2), "t_cand": round(near[0], 2), "pct_diff_gt32": round(float((d > 32).mean()*100), 2)})
    res["angles"] = angles
    json.dump(res, open(f"{out}/resumo.json", "w"), indent=1); print(json.dumps(res, indent=1))

if __name__ == "__main__":
    if sys.argv[1] == "run":
        run_cmd(sys.argv[2], sys.argv[3], sys.argv[4], sys.argv[5], int(sys.argv[6]) if len(sys.argv) > 6 else 60)
    else:
        giro = sys.argv[sys.argv.index("--giro") + 1] if "--giro" in sys.argv else None
        masks = [tuple(int(v) for v in sys.argv[i+1].split(",")) for i, s in enumerate(sys.argv) if s == "--mask"]
        compare(sys.argv[2], sys.argv[3], sys.argv[4], giro, masks)
