use super::subcommands::{
    audio, color, edit, info, mux, overlay, play, record, screenshot, shader, subtitle, transcode,
};
use super::types::Commands;

pub fn execute(cmd: Commands) {
    match cmd {
        Commands::Play {
            input,
            decoder: _,
            no_audio,
            loop_video,
        } => {
            play::run(&input, no_audio, loop_video);
        }

        Commands::Transcode {
            input,
            output,
            codec,
            preset,
            bitrate,
            scale,
            transcode_audio,
            audio_codec,
        } => {
            transcode::run(
                &input,
                &output,
                &codec,
                &preset,
                bitrate.as_deref(),
                scale.as_deref(),
                transcode_audio,
                audio_codec.as_deref(),
            );
        }

        Commands::Screenshot {
            input,
            output,
            time_ms,
        } => {
            screenshot::run(&input, &output, time_ms);
        }

        Commands::Subtitle {
            input,
            output,
            operation,
            sub_file,
            shift_ms,
            track_index,
        } => {
            subtitle::run(
                &input,
                &output,
                &operation,
                sub_file.as_deref(),
                shift_ms,
                track_index,
            );
        }

        Commands::Edit {
            input,
            output,
            operation,
            start_time,
            end_time,
            duration,
            crop,
            rotate,
            flip,
            scale,
            loop_count,
            fade_in,
            fade_out,
            overlay_file,
            watermark_text,
            position,
            additional_inputs,
        } => {
            edit::run(
                &input,
                &output,
                &operation,
                start_time.as_deref(),
                end_time.as_deref(),
                duration.as_deref(),
                crop.as_deref(),
                rotate.as_deref(),
                flip.as_deref(),
                scale.as_deref(),
                loop_count,
                fade_in.as_deref(),
                fade_out.as_deref(),
                overlay_file.as_deref(),
                watermark_text.as_deref(),
                position.as_deref(),
                &additional_inputs,
            );
        }

        Commands::Color {
            input,
            output,
            operation,
            lut_file,
            gamma,
            shadows,
            midtones,
            highlights,
            colorspace,
            temperature,
            brightness,
            contrast,
            saturation,
            tonemap,
        } => {
            color::run(
                &input,
                &output,
                &operation,
                lut_file.as_deref(),
                gamma.as_deref(),
                shadows.as_deref(),
                midtones.as_deref(),
                highlights.as_deref(),
                colorspace.as_deref(),
                temperature,
                brightness,
                contrast,
                saturation,
                tonemap.as_deref(),
            );
        }

        Commands::Audio {
            input,
            output,
            operation,
            volume,
            noise_reduction,
            threshold,
            ratio,
            limit,
            gain,
            frequency,
            pitch,
            tempo,
            loudness,
            silence_db,
            silence_duration,
        } => {
            audio::run(
                &input,
                &output,
                &operation,
                volume.as_deref(),
                noise_reduction.as_deref(),
                threshold.as_deref(),
                ratio.as_deref(),
                limit.as_deref(),
                gain.as_deref(),
                frequency.as_deref(),
                pitch,
                tempo,
                loudness.as_deref(),
                silence_db.as_deref(),
                silence_duration.as_deref(),
            );
        }

        Commands::Record {
            output,
            fps,
            bitrate,
            duration_sec,
        } => {
            record::run(&output, fps, bitrate.as_deref(), duration_sec);
        }

        Commands::Overlay {
            input,
            output,
            overlay_file,
            position,
            overlay_type,
        } => {
            overlay::run(
                &input,
                &output,
                &overlay_file,
                position.as_deref(),
                overlay_type.as_deref(),
            );
        }

        Commands::Shader {
            input,
            output,
            shader_file,
        } => {
            shader::run(&input, &output, &shader_file);
        }

        Commands::Mux {
            video_input,
            audio_input,
            output,
        } => {
            mux::run(&video_input, &audio_input, &output);
        }

        Commands::Info => {
            info::run();
        }
    }
}
