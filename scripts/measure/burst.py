r"""burst.py run RUN OUT [--skip 2] [--window 40]   |   burst.py sheet OUT N RUN...
run: finds the bursts (>= 8 consecutive captured frames) in RUN\cap, and per burst prints JSON with the share of 16px blocks
that flicker (change in one frame and come back in the next) and the share of pixels that boil (std x share of sign flips > 3,
over WINDOW frames after the first SKIP). Writes next to OUT: -mundo.png (world sheet), -pisca<i>.png (flicker heat map),
-seq<i>.png (3 consecutive frames), -rajada<i>.gif, -ferve<i>.png (mean | boil heat map).
sheet: contact sheet, one row per run, N frames spread by capture time. Needs pillow and numpy; reads PNGs only."""
import sys, os, json, argparse
import numpy as np
from PIL import Image, ImageDraw

B = 16        # block size for the flicker test
MIN_BURST = 8
MIN_BYTES = 100000  # smaller PNGs are black/loading frames

def load(cap, n): return Image.open(os.path.join(cap, "frame-%d.png" % n)).convert("RGB")

def find_bursts(fs):
    bursts, cur = [], [fs[0]]
    for a, b in zip(fs, fs[1:]):
        if b == a + 1: cur.append(b)
        else:
            if len(cur) >= MIN_BURST: bursts.append(cur)
            cur = [b]
    if len(cur) >= MIN_BURST: bursts.append(cur)
    return bursts

def flicker(arr):
    """Share of blocks that flicker; arr = RGB int16 frames."""
    H, W, _ = arr[0].shape
    hb, wb = H // B, W // B
    def blocks(x): return np.abs(x).mean(axis=2)[:hb * B, :wb * B].reshape(hb, B, wb, B).mean(axis=(1, 3))
    flick = np.zeros((hb, wb)); change = np.zeros((hb, wb))
    for i in range(1, len(arr) - 1):
        d_prev = blocks(arr[i] - arr[i - 1]); d_skip = blocks(arr[i + 1] - arr[i - 1])
        flick += ((d_prev > 18) & (d_skip < d_prev * 0.5)).astype(float)
        change += (d_prev > 18).astype(float)
    n = max(1, len(arr) - 2)
    return flick, change, n

def boil(gray):
    """Per-pixel boil = std over time x share of sign flips of the frame-to-frame change (boiling, not motion)."""
    d = np.sign(np.diff(gray, axis=0))
    return gray.std(axis=0) * (d[1:] * d[:-1] < 0).mean(axis=0)

