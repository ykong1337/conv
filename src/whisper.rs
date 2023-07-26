use std::io::{Error, ErrorKind};
use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext};

use crate::config::{Language, Model};
use crate::utils;

#[derive(Debug, Serialize, Deserialize)]
pub struct Transcript {
    pub processing_time: Duration,
    pub utterances: Vec<Utterance>,
    pub word_utterances: Option<Vec<Utterance>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Utterance {
    pub start: i64,
    pub end: i64,
    pub text: String,
}

pub struct Whisper {
    ctx: WhisperContext,
    lang: Language,
}

impl Whisper {
    pub async fn new(lang: Language, model: Model) -> std::io::Result<Self> {
        model.download().await?;
        Ok(Self {
            ctx: WhisperContext::new(model.get_path().to_str().unwrap()).map_err(|_| Error::from(ErrorKind::InvalidData))?,
            lang,
        })
    }

    pub fn transcribe<P: AsRef<Path>>(&mut self, audio: P, translate: bool, word_timestamps: bool) -> anyhow::Result<Transcript> {
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        params.set_translate(translate);
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_token_timestamps(word_timestamps);
        params.set_language(Some(<&str>::from(self.lang)));

        let audio = utils::read_file(audio)?;

        let st = Instant::now();
        let mut state = self.ctx.create_state().expect("failed to create state");
        state.full(params, &audio).expect("failed to transcribe");

        let num_segments = state.full_n_segments().expect("failed to get segments");
        if num_segments == 0 {
            return Err(anyhow!("No segments found"));
        };

        let mut words = vec![];
        let mut utterances = vec![];
        for s in 0..num_segments {
            let text = state
                .full_get_segment_text(s)
                .map_err(|e| anyhow!("failed to get segment due to {:?}", e))?;
            let start = state
                .full_get_segment_t0(s)
                .map_err(|e| anyhow!("failed to get segment due to {:?}", e))?;
            let end = state
                .full_get_segment_t1(s)
                .map_err(|e| anyhow!("failed to get segment due to {:?}", e))?;

            utterances.push(Utterance { text, start, end });

            if !word_timestamps {
                continue;
            }

            let num_tokens = state
                .full_n_tokens(s)
                .map_err(|e| anyhow!("failed to get segment due to {:?}", e))?;

            for t in 0..num_tokens {
                let text = state
                    .full_get_token_text(s, t)
                    .map_err(|e| anyhow!("failed to get token due to {:?}", e))?;
                let token_data = state
                    .full_get_token_data(s, t)
                    .map_err(|e| anyhow!("failed to get token due to {:?}", e))?;

                if text.starts_with("[_") {
                    continue;
                }

                words.push(Utterance {
                    text,
                    start: token_data.t0,
                    end: token_data.t1,
                });
            }
        }

        Ok(Transcript {
            utterances,
            processing_time: Instant::now().duration_since(st),
            word_utterances: if word_timestamps { Some(words) } else { None },
        })
    }
}

impl Transcript {
    pub fn as_lrc(&self) -> String {
        self.word_utterances
            .as_ref()
            .unwrap_or(&self.utterances)
            .iter()
            .fold(String::new(), |lrc, fragment| {
                lrc +
                    &format!(
                        "[{:02}:{:02}.{:02}]{}\n[{:02}:{:02}.{:02}]\n",
                        fragment.start / 100 / 60,
                        fragment.start / 100 % 60,
                        fragment.start % 100,
                        fragment.text,
                        fragment.end / 100 / 60,
                        fragment.end / 100 % 60,
                        fragment.end % 100,
                    )
            })
    }
}