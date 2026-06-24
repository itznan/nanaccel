use std::time::{Duration, Instant};
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Direct3D::*;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::Graphics::Dxgi::*;
use windows::Win32::Media::MediaFoundation::*;
use windows::Win32::System::Com::*;
use windows::core::*;

use nvenc::session::InitParams;
use nvenc::session::Session;

fn extract_sps_pps(annex_b: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let mut sps = Vec::new();
    let mut pps = Vec::new();
    let mut i = 0;
    let mut nal_starts = Vec::new();
    while i < annex_b.len() {
        if i + 3 < annex_b.len() && annex_b[i..i + 4] == [0, 0, 0, 1] {
            nal_starts.push((i + 4, 4));
            i += 4;
        } else if i + 2 < annex_b.len() && annex_b[i..i + 3] == [0, 0, 1] {
            nal_starts.push((i + 3, 3));
            i += 3;
        } else {
            i += 1;
        }
    }

    for idx in 0..nal_starts.len() {
        let (start, _code_len) = nal_starts[idx];
        let end = if idx + 1 < nal_starts.len() {
            let next_start = nal_starts[idx + 1].0;
            let next_code_len = nal_starts[idx + 1].1;
            next_start - next_code_len
        } else {
            annex_b.len()
        };

        let payload = &annex_b[start..end];
        if payload.is_empty() {
            continue;
        }
        let nal_type = payload[0] & 0x1F;
        if nal_type == 7 {
            sps = payload.to_vec();
        } else if nal_type == 8 {
            pps = payload.to_vec();
        }
    }
    (sps, pps)
}

