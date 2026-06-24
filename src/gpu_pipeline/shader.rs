use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Direct3D::*;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::Graphics::Dxgi::*;
use windows::Win32::Media::MediaFoundation::*;
use windows::Win32::System::Com::*;
use windows::core::*;

// For D3DCompile
use windows::Win32::Graphics::Direct3D::Fxc::*;

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
pub fn shader_gpu(
    input_path: &str,
    output_path: &str,
    shader_file: &str,
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

        D3D11CreateDevice(
            None::<&IDXGIAdapter>,
            D3D_DRIVER_TYPE_HARDWARE,
            HMODULE(std::ptr::null_mut()),
            D3D11_CREATE_DEVICE_FLAG(D3D11_CREATE_DEVICE_BGRA_SUPPORT.0 | D3D11_CREATE_DEVICE_VIDEO_SUPPORT.0),
            Some(&levels),
            D3D11_SDK_VERSION,
            Some(&mut d3d_device as *mut _),
            Some(&mut feature_level as *mut _),
            Some(&mut d3d_context as *mut _),
        )
        .map_err(|e| format!("Failed to create D3D11 Device: {}", e))?;
        let device: ID3D11Device = d3d_device.unwrap();
        let context = d3d_context.unwrap();

        // Enable multithread protection on D3D11 device
        let multithread: ID3D11Multithread = device.cast().map_err(|e| format!("Cast to ID3D11Multithread failed: {}", e))?;
        let _ = multithread.SetMultithreadProtected(true);

        // 2. Create Device Manager
        let mut token = 0;
        let mut manager_opt = None;
        MFCreateDXGIDeviceManager(&mut token, &mut manager_opt)
            .map_err(|e| format!("MFCreateDXGIDeviceManager failed: {}", e))?;
        let manager = manager_opt.unwrap();
        manager
            .ResetDevice(&device, token)
            .map_err(|e| format!("ResetDevice failed: {}", e))?;

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

        // 6. Compile Shaders
        let shader_src = std::fs::read(shader_file)
            .map_err(|e| format!("Failed to read shader file: {}", e))?;

        let mut vs_blob: Option<ID3DBlob> = None;
        let mut ps_blob: Option<ID3DBlob> = None;
        let mut error_blob: Option<ID3DBlob> = None;

        let vs_code = r#"
            struct VS_OUTPUT {
                float4 Pos : SV_POSITION;
                float2 Tex : TEXCOORD0;
            };
            VS_OUTPUT VS(uint id : SV_VertexID) {
                VS_OUTPUT output;
                output.Tex = float2((id << 1) & 2, id & 2);
                output.Pos = float4(output.Tex * float2(2.0, -2.0) + float2(-1.0, 1.0), 0.0, 1.0);
                return output;
            }
        "#;

        D3DCompile(
            vs_code.as_ptr() as *const _,
            vs_code.len(),
            PCSTR(b"vs_code\0".as_ptr()),
            None,
            None::<&ID3DInclude>,
            PCSTR(b"VS\0".as_ptr()),
            PCSTR(b"vs_5_0\0".as_ptr()),
            0,
            0,
            &mut vs_blob,
            Some(&mut error_blob),
        )
        .map_err(|e| {
            if let Some(err) = &error_blob {
                let msg = std::slice::from_raw_parts(err.GetBufferPointer() as *const u8, err.GetBufferSize());
                format!("VS Compile Error: {}\n{:?}", String::from_utf8_lossy(msg), e)
            } else {
                format!("Failed to compile default vertex shader: {}", e)
            }
        })?;

        error_blob = None;
        let path_c_str = format!("{}\0", shader_file);
        D3DCompile(
            shader_src.as_ptr() as *const _,
            shader_src.len(),
            PCSTR(path_c_str.as_ptr()),
            None,
            None::<&ID3DInclude>,
            PCSTR(b"main\0".as_ptr()),
            PCSTR(b"ps_5_0\0".as_ptr()),
            0,
            0,
            &mut ps_blob,
            Some(&mut error_blob),
        )
        .map_err(|e| {
            if let Some(err) = &error_blob {
                let msg = std::slice::from_raw_parts(err.GetBufferPointer() as *const u8, err.GetBufferSize());
                format!("PS Compile Error: {}\n{:?}", String::from_utf8_lossy(msg), e)
            } else {
                format!("Failed to compile pixel shader: {}", e)
            }
        })?;

        let vs_blob = vs_blob.unwrap();
        let ps_blob = ps_blob.unwrap();

        let mut vs = None;
        device.CreateVertexShader(
            std::slice::from_raw_parts(vs_blob.GetBufferPointer() as *const u8, vs_blob.GetBufferSize()),
            None,
            Some(&mut vs)
        ).map_err(|e| format!("CreateVertexShader failed: {}", e))?;
        let vs = vs.unwrap();

        let mut ps = None;
        device.CreatePixelShader(
            std::slice::from_raw_parts(ps_blob.GetBufferPointer() as *const u8, ps_blob.GetBufferSize()),
            None,
            Some(&mut ps)
        ).map_err(|e| format!("CreatePixelShader failed: {}", e))?;
        let ps = ps.unwrap();

        // 7. Setup Textures and Video Processor
        let video_device: ID3D11VideoDevice = device
            .cast()
            .map_err(|e| format!("Cast to ID3D11VideoDevice failed: {}", e))?;
        let video_context: ID3D11VideoContext = context
            .cast()
            .map_err(|e| format!("Cast to ID3D11VideoContext failed: {}", e))?;

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
            OutputWidth: width,
            OutputHeight: height,
            Usage: D3D11_VIDEO_USAGE_PLAYBACK_NORMAL,
        };

        let enumerator = video_device
            .CreateVideoProcessorEnumerator(&vp_desc)
            .map_err(|e| format!("CreateVideoProcessorEnumerator failed: {}", e))?;

        let processor = video_device
            .CreateVideoProcessor(&enumerator, 0)
            .map_err(|e| format!("CreateVideoProcessor failed: {}", e))?;

        // Intermediate RGBA Texture
        let rgba_desc = D3D11_TEXTURE2D_DESC {
            Width: width,
            Height: height,
            MipLevels: 1,
            ArraySize: 1,
            Format: DXGI_FORMAT_R8G8B8A8_UNORM,
            SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: D3D11_BIND_RENDER_TARGET.0 as u32 | D3D11_BIND_SHADER_RESOURCE.0 as u32,
            CPUAccessFlags: 0,
            MiscFlags: 0,
        };

        let mut rgba_tex = None;
        device.CreateTexture2D(&rgba_desc, None, Some(&mut rgba_tex))
            .map_err(|e| format!("CreateTexture2D (RGBA) failed: {}", e))?;
        let rgba_tex = rgba_tex.unwrap();

        // Processed RGBA Texture
        let mut processed_tex = None;
        device.CreateTexture2D(&rgba_desc, None, Some(&mut processed_tex))
            .map_err(|e| format!("CreateTexture2D (Processed) failed: {}", e))?;
        let processed_tex = processed_tex.unwrap();

        // Final NV12 Texture for NVENC
        let nv12_desc = D3D11_TEXTURE2D_DESC {
            Width: width,
            Height: height,
            MipLevels: 1,
            ArraySize: 1,
            Format: DXGI_FORMAT_NV12,
            SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: D3D11_BIND_RENDER_TARGET.0 as u32 | D3D11_BIND_SHADER_RESOURCE.0 as u32,
            CPUAccessFlags: 0,
            MiscFlags: 0,
        };

        let mut nvenc_texture = None;
        device.CreateTexture2D(&nv12_desc, None, Some(&mut nvenc_texture))
            .map_err(|e| format!("CreateTexture2D (NVENC) failed: {}", e))?;
        let nvenc_texture = nvenc_texture.unwrap();

        // Views for Video Processor and HLSL Rendering
        let out_view_desc_rgba = D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC {
            ViewDimension: D3D11_VPOV_DIMENSION_TEXTURE2D,
            Anonymous: D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC_0 {
                Texture2D: D3D11_TEX2D_VPOV { MipSlice: 0 },
            },
        };

        let mut output_view_rgba = None;
        video_device.CreateVideoProcessorOutputView(&rgba_tex, &enumerator, &out_view_desc_rgba, Some(&mut output_view_rgba))
            .map_err(|e| format!("CreateVideoProcessorOutputView (RGBA) failed: {}", e))?;
        let output_view_rgba = output_view_rgba.unwrap();

        let mut output_view_nv12 = None;
        video_device.CreateVideoProcessorOutputView(&nvenc_texture, &enumerator, &out_view_desc_rgba, Some(&mut output_view_nv12))
            .map_err(|e| format!("CreateVideoProcessorOutputView (NV12) failed: {}", e))?;
        let output_view_nv12 = output_view_nv12.unwrap();

        let mut rtv_rgba = None;
        device.CreateRenderTargetView(&processed_tex, None, Some(&mut rtv_rgba))
            .map_err(|e| format!("CreateRenderTargetView failed: {}", e))?;
        let rtv_rgba = rtv_rgba.unwrap();

        let mut srv_rgba = None;
        device.CreateShaderResourceView(&rgba_tex, None, Some(&mut srv_rgba))
            .map_err(|e| format!("CreateShaderResourceView failed: {}", e))?;
        let srv_rgba = srv_rgba.unwrap();

        let in_view_desc_rgba = D3D11_VIDEO_PROCESSOR_INPUT_VIEW_DESC {
            FourCC: 0,
            ViewDimension: D3D11_VPIV_DIMENSION_TEXTURE2D,
            Anonymous: D3D11_VIDEO_PROCESSOR_INPUT_VIEW_DESC_0 {
                Texture2D: D3D11_TEX2D_VPIV {
                    ArraySlice: 0,
                    MipSlice: 0,
                },
            },
        };

        let mut input_view_rgba = None;
        video_device.CreateVideoProcessorInputView(&processed_tex, &enumerator, &in_view_desc_rgba, Some(&mut input_view_rgba))
            .map_err(|e| format!("CreateVideoProcessorInputView failed: {}", e))?;
        let input_view_rgba = input_view_rgba.unwrap();

        // 8. Initialize NVENC Session
        let session = Session::open_dx(&device)
            .map_err(|e| format!("Failed to open NVENC DX session: {:?}", e))?;

        let (session, mut nv_config) = session
            .get_encode_preset_config_ex(
                nvenc::sys::guids::NV_ENC_CODEC_H264_GUID.clone(),
                nvenc::sys::guids::NV_ENC_PRESET_P4_GUID.clone(),
                nvenc::sys::enums::NVencTuningInfo::LowLatency,
            )
            .map_err(|e| format!("Preset config failed: {:?}", e))?;

        let init_params = InitParams {
            encode_guid: nvenc::sys::guids::NV_ENC_CODEC_H264_GUID.clone(),
            preset_guid: nvenc::sys::guids::NV_ENC_PRESET_P4_GUID.clone(),
            aspect_ratio: [16, 9],
            encode_config: &mut nv_config.preset_cfg,
            tuning_info: nvenc::sys::enums::NVencTuningInfo::LowLatency,
            buffer_format: nvenc::sys::enums::NVencBufferFormat::NV12,
            frame_rate: [fps.round() as u32, 1],
            resolution: [width, height],
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

        // Pipeline States
        context.IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
        context.VSSetShader(&vs, None);
        context.PSSetShader(&ps, None);

        let viewport = D3D11_VIEWPORT {
            TopLeftX: 0.0,
            TopLeftY: 0.0,
            Width: width as f32,
            Height: height as f32,
            MinDepth: 0.0,
            MaxDepth: 1.0,
        };
        context.RSSetViewports(Some(&[viewport]));

        // 9. Loop and read frames
        let mut muxer: Option<crate::mux::Muxer> = None;
        let mut frame_count = 0;

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
                        
                        // Pass 1: YUV -> RGBA
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
                        video_device.CreateVideoProcessorInputView(&src_texture, &enumerator, &in_view_desc, Some(&mut input_view))
                            .map_err(|e| format!("CreateVideoProcessorInputView (YUV) failed: {}", e))?;
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

                        video_context.VideoProcessorBlt(&processor, &output_view_rgba, 0, &[stream])
                            .map_err(|e| format!("VideoProcessorBlt (YUV -> RGBA) failed: {}", e))?;

                        // Pass 2: Run Custom HLSL Shader on the GPU
                        context.OMSetRenderTargets(Some(&[Some(rtv_rgba.clone())]), None);
                        context.PSSetShaderResources(0, Some(&[Some(srv_rgba.clone())]));
                        context.Draw(3, 0);

                        // Unbind SRV to allow it to be used as output next frame
                        context.PSSetShaderResources(0, Some(&[None]));

                        // Pass 3: RGBA -> YUV (NV12 for NVENC)
                        let stream_rgba = D3D11_VIDEO_PROCESSOR_STREAM {
                            Enable: true.into(),
                            OutputIndex: 0,
                            InputFrameOrField: 0,
                            PastFrames: 0,
                            FutureFrames: 0,
                            ppPastSurfaces: std::ptr::null_mut(),
                            pInputSurface: std::mem::ManuallyDrop::new(Some(input_view_rgba.clone())),
                            ppFutureSurfaces: std::ptr::null_mut(),
                            ppPastSurfacesRight: std::ptr::null_mut(),
                            pInputSurfaceRight: std::mem::ManuallyDrop::new(None),
                            ppFutureSurfacesRight: std::ptr::null_mut(),
                        };

                        video_context.VideoProcessorBlt(&processor, &output_view_nv12, 0, &[stream_rgba])
                            .map_err(|e| format!("VideoProcessorBlt (RGBA -> YUV) failed: {}", e))?;

                        // Pass 4: Encode and Mux
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
                                let frame_duration = (1000.0 / fps) as u32;
                                let is_keyframe = encoded_bytes.contains(&0x05) || encoded_bytes.contains(&0x07);
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
