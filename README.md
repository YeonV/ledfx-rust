# LedFx-rs

> An audio-reactive LED effect engine, rewritten in Rust for high performance, safety, and a modern, cross-platform experience.

[![Build Status](https://github.com/YeonV/ledfx-rust/actions/workflows/builder.yml/badge.svg)](https://github.com/YeonV/ledfx-rust/actions/workflows/builder.yml)
[![Latest Release](https://img.shields.io/github/v/release/YeonV/ledfx-rust)](https://github.com/YeonV/ledfx-rust/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Inspired by the original Python LedFx, this project is a ground-up rewrite that leverages the power of Rust for its core processing and Tauri for a modern, unified frontend across all platforms. The result is a highly efficient, real-time lighting controller with a sophisticated feature set.

![Application Screenshot](https://raw.githubusercontent.com/YeonV/ledfx-rust/main/assets/screenshot.png)

---

## ✨ Key Features

This project was built from the ground up with a focus on performance, flexibility, and a best-in-class user experience. Here are some of the amazing things we've accomplished:

*   **🚀 High-Performance Rust Core:** The entire audio processing and effect rendering engine is written in Rust, offering memory safety and blazing-fast performance suitable for real-time applications, even on low-powered devices like a Raspberry Pi.

*   **🎧 Advanced Real-time DSP:** A sophisticated, multi-threaded audio pipeline captures and analyzes audio with minimal latency. The entire DSP chain is fully configurable via the UI, including:
    *   FFT Size (trading time vs. frequency resolution)
    *   Number of Frequency Bands
    *   Min/Max Frequency Range
    *   Forced Sample Rate (for recreating the "LedFx Original" feel)
    *   Multiple Filterbank Types (`Balanced`, `Precision`, `Vocal`, `Blade`)
    *   A fully custom **"BladePlus" Filterbank Designer** with a real-time curve visualizer.

*   **🎨 Dynamic, Schema-Driven Effects:** The effects system is fully modular. New effects can be added purely in Rust. Each effect defines its own configuration and UI "schema" (`sliders`, `color pickers`, etc.), and the frontend dynamically generates the settings panel. **No frontend code changes are needed to add new effects.**

*   **💡 Powerful Virtualization Engine:** Abstract away your physical hardware. Create "Virtual Strips" by composing segments from one or more physical LED controllers, adding gaps, and re-arranging them to match your physical layout perfectly.

*   **🎬 Creative Workflow with Presets & Scenes:**
    *   **Presets:** Save and load your favorite effect configurations as reusable presets. Comes with built-in presets to get you started!
    *   **Scenes:** Take a snapshot of your entire setup—which effects and presets are active on which virtuals—and recall it with a single click.
    *   **Smart Scene Saving:** An intelligent dialog helps you manage "dirty" (unsaved) effect settings, encouraging you to create presets on the fly and keep your library organized.

*   **🌐 Full Remote Control via HTTP API:** A built-in asynchronous HTTP REST API server allows for deep integration with third-party tools.
    *   Activate scenes (`GET /scenes/{id}/activate`) from a browser bookmark or a simple script.
    *   Control virtuals, get device lists, and more.
    *   Perfect for integration with Stream Decks, Touch Portal, Home Assistant, or custom scripts.

*   **🖥️ Truly Cross-Platform:** Thanks to Tauri and a careful CI/CD pipeline, this application is built and tested for:
    *   Windows (x64)
    *   macOS (Intel & Apple Silicon)
    *   Linux (x64 AppImage)
    *   Raspberry Pi (ARM64 AppImage)
    *   Android

*   **💅 Modern, Customizable UI:**
    *   Built with React, TypeScript, Vite, and Zustand for a fast and reactive experience.
    *   Features a fully **customizable TopBar**. Long-press or right-click to enter "Edit Mode," then drag-and-drop to reorder buttons or toggle their visibility. Your layout is saved and remembered.

---

## 🚀 Getting Started (for Developers)

To get a local development environment running, follow these steps.

### Prerequisites

*   **Rust:** Install the Rust toolchain via [rustup](https://rustup.rs/).
*   **Node.js and Yarn:** Install Node.js (v18+) and enable Corepack to get Yarn (`corepack enable`).
*   **System Dependencies:** Follow the [Tauri setup guide](https://tauri.app/v2/guides/getting-started/prerequisites) for your specific operating system to install the necessary webview and build tools.

### Installation & Running

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/YeonV/ledfx-rust.git
    cd ledfx-rust
    ```

2.  **Install frontend dependencies:**
    ```bash
    yarn install
    ```

3.  **Run the development server:**
    ```bash
    yarn dev
    ```
    This will compile the Rust backend and launch the Tauri application with hot-reloading for both the frontend and backend.

---

## 🛠️ Tech Stack

| Backend | Frontend |
| :--- | :--- |
| 🦀 **Rust** | ⚛️ **React** |
| 🪟 Tauri (v2) | 🔷 **TypeScript** |
| ⚡ Tokio | 🚀 **Vite** |
| 🗼 Axum (HTTP Server) | 🐻 **Zustand** (State Management) |
| 🎧 CPAL (Audio Capture) | 🎨 **MUI** (Component Library) |
| 🎼 RustFFT (DSP) | 🖐️ **dnd-kit** (Drag and Drop) |
| 🧬 Serde (Serialization) | |
| 🌉 Specta (Type Generation) | |


---

## 📁 Project Structure

This repository is a Tauri project with a Rust backend and a TypeScript/React frontend.

*   `src/`: Contains all frontend code (React components, hooks, state, etc.).
*   `src-tauri/src/`: Contains all backend Rust code.
    *   `api/`: The Axum HTTP API server.
    *   `audio/`: Cross-platform audio capture and DSP logic.
    *   `engine/`: The core real-time rendering loop and state machine.
    *   `effects/`: The modular effects system and effect definitions.
    *   `store/`: Data structures and persistence logic.
*   `scripts/`: Contains the `generate-effects.js` script that auto-generates Rust and TypeScript code to integrate new effects.

---

## 🤝 Contributing

Contributions are welcome! If you have a feature request, found a bug, or want to add a new effect, please open an issue to discuss it first. Pull requests are greatly appreciated.

## 📜 License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.