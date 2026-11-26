import {defaultContext, GenerateContext} from './random';

import * as util from './util';

import {Tune, Cadence, Chord} from './tune';


export namespace RootGenerator {
  export type Parameters = {
    seed: util.RandomSeed;
    probabilities: [Cadence, util.WeightedItem<number>[]][];
  };
  export function generate(tune: Tune, params: Parameters, ctx: GenerateContext = defaultContext): util.Timeline<number> {
    let rand = ctx.rngFactory(params.seed, 'chord.root');
    let probabilities = new Map(params.probabilities);
    let events = tune.cadence.list().map( c => {
      let items = probabilities.get(c.value);
      util.assertIsDefined(items);
      return {
        t: c.t,
        value: (new util.WeightedRandom(rand, items)).get(),
      }
    } );
    return util.Timeline.fromItems(events);
  }
}

export namespace ToneGenerator {
  export type Parameters = {
    rand_tones: util.WeightedRandomParameters<number[]>;
  };
  export function generate(tune:Tune, roots: util.Timeline<number>, params: Parameters, ctx: GenerateContext = defaultContext): util.Timeline<Chord> {
    let rand = ctx.rngFactory(params.rand_tones.seed, 'chord.tone');
    let rand_tones = new util.WeightedRandom(rand, params.rand_tones.items);
    let events = roots.list().map( c => {return {
      t: c.t,
      value: new Chord(
        tune.scale,
        c.value,
        rand_tones.get()
      )
    }} );
    return util.Timeline.fromItems(events);
  }
}

export namespace ToneModifier {
  export type Parameters = {}

}


export type Parameters = {
  root: RootGenerator.Parameters;
  tone: ToneGenerator.Parameters;
  modifier: ToneModifier.Parameters;
};
export function generate(tune: Tune, params: Parameters, ctx: GenerateContext = defaultContext): util.Timeline<Chord> {
    return ToneGenerator.generate(tune, RootGenerator.generate(tune, params.root, ctx), params.tone, ctx);
}
