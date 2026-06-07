use crate::registry::{self, Hive};

// ─── Tweak Status ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TweakStatus {
    Applied,
    Default,
    Unknown,
    Error,
}

impl TweakStatus {
    pub fn symbol(self) -> &'static str {
        match self {
            TweakStatus::Applied => "●",
            TweakStatus::Default => "○",
            TweakStatus::Unknown => "?",
            TweakStatus::Error => "✗",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            TweakStatus::Applied => "APPLIED",
            TweakStatus::Default => "DEFAULT",
            TweakStatus::Unknown => "UNKNOWN",
            TweakStatus::Error => "ERROR",
        }
    }
}

// ─── Value Type ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegValueType {
    Dword,
    String,
}

// ─── Registry Tweak ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Tweak {
    pub name: &'static str,
    pub description: &'static str,
    pub hive: Hive,
    pub subkey: &'static str,
    pub value_name: &'static str,
    pub reg_type: RegValueType,
    pub optimized: TweakValue,
    pub default: TweakValue,
    pub note: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TweakValue {
    Dword(u32),
    Str(&'static str),
    DeleteValue, // revert = delete the value entirely
}

impl TweakValue {
    pub fn to_display(&self) -> String {
        match self {
            TweakValue::Dword(v) => format!("0x{:X} ({})", v, v),
            TweakValue::Str(s) => format!("\"{}\"", s),
            TweakValue::DeleteValue => "(delete)".to_string(),
        }
    }
}

impl Tweak {
    pub fn read_current(&self) -> Option<String> {
        match self.reg_type {
            RegValueType::Dword => {
                let v = registry::read_dword(self.hive, self.subkey, self.value_name)?;
                Some(format!("0x{:X} ({})", v, v))
            }
            RegValueType::String => {
                let v = registry::read_string(self.hive, self.subkey, self.value_name)?;
                Some(format!("\"{}\"", v))
            }
        }
    }

    pub fn get_status(&self) -> TweakStatus {
        match self.reg_type {
            RegValueType::Dword => {
                let current = match registry::read_dword(self.hive, self.subkey, self.value_name) {
                    Some(v) => v,
                    None => return TweakStatus::Unknown,
                };
                let opt = match self.optimized {
                    TweakValue::Dword(v) => v,
                    _ => return TweakStatus::Unknown,
                };
                if current == opt {
                    return TweakStatus::Applied;
                }
                let def = match self.default {
                    TweakValue::Dword(v) => v,
                    _ => return TweakStatus::Unknown,
                };
                if current == def {
                    return TweakStatus::Default;
                }
                TweakStatus::Unknown
            }
            RegValueType::String => {
                let current = match registry::read_string(self.hive, self.subkey, self.value_name) {
                    Some(v) => v,
                    None => return TweakStatus::Unknown,
                };
                let opt = match self.optimized {
                    TweakValue::Str(s) => s,
                    _ => return TweakStatus::Unknown,
                };
                if current == opt {
                    return TweakStatus::Applied;
                }
                let def = match self.default {
                    TweakValue::Str(s) => s,
                    _ => return TweakStatus::Unknown,
                };
                if current == def {
                    return TweakStatus::Default;
                }
                TweakStatus::Unknown
            }
        }
    }

    pub fn apply(&self) -> Result<(), String> {
        match (&self.reg_type, &self.optimized) {
            (RegValueType::Dword, TweakValue::Dword(v)) => {
                registry::write_dword(self.hive, self.subkey, self.value_name, *v)
                    .map_err(|e| e.to_string())
            }
            (RegValueType::String, TweakValue::Str(s)) => {
                registry::write_string(self.hive, self.subkey, self.value_name, s)
                    .map_err(|e| e.to_string())
            }
            _ => Err("Incompatible value type".to_string()),
        }
    }

    pub fn revert(&self) -> Result<(), String> {
        match (&self.reg_type, &self.default) {
            (_, TweakValue::DeleteValue) => {
                registry::delete_value(self.hive, self.subkey, self.value_name)
                    .map_err(|e| e.to_string())
            }
            (RegValueType::Dword, TweakValue::Dword(v)) => {
                registry::write_dword(self.hive, self.subkey, self.value_name, *v)
                    .map_err(|e| e.to_string())
            }
            (RegValueType::String, TweakValue::Str(s)) => {
                registry::write_string(self.hive, self.subkey, self.value_name, s)
                    .map_err(|e| e.to_string())
            }
            _ => Err("Incompatible value type".to_string()),
        }
    }
}

// ─── Category ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Category {
    pub name: &'static str,
    pub icon: &'static str,
    pub description: &'static str,
    pub tweaks: Vec<Tweak>,
}

