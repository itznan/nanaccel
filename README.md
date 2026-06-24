<p align="center">
  <img src="logo.png" alt="nanaccel Logo" width="200"/>
</p>

# nanaccel 🚀

**nanaccel** is a next-generation, high-performance video CLI tool and engine built in pure Rust, designed as a lightweight and zero-dependency competitor to FFmpeg for hardware-accelerated video decoding, rendering, and encoding on NVIDIA GPUs. 

Unlike other software that wraps or spawns FFmpeg subprocesses, **nanaccel** is compiled into a single standalone binary that interacts directly with Windows Media Foundation (WMF), Direct3D 11, and NVIDIA NVENC APIs at the native C interface level.

---

## 💎 Core Philosophy

1. **0% CPU Decoder/Encoder Overhead**: Video decoding, scaling, pixel format conversion, and encoding are kept entirely on the GPU. Zero frames are copied back to CPU memory during transcoding.
2. **No FFmpeg Subprocesses**: Fully standalone engine with no dependency on local FFmpeg installations or external codecs.
3. **NVIDIA GPU Mandatory**: Strict hardware checking. If a compatible NVIDIA GPU is not detected via `nvcuda` or `nvidia-smi` at startup, the program prints `gpu not detected` and exits.

---

## 📂 Project Directory Structure

```text
nanaccel/
├── .github/
│   └── workflows/
│       └── ci.yml          # GitHub Actions CI workflow for builds & style checks
├── docs/                   # Technical documentation
│   ├── architecture.md     # Pipeline design and GPU flow diagram
│   ├── getting_started.md  # Setup, requirements, and basic commands
│   └── shaders.md          # HLSL Pixel Shader writing guide for video processing
├── src/
│   ├── main.rs             # CLI router and live GPU telemetry reporter
│   ├── gpu_pipeline/       # Direct WMF NVDEC -> D3D11 VPP -> NVENC GPU pipelines
│   ├── commands/           # Modular subcommand parsing and handler functions
│   └── mux.rs              # Native MP4 muxer for wrapping GPU stream packets
├── nanaccel-windows-x86_64/
│   ├── nanaccel.exe        # Pre-built release binary
│   ├── grayscale.hlsl      # Sample HLSL pixel shader
│   └── verify.py           # Automated diagnostic python script
├── .gitattributes          # Line-ending normalization
├── .gitignore              # Dependency targets and workspace exclusions
├── Cargo.toml              # Build configurations and dependency definitions
└── README.md               # User manual and project description
```

---

## 🚀 Getting Started

Launch commands via the CLI to check GPU capabilities, play, transcode, screenshot, or process HLSL shaders:

### 1. Show GPU Info & Telemetry
Queries NVIDIA system telemetry for driver versions, active core utilization, VRAM metrics, power draw, and temperature:
```bash
nanaccel info
```

### 2. GPU-Accelerated Interactive Playback
Decodes and presents video directly into a hardware-accelerated Direct3D 11 window at native frame rates:
```bash
nanaccel play path/to/video.mp4
```
**Interactive Keyboard Controls:**
* `Spacebar` : Play / Pause the video playback.
* `Esc` / `Q` : Instantly close the player window and exit.

**Options:**
* `--no-audio` : Disables audio rendering.
* `--loop` : Infinite loop playback.

### 3. GPU-to-GPU HLSL Filter Pipeline (Shaders)
Compile a custom HLSL pixel shader file dynamically and run it on the video stream entirely on the GPU:
```bash
nanaccel shader input.mp4 output.mp4 path/to/shader.hlsl
```
*Processes the video frame loop completely on the GPU: Decoder (NV12) $\rightarrow$ Video Processor (RGBA) $\rightarrow$ Shader RTV (RGBA) $\rightarrow$ Video Processor (NV12) $\rightarrow$ NVENC.*

### 4. Pure GPU Transcoding
Transcodes H.264 or HEVC inputs directly on the GPU, with optional hardware scaling and custom bitrates:
```bash
nanaccel transcode input.mp4 output.mp4 -c h264 -p p4 -b 5M --scale 1280x720
```
**Options:**
* `-c, --codec <codec>` : Target codec (`h264`, `hevc`). Default is H.264.
* `-p, --preset <preset>` : NVENC preset configuration from `p1` (fastest) to `p7` (highest quality).
* `-b, --bitrate <bitrate>` : Target bitrate (e.g. `5M`, `800k`, `3000000`).
* `--scale <width>x<height>` : High-speed GPU resizing (e.g., `--scale 1920x1080`).

### 5. GPU-Accelerated Screenshot (Frame Extraction)
Decodes a specific frame on the GPU and saves it directly to a high-fidelity image file:
```bash
nanaccel screenshot input.mp4 output.png -t 5000
```
**Options:**
* `-t, --time <ms>` : Timestamp in milliseconds (default: 0).

---

## 📦 Building

To build the executable, ensure you have the **Windows SDK** and **C++ Build Tools** installed along with **Rust** and a compatible **NVIDIA GPU Driver**.

```bash
# Build the binary in release mode
cargo build --release
```

The compiled binary will be located at `target/release/nanaccel.exe`.

---

## 📄 License

This project is licensed under the MIT License - see the `LICENSE` file for details.