def run(a):
    cap = os.path.join(a.run, "cap")
    fs = sorted(int(f[6:-4]) for f in os.listdir(cap) if f.startswith("frame-") and f.endswith(".png") and os.path.getsize(os.path.join(cap, f)) > MIN_BYTES)
    if not fs: raise SystemExit("no frames in " + cap)
    bursts = find_bursts(fs)
    inb = set(n for b in bursts for n in b)
    single = [n for n in fs if n not in inb]
    info = {"frames": len(fs), "bursts": [[b[0], b[-1], len(b)] for b in bursts], "singles": len(single)}
    w = single[int(len(single) * 0.3):]  # world sheet: 8 singles over the last 70%
    pick = [w[int(i * (len(w) - 1) / 7)] for i in range(8)] if len(w) >= 8 else w
    if pick:
        im = load(cap, pick[0]); cw, ch = im.size
        sheet = Image.new("RGB", (cw * 4, ch * 2))
        for i, n in enumerate(pick): sheet.paste(load(cap, n), ((i % 4) * cw, (i // 4) * ch))
        sheet.save(a.out + "-mundo.png"); info["sheet_frames"] = pick
    for bi, b in enumerate(bursts):
        arr = [np.asarray(load(cap, n)).astype(np.int16) for n in b]
        flick, change, n = flicker(arr)
        win = arr[a.skip:a.skip + a.window]
        res = {"first": b[0], "frames": len(b), "pct_blocks_flick_any": round(float((flick > 0).mean() * 100), 2),
               "pct_blocks_flick_often": round(float((flick >= n * 0.25).mean() * 100), 2),
               "pct_blocks_changing_often": round(float((change >= n * 0.5).mean() * 100), 2)}
        if len(win) >= 4:  # boil needs at least 3 diffs to have sign flips
            gray = np.stack([np.asarray(Image.fromarray(np.clip(f, 0, 255).astype(np.uint8)).convert("L")) for f in win]).astype(np.float32)
            bo = boil(gray); mean = gray.mean(axis=0)
            res.update({"boil_first": b[a.skip], "boil_frames": len(win), "pct_px_boil_gt3": round(float((bo > 3).mean() * 100), 2), "boil_mean": round(float(bo.mean()), 2)})
            vis = np.clip(bo * 12, 0, 255).astype(np.uint8)
            heat = np.zeros(mean.shape + (3,), np.uint8); heat[..., 0] = vis; heat[..., 1] = heat[..., 2] = (mean * 0.45).astype(np.uint8)
            side = Image.new("RGB", (mean.shape[1] * 2, mean.shape[0]))
            side.paste(Image.fromarray(mean.astype(np.uint8)).convert("RGB"), (0, 0)); side.paste(Image.fromarray(heat), (mean.shape[1], 0))
            side.save(a.out + "-ferve%d.png" % bi)
        info["burst%d" % bi] = res
        hb, wb = flick.shape
        base = Image.fromarray(np.clip(np.mean(arr, axis=0), 0, 255).astype(np.uint8))
        heat = Image.new("RGBA", base.size, (0, 0, 0, 0)); d = ImageDraw.Draw(heat)
        for y in range(hb):
            for x in range(wb):
                v = flick[y, x] / n
                if v > 0.05: d.rectangle([x * B, y * B, x * B + B - 1, y * B + B - 1], fill=(255, 0, 0, int(min(1.0, v * 2.5) * 190)))
        Image.alpha_composite(base.convert("RGBA"), heat).convert("RGB").save(a.out + "-pisca%d.png" % bi)
        m = len(arr) // 2; w0, h0 = arr[0].shape[1], arr[0].shape[0]
        tri = Image.new("RGB", (w0 * 3, h0))
        for k in range(3): tri.paste(Image.fromarray(np.clip(arr[m - 1 + k], 0, 255).astype(np.uint8)), (k * w0, 0))
        tri.save(a.out + "-seq%d.png" % bi)
        gif = [Image.fromarray(np.clip(f, 0, 255).astype(np.uint8)).resize((640, 360)) for f in arr]
        gif[0].save(a.out + "-rajada%d.gif" % bi, save_all=True, append_images=gif[1:], duration=120, loop=0)
    print(json.dumps(info))

def sheet(out, n, runs):
    W, H = 480, 270
    img = Image.new("RGB", (W * n, (H + 20) * len(runs)), "black"); d = ImageDraw.Draw(img)
    for row, r in enumerate(runs):
        cap = os.path.join(r, "cap")
        fs = sorted((f for f in os.listdir(cap) if f.endswith(".png") and os.path.getsize(os.path.join(cap, f)) > MIN_BYTES), key=lambda f: os.path.getmtime(os.path.join(cap, f)))
        pick = [fs[int(i * (len(fs) - 1) / max(1, n - 1))] for i in range(n)] if fs else []
        for col, f in enumerate(pick): img.paste(Image.open(os.path.join(cap, f)).convert("RGB").resize((W, H)), (col * W, row * (H + 20) + 20))
        d.text((4, row * (H + 20) + 4), os.path.basename(r) + "  " + " ".join(pick), fill="yellow")
    img.save(out)

if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "sheet":
        sheet(sys.argv[2], int(sys.argv[3]), sys.argv[4:])
    elif len(sys.argv) > 1 and sys.argv[1] == "run":
        p = argparse.ArgumentParser(); p.add_argument("cmd"); p.add_argument("run"); p.add_argument("out")
        p.add_argument("--skip", type=int, default=2); p.add_argument("--window", type=int, default=40)
        run(p.parse_args())
    else: raise SystemExit(__doc__)
