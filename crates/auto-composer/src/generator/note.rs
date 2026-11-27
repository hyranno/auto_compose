//! Notes: a pitch for each leaf of the rhythm tree.

use crate::error::{Error, Result};
use crate::random::Rng;
use crate::tune::{Note, NoteNode, Notenum, Tune};
use crate::util::{Timeline, WeightedItem, weighted_pick};

use super::pitch_weight;

/// Draws the pitch of `note` from `prev - 12 ..= prev + 11`, weighted by
/// [`pitch_weight::calc`]. Rests draw too.
pub fn generate_pitch(
    tune: &Tune,
    rng: &mut impl Rng,
    prev: Notenum,
    note: &mut NoteNode,
    params: &pitch_weight::Parameters,
) -> Result<()> {
    let items = (prev - 12..prev + 12)
        .map(|pitch| {
            let candidate = Note {
                pitch,
                duration: note.rhythm.duration,
                is_note_on: note.rhythm.is_note_on,
            };
            let weight = pitch_weight::calc(tune, prev, &candidate, note.rhythm.t, params)?;
            Ok(WeightedItem::new(weight, pitch))
        })
        .collect::<Result<Vec<_>>>()?;
    note.pitch = Some(*weighted_pick(&items, rng)?);
    Ok(())
}

/// Sets the pitches of the leaves of `notes` from left to right. The first previous pitch is
/// the key of the scale.
pub fn pitch_tree(
    tune: &Tune,
    mut notes: NoteNode,
    params: &pitch_weight::Parameters,
    rng: &mut impl Rng,
) -> Result<NoteNode> {
    let mut prev = tune.scale.key;
    for note in notes.leaves_mut() {
        generate_pitch(tune, rng, prev, note, params)?;
        prev = note.pitch.ok_or(Error::MissingPitch)?;
    }
    Ok(notes)
}

/// The leaves of a tree as a timeline.
pub fn tree_to_timeline(tree: &NoteNode) -> Result<Timeline<Note>> {
    let events = tree
        .leaves()
        .into_iter()
        .map(|n| n.to_timeline_item().ok_or(Error::MissingPitch))
        .collect::<Result<Vec<_>>>()?;
    Ok(Timeline::from_items(events))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::rhythm;
    use crate::generator::test_util::{Counting, SEEDS, default_params, seeded, simple_params};
    use crate::generator::tune as tune_generator;

    #[test]
    fn one_draw_per_leaf_and_pitches_near_the_previous_one() {
        for (name, base) in [("default", default_params()), ("simple", simple_params())] {
            for seed in SEEDS {
                let params = tune_generator::Parameters {
                    seed,
                    ..base.clone()
                };
                let tune = tune_generator::generate(&params).unwrap();
                let what = format!("{name} seed={seed}");
                // One pitch draw per leaf, rests included.
                let tree = rhythm::generate(&tune, &params.rhythm, &mut seeded(seed));
                let leaves = tree.leaves().len();
                let mut rng = Counting {
                    inner: seeded(seed),
                    consumed: 0,
                };
                pitch_tree(&tune, tree, &params.pitch, &mut rng).unwrap();
                assert_eq!(rng.consumed, leaves, "{what}");
                let mut prev = tune.scale.key;
                for e in tune.notes.list() {
                    assert!(
                        prev - 12 <= e.value.pitch && e.value.pitch <= prev + 11,
                        "{what}"
                    );
                    prev = e.value.pitch;
                }
            }
        }
    }

    #[test]
    fn leaves_and_notes_match() {
        let params = default_params();
        let tune = tune_generator::generate(&params).unwrap();
        let tree = tune_generator::generate_notes(&tune, &params).unwrap();
        assert_eq!(tree_to_timeline(&tree).unwrap(), tune.notes);
    }
}
