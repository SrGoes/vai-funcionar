//! The F1 settings menu, from trustno1's fork (22c6be2, adapted). F1 shows and
//! hides it while our window has the focus; Up/Down pick a setting, Left/Right
//! change it, Enter runs the line picked (save, reset, close) or changes the
//! setting, Esc closes. While it is shown the title sees no keyboard
//! (`crate::SETTINGS_MENU_OPEN`, read by the input module); a pad still works.
//!
//! The settings are read once at start, so a change is saved to the file
//! `VICTORY_MENU_FILE` names (`scripts\play.ps1` sets it to
//! `victory-settings.txt` in the repository's folder and applies it over its
//! kit at every start) and takes effect at the next start, but for the two
//! marked LIVE (the frame rate at the top, the stats panel), which change at
//! once. Every line is a switch this build reads, but `VICTORY_MENU_CONSOLE`,
//! which is the launcher's own (`-Console`).
//!
//! In every build, as the stats panel is. Drawn as that panel is: white text
//! on a black box, copied onto the swap chain's buffer, in the middle of the
//! picture.
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use windows::Win32::Graphics::Direct3D12::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VK_DOWN, VK_ESCAPE, VK_F1, VK_LEFT, VK_RETURN, VK_RIGHT, VK_UP,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

use super::font::{colours, glyph};
use super::{Host, HostList, transition};

const GLYPH: (u32, u32) = (5, 7);
const CELL: (u32, u32) = (6, 9);
const COLUMNS: u32 = 60;
/// Lines of the box: a title, a help line, a blank, the settings shown, a
/// blank and a note.
const ROWS: usize = 12;
const LINES: u32 = ROWS as u32 + 5;
const RIM: u32 = 4;
const UPLOADS: usize = 4;

/// A setting: the variable, its name in the menu, the values it takes
/// (`""`: not set, the build's own default), whether a change applies at once,
/// and the other variables set with it (`(variable, value when this one is
/// set)`; they are unset when it is).
struct Item {
    var: &'static str,
    label: &'static str,
    values: &'static [(&'static str, &'static str)],
    live: bool,
    with: &'static [(&'static str, &'static str)],
}

const UNSET_ON: &[(&str, &str)] = &[("", "OFF"), ("1", "ON")];
const ON_UNSET: &[(&str, &str)] = &[("1", "ON"), ("", "OFF")];

/// The first value of each is what the launcher starts with (the nvidia kit).
#[rustfmt::skip]
const ITEMS: &[Item] = &[
    // The title's frame rate (`victory-loader/src/frame_rate.rs`): the interval in ms.
    Item { var: "VICTORY_FRAME_INTERVAL_MS", label: "FRAME RATE CAP", values: &[("", "DEFAULT"), ("33.333", "30"), ("16.667", "60"), ("8.333", "120"), ("0", "NONE")], live: false, with: &[] },
    Item { var: "VICTORY_FPS_LINE", label: "FPS AT THE TOP", values: UNSET_ON, live: true, with: &[] },
    Item { var: "VICTORY_PANEL", label: "STATS PANEL (F3)", values: UNSET_ON, live: true, with: &[] },
    // The launcher's own (`scripts\play.ps1 -Console`).
    Item { var: "VICTORY_MENU_CONSOLE", label: "CONSOLE", values: &[("auto", "AUTO"), ("series-s", "SERIES S"), ("series-x", "SERIES X")], live: false, with: &[] },
    Item { var: "VICTORY_AMBIENT", label: "LIGHT ESTIMATE", values: &[("constant:0.8", "CONSTANT 0.8"), ("0.8", "ESTIMATE 0.8"), ("1", "ESTIMATE 1"), ("0.5", "ESTIMATE 0.5"), ("0", "OFF")], live: false, with: &[] },
    Item { var: "VICTORY_AMBIENT_AIR", label: "FOG CAP", values: &[("1.5", "1.5"), ("1", "1"), ("2", "2"), ("", "NONE")], live: false, with: &[] },
    // The translator reads VICTORY_RAYS: the other choice has its own shader cache (warm it).
    Item { var: "VICTORY_BVH_BUILDER", label: "RAY TRACING (NEEDS WARM-UP)", values: ON_UNSET, live: false, with: &[("VICTORY_RAYS", "1"), ("VICTORY_BVH_QUARANTINE", "1"), ("VICTORY_PIPELINE_COMPUTE_WAIT", "0")] },
    Item { var: "VICTORY_HAIR", label: "HAIR", values: ON_UNSET, live: false, with: &[] },
];

