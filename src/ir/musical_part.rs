use super::notation::MusicElement;

#[derive(Eq, PartialEq, Default, Debug, Clone)]
pub struct MusicalPart {
    elems: Vec<MusicElement>,
    divisions: Option<u32>,
    num_voices: Option<usize>,
}

impl MusicalPart {
    pub fn new_from_elems(
        divisions: u32,
        num_voices: usize,
        elems: Vec<MusicElement>,
    ) -> MusicalPart {
        MusicalPart {
            elems,
            divisions: Some(divisions),
            num_voices: Some(num_voices),
        }
    }

    pub fn new() -> MusicalPart {
        MusicalPart {
            elems: vec![],
            divisions: None,
            num_voices: None,
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

    pub fn set_num_voices(&mut self, num_voices: usize) {
        self.num_voices = Some(num_voices);
    }
    pub fn set_divisions(&mut self, divisions: u32) {
        self.divisions = Some(divisions);
    }
    pub fn get_divisions(&self) -> Option<u32> {
        self.divisions
    }
    pub fn get_num_voices(&self) -> Option<usize> {
        self.num_voices
    }
    pub fn push(&mut self, elem: MusicElement) {
        self.elems.push(elem);
    }
    pub fn append(&mut self, elem: &mut Vec<MusicElement>) {
        self.elems.append(elem);
    }
}

impl AsRef<MusicalPart> for MusicalPart {
    fn as_ref(&self) -> &Self {
        self
    }
}
