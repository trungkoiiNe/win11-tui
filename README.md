<div align="center">

# win11-tui

### Your potato PC called. It wants its FPS back.

```
 ██╗    ██╗██╗███╗   ██╗██╗
 ██║    ██║██║████╗  ██║██║
 ██║ █╗ ██║██║██╔██╗ ██║██║
 ██║███╗██║██║██║╚██╗██║╚═╝
 ╚███╔███╔╝██║██║ ╚████║██╗
  ╚══╝╚══╝ ╚═╝╚═╝  ╚═══╝╚═╝
```

**74 Registry Tweaks. 8 Categories. One terminal. Zero bloat.**

[![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust)](https://rust-lang.org)
[![Windows](https://img.shields.io/badge/Windows-11-0078D4?style=flat&logo=windows)](https://microsoft.com)

</div>

---

## What is this?

A **terminal-based Windows 11 optimizer** that slaps your registry into shape. No electron app. No 200MB installer. No "sign up for our newsletter." Just a single 593KB binary that runs in your terminal and makes Windows feel like it actually wants to be used.

**Built for people whose PC sounds like a jet engine just opening Task Manager.**

---

## What does it actually do?

| Category | Tweaks | What you get |
|----------|--------|--------------|
| **Performance & Speed** | 8 | Priority separation, disable prefetch/superfetch, kill power throttling |
| **Privacy & Telemetry** | 11 | Kill telemetry, ads, activity tracking, clipboard spyware |
| **Gaming Optimization** | 13 | Disable Game DVR, MMCSS tuning, HAGS, mouse acceleration off |
| **Network & Internet** | 6 | Kill Nagle's algorithm, zero-delay ACK, window scaling |
| **UI & Visual Tweaks** | 11 | Classic context menu, kill bloat suggestions, spotlight ads |
| **Bloatware & Debloat** | 7 | Strip pre-installed garbage |
| **Power & Security** | 7 | Unhide core parking, unlock CPU boost, disable VBS/HVCI |
| **Startup & Services** | 8 | Kill startup delay, auto-end hung tasks, faster shutdown |

---

## Screenshot

```
+-- Win11 Optimizer | 0/74 applied | Dashboard -------------------+
|                                                                 |
|  * Performance & Speed                          3/8 applied     |
|  o Privacy & Telemetry                          0/11 applied    |
|  o Gaming Optimization                          0/13 applied    |
|  o Network & Internet                           0/6 applied     |
|  o UI & Visual Tweaks                           0/11 applied    |
|  o Bloatware & Debloat                          0/7 applied     |
|  o Power Management & Security                  0/7 applied     |
|  o Startup & Services                           0/8 applied     |
|                                                                 |
| [up/dn] Navigate  [Enter] Open  [A] Apply All  [q] Quit        |
| Ready                                                           |
+-----------------------------------------------------------------+
```

---

## Quick Start

### Prerequisites
- **Windows 11** (10 might work, but you're on your own)
- **Rust** installed ([rustup.rs](https://rustup.rs))
- **Run as Administrator** (it needs registry access, duh)

### Build & Run

```bash
git clone https://github.com/YOUR_USERNAME/win11-tui.git
cd win11-tui
cargo build --release

# NOW RUN IT AS ADMIN. seriously.
target/release/win11-tui.exe
```

---

## Controls

| Key | Action |
|-----|--------|
| `Up/Down` or `j/k` | Navigate |
| `Enter` / `Space` | Open category / Toggle tweak |
| `A` | Apply ALL optimizations |
| `R` | Revert ALL to Windows defaults |
| `Esc` / `b` | Go back |
| `g` / `G` | Jump to top / bottom |
| `q` | Quit |
| `Ctrl+C` | Force quit (you animal) |

---

## Safety First

- **Every tweak is reversible** -- hit `R` to revert everything to Windows defaults
- **Confirmation dialogs** -- no accidental nukes
- **Individual toggles** -- cherry-pick what you want
- **Status indicators** -- see what's applied (*) and what's default (o) at a glance
- **No telemetry, no network calls** -- this thing is offline-only

> Modifying the registry can break things. Back up your system. If you don't know what `Win32PrioritySeparation` means, maybe start with "Apply All" and pray.

---

## Tech Stack

- **Rust** -- because we're not savages
- **ratatui** -- TUI framework (this thing is pretty)
- **crossterm** -- terminal magic
- **winreg** -- direct Windows registry access
- **tokio** -- async runtime (yes, for a TUI, fight me)

---

## Project Structure

```
src/
  main.rs          # Entry point, terminal setup
  app.rs           # App state, Screen/Focus hierarchy, key handling
  tweaks.rs        # All 74 registry tweaks (the meat)
  registry.rs      # Low-level registry read/write
  theme.rs         # NVIDIA green aesthetic (you know the one)
  event.rs         # Keyboard event handler
  ui/
    mod.rs         # Layout: title | content | nav | status
    dashboard.rs   # Category overview screen
    category.rs    # Individual tweak list screen
```

---

## Contributing

1. Fork it
2. `cargo fmt` your code (we're not animals)
3. Add your tweak to `tweaks.rs`
4. PR it

**Adding a new tweak is literally 12 lines of code.** Check the existing ones for the pattern.

---

## License

MIT -- do whatever you want. Just don't blame me when your PC runs so fast it achieves liftoff.

---

<div align="center">

**Made with a potato, by a potato PC user, for potato PC users**

*"I upgraded from a HDD to an SSD and my PC is still slow"*
*-- everyone who needs this tool*

</div>