/// The lines after the settings.
const ACTIONS: [&str; 3] = ["SAVE FOR NEXT START", "RESET ALL TO PLAY DEFAULTS", "CLOSE"];

fn get(var: &str) -> String {
    std::env::var(var).unwrap_or_default()
}

/// Whether the frame-rate line is shown at the top while the menu is closed
/// (`VICTORY_FPS_LINE=1`; off by default so that pictures taken to compare carry nothing of ours;
/// the menu changes it).
fn fps_line_on() -> &'static AtomicBool {
    static ON: OnceLock<AtomicBool> = OnceLock::new();
    ON.get_or_init(|| AtomicBool::new(std::env::var_os("VICTORY_FPS_LINE").is_some_and(|v| v == "1")))
}

/// That line: the frames rendered a second.
fn fps_line() -> Option<String> {
    if !fps_line_on().load(Ordering::Relaxed) {
        return None;
    }
    // The stats panel has that corner (F3), and says the same.
    if super::panel::shown().load(Ordering::Relaxed) {
        return None;
    }
    let fps = f64::from(super::METER_FPS_TENTHS.load(Ordering::Relaxed)) / 10.0;
    if fps <= 0.0 {
        return None;
    }
    Some(format!("FPS: {fps:.1}"))
}

/// The index of `value` among the item's values (an unknown value: shown as
/// itself, at the end).
fn index_of(item: &Item, value: &str) -> Option<usize> {
    item.values.iter().position(|(v, _)| *v == value)
}

/// What the menu shows for `value`: its label, an unknown value as itself,
/// and a value the launcher did not set (a start without play.ps1) as
/// "NOT SET" rather than a blank.
fn shown_value(item: &Item, value: &str) -> String {
    match index_of(item, value) {
        Some(i) => item.values[i].1.to_owned(),
        None if value.is_empty() => "NOT SET".to_owned(),
        None => value.to_ascii_uppercase(),
    }
}

/// Keys pressed, gathered on a thread of their own every 10 ms: a frame
/// lasts 50 ms and more, and a short press would fall between two frames.
fn keys() -> &'static Mutex<Vec<u16>> {
    static KEYS: OnceLock<Mutex<Vec<u16>>> = OnceLock::new();
    KEYS.get_or_init(|| {
        let _ = std::thread::Builder::new().name("settings menu keys".into()).spawn(|| {
            const WATCHED: [u16; 7] = [VK_F1, VK_UP, VK_DOWN, VK_LEFT, VK_RIGHT, VK_RETURN, VK_ESCAPE];
            let mut was = [false; WATCHED.len()];
            loop {
                std::thread::sleep(Duration::from_millis(10));
                let mut process = 0;
                // SAFETY: plain queries; room for the answer.
                unsafe { GetWindowThreadProcessId(GetForegroundWindow(), &mut process) };
                let ours = process == std::process::id();
                for (i, &vk) in WATCHED.iter().enumerate() {
                    // SAFETY: a plain query.
                    let down = ours && unsafe { GetAsyncKeyState(i32::from(vk)) } < 0;
                    if down && !was[i] {
                        keys().lock().unwrap_or_else(|e| e.into_inner()).push(vk);
                    }
                    was[i] = down;
                }
            }
        });
        Mutex::new(Vec::new())
    })
}

