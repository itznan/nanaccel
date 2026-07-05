use std::time::{Duration, Instant};
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Direct3D::*;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dxgi::Common::*;
use windows::Win32::Graphics::Dxgi::*;
use windows::Win32::Media::MediaFoundation::*;
use windows::Win32::System::Com::StructuredStorage::PROPVARIANT;
use windows::Win32::System::Com::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::*;

use super::get_texture_from_buffer;

use std::sync::atomic::{AtomicBool, Ordering};
static IS_PAUSED: AtomicBool = AtomicBool::new(false);

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            WM_KEYDOWN => {
                let key = wparam.0 as i32;
                if key == 0x20 {
                    // Space key
                    let prev = IS_PAUSED.load(Ordering::Relaxed);
                    IS_PAUSED.store(!prev, Ordering::Relaxed);
                    println!(
                        "[NanAccel Video Player] {}",
                        if !prev { "Paused" } else { "Resumed" }
                    );
                } else if key == 0x1B || key == 0x51 {
                    // ESC or Q key
                    PostQuitMessage(0);
                }
                LRESULT(0)
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

pub fn create_video_window(width: u32, height: u32) -> Result<HWND> {
    unsafe {
        let instance = windows::Win32::System::LibraryLoader::GetModuleHandleW(None)?;
        let class_name: Vec<u16> = "NanAccelVideoPlayerClass\0".encode_utf16().collect();

        let wnd_class = WNDCLASSW {
            lpfnWndProc: Some(window_proc),
            hInstance: instance.into(),
            lpszClassName: PCWSTR(class_name.as_ptr()),
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            ..Default::default()
        };

        windows::Win32::UI::WindowsAndMessaging::RegisterClassW(&wnd_class);

        let window_title: Vec<u16> = "NanAccel Video Player - GPU Accelerated\0"
            .encode_utf16()
            .collect();

        let mut rect = RECT {
            left: 0,
            top: 0,
            right: width as i32,
            bottom: height as i32,
        };
        let _ = AdjustWindowRect(&mut rect, WS_OVERLAPPEDWINDOW, false);

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            PCWSTR(class_name.as_ptr()),
            PCWSTR(window_title.as_ptr()),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            rect.right - rect.left,
            rect.bottom - rect.top,
            None,
            None,
            Some(HINSTANCE(instance.0)),
            None,
        )?;

        Ok(hwnd)
    }
}

