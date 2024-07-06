mod musical_part;
mod muxml_parser;
mod part_map;

pub mod ir_to_xml;
pub mod measure_checker;
pub mod notation;
pub mod xml_to_ir;
pub mod multipartxml_to_ir;

pub use musical_part::MusicalPart;
use notation::{TimeModification, TupletActual, TupletNormal};
pub use notation::{MusicElement, TupletNumber};
pub use part_map::PartMap;

pub use xml_to_ir::xml_to_ir;
pub use multipartxml_to_ir::multipartxml_to_ir;

use muxml::muxml_types::TimeModificationElement;

fn convert_time_modification(t_mod: &TimeModificationElement) -> TimeModification {
    let tup_ac = TupletActual::try_from(t_mod.actual_notes.as_ref())
        .expect("Cannot convert this TupletActual string.");
    let tup_norm = TupletNormal::try_from(t_mod.normal_notes.as_ref())
        .expect("Cannot convert this TupletNormal string.");

    TimeModification::new(tup_ac, tup_norm)
}

impl From<TimeModificationElement> for TimeModification {
    fn from(time_mod_elem: TimeModificationElement) -> Self {
        convert_time_modification(&time_mod_elem)
    }
}

impl From<&TimeModificationElement> for TimeModification {
    fn from(time_mod_elem: &TimeModificationElement) -> Self {
        convert_time_modification(time_mod_elem)
    }
}