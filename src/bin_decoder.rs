use crate::bin_encoder::*;
use crate::error;
use crate::notation::*;
use io::Read;
use log::error;
use nom::bits::{bits, streaming::take};
use nom::combinator::all_consuming;
use nom::error::{Error, ErrorKind};
use nom::multi::{count, many0};
use nom::sequence::tuple;
use nom::{Err, IResult, Needed};
use num_traits::FromPrimitive;
use std::fs::File;
use std::io;
use std::io::BufReader;

fn parse_measure_init(input: &[u8]) -> IResult<&[u8], MusicElement> {
    let take_bits = tuple((
        take(2usize),
        take(3usize),
        take(2usize),
        take(4usize),
        take(7usize),
        take(8usize),
        take(5usize),
    ));
    bits::<_, _, Error<(&[u8], usize)>, _, _>(take_bits)(input).and_then(
        |(inp, (id, beats, beat_type, fifths, tempo, reserve_bits, reserve_bits_2))| {
            let _id: MusicTagIdentifiers =
                FromPrimitive::from_u8(id).ok_or(Err::Error(Error::new(input, ErrorKind::Alt)))?;
            let beats = FromPrimitive::from_u8(beats)
                .ok_or(Err::Error(Error::new(input, ErrorKind::Alt)))?;
            let beat_type = FromPrimitive::from_u8(beat_type)
                .ok_or(Err::Error(Error::new(input, ErrorKind::Alt)))?;
            let key_sig = FromPrimitive::from_u8(fifths)
                .ok_or(Err::Error(Error::new(input, ErrorKind::Alt)))?;
            let tempo = Tempo::new_from_raw(tempo);
            let _throwaway: u8 = reserve_bits;
            let _throwaway2: u8 = reserve_bits_2;
            Ok((
                inp,
                MusicElement::MeasureInit(MeasureInitializer {
                    beats,
                    beat_type,
                    key_sig,
                    tempo,
                }),
            ))
        },
    )
}

fn parse_measure_meta(input: &[u8]) -> IResult<&[u8], MusicElement> {
    let take_bits = tuple((
        take(2usize),
        take(2usize),
        take(2usize),
        take(3usize),
        take(7usize),
        count(take(8usize), 2),
    ));
    bits::<_, _, Error<(&[u8], usize)>, _, _>(take_bits)(input).and_then(
        |(inp, (id, start_end, ending, dal_segno, throwaway, throwaway_vec))| {
            let _id: MusicTagIdentifiers =
                FromPrimitive::from_u8(id).ok_or(Err::Error(Error::new(input, ErrorKind::Alt)))?;
            let start_end = FromPrimitive::from_u8(start_end)
                .ok_or(Err::Error(Error::new(input, ErrorKind::Alt)))?;
            let ending = FromPrimitive::from_u8(ending)
                .ok_or(Err::Error(Error::new(input, ErrorKind::Alt)))?;
            let dal_segno = FromPrimitive::from_u8(dal_segno)
                .ok_or(Err::Error(Error::new(input, ErrorKind::Alt)))?;
            let _throwaway: u8 = throwaway;
            let _throwaway_vec: Vec<u8> = throwaway_vec;
            Ok((
                inp,
                MusicElement::MeasureMeta(MeasureMetaData {
                    start_end,
                    ending,
                    dal_segno,
                }),
            ))
        },
    )
}

