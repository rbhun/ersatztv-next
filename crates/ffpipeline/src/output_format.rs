use std::time::Duration;

use crate::ArgVec;
use crate::frame_rate::FrameRate;
use crate::pipeline::{KEYFRAME_INTERVAL_SECONDS, OutputContext, SEGMENT_SECONDS};
use crate::video_codec::VideoEncoder;

pub(crate) fn gop_frames(frame_rate: &FrameRate) -> u32 {
    frame_rate.parsed_frame_rate.round_ties_even() as u32 * KEYFRAME_INTERVAL_SECONDS
}

#[derive(Debug, Clone)]
pub enum OutputFormat {
    Hls {
        playlist: String,
        segment_template: String,
        troubleshoot: bool,
        /// When true, write MPEG-TS segments with the `segment` muxer so in-band
        /// subtitle streams (DVB) stay inside the `.ts` files. The HLS muxer only
        /// accepts WebVTT sidecars and rejects dvdsub/dvbsub / multiple subtitle tracks.
        in_band_subtitles: bool,
    },
}

impl OutputFormat {
    pub(crate) fn as_arg(&self, output_context: &OutputContext) -> ArgVec {
        let force_key_frames_expr = format!("expr:gte(t,n_forced*{KEYFRAME_INTERVAL_SECONDS})");
        let segment_seconds = format!("{SEGMENT_SECONDS}");
        // TODO: 1-second GOP for qsv
        let gop = format!("{}", gop_frames(&output_context.frame_rate));
        let keyint_min = gop.clone();

        let mut args: ArgVec = Vec::new();

        match self {
            OutputFormat::Hls {
                playlist,
                segment_template,
                troubleshoot,
                in_band_subtitles,
            } => {
                if output_context.video_encoder != VideoEncoder::Copy {
                    args.extend(args![
                        "-g",
                        gop,
                        "-keyint_min",
                        keyint_min,
                        "-force_key_frames",
                        force_key_frames_expr,
                    ]);
                }

                if *in_band_subtitles {
                    // HLS muxer cannot put bitmap subtitles into MPEG-TS segments.
                    args.extend(args![
                        "-f",
                        "segment",
                        "-segment_format",
                        "mpegts",
                        "-segment_time",
                        segment_seconds,
                        "-segment_list_size",
                        "0",
                        "-segment_list_flags",
                        "+live",
                        "-segment_list_type",
                        "m3u8",
                        "-segment_list",
                        playlist.to_owned(),
                    ]);

                    match output_context.pts_offset {
                        Some(pts_offset) if pts_offset.duration > Duration::ZERO => {}
                        _ => args.extend(args![
                            "-segment_format_options",
                            "mpegts_flags=+initial_discontinuity",
                        ]),
                    }

                    // keep append behaviour for continuous channel output
                    if !*troubleshoot {
                        args.extend(args!["-break_non_keyframes", "1"]);
                    }

                    args.extend(args![segment_template.to_owned()]);
                } else {
                    args.extend(args![
                        "-f",
                        "hls",
                        "-hls_time",
                        segment_seconds,
                        "-hls_list_size",
                        "0",
                        "-segment_list_flags",
                        "+live",
                        "-hls_segment_filename",
                        segment_template.to_owned(),
                        "-hls_segment_type",
                        "mpegts",
                    ]);

                    if *troubleshoot {
                        args.extend(args!["-hls_flags", "append_list+independent_segments"]);
                    } else {
                        args.extend(args![
                            "-hls_flags",
                            "program_date_time+omit_endlist+append_list+independent_segments"
                        ]);
                    }

                    match output_context.pts_offset {
                        Some(pts_offset) if pts_offset.duration > Duration::ZERO => {}
                        _ => args.extend(args![
                            "-hls_segment_options",
                            "mpegts_flags=+initial_discontinuity",
                        ]),
                    }

                    args.extend(args![playlist.to_owned()]);
                }
            }
        }

        args
    }
}
