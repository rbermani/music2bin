use crate::layout::*;
use crate::notation::*;

use failure::Error;
use io::Write;
use std::io;

pub struct MusicEncoder<W: Write> {
    w: W,
}

impl<W: Write> MusicEncoder<W> {
    pub fn new(w: W) -> MusicEncoder<W> {
        MusicEncoder { w }
    }

    fn write_chunk(&mut self, data: &[u8]) -> Result<(), Error> {
        self.w.write(data)?;
        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), Error> {
        self.w.flush()?;
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
        measure_initializer.set_treble_dynamics(measure_init.treble_dynamics as u8);
        measure_initializer.set_bass_dynamics(measure_init.bass_dynamics as u8);
        measure_initializer.set_tempo(measure_init.tempo.get_raw());
        self.write_chunk(&data)
    }

    pub fn insert_measure_metadata(&mut self, measure_meta: MeasureMetaData) -> Result<(), Error> {
        let mut data: [u8; 1] = [0; 1];
        let mut measure_metadata = MeasureMetaDataBin(&mut data);
        measure_metadata.set_identifier(MusicTagIdentifiers::MeasureMetaDataTag as u8);
        measure_metadata.set_start_end(bool::from(measure_meta.start_end));
        measure_metadata.set_repeat(bool::from(measure_meta.repeat));
        measure_metadata.set_dal_segno(measure_meta.dal_segno as u8);
        self.write_chunk(&data)
    }

    pub fn insert_note_data(&mut self, note_data: NoteData) -> Result<(), Error> {
        let mut data: [u8; 4] = [0; 4];
        let mut note_data_bin = NoteDataBin(&mut data);
        note_data_bin.set_identifier(MusicTagIdentifiers::NoteDataTag as u8);
        note_data_bin.set_note(note_data.note_rest.get_numeric_value());
        note_data_bin.set_phrase_dynamics(note_data.phrase_dynamics as u8);
        note_data_bin.set_rhythm_value(note_data.rhythm_value as u8);
        note_data_bin.set_arpeggiation(bool::from(note_data.arpeggiate));
        note_data_bin.set_special_note(note_data.special_note as u8);
        note_data_bin.set_articulation(note_data.articulation as u8);
        note_data_bin.set_trill(note_data.trill as u8);
        note_data_bin.set_ties(note_data.ties as u8);
        note_data_bin.set_treble_bass(bool::from(note_data.treble_bass));
        note_data_bin.set_chord(bool::from(note_data.chord));
        self.write_chunk(&data)
    }

    pub fn insert_tuplet_data(&mut self, tuplet_data: TupletData) -> Result<(), Error> {
        let mut data: [u8; 2] = [0; 2];
        let mut tuplet_data_bin = TupletDataBin(&mut data);
        tuplet_data_bin.set_identifier(MusicTagIdentifiers::NoteDataTag as u8);
        tuplet_data_bin.set_startstop(tuplet_data.start_stop as u8);
        tuplet_data_bin.set_tuplet_number(tuplet_data.tuplet_number as u8);
        tuplet_data_bin.set_actual_note(tuplet_data.actual_notes as u8);
        tuplet_data_bin.set_normal_note(tuplet_data.normal_notes as u8);
        tuplet_data_bin.set_dotted(bool::from(tuplet_data.dotted));
        self.write_chunk(&data)
    }
}