fn parse_note_data_rest(input: &[u8]) -> IResult<&[u8], MusicElement> {
    let take_bits = tuple((
        take(2usize),
        take(7usize),
        take(4usize),
        take(3usize),
        take(1usize),
        take(1usize),
        take(2usize),
        take(2usize),
        take(2usize),
        take(2usize),
        take(1usize),
        take(1usize),
        take(2usize),
        take(2usize),
    ));
    bits::<_, _, Error<(&[u8], usize)>, _, _>(take_bits)(input).and_then(
        |(
            inp,
            (
                id,
                note_rest,
                phrase_dynamics,
                rhythm_value,
                dotted,
                arpeggiate,
                special_note,
                articulation,
                trill,
                ties,
                stress,
                chord,
                slur,
                voice,
            ),
        )| {
            let _id: MusicTagIdentifiers =
                FromPrimitive::from_u8(id).ok_or(Err::Error(Error::new(input, ErrorKind::Alt)))?;
            let note_rest = NoteRestValue::new_from_numeric(note_rest);
            let phrase_dynamics = FromPrimitive::from_u8(phrase_dynamics)
                .ok_or(Err::Error(Error::new(input, ErrorKind::Alt)))?;
            let rhythm_value = FromPrimitive::from_u8(rhythm_value)
                .ok_or(Err::Error(Error::new(input, ErrorKind::Alt)))?;
            let dotted: u8 = dotted;
            let dotted = dotted != 0u8;
            let arpeggiate = FromPrimitive::from_u8(arpeggiate)
                .ok_or(Err::Error(Error::new(input, ErrorKind::Alt)))?;
            let special_note = FromPrimitive::from_u8(special_note)
                .ok_or(Err::Error(Error::new(input, ErrorKind::Alt)))?;
            let articulation = FromPrimitive::from_u8(articulation)
                .ok_or(Err::Error(Error::new(input, ErrorKind::Alt)))?;
            let trill = FromPrimitive::from_u8(trill)
                .ok_or(Err::Error(Error::new(input, ErrorKind::Alt)))?;
            let ties = FromPrimitive::from_u8(ties)
                .ok_or(Err::Error(Error::new(input, ErrorKind::Alt)))?;
            let stress = FromPrimitive::from_u8(stress)
                .ok_or(Err::Error(Error::new(input, ErrorKind::Alt)))?;
            let chord = FromPrimitive::from_u8(chord)
                .ok_or(Err::Error(Error::new(input, ErrorKind::Alt)))?;
            let slur = FromPrimitive::from_u8(slur)
                .ok_or(Err::Error(Error::new(input, ErrorKind::Alt)))?;
            let voice = FromPrimitive::from_u8(voice)
                .ok_or(Err::Error(Error::new(input, ErrorKind::Alt)))?;
            Ok((
                inp,
                MusicElement::NoteRest(NoteData {
                    note_rest,
                    phrase_dynamics,
                    note_type: rhythm_value,
                    dotted,
                    arpeggiate,
                    special_note,
                    articulation,
                    trill,
                    ties,
                    stress,
                    chord,
                    slur,
                    voice,
                }),
            ))
        },
    )
}

fn parse_tuplet_data(input: &[u8]) -> IResult<&[u8], MusicElement> {
    let take_bits = tuple((
        take(2usize),
        take(2usize),
        take(3usize),
        take(3usize),
        take(3usize),
        take(1usize),
        take(1usize),
        count(take(8usize), 2),
    ));
    bits::<_, _, Error<(&[u8], usize)>, _, _>(take_bits)(input).and_then(
        |(
            inp,
            (
                id,
                start_stop,
                tuplet_number,
                actual_notes,
                normal_notes,
                dotted,
                reservebit,
                throwaway,
            ),
        )| {
            let _id: MusicTagIdentifiers =
                FromPrimitive::from_u8(id).ok_or(Err::Error(Error::new(input, ErrorKind::Alt)))?;
            let start_stop = FromPrimitive::from_u8(start_stop)
                .ok_or(Err::Error(Error::new(input, ErrorKind::Alt)))?;
            let tuplet_number = FromPrimitive::from_u8(tuplet_number)
                .ok_or(Err::Error(Error::new(input, ErrorKind::Alt)))?;
            let dotted: u8 = dotted;
            let dotted = dotted != 0u8;
            let _reservebit: u8 = reservebit;
            let _throwaway: Vec<u8> = throwaway;

            Ok((
                inp,
                MusicElement::Tuplet(TupletData {
                    start_stop,
                    tuplet_number,
                    actual_notes,
                    normal_notes,
                    dotted,
                }),
            ))
        },
    )
}

fn parse_id(input: &[u8]) -> IResult<&[u8], MusicTagIdentifiers> {
    bits::<_, _, Error<(&[u8], usize)>, _, _>(take(2usize))(input).and_then(|id| {
        let tag_id: Option<MusicTagIdentifiers> = FromPrimitive::from_u8(id.1);
        match tag_id {
            Some(tag_id) => {
                return Ok((input, tag_id));
            }
            _ => return Err(Err::Error(Error::new(input, ErrorKind::Alt))),
        }
    })
}

fn music_element(input: &[u8]) -> IResult<&[u8], MusicElement> {
    if input.len() == 0 {
        // This error is expected for EOF condition/ completion of parsing
        return Err(Err::Error(Error::new(input, ErrorKind::Eof)));
    }

    let id = parse_id(input).expect("Not enough bits for identifier.");
    match id.1 {
        MusicTagIdentifiers::MeasureInitializerTag => parse_measure_init(id.0),
        MusicTagIdentifiers::MeasureMetaDataTag => parse_measure_meta(id.0),
        MusicTagIdentifiers::NoteDataTag => parse_note_data_rest(id.0),
        MusicTagIdentifiers::TupletTag => parse_tuplet_data(id.0),
    }
}

fn parse_music_bin(input: &[u8], size: usize) -> IResult<&[u8], Vec<MusicElement>> {
    if input.len() < size {
        error!("input length of vector less than specified size");
        return Err(Err::Incomplete(Needed::new(size)));
    }

    if size < 1 {
        error!("input length too short.");
        return Err(Err::Incomplete(Needed::new(1)));
    }
    let results = all_consuming(many0(music_element))(input);
    return results;
}

