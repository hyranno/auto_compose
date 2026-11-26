import * as util from './util';
import {defaultContext, GenerateContext} from './random';

import * as CadenceGenerator from './cadence';
import * as ChordGenerator from './chord';
import * as NoteGenerator from './note';

import {Tune, Scale, Resolution} from './tune';


export type Parameters = {
  scale: {key: number, tones: number[]};
  time_measure: [number, number];
  resolution: Resolution;
  cadence: CadenceGenerator.Parameters;
  chord: ChordGenerator.Parameters;
  note: NoteGenerator.Parameters;
};

export function generate(params: Parameters, ctx: GenerateContext = defaultContext): Tune {
  util.assertIsDefined(params);
  let tune = new Tune();
  tune.length = params.time_measure[0] * params.time_measure[1];
  tune.scale = new Scale(params.scale.key, params.scale.tones);
  tune.time_measure = params.time_measure;
  tune.max_beat_division_depth = params.note.rhythm.max_beat_division_depth;
  tune.resolution = util.Timeline.fromItems([{t:0, value:params.resolution}]);
  tune.cadence = CadenceGenerator.generate(tune, params.cadence, ctx);
  tune.chord = ChordGenerator.generate(tune, params.chord, ctx);
  tune.notes = NoteGenerator.generate(tune, params.note, ctx);
  return tune;
}
