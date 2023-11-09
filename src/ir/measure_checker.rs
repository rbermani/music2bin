use super::notation::{
    BeatType, Beats, Chord, MeasureInitializer, MusicElement, NoteData, SpecialNote,
    TimeModification, Voice,
};
use log::{error, info, trace, warn};
use num::integer::lcm;
use num_traits::FromPrimitive;
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::convert::From;

pub struct MeasureChecker {
    measure: Vec<MusicElement>,
    elems_since_backup: usize,
    quarter_division: u32,
    beats: Beats,
    beat_type: BeatType,
    part_str: String,
    measure_idx: usize,
    forward_duration: usize,
}

impl MeasureChecker {
    pub const MAX_SUPPORTED_VOICES: usize = 4;
    pub fn new(
        quarter_division: u32,
        measure_init: &MeasureInitializer,
        part_str: &str,
        measure_idx: usize,
        forward_duration: usize,
    ) -> MeasureChecker {
        MeasureChecker {
            measure: vec![],
            elems_since_backup: 0,
            quarter_division,
            beats: measure_init.beats,
            beat_type: measure_init.beat_type,
            part_str: part_str.to_string(),
            measure_idx,
            forward_duration,
        }
    }

    pub fn push_elem(&mut self, elem: MusicElement) {
        //debug!("{:?}", elem);
        self.measure.push(elem);
        self.elems_since_backup += 1;
    }

    pub fn quarter_division(&self) -> u32 {
        self.quarter_division
    }

    pub fn measure_idx(&self) -> usize {
        self.measure_idx
    }

    pub fn conform_backup_placeholder_rests(&mut self, backup_duration: usize) {
        // Backup elements are only inserted when voice changes happen.
        // Calculate duration to current point, since previous voice began, based on notes in the measure, and accounting for corresponding
        // time modifying elements
        let actual_duration = backup_duration - self.forward_duration;
        let last_backup_idx = self.measure.len() - self.elems_since_backup;
        let mut time_mod: Option<TimeModification> = None;
        let mut current_voice = Voice::One;
        let duration_since_backup: usize = self.measure[last_backup_idx..]
            .iter()
            .cloned()
            .map(|element| match element {
                MusicElement::NoteRest(n) => {
                    // chords have a duration in musicxml, but this duration is always identical to the previous note it's
                    // attached to. Chord duration shouldn't impact the total summation.
                    current_voice = n.voice;
                    if n.chord == Chord::NoChord {
                        n.get_duration_numeric(
                            self.quarter_division,
                            u32::from(self.beats),
                            u32::from(self.beat_type),
                            time_mod,
                        ) as usize
                    } else {
                        // Chord notes do not directly impact duration
                        0
                    }
                }
                MusicElement::Tuplet(t) => {
                    time_mod = t.into();
                    0 // does not directly impact sum
                }
                _ => {
                    0 // does not impact sum
                }
            })
            .sum();
        //duration_since_backup -= self.forward_duration;

        match actual_duration.cmp(&duration_since_backup) {
            Ordering::Less => {
                let discrepancy = duration_since_backup - actual_duration;
                println!("{}M{} duration tally {} did not match the backup element's duration {actual_duration}, qtr_div: {} inserting rests to accommodate {discrepancy} discrepancy.", self.part_str.as_str(), self.measure_idx, duration_since_backup, self.quarter_division);

                match NoteData::from_numeric_duration(discrepancy as u32, self.quarter_division) {
                    Some((duration, is_dotted, time_mod)) => {
                        if time_mod.is_some() {
                            warn!("time modification for rest is present, but not being used.")
                        }
                        // The new rest should begin on the next voice after the current one.
                        self.measure
                            .push(MusicElement::NoteRest(NoteData::new_default_rest(
                                duration,
                                is_dotted,
                                current_voice.next(),
                            )));
                    }
                    None => {
                        panic!(
                            "Could not convert {} in a rest duration value.",
                            discrepancy
                        );
                    }
                }
            }
            Ordering::Greater => {
                info!(
                    "Backup_duration {} was > duration_since_backup {} Assuming beginning of measure",
                    actual_duration, duration_since_backup
                );
            }
            Ordering::Equal => {
                // No additional action needed
            }
        }

        self.clear_elems_since_backup();
    }

    fn clear_elems_since_backup(&mut self) {
        self.elems_since_backup = 0;
    }

    pub fn as_inner(&mut self) -> &mut Vec<MusicElement> {
        &mut self.measure
    }

