//! Notes: a pitch for each leaf of the rhythm tree.

use crate::error::Result;
use crate::random::Rng;
use crate::tune::{NoteNode, Notenum, Rhythm, RhythmNode, Tune};
use crate::util::{WeightedItem, weighted_pick};

use super::pitch_weight;

/// Draws the pitch of a note of `rhythm` from `prev - 12 ..= prev + 11`, weighted by
/// [`pitch_weight::calc`].
pub fn draw_pitch(
    tune: &Tune,
    rng: &mut impl Rng,
    prev: Notenum,
    rhythm: &Rhythm,
    params: &pitch_weight::Parameters,
) -> Result<Notenum> {
    let items = (prev - 12..prev + 12)
        .map(|pitch| {
            let weight = pitch_weight::calc(tune, prev, pitch, rhythm, params)?;
            Ok(WeightedItem::new(weight, pitch))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(*weighted_pick(&items, rng)?)
}

/// Draws the pitches of the notes of `rhythm` from left to right, each after the previous
/// note. The first previous pitch is the key of the scale. The rests draw nothing.
pub fn pitch_tree(
    tune: &Tune,
    rhythm: RhythmNode,
    params: &pitch_weight::Parameters,
    rng: &mut impl Rng,
) -> Result<NoteNode> {
    let mut prev = tune.scale.key;
    rhythm.try_map_leaves(&mut |r, is_note_on| {
        if !is_note_on {
            return Ok(None);
        }
        prev = draw_pitch(tune, rng, prev, &r, params)?;
        Ok(Some(prev))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::rhythm;
    use crate::generator::test_util::{Counting, SEEDS, default_params, seeded, simple_params};
    use crate::generator::tune as tune_generator;

    #[test]
    fn one_draw_per_note_and_pitches_near_the_previous_note() {
        let mut rests = 0;
        for (name, base) in [("default", default_params()), ("simple", simple_params())] {
            for seed in SEEDS {
                let params = tune_generator::Parameters {
                    seed,
                    ..base.clone()
                };
                let tune = tune_generator::generate(&params).unwrap();
                let what = format!("{name} seed={seed}");
                // One pitch draw per note, none for the rests.
                let tree = rhythm::generate(&tune, &params.rhythm, &mut seeded(seed));
                let notes = tree.leaves().iter().filter(|&&(_, &on)| on).count();
                rests += tree.leaves().len() - notes;
                let mut rng = Counting {
                    inner: seeded(seed),
                    consumed: 0,
                };
                pitch_tree(&tune, tree, &params.pitch, &mut rng).unwrap();
                assert_eq!(rng.consumed, notes, "{what}");
                let mut prev = tune.scale.key;
                for pitch in tune.notes.list().iter().filter_map(|e| e.value.pitch) {
                    assert!(prev - 12 <= pitch && pitch <= prev + 11, "{what}");
                    prev = pitch;
                }
            }
        }
        assert!(rests > 0, "no rests to test");
    }

    #[test]
    fn leaves_and_notes_match() {
        let params = default_params();
        let tune = tune_generator::generate(&params).unwrap();
        let tree = tune_generator::generate_notes(&tune, &params).unwrap();
        assert_eq!(tree.to_timeline(), tune.notes);
    }
}
