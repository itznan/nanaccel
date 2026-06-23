use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Direct3D::*;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::Graphics::Dxgi::*;
use windows::Win32::Graphics::Imaging::*;
use windows::Win32::Media::MediaFoundation::*;
use windows::Win32::System::Com::StructuredStorage::PROPVARIANT;
use windows::Win32::System::Com::*;
use windows::core::*;

pub unsafe fn save_pixels_to_file(
    file_path: &str,
    width: u32,
    height: u32,
    stride: u32,
    pixels: &[u8],
) -> std::result::Result<(), String> {
    unsafe {
        let factory: IWICImagingFactory =
            CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)
                .map_err(|e| format!("CoCreateInstance CLSID_WICImagingFactory failed: {}", e))?;

        let ext = std::path::Path::new(file_path)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        let encoder_clsid = match ext.as_str() {
            "jpg" | "jpeg" => GUID_ContainerFormatJpeg,
            "bmp" => GUID_ContainerFormatBmp,
            "gif" => GUID_ContainerFormatGif,
            "tiff" | "tif" => GUID_ContainerFormatTiff,
            "webp" => GUID {
                data1: 0xe094b661,
                data2: 0x17bd,
                data3: 0x4be2,
                data4: [0xb7, 0x58, 0xf4, 0x6b, 0x6e, 0xc5, 0x90, 0x05],
            },
            "heif" | "heic" | "avif" => GUID {
                data1: 0x0d62f838,
                data2: 0x5e0c,
                data3: 0x4a0f,
                data4: [0x90, 0x2c, 0x57, 0x24, 0x8c, 0x80, 0xa3, 0xa6],
            },
            _ => GUID_ContainerFormatPng,
        };

        let encoder = factory
            .CreateEncoder(&encoder_clsid, std::ptr::null())
            .map_err(|e| format!("CreateEncoder failed: {}", e))?;

        let file_hstring = HSTRING::from(file_path);
        let stream = factory
            .CreateStream()
            .map_err(|e| format!("CreateStream failed: {}", e))?;

        stream
            .InitializeFromFilename(&file_hstring, 0x40000000)
            .map_err(|e| format!("InitializeFromFilename failed: {}", e))?;

        encoder
            .Initialize(&stream, WICBitmapEncoderNoCache)
            .map_err(|e| format!("Initialize encoder failed: {}", e))?;

        let mut frame = None;
        let mut encoder_options = None;
        encoder
            .CreateNewFrame(&mut frame, &mut encoder_options)
            .map_err(|e| format!("CreateNewFrame failed: {}", e))?;
        let frame = frame.unwrap();

        frame
            .Initialize(encoder_options.as_ref())
            .map_err(|e| format!("Initialize frame failed: {}", e))?;
        frame
            .SetSize(width, height)
            .map_err(|e| format!("SetSize failed: {}", e))?;

        let mut format = GUID_WICPixelFormat32bppBGRA;
        frame
            .SetPixelFormat(&mut format)
            .map_err(|e| format!("SetPixelFormat failed: {}", e))?;

        frame
            .WritePixels(height, stride, pixels)
            .map_err(|e| format!("WritePixels failed: {}", e))?;

        frame
            .Commit()
            .map_err(|e| format!("Commit frame failed: {}", e))?;
        encoder
            .Commit()
            .map_err(|e| format!("Commit encoder failed: {}", e))?;

        Ok(())
    }
}