/// The box's size in pixels.
const fn size(scale: u32) -> (u32, u32) {
    ((COLUMNS * CELL.0 + 2 * RIM) * scale, (LINES * CELL.1 + 2 * RIM) * scale)
}

fn lit(lines: &[String], x: u32, y: u32) -> bool {
    let (Some(x), Some(y)) = (x.checked_sub(RIM), y.checked_sub(RIM)) else {
        return false;
    };
    if x % CELL.0 >= GLYPH.0 || y % CELL.1 >= GLYPH.1 || x / CELL.0 >= COLUMNS {
        return false;
    }
    let Some(&c) = lines.get((y / CELL.1) as usize).and_then(|l| l.as_bytes().get((x / CELL.0) as usize)) else {
        return false;
    };
    (glyph(c.to_ascii_uppercase())[(y % CELL.1) as usize] >> (GLYPH.0 - 1 - x % CELL.0)) & 1 != 0
}

fn painted(lines: &[String], scale: u32, colours: [&[u8]; 2], pitch: usize) -> Vec<u8> {
    let (width, height) = size(scale);
    let bytes = colours[0].len();
    let mut out = vec![0; pitch * height as usize];
    for y in 0..height / scale {
        let top = pitch * (y * scale) as usize;
        for x in 0..width / scale {
            let colour = colours[usize::from(lit(lines, x, y))];
            for i in 0..scale {
                let at = top + (x * scale + i) as usize * bytes;
                out[at..at + bytes].copy_from_slice(colour);
            }
        }
        for i in 1..scale as usize {
            out.copy_within(top..top + pitch, top + i * pitch);
        }
    }
    out
}

struct Buffers {
    format: DXGI_FORMAT,
    scale: u32,
    pitch: u32,
    uploads: Vec<(ID3D12Resource, *mut u8)>,
    card: ID3D12Resource,
    turn: usize,
    fresh: bool,
    /// The card's buffer was copied into before (it is in the copy-source state;
    /// at first it is in the common state, promoted by the first copy).
    used: bool,
}

// SAFETY: free-threaded resources, and memory only written under the presenter's lock.
unsafe impl Send for Buffers {}

impl Buffers {
    fn new(host: &Host, format: DXGI_FORMAT, scale: u32) -> Option<Buffers> {
        let (width, height) = size(scale);
        let pitch = (width * colours(format)?[0].len() as u32).next_multiple_of(256);
        let bytes = u64::from(pitch) * u64::from(height);
        let uploads = (0..UPLOADS).map(|_| host.upload_buffer(bytes)).collect::<Option<Vec<_>>>()?;
        let heap = D3D12_HEAP_PROPERTIES { Type: D3D12_HEAP_TYPE_DEFAULT, ..Default::default() };
        let desc = D3D12_RESOURCE_DESC {
            Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
            Width: bytes,
            Height: 1,
            DepthOrArraySize: 1,
            MipLevels: 1,
            SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
            Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
            ..Default::default()
        };
        let mut card: Option<ID3D12Resource> = None;
        // SAFETY: plain creation of a buffer.
        unsafe {
            host.device
                .CreateCommittedResource(&heap, D3D12_HEAP_FLAG_NONE, &desc, D3D12_RESOURCE_STATE_COMMON, None, &mut card)
                .ok()?;
        }
        Some(Buffers { format, scale, pitch, uploads, card: card?, turn: 0, fresh: false, used: false })
    }

    fn write(&mut self, lines: &[String]) {
        let Some(colours) = colours(self.format) else { return };
        let picture = painted(lines, self.scale, colours, self.pitch as usize);
        self.turn = (self.turn + 1) % UPLOADS;
        // SAFETY: a mapped upload buffer of the picture's size, which the card read frames ago.
        unsafe { std::ptr::copy_nonoverlapping(picture.as_ptr(), self.uploads[self.turn].1, picture.len()) };
        self.fresh = true;
    }
}

