use std::time::Instant;
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

#[allow(clippy::collapsible_if)]
pub fn transcode_gpu(
    input_path: &str,
    output_path: &str,
    codec: &str,
    preset: &str,
    bitrate: Option<&str>,
    scale: Option<&str>,
) -> std::result::Result<(), String> {
    unsafe {
        // Init COM & WMF
        CoInitializeEx(None, COINIT_MULTITHREADED)
            .ok()
            .map_err(|e| e.to_string())?;
        MFStartup(MF_VERSION, MFSTARTUP_FULL).map_err(|e| e.to_string())?;

        // 1. Create D3D11 Device
        let mut d3d_device: Option<ID3D11Device> = None;
        let mut d3d_context: Option<ID3D11DeviceContext> = None;
        let mut feature_level = D3D_FEATURE_LEVEL_11_0;
        let levels = [D3D_FEATURE_LEVEL_11_0];

        println!("[NanAccel Debug] Creating D3D11 Device with BGRA & Video support...");
        D3D11CreateDevice(
            None::<&IDXGIAdapter>,
            D3D_DRIVER_TYPE_HARDWARE,
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
        let device: ID3D11Device = d3d_device.unwrap();
        let context = d3d_context.unwrap();

        println!("[NanAccel Debug] Enabling multithread protection on D3D11 device...");
        let multithread: ID3D11Multithread = device
            .cast()
            .map_err(|e| format!("Cast to ID3D11Multithread failed: {}", e))?;
        let _ = multithread.SetMultithreadProtected(true);

        println!("[NanAccel Debug] Creating DXGI device manager...");
        let mut token = 0;
        let mut manager_opt = None;
        MFCreateDXGIDeviceManager(&mut token, &mut manager_opt)
            .map_err(|e| format!("MFCreateDXGIDeviceManager failed: {}", e))?;
        let manager = manager_opt.unwrap();
        manager
            .ResetDevice(&device, token)
            .map_err(|e| format!("ResetDevice failed: {}", e))?;

        println!(
            "[NanAccel Debug] Initializing Media Foundation source reader from URL: {} ...",
            input_path
        );

        // 3. Create Attributes
        let mut attr_opt = None;
        MFCreateAttributes(&mut attr_opt, 1)
            .map_err(|e| format!("MFCreateAttributes failed: {}", e))?;
        let attr = attr_opt.unwrap();
        attr.SetUnknown(&MF_SOURCE_READER_D3D_MANAGER, &manager)
            .map_err(|e| format!("SetUnknown failed: {}", e))?;

        // 4. Create Source Reader
        let url = HSTRING::from(input_path);
        let reader = MFCreateSourceReaderFromURL(&url, Some(&attr))
            .map_err(|e| format!("MFCreateSourceReaderFromURL failed: {}", e))?;

        // 5. Get Stream Dimensions
        let mut width = 640;
        let mut height = 360;
        let mut fps: f64 = 30.0;
        if let Ok(current_media_type) =
            reader.GetCurrentMediaType(MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32)
        {
            let size = current_media_type.GetUINT64(&MF_MT_FRAME_SIZE).unwrap_or(0);
            if size > 0 {
                width = ((size >> 32) as u32 / 2) * 2;
                height = ((size & 0xFFFFFFFF) as u32 / 2) * 2;
            }
            let ratio = current_media_type.GetUINT64(&MF_MT_FRAME_RATE).unwrap_or(0);
            if ratio > 0 {
                let num = (ratio >> 32) as u32;
                let den = (ratio & 0xFFFFFFFF) as u32;
                if den > 0 {
                    fps = num as f64 / den as f64;
                }
            }
        }

        // Set output type to NV12
        let mt = MFCreateMediaType().map_err(|e| e.to_string())?;
        mt.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)
            .map_err(|e| e.to_string())?;
        mt.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_NV12)
            .map_err(|e| e.to_string())?;
        reader
            .SetCurrentMediaType(MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32, None, &mt)
            .map_err(|e| e.to_string())?;

        // Handle Scale resolution if option provided
        let mut out_width = width;
        let mut out_height = height;
        if let Some(scale_str) = scale {
            let parts: Vec<&str> = scale_str.split('x').collect();
            if parts.len() == 2 {
                if let (Ok(w), Ok(h)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                    out_width = (w / 2) * 2;
                    out_height = (h / 2) * 2;
                }
            }
        } else if out_width > 4096 || out_height > 4096 {
            // Auto cap to 1080p for safe H.264 encoding when input is 5K/8K
            let aspect = width as f64 / height as f64;
            if aspect > 1.0 {
                out_width = 1920;
                out_height = ((1920.0 / aspect).round() as u32 / 2) * 2;
            } else {
                out_height = 1080;
                out_width = ((1080.0 * aspect).round() as u32 / 2) * 2;
            }
        }

        // Query Video Device & Context
        let video_device: ID3D11VideoDevice = device
            .cast()
            .map_err(|e| format!("Cast to ID3D11VideoDevice failed: {}", e))?;
        let video_context: ID3D11VideoContext = context
            .cast()
            .map_err(|e| format!("Cast to ID3D11VideoContext failed: {}", e))?;

        // Create Video Processor Enumerator & Processor
        let vp_desc = D3D11_VIDEO_PROCESSOR_CONTENT_DESC {
            InputFrameFormat: D3D11_VIDEO_FRAME_FORMAT_PROGRESSIVE,
            InputFrameRate: DXGI_RATIONAL {
                Numerator: fps.round() as u32,
                Denominator: 1,
            },
            InputWidth: width,
            InputHeight: height,
            OutputFrameRate: DXGI_RATIONAL {
                Numerator: fps.round() as u32,
                Denominator: 1,
            },
            OutputWidth: out_width,
            OutputHeight: out_height,
            Usage: D3D11_VIDEO_USAGE_PLAYBACK_NORMAL,
        };

        let enumerator = video_device
            .CreateVideoProcessorEnumerator(&vp_desc)
            .map_err(|e| format!("CreateVideoProcessorEnumerator failed: {}", e))?;

        let processor = video_device
            .CreateVideoProcessor(&enumerator, 0)
            .map_err(|e| format!("CreateVideoProcessor failed: {}", e))?;

        // 6. Create NVENC Input Texture
        let mut nvenc_texture = None;
        let texture_desc = D3D11_TEXTURE2D_DESC {
            Width: out_width,
            Height: out_height,
            MipLevels: 1,
            ArraySize: 1,
            Format: DXGI_FORMAT_NV12, // nvenc NV12
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: D3D11_BIND_RENDER_TARGET.0 as u32 | D3D11_BIND_SHADER_RESOURCE.0 as u32,
            CPUAccessFlags: 0,
            MiscFlags: 0,
        };
        device
            .CreateTexture2D(&texture_desc, None, Some(&mut nvenc_texture))
            .map_err(|e| format!("CreateTexture2D failed: {}", e))?;
        let nvenc_texture = nvenc_texture.unwrap();

        // Create Video Processor Output View on nvenc_texture
        let out_view_desc = D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC {
            ViewDimension: D3D11_VPOV_DIMENSION_TEXTURE2D,
            Anonymous: D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC_0 {
                Texture2D: D3D11_TEX2D_VPOV { MipSlice: 0 },
            },
        };
        let mut output_view = None;
        video_device
            .CreateVideoProcessorOutputView(
                &nvenc_texture,
                &enumerator,
                &out_view_desc,
                Some(&mut output_view),
            )
            .map_err(|e| format!("CreateVideoProcessorOutputView failed: {}", e))?;
        let output_view = output_view.unwrap();

        // 7. Initialize NVENC Session
        use nvenc::sys::guids::{
            NV_ENC_CODEC_H264_GUID, NV_ENC_CODEC_HEVC_GUID, NV_ENC_PRESET_P1_GUID,
            NV_ENC_PRESET_P2_GUID, NV_ENC_PRESET_P3_GUID, NV_ENC_PRESET_P4_GUID,
            NV_ENC_PRESET_P5_GUID, NV_ENC_PRESET_P6_GUID, NV_ENC_PRESET_P7_GUID,
        };

        let session = Session::open_dx(&device)
            .map_err(|e| format!("Failed to open NVENC DX session: {:?}", e))?;

        let codec_guid = if codec.to_lowercase() == "hevc" {
            NV_ENC_CODEC_HEVC_GUID
        } else {
            NV_ENC_CODEC_H264_GUID
        };

        let preset_guid = match preset.to_lowercase().as_str() {
            "p1" => NV_ENC_PRESET_P1_GUID,
            "p2" => NV_ENC_PRESET_P2_GUID,
            "p3" => NV_ENC_PRESET_P3_GUID,
            "p4" => NV_ENC_PRESET_P4_GUID,
            "p5" => NV_ENC_PRESET_P5_GUID,
            "p6" => NV_ENC_PRESET_P6_GUID,
            "p7" => NV_ENC_PRESET_P7_GUID,
            _ => NV_ENC_PRESET_P4_GUID,
        };

        let (session, mut nv_config) = session
            .get_encode_preset_config_ex(
                codec_guid.clone(),
                preset_guid.clone(),
                nvenc::sys::enums::NVencTuningInfo::LowLatency,
            )
            .map_err(|e| format!("Preset config failed: {:?}", e))?;

        // Bitrate setup
        if let Some(br_str) = bitrate {
            let mut val = 5_000_000;
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
            nv_config.preset_cfg.rc_params.rate_control_mode =
                nvenc::sys::enums::NVencParamsRcMode::VBR;
            nv_config.preset_cfg.rc_params.average_bit_rate = val;
        }

        let init_params = InitParams {
            encode_guid: codec_guid,
            preset_guid,
            aspect_ratio: [16, 9],
            encode_config: &mut nv_config.preset_cfg,
            tuning_info: nvenc::sys::enums::NVencTuningInfo::LowLatency,
            buffer_format: nvenc::sys::enums::NVencBufferFormat::NV12,
            frame_rate: [fps.round() as u32, 1],
            resolution: [out_width, out_height],
            enable_ptd: true,
            max_encoder_resolution: [0, 0],
        };

        let encoder = session
            .init_encoder(init_params)
            .map_err(|e| format!("init_encoder failed: {:?}", e))?;

        let registered = encoder
            .register_resource_dx11(
                &nvenc_texture,
                nvenc::sys::enums::NVencBufferFormat::NV12,
                0,
            )
            .map_err(|e| format!("register_resource_dx11 failed: {:?}", e))?;

        // 8. Loop and read frames
        let mut muxer: Option<crate::mux::Muxer> = None;
        let mut frame_count = 0;
        let _start_time = Instant::now();

        loop {
            let mut actual_stream_index = 0;
            let mut flags = 0;
            let mut timestamp = 0;
            let mut sample = None;

            reader
                .ReadSample(
                    MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32,
                    0,
                    Some(&mut actual_stream_index),
                    Some(&mut flags),
                    Some(&mut timestamp),
                    Some(&mut sample),
                )
                .map_err(|e| format!("ReadSample failed: {}", e))?;

            if flags & MF_SOURCE_READERF_ENDOFSTREAM.0 as u32 != 0 {
                break;
            }

            if let Some(sample) = sample {
                if let Ok(buffer) = sample.GetBufferByIndex(0) {
                    if let Ok(src_texture) = crate::gpu_pipeline::get_texture_from_buffer(&buffer) {
                        // Use Video Processor to downscale and copy src_texture to nvenc_texture on GPU
                        let in_view_desc = D3D11_VIDEO_PROCESSOR_INPUT_VIEW_DESC {
                            FourCC: 0,
                            ViewDimension: D3D11_VPIV_DIMENSION_TEXTURE2D,
                            Anonymous: D3D11_VIDEO_PROCESSOR_INPUT_VIEW_DESC_0 {
                                Texture2D: D3D11_TEX2D_VPIV {
                                    ArraySlice: 0,
                                    MipSlice: 0,
                                },
                            },
                        };
                        let mut input_view = None;
                        video_device
                            .CreateVideoProcessorInputView(
                                &src_texture,
                                &enumerator,
                                &in_view_desc,
                                Some(&mut input_view),
                            )
                            .map_err(|e| format!("CreateVideoProcessorInputView failed: {}", e))?;
                        let input_view = input_view.unwrap();

                        let stream = D3D11_VIDEO_PROCESSOR_STREAM {
                            Enable: true.into(),
                            OutputIndex: 0,
                            InputFrameOrField: 0,
                            PastFrames: 0,
                            FutureFrames: 0,
                            ppPastSurfaces: std::ptr::null_mut(),
                            pInputSurface: std::mem::ManuallyDrop::new(Some(input_view)),
                            ppFutureSurfaces: std::ptr::null_mut(),
                            ppPastSurfacesRight: std::ptr::null_mut(),
                            pInputSurfaceRight: std::mem::ManuallyDrop::new(None),
                            ppFutureSurfacesRight: std::ptr::null_mut(),
                        };

                        video_context
                            .VideoProcessorBlt(&processor, &output_view, 0, &[stream])
                            .map_err(|e| format!("VideoProcessorBlt failed: {}", e))?;

                        // Now encode it using NVENC
                        let bitstream = encoder
                            .create_bitstream_buffer()
                            .map_err(|e| format!("create_bitstream_buffer failed: {:?}", e))?;

                        encoder
                            .encode_picture(
                                &registered,
                                &bitstream,
                                frame_count,
                                (frame_count as f64 * 1000.0 / fps) as u64,
                                nvenc::sys::enums::NVencBufferFormat::NV12,
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
                            // On first encoded frame, initialize the Muxer using SPS and PPS
                            if muxer.is_none() {
                                let is_hevc = codec.to_lowercase() == "hevc";
                                let (sps, pps) = if is_hevc {
                                    (Vec::new(), Vec::new())
                                } else {
                                    extract_sps_pps(encoded_bytes)
                                };
                                muxer = Some(crate::mux::Muxer::create(
                                    output_path,
                                    out_width as u16,
                                    out_height as u16,
                                    &sps,
                                    &pps,
                                    is_hevc,
                                )?);
                            }

                            if let Some(m) = &mut muxer {
                                // Frame duration in milliseconds
                                let frame_duration = (1000.0 / fps) as u32;
                                let is_hevc = codec.to_lowercase() == "hevc";
                                let is_keyframe = if is_hevc {
                                    // HEVC keyframe check (NAL unit type 19 or 20)
                                    let mut has_keyframe = false;
                                    let mut idx_bytes = 0;
                                    while idx_bytes < encoded_bytes.len() {
                                        if idx_bytes + 3 < encoded_bytes.len()
                                            && encoded_bytes[idx_bytes..idx_bytes + 4]
                                                == [0, 0, 0, 1]
                                        {
                                            let nal_type =
                                                (encoded_bytes[idx_bytes + 4] >> 1) & 0x3F;
                                            if nal_type == 19 || nal_type == 20 {
                                                has_keyframe = true;
                                                break;
                                            }
                                            idx_bytes += 4;
                                        } else if idx_bytes + 2 < encoded_bytes.len()
                                            && encoded_bytes[idx_bytes..idx_bytes + 3] == [0, 0, 1]
                                        {
                                            let nal_type =
                                                (encoded_bytes[idx_bytes + 3] >> 1) & 0x3F;
                                            if nal_type == 19 || nal_type == 20 {
                                                has_keyframe = true;
                                                break;
                                            }
                                            idx_bytes += 3;
                                        } else {
                                            idx_bytes += 1;
                                        }
                                    }
                                    has_keyframe
                                } else {
                                    encoded_bytes.contains(&0x05) || encoded_bytes.contains(&0x07)
                                };
                                m.write_video_frame(encoded_bytes, frame_duration, is_keyframe)?;
                            }
                        }

                        frame_count += 1;
                    }
                }
            }
        }

        if let Some(m) = muxer {
            m.close()?;
        }

        Ok(())
    }
}
