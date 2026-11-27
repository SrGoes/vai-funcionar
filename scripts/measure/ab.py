r"""ab.py RUN_A RUN_B OUT_DIR OFFSETS [--mask x0,y0,x1,y1]...   (offsets: frames after the world's first frame, e.g. 304,400,600)
Per offset: takes A's frame, picks the most similar frame of B within +-40 frames (same view), writes
heat map, side-by-side and crops of the 4 worst 96x96 blocks; prints metrics as JSON. Reads PNGs only."""
import sys, re, json, glob, os
import numpy as np
from PIL import Image, ImageFilter
ANSI = re.compile(r"\x1b\[[\d;]*m")
def world(run):
    for l in open(os.path.join(run, "err.log"), encoding="utf8", errors="replace"):
        m = re.search(r"first frame of 500 draws or more.*?frame=(\d+)", ANSI.sub("", l))
        if m: return int(m[1])
    raise SystemExit("no world frame in " + run)
def frames(run, w):
    return {int(re.search(r"frame-(\d+)\.png", p)[1]) - w: p for p in glob.glob(os.path.join(run, "cap", "frame-*.png")) if re.search(r"frame-(\d+)\.png$", p)}
a_run, b_run, out = sys.argv[1:4]; offs = [int(x) for x in sys.argv[4].split(",")]
masks = [tuple(int(v) for v in sys.argv[i + 1].split(",")) for i, s in enumerate(sys.argv) if s == "--mask"]
os.makedirs(out, exist_ok=True)
wa, wb = world(a_run), world(b_run); fa, fb = frames(a_run, wa), frames(b_run, wb)
small = lambda im: np.asarray(im.convert("L").resize((96, 54)), dtype=float)
res = []
for o in offs:
    if o not in fa: res.append({"offset": o, "error": "A has no such frame"}); continue
    A = Image.open(fa[o]).convert("RGB"); sa = small(A)
    cands = [(np.abs(small(Image.open(fb[k])) - sa).mean(), k) for k in fb if abs(k - o) <= 40]
    if not cands: res.append({"offset": o, "error": "B has no frame near"}); continue
    sim, k = min(cands); B = Image.open(fb[k]).convert("RGB")
    d = np.abs(np.asarray(A.filter(ImageFilter.GaussianBlur(2)), float) - np.asarray(B.filter(ImageFilter.GaussianBlur(2)), float)).max(axis=2)
    for x0, y0, x1, y1 in masks: d[y0:y1, x0:x1] = 0
    blocks = sorted(((d[y:y + 96, x:x + 96].mean(), x, y) for y in range(0, d.shape[0] - 95, 96) for x in range(0, d.shape[1] - 95, 96)), reverse=True)
    heat = Image.fromarray(np.clip(d * 4, 0, 255).astype(np.uint8)).convert("RGB")
    W, H = A.size; sheet = Image.new("RGB", (W * 3, H)); [sheet.paste(im, (i * W, 0)) for i, im in enumerate((A, B, heat))]
    sheet.save(f"{out}/o{o}-sidebyside.png")
    for n, (m, x, y) in enumerate(blocks[:4]):
        c = Image.new("RGB", (288 * 3, 288)); [c.paste(im.crop((x, y, x + 96, y + 96)).resize((288, 288), Image.NEAREST), (i * 288, 0)) for i, im in enumerate((A, B, heat))]
        c.save(f"{out}/o{o}-block{n}-x{x}y{y}.png")
    res.append({"offset": o, "b_frame_offset": k, "view_match_L1": round(sim, 2), "mean_abs": round(float(np.abs(np.asarray(A, float) - np.asarray(B, float)).mean()), 2),
                "pct_blurred_diff_gt32": round(float((d > 32).mean() * 100), 2), "p99": round(float(np.percentile(d, 99)), 1), "worst_blocks": [(round(float(m), 1), x, y) for m, x, y in blocks[:4]]})
json.dump(res, open(f"{out}/metrics.json", "w"), indent=1); print(json.dumps(res, indent=1))
