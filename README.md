
# AURORA
<div align="center">
  <img src="https://raw.githubusercontent.com/ahmoodio/aurora/main/assets/logo.png" alt="Aurora logo" width="180"/>

  <p><em>A modern, Wayland-first GUI for Arch Linux package management</em></p>

  [![last-commit](https://img.shields.io/github/last-commit/ahmoodio/aurora?style=flat&logo=git&logoColor=white&color=7C3AED)](https://github.com/ahmoodio/aurora)
  [![repo-top-language](https://img.shields.io/github/languages/top/ahmoodio/aurora?style=flat&color=7C3AED)](https://github.com/ahmoodio/aurora)
  [![license](https://img.shields.io/github/license/ahmoodio/aurora?style=flat&color=7C3AED)](https://github.com/ahmoodio/aurora/blob/main/LICENSE)
  [![Arch Linux](https://img.shields.io/badge/Arch%20Linux-1793D1?logo=arch-linux&logoColor=white)](https://archlinux.org/)
  [![Wayland](https://img.shields.io/badge/Wayland-ready-success)](https://wayland.freedesktop.org/)

  <p><em>Built with:</em></p>
  <img alt="Rust" src="https://img.shields.io/badge/Rust-000000.svg?style=flat&logo=rust&logoColor=white">
  <img alt="GTK4" src="https://img.shields.io/badge/GTK4-4A86CF.svg?style=flat&logo=gtk&logoColor=white">
  <img alt="libadwaita" src="https://img.shields.io/badge/libadwaita-3D3D3D.svg?style=flat">
  <img alt="Pacman" src="https://img.shields.io/badge/pacman-1793D1.svg?style=flat&logo=arch-linux&logoColor=white">

  <p>
    <a href="#-overview">🌌 Overview</a> •
    <a href="#-features">✨ Features</a> •
    <a href="#-installation">📥 Installation</a> •
    <a href="#-security-model">🔐 Security</a> •
    <a href="#-development">🧩 Development</a>
  </p>
</div>

---

## 🌌 Overview

**Aurora** is a **modern, native GUI package manager for Arch Linux**, designed from the ground up for **Wayland**, **GTK4**, and **libadwaita**.

Unlike traditional wrappers, Aurora focuses on:
- clarity over magic  
- safety over convenience  
- transparency over hidden automation  

Aurora integrates:
- **pacman** for official repositories  
- **AUR helpers (yay / paru)** for community packages  
- **AppStream metadata** for rich visuals (icons, screenshots, descriptions)

All wrapped in a clean, store-like interface inspired by modern Linux desktops.

---

## ✨ Features

- **Unified Package Management**
  - Manage official repo packages (pacman)
  - Manage AUR packages via yay or paru
  - Clear source badges: Repo / AUR

- **Wayland-First UI**
  - Built with GTK4 + libadwaita
  - Smooth animations and adaptive layouts
  - Native feel on GNOME, COSMIC, and modern Wayland compositors

- **Rich App Details**
  - AppStream integration for icons and screenshots
  - Clean app detail pages with versions, descriptions, and metadata

- **Transaction Queue & Review**
  - Queue installs, removals, and updates
  - Review all actions before execution
  - No hidden system changes

- **Live Logs & Feedback**
  - Real-time stdout/stderr streaming
  - Copy or save logs for debugging
  - Clear error messages (no silent failures)

- **Configurable AUR Backend**
  - Switch between `yay` and `paru`
  - Interactive by default (no forced `--noconfirm`)

---
## 🌟 Why Aurora vs Pamac / Octopi?

Aurora is **not** trying to replace Pamac or Octopi — it exists to solve a *different set of problems*.

### 🆚 Aurora vs Pamac

| Aurora | Pamac |
|------|-------|
| Wayland-first (GTK4 + libadwaita) | Primarily GTK3 / mixed backends |
| Focused on transparency and safety | Convenience-first abstractions |
| Explicit transaction review | Often hides low-level pacman details |
| Strict polkit helper model | Broader privileged execution |
| Designed for Arch users | Originally designed for Manjaro |

**Aurora’s philosophy:**  
> *Show the user exactly what will happen — and let them decide.*

---

### 🆚 Aurora vs Octopi

| Aurora | Octopi |
|------|-------|
| Modern, store-like UI | Traditional utility-style UI |
| AppStream icons & screenshots | Text-focused package lists |
| Designed for Wayland + GTK4 | Qt / X11-centric |
| Guided install & review flow | Power-user workflows |
| Visual clarity for discovery | Optimized for speed & control |

**Aurora’s philosophy:**  
> *Package management can be powerful **and** approachable.*

---

### 🎯 Who Aurora Is For

Aurora is ideal if you want:
- A **modern, native Wayland experience**
- Visual app discovery with real metadata
- Clear, reviewable system changes
- A GUI that respects Arch’s philosophy without hiding it

Aurora intentionally avoids:
- Running entire sessions as root
- Silent background changes
- “Magic” automation without visibility

---

### 🧠 Design Goal

Aurora aims to sit **between**:
- **Pamac** (convenience)
- **Octopi** (raw control)

…offering a **safe, modern middle ground** for Arch users who want clarity without friction.

---

## 📥 Installation

### From AUR (recommended)

Using your favorite AUR helper:

```bash
yay -S aurora-gui-git
````

or

```bash
paru -S aurora-gui-git
```

---

### Manual build (for testing)

```bash
git clone https://github.com/ahmoodio/aurora.git
cd aurora
cargo build --release
./target/release/aurora
```

> ⚠️ Manual builds do **not** install polkit rules or desktop files.

---

## 🔐 Security Model

Aurora is designed with **least privilege** in mind:

* The GUI runs **unprivileged**
* All system-level operations are executed via:

  * a **dedicated helper binary**
  * invoked through **polkit (pkexec)**
* Only **whitelisted pacman actions** are allowed
* No shell execution, no arbitrary commands

This makes Aurora **safer than traditional GUI wrappers** that run entire sessions as root.

---

## 🧩 Development

### Requirements

* Rust (stable)
* GTK4
* libadwaita
* pkgconf

### Run in development mode

```bash
cargo run
```

---

## 📄 License

MIT License.
See [`LICENSE`](https://github.com/ahmoodio/aurora/blob/main/LICENSE).

---

## ⭐ Support

If Aurora helps you:

* ⭐ Star the repo
* 🐞 Report issues
* 💡 Suggest features
* 🔧 Open pull requests