impl Category {
    pub fn applied_count(&self) -> usize {
        self.tweaks.iter().filter(|t| t.get_status() == TweakStatus::Applied).count()
    }

    pub fn apply_all(&self) -> (usize, usize) {
        let ok = self.tweaks.iter().filter(|t| t.apply().is_ok()).count();
        (ok, self.tweaks.len())
    }

    pub fn revert_all(&self) -> (usize, usize) {
        let ok = self.tweaks.iter().filter(|t| t.revert().is_ok()).count();
        (ok, self.tweaks.len())
    }
}

// ─── Build All Categories ────────────────────────────────────────────────────

pub fn build_categories() -> Vec<Category> {
    vec![
        // ═══════════════════════════════════════════════════════════════════════
        // 1. Performance & Speed
        // ═══════════════════════════════════════════════════════════════════════
        Category {
            name: "Performance & Speed",
            icon: "⚡",
            description: "CPU scheduling, memory, timer resolution, power throttling",
            tweaks: vec![
                Tweak {
                    name: "Win32PrioritySeparation",
                    description: "CPU scheduler: thread quantum & foreground boost",
                    hive: Hive::Hklm,
                    subkey: r"SYSTEM\CurrentControlSet\Control\PriorityControl",
                    value_name: "Win32PrioritySeparation",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(0x2A),
                    default: TweakValue::Dword(0x2),
                    note: "0x28=balanced, 0x2A=aggressive foreground",
                },
                Tweak {
                    name: "MMCSS CPU Reserve",
                    description: "Reduce background CPU reservation from 20% to 0%",
                    hive: Hive::Hklm,
                    subkey: r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile",
                    value_name: "SystemResponsiveness",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(0),
                    default: TweakValue::Dword(20),
                    note: "",
                },
                Tweak {
                    name: "DisablePagingExecutive",
                    description: "Keep kernel drivers in physical RAM",
                    hive: Hive::Hklm,
                    subkey: r"SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management",
                    value_name: "DisablePagingExecutive",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(1),
                    default: TweakValue::Dword(0),
                    note: "8GB+ RAM recommended",
                },
                Tweak {
                    name: "Disable Prefetcher",
                    description: "Disable application pre-loading into RAM",
                    hive: Hive::Hklm,
                    subkey: r"SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management\PrefetchParameters",
                    value_name: "EnablePrefetcher",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(0),
                    default: TweakValue::Dword(3),
                    note: "Slower cold starts, frees RAM",
                },
                Tweak {
                    name: "Disable Superfetch",
                    description: "Disable intelligent RAM pre-fetching",
                    hive: Hive::Hklm,
                    subkey: r"SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management\PrefetchParameters",
                    value_name: "EnableSuperfetch",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(0),
                    default: TweakValue::Dword(3),
                    note: "",
                },
                Tweak {
                    name: "TimeStampInterval",
                    description: "Tighten timer resolution to 1ms",
                    hive: Hive::Hklm,
                    subkey: r"SOFTWARE\Microsoft\Windows\CurrentVersion\Reliability",
                    value_name: "TimeStampInterval",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(1),
                    default: TweakValue::Dword(0),
                    note: "",
                },
                Tweak {
                    name: "PowerThrottlingOff",
                    description: "Disable CPU power throttling",
                    hive: Hive::Hklm,
                    subkey: r"SYSTEM\CurrentControlSet\Control\Power\PowerThrottling",
                    value_name: "PowerThrottlingOff",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(1),
                    default: TweakValue::Dword(0),
                    note: "⚠ Decreases battery life",
                },
                Tweak {
                    name: "MenuShowDelay",
                    description: "Remove UI menu animation delay",
                    hive: Hive::Hkcu,
                    subkey: r"Control Panel\Desktop",
                    value_name: "MenuShowDelay",
                    reg_type: RegValueType::String,
                    optimized: TweakValue::Str("0"),
                    default: TweakValue::Str("400"),
                    note: "Set 100-300 if too jarring",
                },
            ],
        },

        // ═══════════════════════════════════════════════════════════════════════
        // 2. Privacy & Telemetry
        // ═══════════════════════════════════════════════════════════════════════
        Category {
            name: "Privacy & Telemetry",
            icon: "🔒",
            description: "Telemetry, tracking, advertising ID, activity history",
            tweaks: vec![
                Tweak {
                    name: "AllowTelemetry",
                    description: "Restrict telemetry to minimum",
                    hive: Hive::Hklm,
                    subkey: r"SOFTWARE\Policies\Microsoft\Windows\DataCollection",
                    value_name: "AllowTelemetry",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(0),
                    default: TweakValue::Dword(3),
                    note: "Home/Pro: 0 treated as 1",
                },
                Tweak {
                    name: "Disable Error Reporting",
                    description: "Prevent crash dump transmission",
                    hive: Hive::Hklm,
                    subkey: r"SOFTWARE\Policies\Microsoft\Windows\Windows Error Reporting",
                    value_name: "Disabled",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(1),
                    default: TweakValue::Dword(0),
                    note: "",
                },
                Tweak {
                    name: "Disable Advertising ID",
                    description: "Disable tracking identifier for ads",
                    hive: Hive::Hkcu,
                    subkey: r"Software\Microsoft\Windows\CurrentVersion\AdvertisingInfo",
                    value_name: "Enabled",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(0),
                    default: TweakValue::Dword(1),
                    note: "",
                },
                Tweak {
                    name: "Disable Tailored Experiences",
                    description: "Stop personalized tips/ads from diagnostic data",
                    hive: Hive::Hklm,
                    subkey: r"SOFTWARE\Microsoft\Windows\CurrentVersion\Privacy",
                    value_name: "TailoredExperiencesWithDiagnosticDataEnabled",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(0),
                    default: TweakValue::Dword(1),
                    note: "",
                },
                Tweak {
                    name: "Disable Activity Feed",
                    description: "Disable user application history tracking",
                    hive: Hive::Hklm,
                    subkey: r"SOFTWARE\Policies\Microsoft\Windows\System",
                    value_name: "EnableActivityFeed",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(0),
                    default: TweakValue::Dword(1),
                    note: "",
                },
                Tweak {
                    name: "Prevent Upload Activities",
                    description: "Stop activity records syncing to Microsoft",
                    hive: Hive::Hklm,
                    subkey: r"SOFTWARE\Policies\Microsoft\Windows\System",
                    value_name: "UploadUserActivities",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(0),
                    default: TweakValue::Dword(1),
                    note: "",
                },
                Tweak {
                    name: "Suppress Publish Activities",
                    description: "Prevent local activity feed publishing",
                    hive: Hive::Hklm,
                    subkey: r"SOFTWARE\Policies\Microsoft\Windows\System",
                    value_name: "PublishUserActivities",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(0),
                    default: TweakValue::Dword(1),
                    note: "",
                },
                Tweak {
                    name: "Disable Timeline",
                    description: "Disable Windows Timeline tracking",
                    hive: Hive::Hkcu,
                    subkey: r"SOFTWARE\Microsoft\Windows\CurrentVersion\ContentDeliveryManager",
                    value_name: "SubscribedContent-353698Enabled",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(0),
                    default: TweakValue::Dword(1),
                    note: "",
                },
                Tweak {
                    name: "Disable Feedback Pop-ups",
                    description: "Stop feedback prompts",
                    hive: Hive::Hkcu,
                    subkey: r"SOFTWARE\Microsoft\Siuf\Rules",
                    value_name: "NumberOfSIUFInPeriod",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(0),
                    default: TweakValue::Dword(3),
                    note: "",
                },
                Tweak {
                    name: "Disable Settings Ads",
                    description: "Remove promotional suggestions in Settings",
                    hive: Hive::Hkcu,
                    subkey: r"SOFTWARE\Microsoft\Windows\CurrentVersion\ContentDeliveryManager",
                    value_name: "SubscribedContent-338393Enabled",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(0),
                    default: TweakValue::Dword(1),
                    note: "",
                },
                Tweak {
                    name: "Disable Clipboard History",
                    description: "Prevent OS logging copied text",
                    hive: Hive::Hkcu,
                    subkey: r"SOFTWARE\Microsoft\Clipboard",
                    value_name: "EnableClipboardHistory",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(0),
                    default: TweakValue::Dword(1),
                    note: "",
                },
            ],
        },

        // ═══════════════════════════════════════════════════════════════════════
        // 3. Gaming Optimization
        // ═══════════════════════════════════════════════════════════════════════
        Category {
            name: "Gaming Optimization",
            icon: "🎮",
            description: "Game DVR, GPU scheduling, input lag, MMCSS gaming profile",
            tweaks: vec![
                Tweak {
                    name: "Disable Game DVR",
                    description: "Turn off Xbox Game Bar recording",
                    hive: Hive::Hkcu,
                    subkey: r"System\GameConfigStore",
                    value_name: "GameDVR_Enabled",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(0),
                    default: TweakValue::Dword(1),
                    note: "",
                },
                Tweak {
                    name: "Disable Game DVR Policy",
                    description: "Block Game DVR at policy level",
                    hive: Hive::Hklm,
                    subkey: r"SOFTWARE\Policies\Microsoft\Windows\GameDVR",
                    value_name: "AllowGameDVR",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(0),
                    default: TweakValue::Dword(1),
                    note: "Creates key if missing",
                },
                Tweak {
                    name: "Disable Fullscreen Optimizations",
                    description: "Force exclusive fullscreen, bypass DWM",
                    hive: Hive::Hkcu,
                    subkey: r"SOFTWARE\Microsoft\Windows\CurrentVersion\GameDVR",
                    value_name: "GameDVR_FSEBehaviorMode",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(2),
                    default: TweakValue::Dword(0),
                    note: "Removes DWM overlay input lag",
                },
                Tweak {
                    name: "MMCSS GPU Priority",
                    description: "Elevate GPU thread priority for games",
                    hive: Hive::Hklm,
                    subkey: r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile\Tasks\Games",
                    value_name: "GPU Priority",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(8),
                    default: TweakValue::Dword(2),
                    note: "",
                },
                Tweak {
                    name: "MMCSS Game Priority",
                    description: "Set CPU priority for game threads",
                    hive: Hive::Hklm,
                    subkey: r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile\Tasks\Games",
                    value_name: "Priority",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(6),
                    default: TweakValue::Dword(2),
                    note: "",
                },
                Tweak {
                    name: "MMCSS Scheduling Category",
                    description: "Mark game threads as high-priority",
                    hive: Hive::Hklm,
                    subkey: r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile\Tasks\Games",
                    value_name: "Scheduling Category",
                    reg_type: RegValueType::String,
                    optimized: TweakValue::Str("High"),
                    default: TweakValue::Str("Medium"),
                    note: "",
                },
                Tweak {
                    name: "MMCSS SFIO Priority",
                    description: "Prioritize disk I/O for games",
                    hive: Hive::Hklm,
                    subkey: r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile\Tasks\Games",
                    value_name: "SFIO Priority",
                    reg_type: RegValueType::String,
                    optimized: TweakValue::Str("High"),
                    default: TweakValue::Str("Normal"),
                    note: "",
                },
                Tweak {
                    name: "HAGS (GPU Scheduling)",
                    description: "Enable GPU-level command scheduling",
                    hive: Hive::Hklm,
                    subkey: r"SYSTEM\CurrentControlSet\Control\GraphicsDrivers",
                    value_name: "HwSchMode",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(2),
                    default: TweakValue::Dword(1),
                    note: "Required for DLSS Frame Gen",
                },
                Tweak {
                    name: "Mouse Buffer Queue",
                    description: "Reduce mouse input buffer for lower latency",
                    hive: Hive::Hklm,
                    subkey: r"SYSTEM\CurrentControlSet\Services\mouclass\Parameters",
                    value_name: "MouseDataQueueSize",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(16),
                    default: TweakValue::Dword(100),
                    note: "",
                },
                Tweak {
                    name: "Keyboard Buffer Queue",
                    description: "Reduce keyboard input buffer",
                    hive: Hive::Hklm,
                    subkey: r"SYSTEM\CurrentControlSet\Services\kbdclass\Parameters",
                    value_name: "KeyboardDataQueueSize",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(16),
                    default: TweakValue::Dword(100),
                    note: "",
                },
                Tweak {
                    name: "Disable Mouse Acceleration",
                    description: "Set MouseSpeed to 0 for raw input",
                    hive: Hive::Hkcu,
                    subkey: r"Control Panel\Mouse",
                    value_name: "MouseSpeed",
                    reg_type: RegValueType::String,
                    optimized: TweakValue::Str("0"),
                    default: TweakValue::Str("1"),
                    note: "",
                },
                Tweak {
                    name: "Network Throttling Index",
                    description: "Remove MMCSS network packet limit",
                    hive: Hive::Hklm,
                    subkey: r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile",
                    value_name: "NetworkThrottlingIndex",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(0xFFFFFFFF),
                    default: TweakValue::Dword(10),
                    note: "Audio crackling? try 15-25",
                },
            ],
        },

        // ═══════════════════════════════════════════════════════════════════════
        // 4. Network & Internet
        // ═══════════════════════════════════════════════════════════════════════
        Category {
            name: "Network & Internet",
            icon: "🌐",
            description: "Nagle algorithm, DNS cache, TCP optimization, throughput",
            tweaks: vec![
                Tweak {
                    name: "Disable Nagle (TcpNoDelay)",
                    description: "Force instant small packet transmission",
                    hive: Hive::Hklm,
                    subkey: r"SYSTEM\CurrentControlSet\Services\Tcpip\Parameters",
                    value_name: "TcpNoDelay",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(1),
                    default: TweakValue::Dword(0),
                    note: "Global — per-adapter needs NIC subkey",
                },
                Tweak {
                    name: "Disable Delayed ACK",
                    description: "Acknowledge every packet immediately",
                    hive: Hive::Hklm,
                    subkey: r"SYSTEM\CurrentControlSet\Services\Tcpip\Parameters",
                    value_name: "TcpAckFrequency",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(1),
                    default: TweakValue::Dword(2),
                    note: "",
                },
                Tweak {
                    name: "Zero Delayed ACK Timer",
                    description: "Set delayed ACK timer to zero",
                    hive: Hive::Hklm,
                    subkey: r"SYSTEM\CurrentControlSet\Services\Tcpip\Parameters",
                    value_name: "TcpDelAckTicks",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(0),
                    default: TweakValue::Dword(2),
                    note: "",
                },
                Tweak {
                    name: "Disable Negative DNS Cache",
                    description: "Don't cache failed DNS lookups",
                    hive: Hive::Hklm,
                    subkey: r"SYSTEM\CurrentControlSet\Services\Dnscache\Parameters",
                    value_name: "MaxNegativeCacheTtl",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(0),
                    default: TweakValue::Dword(5),
                    note: "",
                },
                Tweak {
                    name: "DNS Cache TTL",
                    description: "Reduce positive DNS cache to 6 min",
                    hive: Hive::Hklm,
                    subkey: r"SYSTEM\CurrentControlSet\Services\Dnscache\Parameters",
                    value_name: "MaxCacheTtl",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(360),
                    default: TweakValue::Dword(86400),
                    note: "Default is 24h",
                },
                Tweak {
                    name: "RFC 1323 Window Scaling",
                    description: "Enable TCP window scaling & timestamps",
                    hive: Hive::Hklm,
                    subkey: r"SYSTEM\CurrentControlSet\Services\Tcpip\Parameters",
                    value_name: "Tcp1323Opts",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(3),
                    default: TweakValue::Dword(0),
                    note: "Allows receive window >64KB",
                },
            ],
        },

        // ═══════════════════════════════════════════════════════════════════════
        // 5. UI & Visual Tweaks
        // ═══════════════════════════════════════════════════════════════════════
        Category {
            name: "UI & Visual Tweaks",
            icon: "🎨",
            description: "Context menu, animations, taskbar, start menu, lock screen",
            tweaks: vec![
                Tweak {
                    name: "Restore Classic Context Menu",
                    description: "Bring back Win32 right-click menu",
                    hive: Hive::Hkcu,
                    subkey: r"Software\Classes\CLSID\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}\InprocServer32",
                    value_name: "(Default)",
                    reg_type: RegValueType::String,
                    optimized: TweakValue::Str(""),
                    default: TweakValue::DeleteValue,
                    note: "Needs Explorer restart",
                },
                Tweak {
                    name: "Explorer → This PC",
                    description: "Open Explorer to This PC instead of Quick Access",
                    hive: Hive::Hkcu,
                    subkey: r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced",
                    value_name: "LaunchTo",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(1),
                    default: TweakValue::Dword(2),
                    note: "",
                },
                Tweak {
                    name: "Disable Taskbar Animations",
                    description: "Remove taskbar transition animations",
                    hive: Hive::Hkcu,
                    subkey: r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced",
                    value_name: "TaskbarAnimations",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(0),
                    default: TweakValue::Dword(1),
                    note: "",
                },
                Tweak {
                    name: "Hide Widgets",
                    description: "Remove Widgets/Weather from taskbar",
                    hive: Hive::Hkcu,
                    subkey: r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced",
                    value_name: "TaskbarDa",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(0),
                    default: TweakValue::Dword(1),
                    note: "",
                },
                Tweak {
                    name: "Disable Widgets System",
                    description: "Policy-level block of Widgets Board",
                    hive: Hive::Hklm,
                    subkey: r"SOFTWARE\Policies\Microsoft\Dsh",
                    value_name: "AllowWidgets",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(0),
                    default: TweakValue::Dword(1),
                    note: "",
                },
                Tweak {
                    name: "Hide Teams Chat",
                    description: "Remove Teams Chat from taskbar",
                    hive: Hive::Hkcu,
                    subkey: r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced",
                    value_name: "TaskbarMn",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(0),
                    default: TweakValue::Dword(1),
                    note: "",
                },
                Tweak {
                    name: "Hide Search Box",
                    description: "Hide search from taskbar",
                    hive: Hive::Hkcu,
                    subkey: r"Software\Microsoft\Windows\CurrentVersion\Search",
                    value_name: "SearchboxTaskbarMode",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(0),
                    default: TweakValue::Dword(2),
                    note: "0=hide 1=icon 2=all",
                },
                Tweak {
                    name: "Show Seconds on Clock",
                    description: "Display seconds in taskbar clock",
                    hive: Hive::Hkcu,
                    subkey: r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced",
                    value_name: "ShowSecondsInSystemClock",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(1),
                    default: TweakValue::Dword(0),
                    note: "",
                },
                Tweak {
                    name: "Disable Web Search in Start",
                    description: "Remove Bing/web results from Start search",
                    hive: Hive::Hkcu,
                    subkey: r"Software\Policies\Microsoft\Windows\Explorer",
                    value_name: "DisableSearchBoxSuggestions",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(1),
                    default: TweakValue::Dword(0),
                    note: "",
                },
                Tweak {
                    name: "Disable Bing Search",
                    description: "Turn off Bing in Start Menu",
                    hive: Hive::Hkcu,
                    subkey: r"Software\Microsoft\Windows\CurrentVersion\Search",
                    value_name: "BingSearchEnabled",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(0),
                    default: TweakValue::Dword(1),
                    note: "",
                },
                Tweak {
                    name: "Disable Lock Screen",
                    description: "Skip lock screen, go to login directly",
                    hive: Hive::Hklm,
                    subkey: r"SOFTWARE\Policies\Microsoft\Windows\Personalization",
                    value_name: "NoLockScreen",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(1),
                    default: TweakValue::Dword(0),
                    note: "",
                },
            ],
        },

        // ═══════════════════════════════════════════════════════════════════════
        // 6. Bloatware & Debloat
        // ═══════════════════════════════════════════════════════════════════════
        Category {
            name: "Bloatware & Debloat",
            icon: "🧹",
            description: "Pre-installed apps, ads, Copilot, Cortana, spotlight",
            tweaks: vec![
                Tweak {
                    name: "Disable Silent App Installs",
                    description: "Block sponsored app installs (Candy Crush, etc.)",
                    hive: Hive::Hkcu,
                    subkey: r"SOFTWARE\Microsoft\Windows\CurrentVersion\ContentDeliveryManager",
                    value_name: "SilentInstalledAppsEnabled",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(0),
                    default: TweakValue::Dword(1),
                    note: "",
                },
                Tweak {
                    name: "Disable Windows Copilot",
                    description: "System-wide disable Copilot",
                    hive: Hive::Hklm,
                    subkey: r"SOFTWARE\Policies\Microsoft\Windows\WindowsCopilot",
                    value_name: "TurnOffWindowsCopilot",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(1),
                    default: TweakValue::Dword(0),
                    note: "",
                },
                Tweak {
                    name: "Disable Cortana",
                    description: "Disable legacy Cortana assistant",
                    hive: Hive::Hklm,
                    subkey: r"SOFTWARE\Policies\Microsoft\Windows\Windows Search",
                    value_name: "AllowCortana",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(0),
                    default: TweakValue::Dword(1),
                    note: "",
                },
                Tweak {
                    name: "Disable News & Interests",
                    description: "Block News/Interests data feed",
                    hive: Hive::Hklm,
                    subkey: r"SOFTWARE\Policies\Microsoft\Dsh",
                    value_name: "AllowNewsAndInterests",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(0),
                    default: TweakValue::Dword(1),
                    note: "",
                },
                Tweak {
                    name: "Remove Meet Now Icon",
                    description: "Strip Skype Meet Now from tray",
                    hive: Hive::Hklm,
                    subkey: r"SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer",
                    value_name: "HideSCAMeetNow",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(1),
                    default: TweakValue::Dword(0),
                    note: "",
                },
                Tweak {
                    name: "Disable Start Suggestions",
                    description: "Remove app suggestions from Start",
                    hive: Hive::Hkcu,
                    subkey: r"SOFTWARE\Microsoft\Windows\CurrentVersion\ContentDeliveryManager",
                    value_name: "SubscribedContent-338388Enabled",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(0),
                    default: TweakValue::Dword(1),
                    note: "",
                },
                Tweak {
                    name: "Disable Tips & Tricks",
                    description: "Stop Windows tips in Start/Settings",
                    hive: Hive::Hkcu,
                    subkey: r"SOFTWARE\Microsoft\Windows\CurrentVersion\ContentDeliveryManager",
                    value_name: "SubscribedContent-338389Enabled",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(0),
                    default: TweakValue::Dword(1),
                    note: "",
                },
                Tweak {
                    name: "Disable Welcome Screen Nag",
                    description: "Stop 'Get the most out of Windows' prompts",
                    hive: Hive::Hkcu,
                    subkey: r"SOFTWARE\Microsoft\Windows\CurrentVersion\UserProfileEngagement",
                    value_name: "ScoobeSystemSettingEnabled",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(0),
                    default: TweakValue::Dword(1),
                    note: "",
                },
                Tweak {
                    name: "Disable Desktop Spotlight",
                    description: "Stop rotating cloud wallpaper images",
                    hive: Hive::Hkcu,
                    subkey: r"Software\Policies\Microsoft\Windows\CloudContent",
                    value_name: "DisableSpotlightCollectionOnDesktop",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(1),
                    default: TweakValue::Dword(0),
                    note: "",
                },
                Tweak {
                    name: "Disable Lock Screen Overlays",
                    description: "Remove promotions from lock screen",
                    hive: Hive::Hkcu,
                    subkey: r"Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager",
                    value_name: "RotatingLockScreenOverlayEnabled",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(0),
                    default: TweakValue::Dword(1),
                    note: "",
                },
            ],
        },

        // ═══════════════════════════════════════════════════════════════════════
        // 7. Power Management & Security
        // ═══════════════════════════════════════════════════════════════════════
        Category {
            name: "Power Management & Security",
            icon: "🔋",
            description: "CPU boost, core parking, VBS, Defender, security mitigations",
            tweaks: vec![
                Tweak {
                    name: "Unhide Core Parking",
                    description: "Show CPU core parking in Power Options",
                    hive: Hive::Hklm,
                    subkey: r"SYSTEM\CurrentControlSet\Control\Power\PowerSettings\54533251-82be-4824-96c1-47b60b740d00\0cc5b647-c1df-4637-891a-dec35c318583",
                    value_name: "Attributes",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(0),
                    default: TweakValue::Dword(1),
                    note: "",
                },
                Tweak {
                    name: "Unlock CPU Boost Mode",
                    description: "Show hidden turbo boost aggressiveness",
                    hive: Hive::Hklm,
                    subkey: r"SYSTEM\CurrentControlSet\Control\Power\PowerSettings\54533251-82be-4824-96c1-47b60b740d00\be337238-0d82-4146-a960-4f3749d470c7",
                    value_name: "Attributes",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(2),
                    default: TweakValue::Dword(1),
                    note: "",
                },
                Tweak {
                    name: "Disable VBS",
                    description: "Disable Virtualization-Based Security",
                    hive: Hive::Hklm,
                    subkey: r"SOFTWARE\Policies\Microsoft\Windows\DeviceGuard",
                    value_name: "EnableVirtualizationBasedSecurity",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(0),
                    default: TweakValue::Dword(1),
                    note: "⚠ SECURITY RISK — 5-15% CPU gain",
                },
                Tweak {
                    name: "Disable HVCI",
                    description: "Disable Memory Integrity",
                    hive: Hive::Hklm,
                    subkey: r"SOFTWARE\Policies\Microsoft\Windows\DeviceGuard",
                    value_name: "HVCIMATRequired",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(0),
                    default: TweakValue::Dword(1),
                    note: "⚠ SECURITY RISK",
                },
                Tweak {
                    name: "Disable CFG",
                    description: "Disable Control Flow Guard",
                    hive: Hive::Hklm,
                    subkey: r"SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management",
                    value_name: "EnableCfg",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(0),
                    default: TweakValue::Dword(1),
                    note: "⚠ SECURITY RISK",
                },
                Tweak {
                    name: "Disable SEHOP",
                    description: "Disable SEH Overwrite Protection",
                    hive: Hive::Hklm,
                    subkey: r"SYSTEM\CurrentControlSet\Control\Session Manager\kernel",
                    value_name: "KernelSEHOPEnabled",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(0),
                    default: TweakValue::Dword(1),
                    note: "⚠ SECURITY RISK",
                },
                Tweak {
                    name: "Disable Windows Defender",
                    description: "Disable Defender Antivirus engine",
                    hive: Hive::Hklm,
                    subkey: r"SOFTWARE\Policies\Microsoft\Windows Defender",
                    value_name: "DisableAntiSpyware",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(1),
                    default: TweakValue::Dword(0),
                    note: "⚠ Only with 3rd-party AV",
                },
            ],
        },

        // ═══════════════════════════════════════════════════════════════════════
        // 8. Startup & Services
        // ═══════════════════════════════════════════════════════════════════════
        Category {
            name: "Startup & Services",
            icon: "🚀",
            description: "Boot speed, shutdown time, service isolation, fast startup",
            tweaks: vec![
                Tweak {
                    name: "Eliminate Startup Delay",
                    description: "Remove delay before launching startup apps",
                    hive: Hive::Hkcu,
                    subkey: r"Software\Microsoft\Windows\CurrentVersion\Explorer\Serialize",
                    value_name: "StartupDelayInMSec",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(0),
                    default: TweakValue::Dword(10000),
                    note: "Creates Serialize key",
                },
                Tweak {
                    name: "Disable Fast Startup",
                    description: "Force true cold boot to prevent leaks",
                    hive: Hive::Hklm,
                    subkey: r"SYSTEM\CurrentControlSet\Control\Session Manager\Power",
                    value_name: "HiberbootEnabled",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(0),
                    default: TweakValue::Dword(1),
                    note: "Cleaner boot, slightly longer",
                },
                Tweak {
                    name: "Auto-End Tasks",
                    description: "Force-close unresponsive apps on shutdown",
                    hive: Hive::Hkcu,
                    subkey: r"Control Panel\Desktop",
                    value_name: "AutoEndTasks",
                    reg_type: RegValueType::String,
                    optimized: TweakValue::Str("1"),
                    default: TweakValue::Str("0"),
                    note: "",
                },
                Tweak {
                    name: "WaitToKillAppTimeout",
                    description: "Reduce wait to kill apps (2s)",
                    hive: Hive::Hkcu,
                    subkey: r"Control Panel\Desktop",
                    value_name: "WaitToKillAppTimeout",
                    reg_type: RegValueType::String,
                    optimized: TweakValue::Str("2000"),
                    default: TweakValue::Str("20000"),
                    note: "",
                },
                Tweak {
                    name: "HungAppTimeout",
                    description: "Consider app hung after 2s",
                    hive: Hive::Hkcu,
                    subkey: r"Control Panel\Desktop",
                    value_name: "HungAppTimeout",
                    reg_type: RegValueType::String,
                    optimized: TweakValue::Str("2000"),
                    default: TweakValue::Str("5000"),
                    note: "",
                },
                Tweak {
                    name: "WaitToKillServiceTimeout",
                    description: "Reduce wait to kill services (2s)",
                    hive: Hive::Hklm,
                    subkey: r"SYSTEM\CurrentControlSet\Control",
                    value_name: "WaitToKillServiceTimeout",
                    reg_type: RegValueType::String,
                    optimized: TweakValue::Str("2000"),
                    default: TweakValue::Str("20000"),
                    note: "",
                },
                Tweak {
                    name: "SvcHostSplitThreshold",
                    description: "Isolate services into separate processes",
                    hive: Hive::Hklm,
                    subkey: r"SYSTEM\CurrentControlSet\Control",
                    value_name: "SvcHostSplitThresholdInKB",
                    reg_type: RegValueType::Dword,
                    optimized: TweakValue::Dword(8400000),
                    default: TweakValue::Dword(3670016),
                    note: "Adjust: (GB * 1024²) / 2",
                },
            ],
        },
    ]
}
