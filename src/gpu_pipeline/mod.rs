pub mod play;
pub mod screenshot;
pub mod shader;
pub mod transcode;

pub use play::play_gpu;
pub use screenshot::screenshot_gpu;
pub use shader::shader_gpu;
pub use transcode::transcode_gpu;

// Shared GPU helper
use windows::Win32::Graphics::Direct3D11::ID3D11Texture2D;
use windows::Win32::Media::MediaFoundation::{IMFDXGIBuffer, IMFMediaBuffer};
use windows::core::*;

pub unsafe fn get_texture_from_buffer(buffer: &IMFMediaBuffer) -> Result<ID3D11Texture2D> {
    let dxgi_buffer: IMFDXGIBuffer = buffer.cast()?;
    let mut texture: Option<ID3D11Texture2D> = None;
    unsafe {
        dxgi_buffer.GetResource(&ID3D11Texture2D::IID, &mut texture as *mut _ as *mut *mut _)?;
    }
    Ok(texture.unwrap())
}
