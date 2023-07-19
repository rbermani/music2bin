use crate::layout::*;
use crate::notation::*;
use failure::err_msg;
use failure::Error as FailureError;
use io::Read;
use nom::bits::{bits, streaming::take};
use nom::combinator::all_consuming;
use nom::error::{Error, ErrorKind};
use nom::multi::many0;
use nom::sequence::tuple;
use nom::{Err, IResult, Needed};
use num_traits::FromPrimitive;
use std::fs::File;
use std::io;
use std::io::BufReader;

fn parse_measure_init(input: &[u8]) -> IResult<&[u8], MusicElement> {
    let take_bits = tuple((
        take(3usize),
        take(4usize),
        take(3usize),
        take(2usize),
        take(4usize),
        take(4usize),
        take(4usize),
    ));
    bits::<_, _, Error<(&[u8], usize)>, _, _>(take_bits)(input).and_then(
        |(inp, (id, tempo, beats, beat_type, fifths, t_dynamics, b_dyanamics))| {
            let _id: MusicTagIdentifiers =
                FromPrimitive::from_u8(id).ok_or(Err::Error(Error::new(input, ErrorKind::Alt)))?;
            let tempo = FromPrimitive::from_u8(tempo)
                .ok_or(Err::Error(Error::new(input, ErrorKind::Alt)))?;
            let beats = FromPrimitive::from_u8(beats)
                .ok_or(Err::Error(Error::new(input, ErrorKind::Alt)))?;
            let beat_type = FromPrimitive::from_u8(beat_type)
                .ok_or(Err::Error(Error::new(input, ErrorKind::Alt)))?;
            let key_sig = FromPrimitive::from_u8(fifths)
                .ok_or(Err::Error(Error::new(input, ErrorKind::Alt)))?;
            let treble_dynamics = FromPrimitive::from_u8(t_dynamics)
                .ok_or(Err::Error(Error::new(input, ErrorKind::Alt)))?;
            let bass_dynamics = FromPrimitive::from_u8(b_dyanamics)
                .ok_or(Err::Error(Error::new(input, ErrorKind::Alt)))?;
            Ok((
                inp,
                MusicElement::MeasureInit(MeasureInitializer {
                    tempo,
                    beats,
                    beat_type,
                    key_sig,
                    treble_dynamics,
                    bass_dynamics,
                }),
            ))
        },
    )
}

fn parse_measure_meta(input: &[u8]) -> IResult<&[u8], MusicElement> {
    let take_bits = tuple((take(3usize), take(1usize), take(1usize), take(3usize)));
    bits::<_, _, Error<(&[u8], usize)>, _, _>(take_bits)(input).and_then(
        |(inp, (id, start_end, repeat, dal_segno))| {
            let _id: MusicTagIdentifiers =
                FromPrimitive::from_u8(id).ok_or(Err::Error(Error::new(input, ErrorKind::Alt)))?;
            let start_end = FromPrimitive::from_u8(start_end)
                .ok_or(Err::Error(Error::new(input, ErrorKind::Alt)))?;
            let repeat = FromPrimitive::from_u8(repeat)
                .ok_or(Err::Error(Error::new(input, ErrorKind::Alt)))?;
            let dal_segno = FromPrimitive::from_u8(dal_segno)
                .ok_or(Err::Error(Error::new(input, ErrorKind::Alt)))?;

            Ok((
                inp,
                MusicElement::MeasureMeta(MeasureMetaData {
                    start_end,
                    repeat,
                    dal_segno,
                }),
            ))
        },
    )
}

fn parse_note_data_rest(input: &[u8]) -> IResult<&[u8], MusicElement> {
    let take_bits = tuple((
        take(3usize),
        take(7usize),
        take(4usize),
        take(3usize),
        take(1usize),
        take(2usize),
        take(2usize),
        take(2usize),
        take(2usize),
        take(1usize),
        take(1usize),
    ));
    bits::<_, _, Error<(&[u8], usize)>, _, _>(take_bits)(input).and_then(
        |(
            inp,
            (
                id,
                note_rest,
                phrase_dynamics,
                rhythm_value,
                arpeggiate,
                special_note,
                articulation,
                trill,
                ties,
                rh_lh,
                stress,
            ),
        )| {
            let _id: MusicTagIdentifiers =
                FromPrimitive::from_u8(id).ok_or(Err::Error(Error::new(input, ErrorKind::Alt)))?;
            let note_rest = FromPrimitive::from_u8(note_rest)
                .ok_or(Err::Error(Error::new(input, ErrorKind::Alt)))?;
            let phrase_dynamics = FromPrimitive::from_u8(phrase_dynamics)
                .ok_or(Err::Error(Error::new(input, ErrorKind::Alt)))?;
            let rhythm_value = FromPrimitive::from_u8(rhythm_value)
                .ok_or(Err::Error(Error::new(input, ErrorKind::Alt)))?;
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
            let rh_lh = FromPrimitive::from_u8(rh_lh)
                .ok_or(Err::Error(Error::new(input, ErrorKind::Alt)))?;
            let stress = FromPrimitive::from_u8(stress)
                .ok_or(Err::Error(Error::new(input, ErrorKind::Alt)))?;
            Ok((
                inp,
                MusicElement::NoteRest(NoteData {
                    note_rest,
                    phrase_dynamics,
                    rhythm_value,
                    arpeggiate,
                    special_note,
                    articulation,
                    trill,
                    ties,
                    rh_lh,
                    stress,
                }),
            ))
        },
    )
}

