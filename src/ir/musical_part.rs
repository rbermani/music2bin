use std::collections::BTreeSet;
use num::integer::lcm;
use super::{measure_checker::MeasureChecker, notation::{MeasureInitializer, MeasureMetaData, MusicElement, PhraseDynamics}};
use crate::error::{Result,Error};
use log::{trace,error};

type VoiceIdx = u8;
type MeasureIdx = usize;

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

#[derive(Eq, PartialEq, Default, Debug, Clone)]
pub struct MusicalPart {
    elems: Vec<MusicElement>,
    divisions: Option<u32>,
    measure_checker: Option<MeasureChecker>,
    part_str: String,
    voices: BTreeSet<VoiceIdx>,
    // The index in the vector of elements containing the most recent Measure Initializer
    cur_init_measure_idx: Option<MeasureIdx>,
    pub cur_phrase_dyn: Option<PhraseDynamics>,
}

impl MusicalPart {
    pub const MAX_SUPPORTED_VOICES: usize = 4;
    pub fn new_from_elems(
        part_str: &str,
        elems: Vec<MusicElement>,
    ) -> Result<MusicalPart> {
        let mut temp_mpart = MusicalPart {
            elems,
            divisions: None,
            measure_checker: None,
            part_str: part_str.to_string(),
            voices: BTreeSet::new(),
            cur_init_measure_idx: None,
            cur_phrase_dyn: None,
        };
        temp_mpart.update_divisions_voices()?;
        Ok(temp_mpart)
    }

    pub fn new(part_str: &str) -> MusicalPart {
        MusicalPart {
            elems: vec![],
            divisions: None,
            measure_checker: None,
            part_str: part_str.to_string(),
            voices: BTreeSet::new(),
            cur_init_measure_idx: None,
            cur_phrase_dyn: None,
        }
    }
    pub fn len(&self) -> usize {
        self.elems.len()
    }
    pub fn is_empty(&self) -> bool {
        self.elems.is_empty()
    }
    pub fn inner(&self) -> &Vec<MusicElement> {
        &self.elems
    }

