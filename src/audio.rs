//! Procedural UI sound.
//!
//! a28 settled that audio is synthesized, never sampled; a137(a) scoped this
//! pass to **UI feedback only**: tool select, place, refuse, notification.
//! A sound, like an animation, must indicate something of use (a62) — so
//! there is no ambient bed and no music here, and the four sounds map to
//! exactly four world events, one to one.
//!
//! Synthesis is pure and deterministic: [`synthesize`] turns a [`Sound`] into
//! mono f32 samples with no clock, no rand and no I/O, so a test can assert
//! what a sound *is* (its length, its peak, that it decays to silence) and
//! the same bytes ship every time. The output wrapper is the only impure
//! piece, and it lives behind [`Player`] so the game binary carries it and
//! the lib does not.

/// The four events the interface speaks. Closed on purpose: a fifth event
/// means a design decision, not a new enum variant smuggled in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sound {
    /// A tool was selected. Short, bright, neutral.
    Select,
    /// An action landed in the world — a road leg, a zone paint, a building.
    Place,
    /// The governor refused an action. Distinctly *not* an error tone: lower,
    /// softer, over quickly — the city is unchanged.
    Refuse,
    /// A notification: a ticket validated, an autosave landed.
    Notify,
}

impl Sound {
    /// Length in seconds. Kept short: feedback, not ambience.
    pub fn seconds(self) -> f32 {
        match self {
            Sound::Select => 0.06,
            Sound::Place => 0.10,
            Sound::Refuse => 0.14,
            Sound::Notify => 0.18,
        }
    }
}

/// The one constant the synthesis is built from: samples per second. The
/// player resamples by trusting it, so it is published rather than guessed.
pub const SAMPLE_RATE: u32 = 44_100;

/// Synthesize one sound into mono samples in `[-1, 1]`.
///
/// Each voice is a decaying sine at a frequency chosen to *mean* something:
/// one voice for select and place (place lower, so a landing feels heavier),
/// two voices for refuse (a falling minor third — two tones, because a refusal
/// has a "not that" then a "rather this" shape), and a rising fourth for
/// notify. The envelope is an exponential decay with a short attack, so no
/// click: a click is a defect a player hears a thousand times.
pub fn synthesize(sound: Sound) -> Vec<f32> {
    let total = (sound.seconds() * SAMPLE_RATE as f32) as usize;
    let mut out = vec![0.0f32; total];
    let voices: &[(f32, f32, f32)] = match sound {
        // (start_hz, end_hz, amplitude)
        Sound::Select => &[(880.0, 880.0, 0.30)],
        Sound::Place => &[(330.0, 330.0, 0.42)],
        Sound::Refuse => &[(392.0, 392.0, 0.34), (311.0, 311.0, 0.30)],
        Sound::Notify => &[(440.0, 587.0, 0.30)],
    };
    for &(start_hz, end_hz, amplitude) in voices {
        let voice_start = match sound {
            // The refusal's second tone arrives a beat later.
            Sound::Refuse if end_hz < start_hz => total / 2,
            _ => 0,
        };
        let voice_len = total - voice_start;
        let mut phase = 0.0f32;
        for i in 0..voice_len {
            let t = i as f32 / SAMPLE_RATE as f32;
            let span = voice_len as f32 / SAMPLE_RATE as f32;
            // Linear glide for the notify's rise; static otherwise.
            let hz = start_hz + (end_hz - start_hz) * (t / span);
            phase += 2.0 * std::f32::consts::PI * hz / SAMPLE_RATE as f32;
            // 4 ms attack, exponential decay over the voice's span.
            let attack = (t / 0.004).min(1.0);
            let decay = (-t / (span * 0.45)).exp();
            out[voice_start + i] += amplitude * attack * decay * phase.sin();
        }
    }
    out
}

/// The output side: owns the rodio stream and speaks only in [`Sound`]s.
/// Constructing it can fail (no output device); the game logs that once and
/// runs silent rather than crashing over feedback.
pub struct Player {
    #[allow(dead_code)] // the stream must live for the sound to play at all
    stream: rodio::OutputStream,
    handle: rodio::OutputStreamHandle,
}

impl Player {
    pub fn new() -> Option<Player> {
        let (stream, handle) = rodio::OutputStream::try_default().ok()?;
        Some(Player { stream, handle })
    }

    /// Play one sound. Fire-and-forget: a sound that cannot play is logged,
    /// never surfaced as an error — audio is feedback, not a control path.
    pub fn play(&self, sound: Sound) {
        let samples = synthesize(sound);
        let source: rodio::buffer::SamplesBuffer<f32> =
            rodio::buffer::SamplesBuffer::new(1, SAMPLE_RATE, samples);
        if let Err(err) = self.handle.play_raw(source) {
            tracing::debug!(?sound, %err, "audio feedback could not play");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_sound_is_finite_peaked_and_decays_to_silence() {
        for sound in [Sound::Select, Sound::Place, Sound::Refuse, Sound::Notify] {
            let samples = synthesize(sound);
            let expected = (sound.seconds() * SAMPLE_RATE as f32) as usize;
            assert_eq!(samples.len(), expected, "{sound:?} length");
            assert!(
                samples.iter().all(|s| s.is_finite() && s.abs() <= 1.0),
                "{sound:?} left the [-1, 1] range"
            );
            let peak = samples.iter().fold(0.0f32, |m, s| m.max(s.abs()));
            assert!(peak > 0.1, "{sound:?} is too quiet to hear (peak {peak})");
            // The tail is silence: the envelope really decays.
            let tail = &samples[samples.len() * 9 / 10..];
            let tail_peak = tail.iter().fold(0.0f32, |m, s| m.max(s.abs()));
            assert!(
                tail_peak < peak * 0.35,
                "{sound:?} does not decay (tail {tail_peak} vs peak {peak})"
            );
        }
    }

    #[test]
    fn synthesis_is_deterministic() {
        let a = synthesize(Sound::Place);
        let b = synthesize(Sound::Place);
        assert_eq!(a, b, "the same sound synthesized twice differed");
    }

    #[test]
    fn a_refusal_does_not_sound_like_a_place() {
        // The whole point of distinct tones: the four sounds must differ.
        let refuse = synthesize(Sound::Refuse);
        let place = synthesize(Sound::Place);
        let same = refuse
            .iter()
            .zip(place.iter())
            .map(|(r, p)| (r - p).abs())
            .sum::<f32>();
        assert!(same > 1.0, "refuse and place are nearly the same sound ({same})");
    }

    #[test]
    fn no_sound_starts_with_a_click() {
        // The first millisecond ramps up: its peak is well below the sound's
        // full amplitude, which is what "no click" means measurably.
        for sound in [Sound::Select, Sound::Place, Sound::Refuse, Sound::Notify] {
            let samples = synthesize(sound);
            let first_ms = (0.001 * SAMPLE_RATE as f32) as usize;
            let early = samples[..first_ms].iter().fold(0.0f32, |m, s| m.max(s.abs()));
            let peak = samples.iter().fold(0.0f32, |m, s| m.max(s.abs()));
            assert!(early < peak * 0.75, "{sound:?} clicks (early {early}, peak {peak})");
        }
    }
}
