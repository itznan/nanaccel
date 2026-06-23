pub fn print_help() {
    println!("\x1b[1m\x1b[36mNVIDIA Hardware Accelerated Video CLI Tool (NanAccel)\x1b[0m");
    println!("Written in Rust with zero compile-time dependencies to avoid application controls.");
    println!("\n\x1b[1mUsage:\x1b[0m");
    println!("  nanaccel <subcommand> [options]");
    println!("\n\x1b[1mSubcommands:\x1b[0m");
    println!(
        "  \x1b[32minfo\x1b[0m                              Print NVIDIA GPU capabilities & live status"
    );
    println!(
        "  \x1b[32mplay <input>\x1b[0m                      Play a video file using hardware-accelerated NVDEC"
    );
    println!("       [-d, --decoder <decoder>]    Specify decoder (e.g., h264_cuvid, hevc_cuvid)");
    println!("       [--no-audio]                 Disable audio");
    println!("       [--loop]                     Loop playback infinitely");
    println!(
        "  \x1b[32mtranscode <input> <output>\x1b[0m        Transcode video using NVDEC -> CUDA -> NVENC"
    );
    println!(
        "       [-c, --codec <codec>]        Target video codec: h264, hevc, av1, vp8, vp9, mpeg1, mpeg2, mpeg4, mjpeg, prores, dnxhd, cineform, huffyuv, ffv1, theora, dirac, vc1, wmv, xvid, divx (default: h264)"
    );
    println!(
        "       [-ac, --audio-codec <codec>] Target audio codec: aac, mp3, flac, opus, vorbis, pcm, alac, ac3, e-ac3, dts, amr, speex, wma, gsm, truehd, atmos, dts-hd"
    );
    println!(
        "       [-p, --preset <preset>]      NVENC preset: p1 (fastest) to p7 (slowest) (default: p4)"
    );
    println!("       [-b, --bitrate <bitrate>]    Output video bitrate (e.g., 5M, 800k)");
    println!("       [--scale <width>x<height>]   Scale resolution on the GPU (e.g., 1280x720)");
    println!("       [--transcode-audio]          Transcode audio to AAC (default: copy stream)");
    println!(
        "  \x1b[32mscreenshot <input> <output>\x1b[0m       Extract a single frame from the video at a timestamp"
    );
    println!("       [-t, --time <ms>]            Timestamp in milliseconds (default: 0)");
    println!("  \x1b[32msubtitle <operation> <input> <output>\x1b[0m Subtitle processing utility");
    println!("       Operations:                  extract, convert, burn, sync, merge, remove");
    println!(
        "       Format Support:              SRT, ASS, SSA, VTT, PGS, DVB, DVD subtitles, Teletext"
    );
    println!(
        "       [-s, --sub-file <file>]      Specify subtitle file for burn / merge operations"
    );
    println!(
        "       [-t, --track <idx>]          Specify track index for subtitle extraction (default: 0)"
    );
    println!(
        "       [--shift <ms>]               Specify timestamp shift in milliseconds for sync operation"
    );
    println!("  \x1b[32medit <operation> <input> <output>\x1b[0m  Video editing utility");
    println!(
        "       Operations:                  trim, cut, split, join, concat, crop, rotate, flip, scale,"
    );
    println!(
        "                                    stabilize, denoise, sharpen, deblock, deinterlace, reverse,"
    );
    println!("                                    loop, fade, crossfade, overlay, watermark");
    println!("       [-ss, --start <time>]        Specify start time for trim / cut / fade");
    println!("       [-to, --end <time>]          Specify end time for trim / cut");
    println!("       [-t, --duration <time/sec>]  Specify duration for trim / cut / split / fade");
    println!("       [--crop <w:h:x:y>]           Crop window parameter");
    println!("       [--rotate <angle>]           Rotate angle: 90, 180, 270");
    println!("       [--flip <h|v|both>]          Flip direction");
    println!("       [--scale <w>x<h>]            Target output resolution");
    println!("       [--loop-count <N>]           Specify number of loops");
    println!("       [--fade-in / --fade-out]     Specify fade options (e.g. st=0:d=2)");
    println!(
        "       [-f, --file / --overlay]     Specify secondary overlay / crossfade video file"
    );
    println!("       [--watermark-text <text>]    Specify drawtext watermark text");
    println!("       [--position <x:y>]           Overlay/drawtext positioning");
    println!("  \x1b[32mcolor <operation> <input> <output>\x1b[0m Color processing utility");
    println!(
        "       Operations:                  hdr2sdr, sdr2hdr, lut, gamma, grading, colorspace,"
    );
    println!("                                    whitebalance, adjust, tonemap");
    println!("       [-l, --lut <file>]           Specify LUT file for lut operation");
    println!("       [--gamma <val>]              Specify gamma value (default: 1.0)");
    println!(
        "       [--shadows / --midtones / --highlights <r:g:b>] Lift/Gamma/Gain color balance controls"
    );
    println!(
        "       [--colorspace <space>]       Specify colorspace conversion (e.g. bt709, bt2020)"
    );
    println!(
        "       [--temp <K>]                 Specify color temperature in Kelvin for white balance"
    );
    println!("       [--brightness / --contrast / --saturation <val>] Adjust controls");
    println!("       [--tonemap <algo>]           Tone mapping algorithm");
    println!("  \x1b[32maudio <operation> <input> <output>\x1b[0m Audio editing utility");
    println!(
        "       Operations:                  volume / volume-control, denoise / noise-reduction,"
    );
    println!(
        "                                    compress / compression, limit / limiter, eq / equalizer,"
    );
    println!(
        "                                    pitch / pitch-shift, tempo / tempo-change, reverb, echo,"
    );
    println!(
        "                                    bass / bass-boost, silencedetect / silence-detection,"
    );
    println!("                                    normalize / audio-normalization");
    println!("       [--volume <val>]             Volume scale multiplier or dB (default: 1.0)");
    println!("       [--nr <val>]                 Noise reduction level (default: 12)");
    println!("       [--threshold / --ratio <val>] Compression parameters (default: -21dB / 4)");
    println!(
        "       [--limit <val>]              Lookahead limiter input/output limit (default: 0.1)"
    );
    println!("       [--freq / --gain <val>]      Parametric equalizer band options");
    println!("       [--pitch <val>]              Pitch shift scale (default: 1.0)");
    println!("       [--tempo <val>]              Tempo speed scale (default: 1.0)");
    println!(
        "       [--loudness <val>]           Target LUFS normalization loudness (default: -16)"
    );
    println!("       [--silence-db / --silence-duration <val>] Silence detection parameters");
    println!(
        "  \x1b[32mrecord <output>\x1b[0m                   Ultra-low overhead screen recording"
    );
    println!("       [--fps <fps>]                Record frame rate (default: 60)");
    println!("       [-b, --bitrate <bitrate>]    Encoding bitrate (default: 8M)");
    println!("       [-d, --duration <sec>]       Max duration in seconds (default: 5)");
    println!(
        "  \x1b[32moverlay <input> <output> <overlay>\x1b[0m GPU-accelerated image/text overlay"
    );
    println!("       [-p, --position <x:y>]       Overlay position coordinates (default: 10:10)");
    println!("       [-t, --type <type>]          Type: logo, watermark, timecode (default: logo)");
    println!(
        "  \x1b[32mshader <input> <output> <shader>\x1b[0m  HLSL shader effect rendering pipeline"
    );
    println!(
        "  \x1b[32mmux --video <v_in> --audio <a_in> --output <out>\x1b[0m Multiplex tracks into MP4 container"
    );
    println!();
}