pub struct MusicDecoder {
    r: Option<BufReader<File>>,
    data: Vec<u8>,
}

impl MusicDecoder {
    pub fn new(reader: Option<BufReader<File>>) -> MusicDecoder {
        let r = reader;
        MusicDecoder { r, data: vec![] }
    }

    pub fn reader_read(&mut self) -> error::Result<()> {
        match &mut self.r {
            None => Err(error::Error::MissingReader),
            Some(r) => {
                let _bytes_read = r
                    .read_to_end(&mut self.data)
                    .map_err(|e| error::Error::IoKind(e.kind().to_string()))?;
                //println!("read {} bytes", bytes_read);
                Ok(())
            }
        }
    }

    pub fn clear_data(&mut self) {
        self.data.clear();
    }

    pub fn raw_read(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    pub fn parse_data(&self) -> error::Result<Vec<MusicElement>> {
        match parse_music_bin(&self.data, self.data.len()) {
            Ok((_, r)) => Ok(r),
            _ => Err(error::Error::DecodingError),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MusicDecoder;
    use crate::bin_encoder::*;
    use crate::notation::*;
    use std::io::BufWriter;
    use std::path::PathBuf;
    use std::fs::File;

    #[test]
    fn test_music_note_data_rest_parse() {
        let mut music_dec = MusicDecoder::new(None);
        let note_rest_data: &[u8] = &[0xa0, 0xef, 0x84, 0x01];
        music_dec.raw_read(note_rest_data);
        let elems = music_dec.parse_data();
        assert!(elems.is_ok());

        assert_eq!(
            elems.unwrap().get(0),
            Some(&MusicElement::NoteRest(NoteData {
                note_rest: NoteRestValue::new_from_numeric(65),
                phrase_dynamics: PhraseDynamics::Forte,
                note_type: NoteType::SemiBreve,
                dotted: true,
                arpeggiate: Arpeggiate::NoArpeggiation,
                special_note: SpecialNote::None,
                articulation: Articulation::Marcato,
                trill: Trill::None,
                ties: NoteConnection::NoTie,
                stress: Stress::NotAccented,
                chord: Chord::NoChord,
                slur: SlurConnection::NoSlur,
                voice: Voice::Two,
            }))
        );

        // TODO: Add negative cases that fail
    }
    #[test]
    fn test_music_meta_parse() {
        let mut music_dec = MusicDecoder::new(None);
        let measure_meta_data: &[u8] = &[0x4e, 0x00, 0x00, 0x00];
        music_dec.raw_read(measure_meta_data);
        let elems = music_dec.parse_data();
        assert!(elems.is_ok());

        assert_eq!(
            elems.unwrap().get(0),
            Some(&MusicElement::MeasureMeta(MeasureMetaData {
                start_end: MeasureStartEnd::MeasureStart,
                ending: Ending::Three,
                dal_segno: DalSegno::DaCapo
            }))
        );
        // TODO: Add negative cases that fail
    }
    /// This function is just used for dumping serialized data structures to file to
    /// use for validation test data generation. Can be left commented out
    // #[test]
    // fn test_dump_bin_file() {
    //     let outfile = File::create(PathBuf::from("validation.bin")).expect("IO Error Occurred");
    //     let writer = BufWriter::new(BufWriter::new(outfile));
    //     let mut validation = MusicEncoder::new(writer);
    //     validation.insert_note_data(NoteData {
    //         note_rest: NoteRestValue::new_from_numeric(65),
    //         phrase_dynamics: PhraseDynamics::Forte,
    //         note_type: NoteType::SemiBreve,
    //         dotted: true,
    //         arpeggiate: Arpeggiate::NoArpeggiation,
    //         special_note: SpecialNote::None,
    //         articulation: Articulation::Marcato,
    //         trill: Trill::None,
    //         ties: NoteConnection::NoTie,
    //         stress: Stress::NotAccented,
    //         chord: Chord::NoChord,
    //         slur: SlurConnection::NoSlur,
    //         voice: Voice::Two,
    //     }).unwrap();

    //     validation.flush();
    // }
    #[test]
    fn test_music_init_parse() {
        let mut music_dec = MusicDecoder::new(None);

        // Positive case examples
        let music_init_data: &[u8] = &[0x12, 0x0c, 0x80, 0x00];
        music_dec.raw_read(music_init_data);

        let elems = music_dec.parse_data();
        assert!(elems.is_ok());
        assert_eq!(
            elems.unwrap().get(0),
            Some(&MusicElement::MeasureInit(MeasureInitializer {
                beats: Beats::Four,
                beat_type: BeatType::Four,
                key_sig: KeySignature::CMajorAminor,
                tempo: Tempo::default(),
            }))
        );

        //music_dec.clear_data();
        // TODO: Add negative cases that fail
    }
}
