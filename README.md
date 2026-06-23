# nann 🚀

**nann** is a next-generation, high-performance video CLI tool and engine built in pure Rust, designed as a lightweight and zero-dependency competitor to FFmpeg for hardware-accelerated video decoding, rendering, and encoding on NVIDIA GPUs. 

Unlike other software that wraps or spawns FFmpeg subprocesses, **nann** is compiled into a single standalone binary that interacts directly with Windows Media Foundation (WMF), Direct3D 11, and NVIDIA NVENC APIs at the native C interface level.

---

## 💎 Core Philosophy

1. **0% CPU Decoder/Encoder Overhead**: Video decoding, scaling, pixel format conversion, and encoding are kept entirely on the GPU. Zero frames are copied back to CPU memory during transcoding.
2. **No FFmpeg Subprocesses**: Fully standalone engine with no dependency on local FFmpeg installations or external codecs.
3. **NVIDIA GPU Mandatory**: Strict hardware checking. If a compatible NVIDIA GPU is not detected via `nvcuda` or `nvidia-smi` at startup, the program prints `gpu not detected` and exits.

---

## 📂 Project Directory Structure

A clean, standardized Rust and Git repository layout optimized for collaborative development:

```text
nann/
├── .github/
│   └── workflows/
│       └── ci.yml          # GitHub Actions CI workflow for builds & style checks
├── src/
│   ├── main.rs             # CLI router and live GPU telemetry reporter
│   ├── gpu_pipeline.rs     # Core GPU pipeline (WMF NVDEC -> D3D11 VPP -> NVENC)
│   └── mux.rs              # Native MP4 muxer for wrapping GPU stream packets
├── .gitattributes          # Line-ending normalization and binary files handling
├── .gitignore              # Standard Rust/Cargo git ignores and video file exclusions
├── Cargo.toml              # Rust project manifest & external crate dependencies
├── README.md               # Project documentation and specifications
└── rustfmt.toml            # Code styling rules for cargo fmt
```

---

## 🛠️ Technology Stack

* **Rust**: Memory-safe system language running edition 2024.
* **Windows Media Foundation (WMF)**: Direct hardware-accelerated demuxing and decoding (NVDEC) using the DXGI Device Manager.
* **Direct3D 11 Video Processing (VPP)**: Real-time, GPU-driven color space conversion (`NV12` ⇄ `BGRA`) and resolution downscaling.
* **NVIDIA NVENC SDK**: Direct access to hardware encoding chips.
* **Native MP4 Muxer**: Fast and low-overhead container writer wrapping encoded H.264 streams.

---

## 🚀 Usage

### 1. Show GPU Info & Telemetry
Queries NVIDIA system telemetry for driver versions, active core utilization, VRAM metrics, power draw, and temperature:
```bash
nann info
```

### 2. GPU-Accelerated Playback
Decodes and presents video directly into a Direct3D 11 window at native frame rates:
```bash
nann play path/to/video.mp4
```
**Options:**
* `--no-audio` : Disables audio rendering.
* `--loop` : Infinite loop playback.

### 3. Pure GPU Transcoding
Transcodes H.264 or HEVC inputs directly on the GPU, with optional hardware scaling and custom bitrates:
```bash
nann transcode input.mp4 output.mp4 -c h264 -p p4 -b 5M --scale 1280x720
```
**Options:**
* `-c, --codec <codec>` : Target codec (`h264`, `hevc`). Default is H.264.
* `-p, --preset <preset>` : NVENC preset configuration from `p1` (fastest) to `p7` (highest quality).
* `-b, --bitrate <bitrate>` : Target bitrate (e.g. `5M`, `800k`, `3000000`).
* `--scale <width>x<height>` : High-speed GPU resizing. (e.g., `--scale 1920x1080`).

---

## 📦 Building

To build the executable, ensure you have the **Windows SDK** and **C++ Build Tools** installed along with **Rust** and a compatible **NVIDIA GPU Driver**.

```bash
# Build the binary in release mode
cargo build --release
```

The compiled binary will be located at `target/release/nann.exe`.

---

## 📄 License

This project is licensed under the MIT License - see the `LICENSE` file for details.