pub fn record_gpu(
    output_path: &str,
    fps: Option<u32>,
    bitrate: Option<&str>,
    duration_sec: Option<u32>,
) -> std::result::Result<(), String> {
    unsafe {
        // Init COM & WMF
        CoInitializeEx(None, COINIT_MULTITHREADED)
            .ok()
            .map_err(|e| e.to_string())?;
        MFStartup(MF_VERSION, MFSTARTUP_FULL).map_err(|e| e.to_string())?;

        let target_fps = fps.unwrap_or(60);
        let target_duration = duration_sec.unwrap_or(5);

        // 1. Create DXGI Factory and find default adapter & output
        let factory: IDXGIFactory1 = CreateDXGIFactory1().map_err(|e| format!("Failed to create DXGI Factory: {}", e))?;

        let adapter: IDXGIAdapter1 = factory.EnumAdapters1(0).map_err(|e| format!("EnumAdapters1 failed: {}", e))?;

        let output: IDXGIOutput = adapter.EnumOutputs(0).map_err(|e| format!("EnumOutputs failed: {}", e))?;
        let output1: IDXGIOutput1 = output.cast().map_err(|e| format!("Cast to IDXGIOutput1 failed: {}", e))?;

        // 2. Query Desktop resolution from output description
        let desc = output.GetDesc().map_err(|e| format!("GetDesc failed: {}", e))?;
        let rect = desc.DesktopCoordinates;
        let mut width = (rect.right - rect.left) as u32;
        let mut height = (rect.bottom - rect.top) as u32;

        // Force even width/height as required by NVENC
        width = (width / 2) * 2;
        height = (height / 2) * 2;

        println!("[DXGI] Target screen dimensions: {}x{}", width, height);

        // 3. Create D3D11 Device.
        // NOTE: Since we pass adapter, driver type MUST be D3D_DRIVER_TYPE_UNKNOWN.
        let mut d3d_device: Option<ID3D11Device> = None;
        let mut d3d_context: Option<ID3D11DeviceContext> = None;
        let mut feature_level = D3D_FEATURE_LEVEL_11_0;
        let levels = [D3D_FEATURE_LEVEL_11_0];

        D3D11CreateDevice(
            &adapter,
            D3D_DRIVER_TYPE_UNKNOWN,
            HMODULE(std::ptr::null_mut()),
            D3D11_CREATE_DEVICE_FLAG(
                D3D11_CREATE_DEVICE_BGRA_SUPPORT.0 | D3D11_CREATE_DEVICE_VIDEO_SUPPORT.0,
            ),
            Some(&levels),
            D3D11_SDK_VERSION,
            Some(&mut d3d_device as *mut _),
            Some(&mut feature_level as *mut _),
            Some(&mut d3d_context as *mut _),
        )
        .map_err(|e| format!("Failed to create D3D11 Device: {}", e))?;
        let device = d3d_device.unwrap();
        let context = d3d_context.unwrap();

        // Enable multithread protection on device
        let multithread: ID3D11Multithread = device
            .cast()
            .map_err(|e| format!("Cast to ID3D11Multithread failed: {}", e))?;
        let _ = multithread.SetMultithreadProtected(true);

        // 4. Initialize Desktop Duplication
        let duplication: IDXGIOutputDuplication = output1
            .DuplicateOutput(&device)
            .map_err(|e| format!("DuplicateOutput failed: {}. Make sure you are on a primary monitor.", e))?;

        // 5. Create a permanent input texture in BGRA format (matching screen format)
        let texture_desc = D3D11_TEXTURE2D_DESC {
            Width: width,
            Height: height,
            MipLevels: 1,
            ArraySize: 1,
            Format: DXGI_FORMAT_B8G8R8A8_UNORM,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: D3D11_BIND_RENDER_TARGET.0 as u32 | D3D11_BIND_SHADER_RESOURCE.0 as u32,
            CPUAccessFlags: 0,
            MiscFlags: 0,
        };
        let mut input_texture = None;
        device
            .CreateTexture2D(&texture_desc, None, Some(&mut input_texture))
            .map_err(|e| format!("Create BGRA Texture2D failed: {}", e))?;
        let input_texture = input_texture.unwrap();

        // 6. Initialize NVENC Session in ARGB mode
        use nvenc::sys::guids::{NV_ENC_CODEC_H264_GUID, NV_ENC_PRESET_P4_GUID};

        let session = Session::open_dx(&device)
            .map_err(|e| format!("Failed to open NVENC DX session: {:?}", e))?;

        let (session, mut nv_config) = session
            .get_encode_preset_config_ex(
                NV_ENC_CODEC_H264_GUID,
                NV_ENC_PRESET_P4_GUID,
                nvenc::sys::enums::NVencTuningInfo::LowLatency,
            )
            .map_err(|e| format!("Preset config failed: {:?}", e))?;

        // Bitrate
        let mut val = 8_000_000;
        if let Some(br_str) = bitrate {
            let trimmed = br_str.trim().to_lowercase();
            if trimmed.ends_with('m') {
                if let Ok(num) = trimmed.trim_end_matches('m').parse::<f64>() {
                    val = (num * 1_000_000.0) as u32;
                }
            } else if trimmed.ends_with('k') {
                if let Ok(num) = trimmed.trim_end_matches('k').parse::<f64>() {
                    val = (num * 1_000.0) as u32;
                }
            } else if let Ok(num) = trimmed.parse::<u32>() {
                val = num;
            }
        }
        nv_config.preset_cfg.rc_params.rate_control_mode = nvenc::sys::enums::NVencParamsRcMode::VBR;
        nv_config.preset_cfg.rc_params.average_bit_rate = val;

        let init_params = InitParams {
            encode_guid: NV_ENC_CODEC_H264_GUID,
            preset_guid: NV_ENC_PRESET_P4_GUID,
            aspect_ratio: [16, 9],
            encode_config: &mut nv_config.preset_cfg,
            tuning_info: nvenc::sys::enums::NVencTuningInfo::LowLatency,
            buffer_format: nvenc::sys::enums::NVencBufferFormat::ARGB, // Direct ARGB encoding
            frame_rate: [target_fps, 1],
            resolution: [width, height],
            enable_ptd: true,
            max_encoder_resolution: [0, 0],
        };

        let encoder = session
            .init_encoder(init_params)
            .map_err(|e| format!("init_encoder failed: {:?}", e))?;

        let registered = encoder
            .register_resource_dx11(
                &input_texture,
                nvenc::sys::enums::NVencBufferFormat::ARGB,
                0,
            )
            .map_err(|e| format!("register_resource_dx11 failed: {:?}", e))?;

        // 7. Capture & Encode Loop
        let mut muxer: Option<crate::mux::Muxer> = None;
        let mut frame_count: usize = 0;
        let total_frames = (target_fps * target_duration) as usize;
        let frame_duration = Duration::from_secs_f64(1.0 / target_fps as f64);

        println!("[Record] Screen recording started for {} seconds...", target_duration);

        while frame_count < total_frames {
            let loop_start = Instant::now();
            let mut frame_info = DXGI_OUTDUPL_FRAME_INFO::default();
            let mut desktop_resource: Option<IDXGIResource> = None;

            // Acquire desktop frame
            let acquired = match duplication.AcquireNextFrame(10, &mut frame_info, &mut desktop_resource) {
                Ok(_) => {
                    let res = desktop_resource.unwrap();
                    let tex: ID3D11Texture2D = res.cast().map_err(|e| e.to_string())?;
                    // Copy desktop texture directly to our registered ARGB input texture on the GPU
                    context.CopyResource(&input_texture, &tex);
                    true
                }
                Err(e) => {
                    if e.code() == DXGI_ERROR_ACCESS_LOST {
                        println!("[DXGI] Access lost to desktop output duplication.");
                        break;
                    }
                    // Wait timeout is normal; we reuse the last frame's contents
                    false
                }
            };

            // Encode the ARGB texture with NVENC
            let bitstream = encoder
                .create_bitstream_buffer()
                .map_err(|e| format!("create_bitstream_buffer failed: {:?}", e))?;

            encoder
                .encode_picture(
                    &registered,
                    &bitstream,
                    frame_count,
                    (frame_count as f64 * 1000.0 / target_fps as f64) as u64,
                    nvenc::sys::enums::NVencBufferFormat::ARGB,
                    nvenc::sys::enums::NVencPicStruct::Frame,
                    nvenc::sys::enums::NVencPicType::P,
                    None,
                )
                .map_err(|e| format!("encode_picture failed: {:?}", e))?;

            let lock = bitstream
                .try_lock(true)
                .map_err(|e| format!("try_lock failed: {:?}", e))?;
            let encoded_bytes = lock.as_slice();

            if !encoded_bytes.is_empty() {
                if muxer.is_none() {
                    let (sps, pps) = extract_sps_pps(encoded_bytes);
                    muxer = Some(crate::mux::Muxer::create(
                        output_path,
                        width as u16,
                        height as u16,
                        &sps,
                        &pps,
                        false,
                    )?);
                }

                if let Some(m) = &mut muxer {
                    let is_keyframe = encoded_bytes.contains(&0x05) || encoded_bytes.contains(&0x07);
                    m.write_video_frame(encoded_bytes, (1000 / target_fps) as u32, is_keyframe)?;
                }
            }

            if acquired {
                duplication.ReleaseFrame().map_err(|e| format!("ReleaseFrame failed: {}", e))?;
            }

            frame_count += 1;

            let elapsed = loop_start.elapsed();
            if elapsed < frame_duration {
                std::thread::sleep(frame_duration - elapsed);
            }
        }

        if let Some(m) = muxer {
            m.close()?;
        }

        Ok(())
    }
}
