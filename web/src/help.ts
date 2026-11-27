// The explanations of the fields, shown as tooltips next to their labels.

const LAYER_SEED =
  'Replaces the seed of the tune for this layer only, so that this layer can be changed while the others are kept. ' +
  'Left empty, the layer uses the seed of the tune.';

export const HELP = {
  tune: {
    seed:
      'The seed of the random numbers, an integer from 0 to 4294967295. ' +
      'Another value gives another tune of a similar feel; the value itself has no meaning.',
    key: 'The key note as a MIDI note number (60 is C4, 69 is A4). The melody stays around it.',
    tones:
      'The notes of the scale, in semitones from the key note. ' +
      'The degrees of the chords count the notes of this list from 0.',
    beatsPerBar: 'The beats in a bar: 4 for 4/4 time, 3 for 3/4 time.',
    bars: 'The number of bars of the tune.',
    resolution:
      'How the tune ends (the cadences at its end). ' +
      'Half or Plagal sounds like it continues; Perfect authentic sounds finished.',
  },
  cadence: {
    seed: LAYER_SEED,
    duration:
      'The length of each cadence (harmonic function) in beats, that is how often the chord changes. ' +
      '2 or 4 suits 4/4 time, and 3 suits 3/4 time.',
    transitions:
      'The weights of the next cadence (column) after each cadence (row), as a Markov chain. ' +
      'Each row is made into probabilities. The end of the tune is fixed by the resolution.',
  },
  chord: {
    seed: LAYER_SEED,
    roots:
      'The weights of the root of a chord on this cadence, as a degree of the scale from the key note ' +
      '(0 is I, 3 is IV, 4 is V).',
    tones:
      'The weights of the chord tones, as degrees of the scale from the root. ' +
      '0, 2, 4 is a triad, and 0, 2, 4, 6 a seventh chord.',
  },
  rhythm: {
    seed: LAYER_SEED,
    maxDepth: 'How many times a beat can be halved: the shortest note is a beat divided by 2 to this power.',
    pBranch:
      'The probability to halve a note, by its length in beats: 0 up to edge0, rising smoothly to 1 at edge1. ' +
      'A larger edge0 gives fewer short notes, and a smaller edge1 gives fewer long notes.',
    pNoteOn:
      'The probability that a note sounds rather than rests, by its length in beats: ' +
      '0 up to edge0, rising smoothly to 1 at edge1. Larger values give more rests, mostly on short notes.',
  },
  pitch: {
    seed: LAYER_SEED,
    absPitchFactor:
      'Lowers the weight of a pitch by its distance in semitones from the key note: not at all up to edge0, ' +
      'down to 0 (never chosen) at edge1. Larger values allow a wider range.',
    relPitchFactor:
      'Lowers the weight of a pitch by its distance in semitones from the previous note: not at all up to edge0, ' +
      'down to 0 (never chosen) at edge1. Larger values allow larger leaps.',
    factorInScale: 'Multiplies the weight of a pitch in the scale. Larger values keep the melody in the scale.',
    factorInChord: 'Multiplies the weight of a pitch in the current chord. Larger values keep the melody on the chord.',
    rhythmExponentFactor:
      'The weight of each pitch is raised to the power of 1 + this × the length of the note in beats. ' +
      'Larger values keep long notes on the likely pitches and leave short notes freer.',
  },
  variant: {
    seed: 'The seed of the random numbers of the variant (the second half), an integer from 0 to 4294967295.',
    resolution: 'How the variant ends. It replaces the resolution of the tune.',
    timespan:
      'The weights of the span to change, from start to end in beats from the beginning of the tune. ' +
      'Only what lies strictly inside it changes.',
    layer:
      'The weights of the layer to generate again within the timespan: ' +
      'cadence (and the chords and pitches on it), chord (and the pitches), rhythm, or pitch.',
  },
  bpm: 'The tempo for playing and for the MIDI file, in beats per minute.',
} as const;