/// The menu of a presenter.
pub(super) struct Menu {
    /// The values chosen, by item (as at start until changed).
    values: Vec<String>,
    /// The values at start.
    started: Vec<String>,
    /// The line picked: an item, or an action after them.
    at: usize,
    note: String,
    changed: bool,
    /// What the buffer holds: the frame-rate line, or (none) the menu's lines.
    painted: Option<String>,
    buffers: Option<Buffers>,
    failed: bool,
}

impl Menu {
    pub(super) fn new() -> Menu {
        // Start the key thread now: F1 must be seen while the menu is closed.
        let _ = keys();
        // `VICTORY_MENU_OPEN=1`: shown from the start (to look at it in a run nobody presses keys in).
        if std::env::var_os("VICTORY_MENU_OPEN").is_some_and(|v| v == "1") {
            crate::SETTINGS_MENU_OPEN.store(true, Ordering::Relaxed);
        }
        let values: Vec<String> = ITEMS.iter().map(|i| get(i.var)).collect();
        Menu { started: values.clone(), values, at: 0, note: String::new(), changed: true, painted: None, buffers: None, failed: false }
    }

    fn open() -> bool {
        crate::SETTINGS_MENU_OPEN.load(Ordering::Relaxed)
    }

    fn lines(&self) -> Vec<String> {
        let total = ITEMS.len() + ACTIONS.len();
        let first = self.at.saturating_sub(ROWS / 2).min(total.saturating_sub(ROWS));
        let mut out = vec![
            "SETTINGS                       F1 OR ESC CLOSES".to_owned(),
            "UP/DOWN PICK  LEFT/RIGHT CHANGE  ENTER DOES IT".to_owned(),
            String::new(),
        ];
        for row in first..(first + ROWS).min(total) {
            let cursor = if row == self.at { '>' } else { ' ' };
            let line = if let Some(item) = ITEMS.get(row) {
                let value = &self.values[row];
                let shown = shown_value(item, value);
                let mark = if *value != self.started[row] { if item.live { " *LIVE" } else { " *" } } else if item.live { " LIVE" } else { "" };
                format!("{cursor} {:<27} < {shown} >{mark}", item.label)
            } else {
                format!("{cursor} {}", ACTIONS[row - ITEMS.len()])
            };
            out.push(line.chars().take(COLUMNS as usize).collect());
        }
        while out.len() < 3 + ROWS {
            out.push(String::new());
        }
        out.push(String::new());
        out.push(if self.note.is_empty() { "* = CHANGED: APPLIES AT THE NEXT START UNLESS LIVE".to_owned() } else { self.note.clone() });
        out
    }

    /// Changes item `i` by `step` (+1 / -1) through its values.
    fn change(&mut self, i: usize, step: isize) {
        let item = &ITEMS[i];
        let n = item.values.len() as isize;
        let now = index_of(item, &self.values[i]).map_or(-1, |x| x as isize);
        let next = if now < 0 { if step > 0 { 0 } else { n - 1 } } else { (now + step).rem_euclid(n) };
        self.values[i] = item.values[next as usize].0.to_owned();
        if item.live {
            self.apply_live(i);
        }
        self.note.clear();
    }

    /// A setting the build reads at each use: changed in this run's environment at once.
    fn apply_live(&self, i: usize) {
        let (var, value) = (ITEMS[i].var, &self.values[i]);
        // SAFETY: the variables changed here are only read through the atomics below, and
        // by children of this process (none); Windows' environment block is locked by
        // the process heap's routines, and std holds its own lock.
        unsafe {
            if value.is_empty() {
                std::env::remove_var(var);
            } else {
                std::env::set_var(var, value);
            }
        }
        match var {
            "VICTORY_FPS_LINE" => fps_line_on().store(value == "1", Ordering::Relaxed),
            "VICTORY_PANEL" => super::panel::shown().store(value == "1", Ordering::Relaxed),
            _ => {}
        }
    }