    pub fn remove_incomplete_voices(&mut self, voices: &BTreeSet<u8>) {
        let mut voice_durations: [u32; Self::MAX_SUPPORTED_VOICES] =
            [0; Self::MAX_SUPPORTED_VOICES];
        let mut voice_last_idx: [usize; Self::MAX_SUPPORTED_VOICES] =
            [0; Self::MAX_SUPPORTED_VOICES];

        if voices.len() > Self::MAX_SUPPORTED_VOICES {
            panic!(
                "Set of voices len {} exceeds max supported {}",
                voices.len(),
                Self::MAX_SUPPORTED_VOICES
            );
        }

        let mut time_mod = None;
        let mut prev_voice = 0;

        for (idx, elem) in self.measure.iter().cloned().enumerate() {
            // if self.measure_idx == 68 {
            // println!("{:?}", elem);
            // }
            match elem {
                MusicElement::Tuplet(t) => time_mod = t.into(),
                MusicElement::NoteRest(n) => {
                    // Do not include chord notes or grace notes in the count, as they do not impact measure duration
                    if n.chord == Chord::NoChord && n.special_note == SpecialNote::None {
                        voice_durations[n.voice as usize] += n.get_duration_numeric(
                            self.quarter_division,
                            u32::from(self.beats),
                            u32::from(self.beat_type),
                            time_mod,
                        )
                    }
                    if n.voice as usize > prev_voice {
                        //The voice number of this element is higher than the previous element's voice, store previous index
                        voice_last_idx[prev_voice] = idx - 1;
                    }

                    prev_voice = n.voice as usize;
                }
                _ => {
                    error!("Unhandled element case");
                }
            }
        }
        // if self.measure_idx == 68 {
        //         println!("voice_durations: {:?}", voice_durations);
        // }

        let first_voice_duration = voice_durations[0];
        for (voice_idx, _) in voices.iter().enumerate() {
            //println!("voice {} duration {}", voice_idx, voice_durations[voice_idx]);
            if voice_durations[voice_idx] != 0 && voice_durations[voice_idx] < first_voice_duration
            {
                let discrepancy = first_voice_duration - voice_durations[voice_idx];
                println!(
                    "{}M{} Voice Zero: {first_voice_duration} duration Voice {voice_idx}: {} duration {} discrepancy", self.part_str.as_str(), self.measure_idx,
                    voice_durations[voice_idx],discrepancy
                );
                // insert rest of discrepancy length at index at measure[voice_last_idx[voice_idx]]
                println!("Inserting rest due to voice length incorrect.");
                if let Some((duration, is_dotted, time_mod)) =
                    NoteData::from_numeric_duration(discrepancy, self.quarter_division)
                {
                    if time_mod.is_some() {
                        warn!("time modification for rest is present, but not being used.")
                    }
                    // The new rest should begin on the current voice to correct the total duration.
                    self.measure.insert(
                        voice_last_idx[voice_idx],
                        MusicElement::NoteRest(NoteData::new_default_rest(
                            duration,
                            is_dotted,
                            FromPrimitive::from_u8(voice_idx as u8).unwrap(),
                        )),
                    );
                } else {
                    panic!(
                        "Could not convert {} in a rest duration value.",
                        discrepancy
                    );
                }
            }
        }
    }
}

struct DivisionsVec {
    inner: Vec<u32>,
}

impl DivisionsVec {
    // Create a new, empty DivisionsVec
    pub fn new() -> Self {
        DivisionsVec { inner: vec![] }
    }

    // Add an item to the DivisionsVec, but only if it's not already present
    pub fn add(&mut self, value: u32) {
        if value != 0 && !self.inner.contains(&value) {
            self.inner.push(value);
        }
    }

    pub fn find_lcm(&mut self) -> u32 {
        self.inner.iter().fold(1, |acc, &n| lcm(acc, n))
    }

    // Allow direct access to the inner Vec<u32>
    #[allow(dead_code)]
    pub fn inner(&self) -> &Vec<u32> {
        &self.inner
    }
}

pub fn calc_divisions_voices(music_elems_v: Vec<MusicElement>, dump_input: bool) -> (u32, usize) {
    let mut voices = BTreeSet::new();
    let mut integers_v = DivisionsVec::new();
    let mut time_mod = None;

    for elem in music_elems_v.iter() {
        if dump_input {
            trace!("{:?}", elem);
        }
        match elem {
            MusicElement::Tuplet(t) => {
                time_mod = (*t).into();
            }
            MusicElement::NoteRest(n) => {
                voices.insert(n.voice as u8);
                integers_v.add(n.get_note_multiple(time_mod).map_or_else(|| 0, |v| v));
            }
            _ => {}
        }
    }

    // for (idx, elem) in integers_v.inner().iter().enumerate() {
    //     println!("{idx},{elem}");
    // }

    (integers_v.find_lcm(), voices.len())
}
