//! Host-memory admission for speculative texture work and reusable upload storage.
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering};
use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

const MIB: u64 = 1 << 20;
const GIB: u64 = 1 << 30;
static PRESSURED: AtomicBool = AtomicBool::new(false);
static HEADROOM: AtomicU64 = AtomicU64::new(u64::MAX);

fn memory_status() -> Option<MEMORYSTATUSEX> {
    // SAFETY: the API initializes the fields of a correctly sized structure.
    let mut status: MEMORYSTATUSEX = unsafe { std::mem::zeroed() };
    status.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
    (unsafe { GlobalMemoryStatusEx(&mut status) } != 0).then_some(status)
}

// The texture sweep may run for seconds inside a single submitted frame.
static NEXT_COMMIT_SAMPLE: AtomicU64 = AtomicU64::new(0);
static AVAILABLE_COMMIT: AtomicU64 = AtomicU64::new(u64::MAX);
static RETRY_UPLOAD_AT: AtomicU64 = AtomicU64::new(0);

/// The PC's memory shared with the card, as DXGI counts it for this
/// process: how full against its budget ([`shared_level`]), sampled at most
/// every 32 ms. Commit and free RAM alone do not see it: on a 16 GB PC with
/// a 40 GB page file (RTX 5060, 27/11) commit stayed above 30 GB free while
/// the shared group went past its 7.4 GB budget (staging blocks for the
/// textures of a loading world, 5 GB of them), upload buffers failed with
/// E_OUTOFMEMORY 641 times and the driver crashed closing a list
/// (`nvwgf2umx.dll+0x422e03`, twice). `VICTORY_SHARED_GUARD=0`: not looked at.
static NEXT_SHARED_SAMPLE: AtomicU64 = AtomicU64::new(0);
static SHARED_LEVEL: AtomicU8 = AtomicU8::new(0);

/// How full the shared group is: 0 below 85 % of its budget; 1 (pressure:
/// idle storage let go, prefetch held to 64 MB a frame) from 85 %; 2 (no new
/// staging blocks for speculative textures) from 90 %; 3 (critical: no spare
/// staging blocks kept, prefetch held to 16 MB) from 95 %; 4 (only the
/// textures a draw needs now) from 97 %. A budget of 0 counts as no budget.
fn shared_level(budget: u64, used: u64) -> u8 {
    if budget == 0 {
        return 0;
    }
    let percent = u128::from(used) * 100 / u128::from(budget);
    match percent {
        0..85 => 0,
        85..90 => 1,
        90..95 => 2,
        95..97 => 3,
        _ => 4,
    }
}

fn shared_guard() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("VICTORY_SHARED_GUARD").is_none_or(|v| v != "0"))
}

/// [`SHARED_LEVEL`], sampled again if 32 ms have passed.
fn sample_shared(now: u64) -> u8 {
    if !shared_guard() {
        return 0;
    }
    let next = NEXT_SHARED_SAMPLE.load(Ordering::Relaxed);
    if now >= next
        && NEXT_SHARED_SAMPLE
            .compare_exchange(next, now.saturating_add(32), Ordering::Relaxed, Ordering::Relaxed)
            .is_ok()
        && let Some([_, [budget, used, ..]]) = crate::host::host().and_then(|h| h.video_memory_groups())
    {
        let level = shared_level(budget, used);
        let was = SHARED_LEVEL.swap(level, Ordering::Relaxed);
        if level != was && (level >= 3 || was >= 3) {
            tracing::warn!(
                level,
                was,
                budget_mib = budget >> 20,
                used_mib = used >> 20,
                texture_staging_mib = crate::texture::live_staging_bytes() >> 20,
                "the PC's memory shared with the card: level changed (3 and up hold texture uploads back)"
            );
        }
    }
    SHARED_LEVEL.load(Ordering::Relaxed)
}

pub(crate) fn allocation_error(error: &windows::core::Error) {
    if error.code() == windows::Win32::Foundation::E_OUTOFMEMORY {
        RETRY_UPLOAD_AT.fetch_max(crate::host::coarse_ms().saturating_add(500), Ordering::Relaxed);
    }
}

