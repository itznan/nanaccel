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

To build the executable, ensure you have the **Windows SDK** and **C++ Build Tools** installed along with **Rust** and a compatible **NVIDIA GPU Driver**.

```bash
# Build the binary in release mode
cargo build --release
```

The compiled binary will be located at `target/release/nanaccel.exe`.

---

## 🛠️ CLI Subcommands & Features Reference

**nanaccel** exposes a wide range of GPU-accelerated video/audio subcommands:

### 1. Show GPU Info & Telemetry (`info`)
Queries NVIDIA system telemetry for driver versions, active core utilization, VRAM metrics, power draw, and temperature:
```bash
nanaccel info
```

### 2. GPU-Accelerated Interactive Playback (`play`)
Decodes and presents video directly into a hardware-accelerated Direct3D 11 window at native frame rates:
```bash
nanaccel play path/to/video.mp4 [options]
```
* **Interactive Keyboard Controls:**
  * `Spacebar` : Play / Pause the video playback.
  * `Esc` / `Q` : Instantly close the player window and exit.
* **Options:**
  * `-d, --decoder <decoder>` : Specify a custom decoder.
  * `--no-audio` : Disables audio rendering.
  * `--loop` : Infinite loop playback.

### 3. Pure GPU Transcoding (`transcode`)
Transcodes H.264 or HEVC inputs directly on the GPU, with optional hardware scaling and custom bitrates. Zero B-frames are used to maintain latency and prevent timing drift:
```bash
nanaccel transcode input.mp4 output.mp4 [options]
```
* **Options:**
  * `-c, --codec <codec>` : Target video codec (`h264`, `hevc`). Default is `h264`.
  * `-p, --preset <preset>` : NVENC preset configuration from `p1` (fastest) to `p7` (highest quality).
  * `-b, --bitrate <bitrate>` : Target bitrate (e.g. `5M`, `800k`, `3000000`).
  * `--scale <width>x<height>` : High-speed GPU resizing (e.g. `1280x720`).

### 4. GPU-to-GPU HLSL Filter Pipeline (`shader`)
Compile a custom HLSL pixel shader file dynamically and run it on the video stream entirely on the GPU:
```bash
nanaccel shader input.mp4 output.mp4 path/to/shader.hlsl
```
*Processes the video frame loop completely on the GPU: Decoder (NV12) $\rightarrow$ Video Processor (RGBA) $\rightarrow$ Shader RTV (RGBA) $\rightarrow$ Video Processor (NV12) $\rightarrow$ NVENC.*

### 5. GPU-Accelerated Screen Recording (`record`)
Records your primary desktop monitor directly using DXGI Desktop Duplication and NVENC ARGB encoding. The loop utilizes a self-correcting catch-up timer to capture at stable, accurate real-time speeds:
```bash
nanaccel record output_recording.mp4 [options]
```
* **Options:**
  * `--fps <fps>` : Target frame rate (e.g. `30`, `60`). Default is `60`.
  * `-b, --bitrate <bitrate>` : Target encoding bitrate (e.g. `10M`, `4M`). Default is `8M`.
  * `-d, --duration <seconds>` : Recording duration limit in seconds. Default is `5`.

### 6. GPU-Accelerated Screenshot Frame Extraction (`screenshot`)
Decodes a specific frame on the GPU and saves it directly to a high-fidelity image file using Windows Imaging Component (WIC):
```bash
nanaccel screenshot input.mp4 output.png [options]
```
* **Options:**
  * `-t, --time <ms>` : Timestamp in milliseconds (default: 0).
  * Supported output formats: `PNG`, `JPEG`, `BMP`, `TIFF`, `WebP`, `GIF`, `HEIF`, `AVIF`.

### 7. Native Audio DSP Processor (`audio`)
Performs native digital signal processing (DSP) on audio tracks using `Symphonia` for decoding and writes to a WAV format output:
```bash
nanaccel audio <operation> input.mp3 output.wav [options]
```
* **Available Operations:**
  * `volume` (volume-control)
  * `denoise` (noise-reduction)
  * `compress` (compression)
  * `limit` (limiter)
  * `eq` (equalizer)
  * `pitch` (pitch-shift)
  * `tempo` (tempo-change)
  * `reverb`
  * `echo`
  * `bass` (bass-boost)
  * `silencedetect` (silence-detection)
  * `normalize` (audio-normalization)
* **Options:**
  * `--volume <val>` : Volume multiplier or decibel adjustment (e.g. `1.5` or `-6dB`).
  * `--nr <val>` : Noise reduction strength parameters.
  * `--threshold <val>` : Noise gate or compressor threshold (e.g. `-21dB`).
  * `--ratio <val>` : Compressor dynamic ratio (e.g. `3:1`).
  * `--limit <val>` : Hard limiter threshold amplitude limit (e.g. `0.1`).
  * `--gain <val>` : Equalizer frequency gain (dB).
  * `--freq <val>` : Target equalizer frequency.
  * `--pitch <val>` : Pitch shifting multiplier factor (e.g. `1.2`).
  * `--tempo <val>` : Tempo scaling factor (e.g. `0.9`).
  * `--silence-db <val>` : Silence detection threshold (e.g. `-50dB`).
  * `--silence-duration <val>` : Silence duration threshold in seconds (e.g. `2.0`).

### 8. Native Stream Multiplexing (`mux`)
Combines video and audio streams together into a single ISO-MP4 container with zero transcoding overhead:
```bash
nanaccel mux --video input_video.mp4 --audio input_audio.mp3 --output output_muxed.mp4
```

### 9. GPU Overlay & Watermark Engine (`overlay`)
Overlays image watermarks or logos directly on the GPU canvas using Direct3D 11 compositing:
```bash
nanaccel overlay input.mp4 output.mp4 path/to/logo.png [options]
```
* **Options:**
  * `-p, --position <pos>` : Target position coordinates (e.g. `10:10`).
  * `-t, --type <type>` : Overlay configuration type (`logo`, `watermark`).

### 10. Subtitle Track Control (`subtitle`)
Manage container-level subtitle tracks natively:
```bash
nanaccel subtitle <operation> input.mp4 output [options]
```
* **Operations:** `extract`, `convert`, `burn`, `sync`, `merge`, `remove`.
* **Options:**
  * `-s, --sub-file <file>` : Path to external subtitle file (`.srt`, `.vtt`).
  * `-t, --track <index>` : Index of target subtitle track.
  * `--shift <ms>` : Shift timing offset delay in milliseconds.

### 11. Video Editing Commands (`edit`)
GPU-based video editing triggers:
```bash
nanaccel edit <operation> input.mp4 output.mp4 [options]
```
* **Operations:** `trim`, `cut`, `split`, `join`, `concat`, `crop`, `rotate`, `flip`, `scale`, `stabilize`, `denoise`, `sharpen`, `deblock`, `deinterlace`, `reverse`, `loop`, `fade`, `crossfade`, `overlay`, `watermark`.

### 12. Video Color Grading (`color`)
High-speed GPU pixel color space conversion and grading:
```bash
nanaccel color <operation> input.mp4 output.mp4 [options]
```
* **Operations:** `hdr2sdr`, `sdr2hdr`, `lut`, `gamma`, `grading`, `colorspace`, `whitebalance`, `adjust`, `tonemap`.
* **Options:**
  * `--lut-file <file>` : 3D LUT grading file path.
  * `--brightness <val>`, `--contrast <val>`, `--saturation <val>` : Grade parameters.

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](file:///E:/NAN/Github/nanaccel/LICENSE) file for details.
