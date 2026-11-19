use crate::ArgVec;
use crate::output_settings::OutputSettings;
use crate::probe::ProbeResultAudioStream;

pub struct AudioDecoder {
    stream_index: u32,
    input_codec: String,
    input_channels: u32,
    output_channels: Option<u32>,
}

impl AudioDecoder {
    pub fn new(
        audio_stream: &ProbeResultAudioStream,
        output_settings: &OutputSettings,
    ) -> AudioDecoder {
        AudioDecoder {
            stream_index: audio_stream.stream_index,
            input_codec: audio_stream.codec.to_owned(),
            input_channels: audio_stream.channels,
            output_channels: output_settings.audio.transcode.channels,
        }
    }

    pub(crate) fn as_arg(&self) -> ArgVec {
        if self.input_codec == "ac3" && self.input_channels > 2 && self.output_channels == Some(2) {
            // scoped to this stream so other audio streams read from the same input keep their decoder
            return args![
                format!("-c:{}", self.stream_index),
                "ac3",
                "-downmix",
                "stereo"
            ];
        }

        Vec::new()
    }
}
