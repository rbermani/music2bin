use bitfield::bitfield;
use num_derive::FromPrimitive;

#[derive(Debug, FromPrimitive)]
#[repr(u8)]
pub enum MusicTagIdentifiers {
    MeasureInitializerTag = 0,
    MeasureMetaDataTag = 1,
    NoteDataTag = 2,
    TupletTag = 3,
}

pub enum MusicElementBin {
    MeasureInit(MeasureInitializerBin<[u8; 4]>),
    MeasureMeta(MeasureMetaDataBin<[u8; 1]>),
    NoteRest(NoteDataBin<[u8; 4]>),
    Tuplet(TupletDataBin<[u8; 2]>),
}

// Four bytes with Bit 31 as MSB
bitfield! {
    pub struct MeasureInitializerBin(MSB0 [u8]);
    impl Debug;
    u8;
    pub get_identifier, set_identifier: 2, 0;
    pub get_beats, set_beats: 5, 3;
    pub get_beat_type, set_beat_type: 7, 6;
    pub get_fifths, set_fifths: 11, 8;
    pub get_treble_dynamics, set_treble_dynamics: 15, 12;
    pub get_bass_dynamics, set_bass_dynamics: 19, 16;
    pub get_tempo, set_tempo: 26, 20;
}

// One byte
bitfield! {
    pub struct MeasureMetaDataBin(MSB0 [u8]);
    impl Debug;
    u8;
    pub get_identifier, set_identifier: 2, 0;
    pub get_start_end, set_start_end: 3;
    pub get_repeat, set_repeat: 4;
    pub get_dal_segno, set_dal_segno: 7, 5;
}

// Four bytes with Bit 31 as MSB
bitfield! {
    pub struct NoteDataBin(MSB0 [u8]);
    impl Debug;
    u8;
    pub get_identifier, set_identifier: 2, 0;
    pub get_note, set_note: 9, 3;
    pub get_phrase_dynamics, set_phrase_dynamics: 13, 10;
    pub get_rhythm_value, set_rhythm_value: 16, 14;
    pub get_arpeggiation, set_arpeggiation: 17;
    pub get_special_note, set_special_note: 19, 18;
    pub get_articulation, set_articulation: 21, 20;
    pub get_trill, set_trill: 23, 22;
    pub get_ties, set_ties: 25, 24;
    pub get_treble_bass, set_treble_bass: 26;
    pub get_stress, set_stress: 27;
    pub get_chord, set_chord: 28;
}

// Two bytes
bitfield! {
    pub struct TupletDataBin(MSB0 [u8]);
    impl Debug;
    u8;
    pub get_identifier, set_identifier: 2, 0;
    pub get_startstop, set_startstop: 4, 3;
    pub get_tuplet_number, set_tuplet_number: 7, 5;
    pub get_actual_note, set_actual_note: 10, 8;
    pub get_normal_note, set_normal_note: 13, 11;
    pub get_dotted, set_dotted: 14;
}
