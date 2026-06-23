use mp4::{AvcConfig, MediaConfig, Mp4Config, Mp4Sample, Mp4Writer, TrackConfig};
use std::fs::File;

pub struct Muxer {
    writer: Mp4Writer<File>,
    track_id: u32,
    current_time: u64,
}

impl Muxer {
    pub fn create(
        path: &str,
        width: u16,
        height: u16,
        sps: &[u8],
        pps: &[u8],
    ) -> Result<Self, String> {
        let file =
            File::create(path).map_err(|e| format!("Failed to create output file: {}", e))?;

        let config = Mp4Config {
            major_brand: str::parse("isom").map_err(|_| "Invalid major brand")?,
            minor_version: 512,
            compatible_brands: vec![
                str::parse("isom").map_err(|_| "Invalid brand")?,
                str::parse("iso2").map_err(|_| "Invalid brand")?,
                str::parse("avc1").map_err(|_| "Invalid brand")?,
                str::parse("mp41").map_err(|_| "Invalid brand")?,
            ],
            timescale: 1000,
        };

        let mut writer = Mp4Writer::write_start(file, &config)
            .map_err(|e| format!("Failed to start MP4 writer: {}", e))?;

        let avc = AvcConfig {
            width,
            height,
            seq_param_set: sps.to_vec(),
            pic_param_set: pps.to_vec(),
        };

        let track_config = TrackConfig {
            language: "und".to_string(),
            timescale: 1000, // 1000 ticks per second (1ms resolution)
            track_type: mp4::TrackType::Video,
            media_conf: MediaConfig::AvcConfig(avc),
        };

        writer
            .add_track(&track_config)
            .map_err(|e| format!("Failed to add video track: {}", e))?;

        // Track IDs are 1-indexed in mp4 crate, so the first added track is 1
        let track_id = 1;

        Ok(Self {
            writer,
            track_id,
            current_time: 0,
        })
    }

    pub fn write_video_frame(
        &mut self,
        annex_b: &[u8],
        duration_ms: u32,
        is_sync: bool,
    ) -> Result<(), String> {
        let avcc_data = annex_b_to_avcc(annex_b);
        if avcc_data.is_empty() {
            return Ok(());
        }

        let sample = Mp4Sample {
            bytes: bytes::Bytes::from(avcc_data),
            duration: duration_ms,
            is_sync,
            rendering_offset: 0,
            start_time: self.current_time,
        };

        self.writer
            .write_sample(self.track_id, &sample)
            .map_err(|e| format!("Failed to write sample: {}", e))?;
        self.current_time += duration_ms as u64;

        Ok(())
    }

    pub fn close(mut self) -> Result<(), String> {
        self.writer
            .write_end()
            .map_err(|e| format!("Failed to finalize MP4 file: {}", e))?;
        Ok(())
    }
}

/// Converts Annex B H.264 stream format (start-code prefixed) to AVCC length-prefixed format.
/// Skips SPS (7) and PPS (8) NAL units since they are stored in the container track header.
pub fn annex_b_to_avcc(annex_b: &[u8]) -> Vec<u8> {
    let mut avcc = Vec::new();
    let mut i = 0;

    // Find all start codes and extract payloads
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

        let nal_payload = &annex_b[start..end];
        if nal_payload.is_empty() {
            continue;
        }

        // Filter out SPS (7) and PPS (8)
        let nal_type = nal_payload[0] & 0x1F;
        if nal_type == 7 || nal_type == 8 {
            continue;
        }

        let len = nal_payload.len() as u32;
        avcc.extend_from_slice(&len.to_be_bytes());
        avcc.extend_from_slice(nal_payload);
    }

    avcc
}