#[allow(clippy::collapsible_if)]
pub fn play_gpu(
    input_path: &str,
    _no_audio: bool,
    loop_video: bool,
) -> std::result::Result<(), String> {
    unsafe {
        IS_PAUSED.store(false, Ordering::Relaxed);
        // Init COM & WMF
        CoInitializeEx(None, COINIT_MULTITHREADED)
            .ok()
            .map_err(|e| e.to_string())?;
        MFStartup(MF_VERSION, MFSTARTUP_FULL).map_err(|e| e.to_string())?;

        // Create D3D11 Device
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

        // Create Attributes
        let mut attr_opt = None;
        MFCreateAttributes(&mut attr_opt, 1)
            .map_err(|e| format!("MFCreateAttributes failed: {}", e))?;
        let attr = attr_opt.unwrap();
        attr.SetUnknown(&MF_SOURCE_READER_D3D_MANAGER, &manager)
            .map_err(|e| format!("SetUnknown failed: {}", e))?;

        // Create Source Reader
        let url = HSTRING::from(input_path);
        let reader = MFCreateSourceReaderFromURL(&url, Some(&attr))
            .map_err(|e| format!("MFCreateSourceReaderFromURL failed: {}", e))?;

        // Get dimensions
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

        // Set output type to NV12 (hardware decoding natively outputs NV12)
        let mt = MFCreateMediaType().map_err(|e| e.to_string())?;
        mt.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)
            .map_err(|e| e.to_string())?;
        mt.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_NV12)
            .map_err(|e| e.to_string())?;
        reader
            .SetCurrentMediaType(MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32, None, &mt)
            .map_err(|e| format!("Failed to set output format to NV12: {}", e))?;

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

        // Create window
        let hwnd = create_video_window(width, height)
            .map_err(|e| format!("Failed to create Win32 Window: {}", e))?;

        // Create Swap Chain
        let swap_chain_desc = DXGI_SWAP_CHAIN_DESC {
            BufferDesc: DXGI_MODE_DESC {
                Width: width,
                Height: height,
                RefreshRate: DXGI_RATIONAL {
                    Numerator: 60,
                    Denominator: 1,
                },
                Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                ..Default::default()
            },
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
            BufferCount: 2,
            OutputWindow: hwnd,
            Windowed: true.into(),
            SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
            ..Default::default()
        };

        let dxgi_device: IDXGIDevice = device.cast().map_err(|e| e.to_string())?;
        let dxgi_adapter = dxgi_device.GetAdapter().map_err(|e| e.to_string())?;
        let dxgi_factory = dxgi_adapter
            .GetParent::<IDXGIFactory>()
            .map_err(|e| e.to_string())?;

        let mut swap_chain_opt = None;
        dxgi_factory
            .CreateSwapChain(&device, &swap_chain_desc, &mut swap_chain_opt)
            .ok()
            .map_err(|e| format!("CreateSwapChain failed: {}", e))?;
        let swap_chain = swap_chain_opt.unwrap();

        let back_buffer: ID3D11Texture2D = swap_chain
            .GetBuffer(0)
            .map_err(|e| format!("GetBuffer failed: {}", e))?;

        let mut rtv = None;
        device
            .CreateRenderTargetView(&back_buffer, None, Some(&mut rtv))
            .map_err(|e| format!("CreateRenderTargetView failed: {}", e))?;
        let rtv = rtv.unwrap();
        context.OMSetRenderTargets(Some(&[Some(rtv)]), None);

        // Create Video Processor Output View
        let out_view_desc = D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC {
            ViewDimension: D3D11_VPOV_DIMENSION_TEXTURE2D,
            Anonymous: D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC_0 {
                Texture2D: D3D11_TEX2D_VPOV { MipSlice: 0 },
            },
        };
        let mut output_view = None;
        video_device
            .CreateVideoProcessorOutputView(
                &back_buffer,
                &enumerator,
                &out_view_desc,
                Some(&mut output_view),
            )
            .map_err(|e| format!("CreateVideoProcessorOutputView failed: {}", e))?;
        let output_view = output_view.unwrap();

        // Loop
        let mut start_time = Instant::now();
        let mut msg = MSG::default();

        'playback: loop {
            // Process window messages
            while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
                if msg.message == WM_QUIT {
                    break 'playback;
                }
            }

            if IS_PAUSED.load(Ordering::Relaxed) {
                std::thread::sleep(Duration::from_millis(15));
                start_time += Duration::from_millis(15);
                continue;
            }

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
                if loop_video {
                    let mut var = PROPVARIANT::default();
                    (*var.Anonymous.Anonymous).vt = windows::Win32::System::Variant::VARENUM(20); // VT_I8
                    (*var.Anonymous.Anonymous).Anonymous.hVal = 0;
                    reader
                        .SetCurrentPosition(
                            &GUID::default() as *const GUID,
                            &var as *const PROPVARIANT,
                        )
                        .map_err(|e| format!("Loop seek failed: {}", e))?;
                    start_time = Instant::now();
                    continue;
                } else {
                    break 'playback;
                }
            }

            if let Some(sample) = sample {
                if let Ok(buffer) = sample.GetBufferByIndex(0) {
                    if let Ok(src_texture) = get_texture_from_buffer(&buffer) {
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
                        if video_device
                            .CreateVideoProcessorInputView(
                                &src_texture,
                                &enumerator,
                                &in_view_desc,
                                Some(&mut input_view),
                            )
                            .is_ok()
                        {
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
                            let _ = video_context.VideoProcessorBlt(
                                &processor,
                                &output_view,
                                0,
                                &[stream],
                            );
                            swap_chain
                                .Present(1, DXGI_PRESENT(0))
                                .ok()
                                .map_err(|e| e.to_string())?;
                        }
                    }
                }
            }

            // Wait/align based on actual presentation timestamp (timestamp is in 100ns units)
            let expected_time = start_time + Duration::from_nanos(timestamp as u64 * 100);
            let now = Instant::now();
            if now < expected_time {
                std::thread::sleep(expected_time - now);
            }
        }

        Ok(())
    }
}