pub fn screenshot_gpu(
    input_path: &str,
    output_path: &str,
    time_ms: u32,
) -> std::result::Result<(), String> {
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED)
            .ok()
            .map_err(|e| e.to_string())?;
        MFStartup(MF_VERSION, MFSTARTUP_FULL).map_err(|e| e.to_string())?;

        let mut d3d_device: Option<ID3D11Device> = None;
        let mut d3d_context: Option<ID3D11DeviceContext> = None;
        let mut feature_level = D3D_FEATURE_LEVEL_11_0;
        let levels = [D3D_FEATURE_LEVEL_11_0];

        D3D11CreateDevice(
            None::<&IDXGIAdapter>,
            D3D_DRIVER_TYPE_HARDWARE,
            HMODULE(std::ptr::null_mut()),
            D3D11_CREATE_DEVICE_BGRA_SUPPORT,
            Some(&levels),
            D3D11_SDK_VERSION,
            Some(&mut d3d_device as *mut _),
            Some(&mut feature_level as *mut _),
            Some(&mut d3d_context as *mut _),
        )
        .map_err(|e| format!("Failed to create D3D11 Device: {}", e))?;
        let device: ID3D11Device = d3d_device.unwrap();
        let context = d3d_context.unwrap();

        let mut token = 0;
        let mut manager_opt = None;
        MFCreateDXGIDeviceManager(&mut token, &mut manager_opt)
            .map_err(|e| format!("MFCreateDXGIDeviceManager failed: {}", e))?;
        let manager = manager_opt.unwrap();
        manager
            .ResetDevice(&device, token)
            .map_err(|e| format!("ResetDevice failed: {}", e))?;

        let mut attr_opt = None;
        MFCreateAttributes(&mut attr_opt, 1)
            .map_err(|e| format!("MFCreateAttributes failed: {}", e))?;
        let attr = attr_opt.unwrap();
        attr.SetUnknown(&MF_SOURCE_READER_D3D_MANAGER, &manager)
            .map_err(|e| format!("SetUnknown failed: {}", e))?;

        let url = HSTRING::from(input_path);
        let reader = MFCreateSourceReaderFromURL(&url, Some(&attr))
            .map_err(|e| format!("MFCreateSourceReaderFromURL failed: {}", e))?;

        let mt = MFCreateMediaType().map_err(|e| e.to_string())?;
        mt.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)
            .map_err(|e| e.to_string())?;
        mt.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_NV12)
            .map_err(|e| e.to_string())?;
        reader
            .SetCurrentMediaType(MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32, None, &mt)
            .map_err(|e| e.to_string())?;

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

        if time_ms > 0 {
            let position = time_ms as i64 * 10000;
            let mut var = PROPVARIANT::default();
            let anon = &mut *var.Anonymous.Anonymous;
            anon.vt = windows::Win32::System::Variant::VT_I8;
            anon.Anonymous.hVal = position;

            let guid_null = windows::core::GUID::default();
            reader
                .SetCurrentPosition(&guid_null, &var)
                .map_err(|e| format!("Seek failed: {}", e))?;
        }

        let mut actual_stream_index = 0;
        let mut flags = 0;
        let mut timestamp = 0;
        let mut sample = None;

        for _ in 0..20 {
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

            if sample.is_some() {
                break;
            }
        }

        let sample = sample
            .ok_or_else(|| "Failed to read a valid video frame at this position".to_string())?;
        let buffer = sample
            .GetBufferByIndex(0)
            .map_err(|e| format!("GetBufferByIndex failed: {}", e))?;
        let nv12_texture = crate::gpu_pipeline::get_texture_from_buffer(&buffer)
            .map_err(|e| format!("Failed to extract GPU texture: {}", e))?;

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

        let bgra_desc = D3D11_TEXTURE2D_DESC {
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
        let mut gpu_bgra_texture = None;
        device
            .CreateTexture2D(&bgra_desc, None, Some(&mut gpu_bgra_texture))
            .map_err(|e| format!("Create BGRA texture failed: {}", e))?;
        let gpu_bgra_texture = gpu_bgra_texture.unwrap();

        let out_view_desc = D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC {
            ViewDimension: D3D11_VPOV_DIMENSION_TEXTURE2D,
            Anonymous: D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC_0 {
                Texture2D: D3D11_TEX2D_VPOV { MipSlice: 0 },
            },
        };
        let mut output_view = None;
        video_device
            .CreateVideoProcessorOutputView(
                &gpu_bgra_texture,
                &enumerator,
                &out_view_desc,
                Some(&mut output_view),
            )
            .map_err(|e| format!("CreateVideoProcessorOutputView failed: {}", e))?;
        let output_view = output_view.unwrap();

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
                &nv12_texture,
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

        let staging_desc = D3D11_TEXTURE2D_DESC {
            Width: width,
            Height: height,
            MipLevels: 1,
            ArraySize: 1,
            Format: DXGI_FORMAT_B8G8R8A8_UNORM,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Usage: D3D11_USAGE_STAGING,
            BindFlags: 0,
            CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
            MiscFlags: 0,
        };
        let mut staging_texture = None;
        device
            .CreateTexture2D(&staging_desc, None, Some(&mut staging_texture))
            .map_err(|e| format!("Create staging texture failed: {}", e))?;
        let staging_texture = staging_texture.unwrap();

        context.CopyResource(&staging_texture, &gpu_bgra_texture);

        let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
        context
            .Map(&staging_texture, 0, D3D11_MAP_READ, 0, Some(&mut mapped))
            .map_err(|e| format!("Map staging texture failed: {}", e))?;

        let row_pitch = mapped.RowPitch as usize;
        let data_size = row_pitch * height as usize;
        let pixels_slice = std::slice::from_raw_parts(mapped.pData as *const u8, data_size);

        let save_res =
            save_pixels_to_file(output_path, width, height, mapped.RowPitch, pixels_slice);

        context.Unmap(&staging_texture, 0);

        save_res?;

        Ok(())
    }
}