thread_local! {
    static STAGING_DEFERRED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

pub(crate) fn note_staging_deferred() {
    STAGING_DEFERRED.set(true);
}

pub(crate) fn take_staging_deferred() -> bool {
    STAGING_DEFERRED.replace(false)
}

pub(crate) fn upload_refused() -> bool {
    crate::host::coarse_ms() < RETRY_UPLOAD_AT.load(Ordering::Relaxed)
}

fn sample_commit(now: u64) -> u64 {
    let next = NEXT_COMMIT_SAMPLE.load(Ordering::Relaxed);
    if now >= next
        && NEXT_COMMIT_SAMPLE
            .compare_exchange(next, now.saturating_add(32), Ordering::Relaxed, Ordering::Relaxed)
            .is_ok()
        && let Some(status) = memory_status()
    {
        AVAILABLE_COMMIT.store(status.ullAvailPageFile, Ordering::Relaxed);
    }
    AVAILABLE_COMMIT.load(Ordering::Relaxed)
}

fn has_upload_reserve(commit: u64, bake: bool) -> bool {
    // Keep a protective floor against Rust/game OOM aborts without starving
    // bindless world textures or one-shot ped decoration bakes when steady-state
    // commit headroom sits in the 1-4 GiB range on 32 GiB hosts.
    commit >= if bake { 256 * MIB } else { 512 * MIB }
}

fn has_growth_reserve(commit: u64, bake: bool) -> bool {
    // Growing a fresh 64 MiB upload block for speculative bindless prefetch
    // requires more headroom than reusing slices already committed inside the
    // staging pool.
    commit >= if bake { 256 * MIB } else { 1024 * MIB }
}

/// Leave commit for game allocations and submissions already in flight while
/// still letting bindless textures and one-shot material bakes upload.
pub(crate) fn admit_texture_pass(bake: bool) -> bool {
    let now = crate::host::coarse_ms();
    if now < RETRY_UPLOAD_AT.load(Ordering::Relaxed) {
        return false;
    }
    if !bake && sample_shared(now) >= 4 {
        return false;
    }
    has_upload_reserve(sample_commit(now), bake)
}

/// Whether the staging allocator may commit a brand-new OS upload buffer
/// (reusing existing free ranges in already-committed pool blocks stays open
/// whenever `admit_texture_pass` is true).
pub(crate) fn allow_staging_growth(bake: bool) -> bool {
    let now = crate::host::coarse_ms();
    if now < RETRY_UPLOAD_AT.load(Ordering::Relaxed) {
        return false;
    }
    if sample_shared(now) >= if bake { 4 } else { 2 } {
        return false;
    }
    has_growth_reserve(sample_commit(now), bake)
}

pub(crate) fn limited_host_ram() -> bool {
    static LIMITED: OnceLock<bool> = OnceLock::new();
    *LIMITED.get_or_init(|| memory_status().is_some_and(|s| s.ullTotalPhys <= 32 * GIB))
}

fn next_pressure(was: bool, physical: u64, commit: u64) -> bool {
    if was {
        physical < 5 * GIB || commit < 6 * GIB
    } else {
        physical <= 3 * GIB || commit <= 4 * GIB
    }
}

/// Called by the once-per-second submission sweep. A failed sample keeps the last good state.
pub(crate) fn refresh() {
    let Some(status) = memory_status() else { return };
    let (physical, commit) = (status.ullAvailPhys, status.ullAvailPageFile);
    HEADROOM.store(physical.min(commit), Ordering::Relaxed);
    let was = pressured();
    let pressure = next_pressure(was, physical, commit);
    if pressure != was {
        PRESSURED.store(pressure, Ordering::Relaxed);
        if pressure {
            tracing::warn!(
                pid = std::process::id(),
                frame = crate::frame::current(),
                physical_free_mib = physical >> 20,
                available_commit_mib = commit >> 20,
                buffer_mirror_span_mib = crate::shadow::live_span_bytes() >> 20,
                texture_staging_mib = crate::texture::live_staging_bytes() >> 20,
                spare_scratch_mib = crate::host::ALIVE[4].load(Ordering::Relaxed) >> 20,
                game_filled_gpu_texture_mib = crate::resource::KINDS[1].load(Ordering::Relaxed) >> 20,
                "host memory pressure: limiting texture prefetch and idle upload storage"
            );
        } else {
            tracing::info!(physical_free_mib = physical >> 20, available_commit_mib = commit >> 20, "host memory pressure cleared");
        }
    }
}

pub(crate) fn headroom() -> Option<u64> {
    let bytes = HEADROOM.load(Ordering::Relaxed);
    (bytes != u64::MAX).then_some(bytes)
}

/// Free RAM or commit low ([`refresh`]), or the shared group at 85 % of its
/// budget ([`shared_level`]).
pub(crate) fn pressured() -> bool {
    PRESSURED.load(Ordering::Relaxed) || sample_shared(crate::host::coarse_ms()) >= 1
}

/// True when OS commit headroom is critically low (< 1.5 GiB), the shared
/// group is at 95 % of its budget, or an allocation just failed with
/// `E_OUTOFMEMORY`, requiring emergency throttling.
pub(crate) fn critical_commit() -> bool {
    let now = crate::host::coarse_ms();
    now < RETRY_UPLOAD_AT.load(Ordering::Relaxed) || sample_commit(now) < 1536 * MIB || sample_shared(now) >= 3
}

/// A soft per-frame admission limit. A single texture can cross it. The
/// queue retains unserved slots so they can be retried on later frames.
pub(crate) fn prefetch_limit(bytes: u64, pressure: bool, critical: bool, limited: bool, staging: u64) -> u64 {
    if critical {
        bytes.min(16 << 20)
    } else if pressure || (limited && staging >= GIB) {
        bytes.min(64 << 20)
    } else {
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bake_and_lookahead_leave_room_for_live_game_allocations() {
        for (commit, bake, lookahead, growth) in [
            (128 * MIB, false, false, false),
            (256 * MIB, true, false, false),
            (512 * MIB, true, true, false),
            (768 * MIB, true, true, false),
            (1024 * MIB, true, true, true),
        ] {
            assert_eq!(has_upload_reserve(commit, true), bake);
            assert_eq!(has_upload_reserve(commit, false), lookahead);
            assert_eq!(has_growth_reserve(commit, false), growth);
        }
    }

    #[test]
    fn the_shared_group_is_held_back_before_its_budget() {
        let budget = 7446 * MIB;
        assert_eq!(shared_level(0, 9 * GIB), 0);
        assert_eq!(shared_level(budget, 5955 * MIB), 0);
        assert_eq!(shared_level(budget, budget * 85 / 100 + MIB), 1);
        assert_eq!(shared_level(budget, budget * 91 / 100), 2);
        assert_eq!(shared_level(budget, budget * 96 / 100), 3);
        // Run look720u6 (27/11): 7,694 MB in use of 7,446 when uploads failed.
        assert_eq!(shared_level(budget, 7694 * MIB), 4);
    }

    #[test]
    fn either_constraint_enters_pressure_and_both_must_recover() {
        assert!(next_pressure(false, 12 * GIB, 4 * GIB));
        assert!(next_pressure(false, 3 * GIB, 16 * GIB));
        assert!(next_pressure(true, 4 * GIB, 7 * GIB));
        assert!(next_pressure(true, 6 * GIB, 5 * GIB));
        assert!(!next_pressure(true, 5 * GIB, 6 * GIB));
        assert!(!next_pressure(false, 4 * GIB, 5 * GIB));
    }

    #[test]
    fn prefetch_respects_existing_budgets_and_resumes_after_recovery() {
        assert_eq!(prefetch_limit(512 << 20, false, false, true, GIB), 64 << 20);
        assert_eq!(prefetch_limit(512 << 20, false, false, true, GIB - 1), 512 << 20);
        assert_eq!(prefetch_limit(512 << 20, false, false, false, 2 * GIB), 512 << 20);
        assert_eq!(prefetch_limit(0, true, true, false, 0), 0);
        assert_eq!(prefetch_limit(8 << 20, true, true, false, 0), 8 << 20);
        assert_eq!(prefetch_limit(512 << 20, true, false, false, 0), 64 << 20);
        assert_eq!(prefetch_limit(512 << 20, true, true, false, 0), 16 << 20);
        assert_eq!(prefetch_limit(192 << 20, false, false, false, 0), 192 << 20);
    }
}
