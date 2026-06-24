# Getting Started with nanaccel

A guide to setting up, compiling, and running **nanaccel** on your local machine.

---

## 1. System Requirements

*   **Operating System**: Windows 10/11 (64-bit).
*   **Graphics Card**: NVIDIA GPU (GeForce, Quadro, or Tesla) with modern drivers.
*   **Driver Version**: Latest Game Ready or Studio Driver supporting NVENC.
*   **Tools**:
    *   [Rust toolchain](https://rustup.rs/) (edition 2021/2024).
    *   Windows SDK & C++ Build Tools (installed automatically via Visual Studio Installer).

---

## 2. Setup and Compilation

Clone the repository and build the binary in release mode:

```bash
# Clone the repository
git clone https://github.com/itznan/nanaccel.git
cd nanaccel

# Build the release binary
cargo build --release
```

The optimized executable will be created at:
```text
target/release/nanaccel.exe
```

---

## 3. Basic Walkthrough

### Verify NVIDIA GPU Connection
Verify if the hardware checks are working and telemetry is returned:
```bash
.\nanaccel.exe info
```

### Try Interactive Video Playback
Open a video file in the native D3D11 player window:
```bash
.\nanaccel.exe play classroom.mp4
```
*   Press **`Space`** to pause or resume playback.
*   Press **`Esc`** or **`Q`** to close the window and quit.

### Convert Video using custom Shader
Apply the grayscale pixel shader to the sample video:
```bash
.\nanaccel.exe shader classroom.mp4 output_grayscale.mp4 grayscale.hlsl
```
*This compiles `grayscale.hlsl` at runtime, runs the shader on each video frame using the GPU, and writes the output as a valid H.264 file.*

### Record Desktop Screen
Record your primary desktop monitor directly using hardware-accelerated capture and NVENC encoding:
```bash
.\nanaccel.exe record output_recording.mp4 --duration 10 --fps 60 -b 10M
```
*This captures the screen at 60 FPS using DXGI Desktop Duplication, copies the textures on the GPU, and encodes them using NVENC to `output_recording.mp4` with a 10 Mbps bitrate.*
