use crate::error::Error;
use crate::ir::notation::*;
use bitfield::bitfield;
use io::Write;
use num_derive::FromPrimitive;
use std::io;

pub const MUSIC_ELEMENT_LENGTH: usize = 4;

#[derive(Debug, FromPrimitive)]
#[repr(u8)]
pub enum MusicTagIdentifiers {
    MeasureInitializer = 0,
    MeasureMetaData = 1,
    NoteData = 2,
    Tuplet = 3,
}

pub struct MusicBinHeader {
    identifier: [u8; 4],
    length: usize,
}

impl MusicBinHeader {
    pub const MUSICBIN_MAGIC_NUMBER: [u8; 4] = [b'M', b'u', b'B', b'i'];

    pub fn new(length: usize) -> MusicBinHeader {
        MusicBinHeader {
            identifier: Self::MUSICBIN_MAGIC_NUMBER,
            length,
        }
    }

    pub fn get_chunk_length(&self) -> usize {
        self.length / MUSIC_ELEMENT_LENGTH
    }

    pub fn set_length(&mut self, length: usize) {
        self.length = length;
    }
}

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
    pub get_articulation, set_articulation: 22, 20;
    pub get_trill, set_trill: 24, 23;
    pub get_ties, set_ties: 26, 25;
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
    pub get_tuplet_number, set_tuplet_number: 5, 4;
    pub get_actual_note, set_actual_note: 9, 6;
    pub get_normal_note, set_normal_note: 13, 10;
    pub get_dotted, set_dotted: 14;
}

pub struct MusicEncoder<W: Write> {
    w: W,
}

impl<W: Write> MusicEncoder<W> {
    fn write_chunk(&mut self, data: &[u8]) -> Result<(), Error> {
        self.w
            .write(data)
            .map_err(|e| Error::IoKind(e.kind().to_string()))?;
        Ok(())
    }

    pub fn new(w: W) -> MusicEncoder<W> {
        MusicEncoder { w }
    }

    pub fn create_header(&mut self, length: usize) -> Result<(), Error> {
        let hdr = MusicBinHeader::new(length);
        self.write_chunk(&hdr.identifier)?;
        self.write_chunk(&(hdr.length as u32).to_le_bytes())
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
        measure_initializer.set_identifier(MusicTagIdentifiers::MeasureInitializer as u8);
        measure_initializer.set_beats(measure_init.beats as u8);
        measure_initializer.set_beat_type(measure_init.beat_type as u8);
        measure_initializer.set_fifths(measure_init.key_sig as u8);
        measure_initializer.set_tempo(measure_init.tempo.get_raw());
        self.write_chunk(&data)
    }

    pub fn insert_measure_metadata(&mut self, measure_meta: MeasureMetaData) -> Result<(), Error> {
        let mut data: [u8; 4] = [0; 4];
        let mut measure_metadata = MeasureMetaDataBin(&mut data);
        measure_metadata.set_identifier(MusicTagIdentifiers::MeasureMetaData as u8);
        measure_metadata.set_start_end(measure_meta.start_end as u8);
        measure_metadata.set_ending(measure_meta.ending as u8);
        measure_metadata.set_dal_segno(measure_meta.dal_segno as u8);
        self.write_chunk(&data)
    }

    pub fn insert_note_data(&mut self, note_data: NoteData) -> Result<(), Error> {
        let mut data: [u8; 4] = [0; 4];
        let mut note_data_bin = NoteDataBin(&mut data);
        note_data_bin.set_identifier(MusicTagIdentifiers::NoteData as u8);
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
        tuplet_data_bin.set_identifier(MusicTagIdentifiers::Tuplet as u8);
        tuplet_data_bin.set_startstop(tuplet_data.start_stop as u8);
        tuplet_data_bin.set_tuplet_number(tuplet_data.tuplet_number as u8);
        tuplet_data_bin.set_actual_note(tuplet_data.actual_notes as u8);
        tuplet_data_bin.set_normal_note(tuplet_data.normal_notes as u8);
        tuplet_data_bin.set_dotted(tuplet_data.dotted);
        self.write_chunk(&data)
    }
}
