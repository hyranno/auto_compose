//! Notes: a pitch for each leaf of the rhythm tree.

use crate::error::Result;
use crate::random::Rng;
use crate::tune::{NoteNode, Notenum, Rhythm, RhythmNode, Tune};
use crate::util::{WeightedItem, weighted_pick};

use super::pitch_weight;

/// Draws the pitch of a note of `rhythm` from `prev - 12 ..= prev + 11`, weighted by
/// [`pitch_weight::calc`]. Rests draw too.
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

/// Draws the pitches of the leaves of `rhythm` from left to right. The first previous pitch
/// is the key of the scale.
pub fn pitch_tree(
    tune: &Tune,
    rhythm: RhythmNode,
    params: &pitch_weight::Parameters,
    rng: &mut impl Rng,
) -> Result<NoteNode> {
    let mut prev = tune.scale.key;
    rhythm.try_map_leaves(&mut |r, ()| {
        prev = draw_pitch(tune, rng, prev, &r, params)?;
        Ok(prev)
    })
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
                let leaves = tree.rhythms().len();
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
        assert_eq!(tree.to_timeline(), tune.notes);
    }
}
