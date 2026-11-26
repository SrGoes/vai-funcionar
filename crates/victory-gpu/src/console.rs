//! Which console the title runs on. Titles pick settings (resolution,
//! memory budgets) by it, so the user chooses; Series X unless told.

use std::sync::atomic::{AtomicBool, Ordering};

const GIB: usize = 1 << 30;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Console {
    #[default]
    SeriesX,
    SeriesS,
}

impl Console {
    /// Title memory in bytes: (GPU-optimal, standard). Series X in the
    /// advanced configuration; Series S has one range, counted as standard
    /// (7,776 MiB, the published console-mode figure).
    // shortcut: later system updates gave Series S titles more (amount not
    // published here); the title sizes nothing from this figure
    // (it goes by the device type), so update it if a title does.
    pub const fn title_memory(self) -> (usize, usize) {
        match self {
            Console::SeriesX => (10 * GIB, 7 * GIB / 2),
            Console::SeriesS => (0, 7_776 << 20),
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Console::SeriesX => "Xbox Series X",
            Console::SeriesS => "Xbox Series S",
        }
    }
}

/// What the PC has that the choice of console depends on.
#[derive(Debug, Clone, Copy)]
pub struct HostMemory {
    /// Physical memory, in bytes (0: not known).
    pub ram: u64,
    /// Dedicated video memory of the default adapter (the one the host
    /// device is made on), in bytes (0: not known).
    pub vram: u64,
}

impl HostMemory {
    /// Least memory with which Series X is chosen by `auto`. Series X runs
    /// on a 64 GB / 24 GB PC (README: 40-46 GB private). On 16 GB / 8 GB
    /// (RTX 5060) both Series X runs ran out of physical memory in the
    /// first minutes of the world (7-500 MB free, 0.3-5 fps, E_OUTOFMEMORY
    /// and a device loss or a fault in the driver), while Series S held.
    pub const SERIES_X_RAM: u64 = 48 << 30;
    pub const SERIES_X_VRAM: u64 = 12 << 30;

    /// The console `auto` picks: Series X only where both are known and
    /// large enough.
    pub const fn console(self) -> Console {
        if self.ram >= Self::SERIES_X_RAM && self.vram >= Self::SERIES_X_VRAM {
            Console::SeriesX
        } else {
            Console::SeriesS
        }
    }

    /// This PC's.
    #[cfg(windows)]
    pub fn of_host() -> Self {
        use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory2, DXGI_CREATE_FACTORY_FLAGS, IDXGIFactory4};
        use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
        // SAFETY: the API fills a correctly sized structure.
        let ram = unsafe {
            let mut m: MEMORYSTATUSEX = std::mem::zeroed();
            m.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
            if GlobalMemoryStatusEx(&mut m) != 0 { m.ullTotalPhys } else { 0 }
        };
        // Adapter 0: the one `D3D12CreateDevice` takes when given none.
        // SAFETY: plain DXGI queries.
        let vram = unsafe {
            CreateDXGIFactory2::<IDXGIFactory4>(DXGI_CREATE_FACTORY_FLAGS(0))
                .and_then(|f| f.EnumAdapters1(0))
                .and_then(|a| a.GetDesc1())
                .map_or(0, |d| d.DedicatedVideoMemory as u64)
        };
        Self { ram, vram }
    }

    #[cfg(not(windows))]
    pub fn of_host() -> Self {
        Self { ram: 0, vram: 0 }
    }
}

static SERIES_S: AtomicBool = AtomicBool::new(false);

/// Set before the title starts.
pub fn set(console: Console) {
    SERIES_S.store(console == Console::SeriesS, Ordering::Relaxed);
}

pub fn current() -> Console {
    if SERIES_S.load(Ordering::Relaxed) {
        Console::SeriesS
    } else {
        Console::SeriesX
    }
}

#[cfg(test)]
mod tests {
    use super::{Console, HostMemory};
    const GIB: u64 = 1 << 30;

    /// Series X only where both memories are known and large enough.
    #[test]
    fn auto_picks_series_x_only_with_the_memory_it_needs() {
        let pick = |ram, vram| HostMemory { ram: ram * GIB, vram: vram * GIB }.console();
        assert_eq!(pick(16, 8), Console::SeriesS); // RTX 5060 PC of the black-render runs
        assert_eq!(pick(32, 32), Console::SeriesS); // RTX 5090 PC of the pareto runs
        assert_eq!(pick(64, 8), Console::SeriesS);
        assert_eq!(pick(64, 24), Console::SeriesX); // RX 7900 XTX PC of the README
        assert_eq!(pick(0, 0), Console::SeriesS); // unknown
    }
}
