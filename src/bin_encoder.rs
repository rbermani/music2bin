use crate::notation::*;

use crate::error::Error;
use bitfield::bitfield;
use io::Write;
use num_derive::FromPrimitive;
use std::io;

#[derive(Debug, FromPrimitive)]
#[repr(u8)]
pub enum MusicTagIdentifiers {
    MeasureInitializerTag = 0,
    MeasureMetaDataTag = 1,
    NoteDataTag = 2,
    TupletTag = 3,
}

// pub enum MusicElementBin {
//     MeasureInit(MeasureInitializerBin<[u8; 4]>),
//     MeasureMeta(MeasureMetaDataBin<[u8; 4]>),
//     NoteRest(NoteDataBin<[u8; 4]>),
//     Tuplet(TupletDataBin<[u8; 4]>),
// }

// Bit 31 as MSB
bitfield! {
    pub struct MeasureInitializerBin(MSB0 [u8]);
    impl Debug;
    u8;
    pub get_identifier, set_identifier: 1, 0;
    pub get_beats, set_beats: 4, 2;
    pub get_beat_type, set_beat_type: 6, 5;
    pub get_fifths, set_fifths: 10, 7;
    pub get_tempo, set_tempo: 17, 11;
}

bitfield! {
    pub struct MeasureMetaDataBin(MSB0 [u8]);
    impl Debug;
    u8;
    pub get_identifier, set_identifier: 1, 0;
    pub get_start_end, set_start_end: 3, 2;
    pub get_ending, set_ending: 5, 4;
    pub get_dal_segno, set_dal_segno: 8, 6;
}

bitfield! {
    pub struct NoteDataBin(MSB0 [u8]);
    impl Debug;
    u8;
    pub get_identifier, set_identifier: 1, 0;
    pub get_note, set_note: 8, 2;
    pub get_phrase_dynamics, set_phrase_dynamics: 12, 9;
    pub get_rhythm_value, set_rhythm_value: 15, 13;
    pub get_dotted, set_dotted: 16;
    pub get_arpeggiation, set_arpeggiation: 17;
    pub get_special_note, set_special_note: 19, 18;
    pub get_articulation, set_articulation: 21, 20;
    pub get_trill, set_trill: 23, 22;
    pub get_ties, set_ties: 25, 24;
    pub get_stress, set_stress: 26;
    pub get_chord, set_chord: 27;
    pub get_slur, set_slur: 29, 28;
    pub get_voice, set_voice: 31, 30;
}

bitfield! {
    pub struct TupletDataBin(MSB0 [u8]);
    impl Debug;
    u8;
    pub get_identifier, set_identifier: 1, 0;
    pub get_startstop, set_startstop: 3, 2;
    pub get_tuplet_number, set_tuplet_number: 6, 4;
    pub get_actual_note, set_actual_note: 9, 7;
    pub get_normal_note, set_normal_note: 12, 10;
    pub get_dotted, set_dotted: 13;
}

pub struct MusicEncoder<W: Write> {
    w: W,
}

impl<W: Write> MusicEncoder<W> {
    pub fn new(w: W) -> MusicEncoder<W> {
        MusicEncoder { w }
    }

    fn write_chunk(&mut self, data: &[u8]) -> Result<(), Error> {
        self.w
            .write(data)
            .map_err(|e| Error::IoKind(e.kind().to_string()))?;
        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), Error> {
        self.w
            .flush()
            .map_err(|e| Error::IoKind(e.kind().to_string()))?;
        Ok(())
    }

    pub fn insert_measure_initializer(
        &mut self,
        measure_init: MeasureInitializer,
    ) -> Result<(), Error> {
        let mut data: [u8; 4] = [0; 4];
        let mut measure_initializer = MeasureInitializerBin(&mut data);
        measure_initializer.set_identifier(MusicTagIdentifiers::MeasureInitializerTag as u8);
        measure_initializer.set_beats(measure_init.beats as u8);
        measure_initializer.set_beat_type(measure_init.beat_type as u8);
        measure_initializer.set_fifths(measure_init.key_sig as u8);
        measure_initializer.set_tempo(measure_init.tempo.get_raw());
        self.write_chunk(&data)
    }

    pub fn insert_measure_metadata(&mut self, measure_meta: MeasureMetaData) -> Result<(), Error> {
        let mut data: [u8; 4] = [0; 4];
        let mut measure_metadata = MeasureMetaDataBin(&mut data);
        measure_metadata.set_identifier(MusicTagIdentifiers::MeasureMetaDataTag as u8);
        measure_metadata.set_start_end(measure_meta.start_end as u8);
        measure_metadata.set_ending(measure_meta.ending as u8);
        measure_metadata.set_dal_segno(measure_meta.dal_segno as u8);
        self.write_chunk(&data)
    }

    pub fn insert_note_data(&mut self, note_data: NoteData) -> Result<(), Error> {
        let mut data: [u8; 4] = [0; 4];
        let mut note_data_bin = NoteDataBin(&mut data);
        note_data_bin.set_identifier(MusicTagIdentifiers::NoteDataTag as u8);
        note_data_bin.set_note(note_data.note_rest.get_numeric_value());
        note_data_bin.set_phrase_dynamics(note_data.phrase_dynamics as u8);
        note_data_bin.set_rhythm_value(note_data.note_type as u8);
        note_data_bin.set_dotted(note_data.dotted);
        note_data_bin.set_arpeggiation(bool::from(note_data.arpeggiate));
        note_data_bin.set_special_note(note_data.special_note as u8);
        note_data_bin.set_articulation(note_data.articulation as u8);
        note_data_bin.set_trill(note_data.trill as u8);
        note_data_bin.set_ties(note_data.ties as u8);
        note_data_bin.set_chord(bool::from(note_data.chord));
        note_data_bin.set_slur(note_data.slur as u8);
        note_data_bin.set_voice(note_data.voice as u8);
        self.write_chunk(&data)
    }

    pub fn insert_tuplet_data(&mut self, tuplet_data: TupletData) -> Result<(), Error> {
        let mut data: [u8; 4] = [0; 4];
        let mut tuplet_data_bin = TupletDataBin(&mut data);
        tuplet_data_bin.set_identifier(MusicTagIdentifiers::TupletTag as u8);
        tuplet_data_bin.set_startstop(tuplet_data.start_stop as u8);
        tuplet_data_bin.set_tuplet_number(tuplet_data.tuplet_number as u8);
        tuplet_data_bin.set_actual_note(tuplet_data.actual_notes as u8);
        tuplet_data_bin.set_normal_note(tuplet_data.normal_notes as u8);
        tuplet_data_bin.set_dotted(bool::from(tuplet_data.dotted));
        self.write_chunk(&data)
    }
}