    pub fn set_initial_divisions(&mut self, divisions: u32) {
        self.divisions = Some(divisions);
    }
    pub fn get_initial_divisions(&self) -> Option<u32> {
        self.divisions
    }
    pub fn get_num_voices(&self) -> usize {
        self.voices.len()
    }
    pub fn insert_new_voice(&mut self, voice_num: VoiceIdx) -> Result<usize> {
        self.voices.insert(voice_num);
        if self.voices.len() > MeasureChecker::MAX_SUPPORTED_VOICES {
            // Don't let the number of voices in the voices set exceed the maximum
            self.voices.remove(&voice_num);
            Err(Error::OutofBounds)
        } else {
            Ok(self.voices.iter().position(|&x| x == voice_num).unwrap())
        }
    }
    fn push(&mut self, elem: MusicElement) {
        self.elems.push(elem);
    }
    fn append(&mut self, elem: &mut Vec<MusicElement>) {
        self.elems.append(elem);
    }
    pub fn get_cur_init_measure(&self) -> MeasureInitializer {
        if self.cur_init_measure_idx.is_none() {
            MeasureInitializer::default()
        } else {
            if let MusicElement::MeasureInit(measure_init) = self.elems[self.cur_init_measure_idx.unwrap()] {
                measure_init
            } else {
                MeasureInitializer::default()
            }
        }
    }
    pub fn get_cur_init_measure_idx(&self) -> Option<usize> {
        self.cur_init_measure_idx
    }
    pub fn push_init_measure(&mut self, init_measure: MeasureInitializer) {
        self.elems.push(MusicElement::MeasureInit(init_measure));
        self.cur_init_measure_idx = if self.elems.len() > 0 {
            Some(self.elems.len() - 1)
        } else {
            None
        };
    }
    pub fn push_meta_start(&mut self, meta_start: MeasureMetaData, forward_duration: usize, xml_measure_idx: usize) {
        let init_measure_idx = match self.cur_init_measure_idx {
            Some(idx) => idx,
            None => panic!("Attempted to push a meta start measure without an initializer measure"),
        };
        self.measure_checker = if let MusicElement::MeasureInit(cur_init_measure) = self.elems[init_measure_idx].clone() {
            Some(MeasureChecker::new(
                self.divisions.unwrap(),
                &cur_init_measure,
                self.part_str.as_str(),
                xml_measure_idx,
                forward_duration,
            ))
        } else {
            panic!("Could not pattern match MusicElement::MeasureInit at target index.");
        };
        self.elems.push(MusicElement::MeasureMeta(meta_start));
    }
    pub fn push_measure_elem(&mut self, measure_elem: MusicElement) {
        if let Some(measure_checker) = &mut self.measure_checker {
            measure_checker.push_elem(measure_elem);
        } else {
            panic!("Measure Checker is not initialized but measure meta end element push attempted");
        }
    }
    pub fn update_backup_duration(&mut self, duration_val: usize) {
        if let Some(measure_checker) = &mut self.measure_checker {
            measure_checker.conform_backup_placeholder_rests(duration_val);
        } else {
            panic!("Measure Checker is not initialized but request to update backup duration");
        }
    }
    pub fn push_meta_end(&mut self, meta_end: MeasureMetaData) {
        if let Some(measure_checker) = &mut self.measure_checker {
            measure_checker.remove_incomplete_voices(&self.voices);
            self.elems.append(measure_checker.as_inner());
            self.elems.push(MusicElement::MeasureMeta(meta_end));
        } else {
            panic!("Measure Checker is not initialized but measure meta end element push attempted");
        }
    }
    pub fn get_measure_idx(&self) -> usize {
        if let Some(measure_checker) = &self.measure_checker {
            measure_checker.measure_idx()
        } else {
            panic!("Measure Checker is not initialized but request made for measure checker fields");
        }
    }
    pub fn get_cur_quarter_divisions(&self) -> u32 {
        if let Some(measure_checker) = &self.measure_checker {
            measure_checker.quarter_division()
        } else {
            panic!("Measure Checker is not initialized but request made for measure checker fields");
        }
    }

    fn update_divisions_voices(&mut self) -> Result<()> {
        // For tuplets, the associated note type is embedded in the NoteData type. The Tuplet data information element
        // precedes the note data element, so to determine the shortest value represented in the piece, both the tuplet information
        // is needed and all of the notes within the tuplet section. For the minimum, we're looking for the shortest note type
        // that is within a tuplet, and the most actual notes within the number of normal notes indicated in the Tuplet data
        // and finding a LCM (least common multiple) for them

        let mut integers_v = DivisionsVec::new();
        let mut time_mod = None;

        for elem in (&self.elems).iter() {
            trace!("{:?}", elem);
            match elem {
                MusicElement::Tuplet(t) => {
                    time_mod = (*t).into();
                }
                MusicElement::NoteRest(n) => {
                    self.voices.insert(n.voice as u8);
                    integers_v.add(n.get_note_multiple(time_mod).map_or_else(|| 0, |v| v));
                }
                _ => {}
            }
        }
        if self.voices.len() > MusicalPart::MAX_SUPPORTED_VOICES {
            error!(
                "Maximum supported voices is {} but piece contains {}.",
                MusicalPart::MAX_SUPPORTED_VOICES,
                self.voices.len()
            );
            return Err(Error::OutofBounds);
        }
        self.divisions = Some(integers_v.find_lcm());
        // for (idx, elem) in integers_v.inner().iter().enumerate() {
        //     println!("{idx},{elem}");
        // }
        Ok(())
    }
}

impl AsRef<MusicalPart> for MusicalPart {
    fn as_ref(&self) -> &Self {
        self
    }
}