    /// The variables to write for the changed items: each item's own, and its `with`
    /// (the value they have while it is set, or none while it is not).
    fn changes(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        for (i, item) in ITEMS.iter().enumerate() {
            if self.values[i] == self.started[i] {
                continue;
            }
            out.push((item.var.to_owned(), self.values[i].clone()));
            for (var, on) in item.with {
                out.push(((*var).to_owned(), if self.values[i].is_empty() { String::new() } else { (*on).to_owned() }));
            }
        }
        out
    }

    /// Writes the changed values over those in the settings file.
    fn save(&mut self) -> bool {
        let Some(path) = std::env::var_os("VICTORY_MENU_FILE") else {
            self.note = "NO SETTINGS FILE: START WITH SCRIPTS PLAY.PS1".to_owned();
            return false;
        };
        let mut kept: Vec<(String, String)> = std::fs::read_to_string(&path)
            .unwrap_or_default()
            .lines()
            .filter_map(|l| l.split_once('=').map(|(k, v)| (k.trim().to_owned(), v.trim().to_owned())))
            .filter(|(k, _)| k.starts_with("VICTORY_"))
            .collect();
        for (var, value) in self.changes() {
            kept.retain(|(k, _)| *k != var);
            kept.push((var, value));
        }
        let mut text = String::from("# Saved by the F1 menu; applied over the play settings at every start (an empty value: unset).\r\n");
        for (k, v) in &kept {
            text.push_str(&format!("{k}={v}\r\n"));
        }
        match std::fs::write(&path, text) {
            Ok(()) => {
                tracing::info!(path = %std::path::Path::new(&path).display(), "settings menu: saved");
                self.started = self.values.clone();
                self.note = "SAVED. APPLIES AT THE NEXT START".to_owned();
                true
            }
            Err(e) => {
                tracing::warn!(error = %e, "settings menu: not saved");
                self.note = "COULD NOT SAVE THE SETTINGS FILE".to_owned();
                false
            }
        }
    }

    /// Removes the settings file: the launcher's own settings at the next start.
    fn reset(&mut self) {
        if let Some(path) = std::env::var_os("VICTORY_MENU_FILE") {
            let _ = std::fs::remove_file(path);
        }
        self.note = "RESET. PLAY DEFAULTS AT THE NEXT START".to_owned();
    }

    /// A frame was presented: the keys pressed since the last one.
    pub(super) fn frame(&mut self) {
        let pressed = std::mem::take(&mut *keys().lock().unwrap_or_else(|e| e.into_inner()));
        if pressed.is_empty() {
            return;
        }
        let total = ITEMS.len() + ACTIONS.len();
        for vk in pressed {
            let open = Self::open();
            if vk == VK_F1 || (open && vk == VK_ESCAPE) {
                crate::SETTINGS_MENU_OPEN.store(!open && vk == VK_F1, Ordering::Relaxed);
                self.note.clear();
                self.changed = true;
                continue;
            }
            if !open {
                continue;
            }
            match vk {
                VK_UP => self.at = (self.at + total - 1) % total,
                VK_DOWN => self.at = (self.at + 1) % total,
                VK_LEFT if self.at < ITEMS.len() => self.change(self.at, -1),
                VK_RIGHT if self.at < ITEMS.len() => self.change(self.at, 1),
                VK_RETURN if self.at < ITEMS.len() => self.change(self.at, 1),
                VK_RETURN => match self.at - ITEMS.len() {
                    0 => {
                        self.save();
                    }
                    1 => self.reset(),
                    _ => crate::SETTINGS_MENU_OPEN.store(false, Ordering::Relaxed),
                },
                _ => {}
            }
            self.changed = true;
        }
    }

