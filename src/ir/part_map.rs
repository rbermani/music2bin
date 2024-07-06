//use muxml::muxml_types::{ScorePart, PartListElement, Part};
use muxml::score::CompleteParts;

use super::musical_part::MusicalPart;
use crate::error::{Error, Result};
use std::collections::BTreeMap;

// This data type uses an Index Pointer pattern
// TODO: Add logic to actually remove entries from Vec and BTreeMap upon
// item removal and re-conform all of the vector indexes after the item so that
// indexes in the BTreeMap continue to point to the correct Vec indexes

type VecIdx = usize;
type PartCount = usize;
type PartId = String;
type PartIdIndex = Option<VecIdx>;
type PartIdPair<'a> = (&'a PartId, &'a PartIdIndex); // Alias for the reference pair
type PartIdMap = BTreeMap<PartId, PartIdIndex>;
type PartIdValue = Option<MusicalPart>;
type PartIdRefValue<'a> = Option<&'a MusicalPart>;

#[derive(Eq, PartialEq, Default, Debug, Clone)]
pub struct PartMap {
    part_ids: PartIdMap,
    parts: Vec<PartIdValue>,
}

impl PartMap {
    const MAX_SUPPORTED_VOICES: usize = 4;

    pub fn new() -> PartMap {
        PartMap {
            part_ids: PartIdMap::new(),
            parts: vec![],
        }
    }

    pub fn get_removed_parts(&self) -> PartCount {
        self.part_ids.iter().fold(
            0,
            |acc: usize, (_k, val): PartIdPair | {
                if val.is_none() {
                    acc + 1
                } else {
                    acc
                }
            },
        )
    }

    pub fn keys(&self) -> Vec<String> {
        self.part_ids.keys().map(|key| key.to_string()).collect()
    }

    pub fn get_part_ids(&self) -> PartIdMap {
        self.part_ids.clone()
    }

    pub fn num_part_ids(&self) -> PartCount {
        self.part_ids.len()
    }

    pub fn num_parts(&self) -> PartCount {
        self.parts.len()
    }

    pub fn get_part(&self, idx: usize) -> PartIdRefValue {
        if let Some(val) = self.parts.get(idx) {
            val.as_ref()
        } else {
            None
        }
    }

    pub fn remove_part(&mut self, part_key: &str) {
        if self.part_ids.insert(part_key.to_string(), None).is_none() {
            println!("No existing value was present for key");
        }
    }
    /// Combine musical parts (if feasible)
    ///
    /// Combines the parts in the map into one if the number and configuration
    /// of each part is the same
    pub fn combine_parts(&mut self) {

    }
    // pub fn extend_parts(&mut self, musical_parts: Vec<MusicalPart>) {
    //     self.parts.extend(musical_parts);
    // }

    fn insert_part_id(&mut self, part_key: &str, val: PartIdIndex) {
        self.part_ids.insert(part_key.to_string(), val);
    }

    pub fn push_part(&mut self, part_key: &str, part: MusicalPart) -> Result<()> {
        if let Some((_k, v)) = self.part_ids.get_key_value(part_key) {
            if v.is_none() {
                self.parts.push(Some(part));
                // Populate the value at this key to indicate the index into the vector where this part resides
                self.insert_part_id(part_key, Some(self.parts.len() - 1));
                Ok(())
            } else {
                // There is already a value at this location
                Err(Error::ItemExists)
            }
        } else {
            // No initial value was added, so add one now
            self.parts.push(Some(part));
            // Populate the value at this key to indicate the index into the vector where this part resides
            self.insert_part_id(part_key, Some(self.parts.len() - 1));
            Ok(())
        }
    }

    pub fn add_part_id(&mut self, id: &str) -> Result<()> {
        if self.part_ids.contains_key(id) {
            Err(Error::ItemExists)
        } else {
            self.insert_part_id(id, None);
            Ok(())
        }
    }
}

// impl From<&PartMap> for Vec<Part> {
//     fn from(pm: &PartMap) -> Self {
//         let mut p_elems: Vec<Part> = vec![];
//         for (part_id, opt_idx) in pm.get_part_ids() {
//             if let Some(idx) = opt_idx {
//                 println!("part {}", part_id);
//                 let part = pm.get_part(idx).unwrap();
//                 let measure = part.into();
//                 p_elems.push(Part {
//                     id: part_id.to_string(),
//                     measure,
//                 })
//             }
//         }
//         p_elems
//     }
// }

// impl From<&PartMap> for PartListElement {
//     fn from(pm: &PartMap) -> Self {
//         let mut score_parts = vec![];
//         for (part_id, opt_idx) in pm.get_part_ids() {
//             if let Some(_idx) = opt_idx {
//                 score_parts.push(ScorePart {
//                     id: part_id.to_string(),
//                     part_name: "Piano".to_string(),
//                 });
//             }
//         }
//         PartListElement {
//             score_part: score_parts,
//         }
//     }
// }

impl TryFrom<PartMap> for CompleteParts {
    type Error = Error;
    fn try_from(pm: PartMap) -> std::result::Result<Self, Self::Error> {
        let mut complete_parts = CompleteParts::default();
        for (part_id, opt_idx) in pm.get_part_ids() {
            if let Some(idx) = opt_idx {
                println!("Part ID: {}", part_id.as_str());
                complete_parts.add_part(part_id.as_str(), "Piano")?;
                let part = pm.get_part(idx).unwrap();
                let measures = part.into();
                complete_parts.extend_measures(part_id.as_str(), measures)?;
            }
        }
        Ok(complete_parts)
    }
}