fn parse_id(input: &[u8]) -> IResult<&[u8], MusicTagIdentifiers> {
    bits::<_, _, Error<(&[u8], usize)>, _, _>(take(3usize))(input).and_then(|id| {
        let tag_id: Option<MusicTagIdentifiers> = FromPrimitive::from_u8(id.1);
        match tag_id {
            Some(tag_id) => {
                return Ok((input, tag_id));
            }
            _ => return { Err(Err::Error(Error::new(input, ErrorKind::Alt))) },
        }
    })
}

fn music_element(input: &[u8]) -> IResult<&[u8], MusicElement> {
    if input.len() == 0 {
        return Err(Err::Error(Error::new(input, ErrorKind::Eof)));
    }

    let id = parse_id(input).expect("Not enough bits for identifier.");
    println!("music element");
    match id.1 {
        MusicTagIdentifiers::MeasureInitializerTag => parse_measure_init(id.0),
        MusicTagIdentifiers::MeasureMetaDataTag => parse_measure_meta(id.0),
        MusicTagIdentifiers::NoteDataTag => parse_note_data_rest(id.0),
        _ => Err(Err::Error(Error::new(input, ErrorKind::Alt))),
    }
}

fn parse_music_bin(input: &[u8], size: usize) -> IResult<&[u8], Vec<MusicElement>> {
    if input.len() < size {
        return Err(Err::Incomplete(Needed::new(size)));
    }

    if size < 1 {
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

    pub fn reader_read(&mut self) -> Result<(), FailureError> {
        match &mut self.r {
            None => Err(err_msg("Reader is missing.")),
            Some(r) => {
                let bytes_read = r.read_to_end(&mut self.data)?;
                println!("read {} bytes", bytes_read);
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

    pub fn parse_data(&self) -> Result<Vec<MusicElement>, FailureError> {
        match parse_music_bin(&self.data, self.data.len()) {
            Ok((_, r)) => Ok(r),
            _ => Err(err_msg("Unknown")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MusicDecoder;
    use crate::notation::*;
    #[test]
    fn test_music_note_data_rest_parse() {
        let mut music_dec = MusicDecoder::new(None);
        let note_rest_data: &[u8] = &[0x50, 0x42, 0x80, 0x00];
        music_dec.raw_read(note_rest_data);
        let elems = music_dec.parse_data();
        assert!(elems.is_ok());

        assert_eq!(
            elems.unwrap().get(0),
            Some(&MusicElement::NoteRest(NoteData {
                note_rest: 65,
                phrase_dynamics: PhraseDynamics::None,
                rhythm_value: Duration::Crochet,
                arpeggiate: Arpeggiate::NoArpeggiation,
                special_note: SpecialNote::None,
                articulation: Articulation::None,
                trill: Trill::None,
                ties: NoteConnection::NoTie,
                rh_lh: RightHandLeftHand::RightHand,
                stress: Stress::NotAccented,
            }))
        );

        // TODO: Add negative cases that fail
    }
    #[test]
    fn test_music_meta_parse() {
        let mut music_dec = MusicDecoder::new(None);
        let measure_meta_data: &[u8] = &[0x20];
        music_dec.raw_read(measure_meta_data);
        let elems = music_dec.parse_data();
        assert!(elems.is_ok());

        assert_eq!(
            elems.unwrap().get(0),
            Some(&MusicElement::MeasureMeta(MeasureMetaData {
                start_end: MeasureStartEnd::MeasureStart,
                repeat: Repeats::NoRepeat,
                dal_segno: DalSegno::None
            }))
        );
        // TODO: Add negative cases that fail
    }
    #[test]
    fn test_music_init_parse() {
        let mut music_dec = MusicDecoder::new(None);

        // Positive case examples
        let music_init_data: &[u8] = &[0x12, 0x90, 0x17];
        music_dec.raw_read(music_init_data);

        let elems = music_dec.parse_data();

        assert!(elems.is_ok());

        assert_eq!(
            elems.unwrap().get(0),
            Some(&MusicElement::MeasureInit(MeasureInitializer {
                tempo: Tempo::Allegro,
                beats: Beats::Four,
                beat_type: BeatType::Four,
                key_sig: KeySignature::CMajorAminor,
                treble_dynamics: Dynamics::Pianissimo,
                bass_dynamics: Dynamics::Fortississimo,
            }))
        );

        //music_dec.clear_data();

        // TODO: Add negative cases that fail
    }
}