    /// Records the copy of the box onto `back` (in the copy-destination state).
    pub(super) fn draw(&mut self, host: &Host, list: &HostList, back: &ID3D12Resource, (width, height): (u32, u32), format: DXGI_FORMAT) {
        if self.failed {
            return;
        }
        // Closed: the frame-rate line in the top left corner, if it is wanted and there is a rate.
        let open = Self::open();
        let line = if open { None } else { fps_line() };
        if !open && line.is_none() {
            return;
        }
        // Our own copies are not part of a recorded frame (the recorder is in diagnostics builds).
        #[cfg(feature = "diagnostics")]
        let _ours = crate::replay::record::unrecorded();
        let scale = (height / 540).max(1);
        if self.buffers.as_ref().is_none_or(|b| (b.format, b.scale) != (format, scale)) {
            std::mem::forget(self.buffers.take());
            self.buffers = Buffers::new(host, format, scale);
            self.failed = self.buffers.is_none();
            self.changed = true;
        }
        if self.painted != line {
            self.painted = line.clone();
            self.changed = true;
        }
        let lines = if self.changed { Some(line.as_ref().map_or_else(|| self.lines(), |l| vec![l.clone()])) } else { None };
        let Some(buffers) = &mut self.buffers else { return };
        if let Some(lines) = lines {
            self.changed = false;
            buffers.write(&lines);
        }
        // The whole box in the middle, or of it the first line's part in the corner.
        let (w, h) = match &line {
            Some(l) => ((l.len() as u32 * CELL.0 + 2 * RIM) * scale, (CELL.1 + 2 * RIM) * scale),
            None => size(scale),
        };
        let (w, h) = (w.min(width), h.min(height));
        let (x, y) = if line.is_some() { (0, 0) } else { ((width - w) / 2, (height - h) / 2) };
        let list = &list.list;
        // SAFETY: an open list and live resources; the box lies within both.
        unsafe {
            if std::mem::take(&mut buffers.fresh) {
                if std::mem::replace(&mut buffers.used, true) {
                    list.ResourceBarrier(&[transition(&buffers.card, D3D12_RESOURCE_STATE_COPY_SOURCE, D3D12_RESOURCE_STATE_COPY_DEST)]);
                }
                list.CopyResource(&buffers.card, &buffers.uploads[buffers.turn].0);
                list.ResourceBarrier(&[transition(&buffers.card, D3D12_RESOURCE_STATE_COPY_DEST, D3D12_RESOURCE_STATE_COPY_SOURCE)]);
            }
            let from = D3D12_TEXTURE_COPY_LOCATION {
                pResource: std::mem::transmute_copy(&buffers.card),
                Type: D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT,
                Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
                    PlacedFootprint: D3D12_PLACED_SUBRESOURCE_FOOTPRINT {
                        Offset: 0,
                        Footprint: D3D12_SUBRESOURCE_FOOTPRINT { Format: format, Width: size(scale).0, Height: size(scale).1, Depth: 1, RowPitch: buffers.pitch },
                    },
                },
            };
            let to = D3D12_TEXTURE_COPY_LOCATION {
                pResource: std::mem::transmute_copy(back),
                Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
                Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 { SubresourceIndex: 0 },
            };
            let part = D3D12_BOX { left: 0, top: 0, front: 0, right: w, bottom: h, back: 1 };
            list.CopyTextureRegion(&to, x, y, 0, &from, Some(&part));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn menu_with(values: Vec<String>) -> Menu {
        Menu { started: values.clone(), values, at: 0, note: String::new(), changed: false, painted: None, buffers: None, failed: false }
    }

    #[test]
    fn every_line_fits_and_every_character_has_a_picture() {
        let mut menu = menu_with(ITEMS.iter().map(|_| String::new()).collect());
        for note in ["", "SAVED. APPLIES AT THE NEXT START", "NO SETTINGS FILE: START WITH SCRIPTS PLAY.PS1", "COULD NOT SAVE THE SETTINGS FILE", "RESET. PLAY DEFAULTS AT THE NEXT START"] {
            menu.note = note.to_owned();
            for at in 0..ITEMS.len() + ACTIONS.len() {
                menu.at = at;
                let lines = menu.lines();
                assert_eq!(lines.len(), LINES as usize);
                for line in lines {
                    assert!(line.len() <= COLUMNS as usize, "{line}");
                    for c in line.bytes() {
                        assert_ne!(glyph(c.to_ascii_uppercase()), glyph(b'#'), "{line}: {}", c as char);
                    }
                }
            }
        }
    }

    #[test]
    fn every_value_of_every_item_fits_and_has_pictures() {
        let mut menu = menu_with(ITEMS.iter().map(|_| String::new()).collect());
        for (i, item) in ITEMS.iter().enumerate() {
            for (value, _) in item.values {
                menu.values[i] = (*value).to_owned();
                menu.at = i;
                for line in menu.lines() {
                    // (Lines are cut at COLUMNS: one that reaches it may have lost its end.)
                    assert!(line.len() < COLUMNS as usize, "{line}");
                    assert!(line.bytes().all(|c| glyph(c.to_ascii_uppercase()) != glyph(b'#')), "{line}");
                }
            }
        }
    }

    #[test]
    fn values_cycle_both_ways() {
        let mut menu = menu_with(ITEMS.iter().map(|i| i.values[0].0.to_owned()).collect());
        // (An item that is live would change this process's variables: test one that is not.)
        let i = ITEMS.iter().position(|i| !i.live).unwrap();
        let n = ITEMS[i].values.len();
        for _ in 0..n {
            menu.change(i, 1);
        }
        assert_eq!(menu.values[i], ITEMS[i].values[0].0);
        menu.change(i, -1);
        assert_eq!(menu.values[i], ITEMS[i].values[n - 1].0);
    }

    #[test]
    fn ray_tracing_takes_its_companions_along() {
        let i = ITEMS.iter().position(|i| i.var == "VICTORY_BVH_BUILDER").unwrap();
        let mut menu = menu_with(ITEMS.iter().map(|i| i.values[0].0.to_owned()).collect());
        assert!(menu.changes().is_empty());
        menu.values[i] = String::new();
        let off = menu.changes();
        assert_eq!(off.len(), 1 + ITEMS[i].with.len());
        assert!(off.iter().all(|(_, v)| v.is_empty()), "{off:?}");
        menu.started[i] = String::new();
        menu.values[i] = "1".to_owned();
        let on = menu.changes();
        assert!(on.contains(&("VICTORY_RAYS".to_owned(), "1".to_owned())), "{on:?}");
        assert!(on.contains(&("VICTORY_PIPELINE_COMPUTE_WAIT".to_owned(), "0".to_owned())), "{on:?}");
    }

    #[test]
    fn a_saved_change_is_in_the_file_and_kept_by_the_next_save() {
        let dir = std::env::temp_dir().join(format!("victory-menu-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("victory-settings.txt");
        std::fs::write(&file, "# old\r\nVICTORY_KEPT=1\r\nVICTORY_HAIR=1\r\nNOT_OURS=1\r\n").unwrap();
        // SAFETY: only this test reads VICTORY_MENU_FILE.
        unsafe { std::env::set_var("VICTORY_MENU_FILE", &file) };
        let i = ITEMS.iter().position(|i| i.var == "VICTORY_HAIR").unwrap();
        let mut menu = menu_with(ITEMS.iter().map(|i| i.values[0].0.to_owned()).collect());
        menu.values[i] = String::new();
        assert!(menu.save());
        let text = std::fs::read_to_string(&file).unwrap();
        assert!(text.contains("VICTORY_KEPT=1\r\n") && text.contains("VICTORY_HAIR=\r\n"), "{text}");
        assert!(!text.contains("NOT_OURS") && !text.contains("VICTORY_HAIR=1"), "{text}");
        menu.reset();
        assert!(!file.exists());
        // SAFETY: as above.
        unsafe { std::env::remove_var("VICTORY_MENU_FILE") };
        let _ = std::fs::remove_dir_all(&dir);
    }
}
