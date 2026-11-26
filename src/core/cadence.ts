import {defaultContext, GenerateContext} from './random';

import * as util from './util';
import * as markov from './markov';

import {Tune, Cadence, CadencePostfix} from './tune';


export type Parameters = {
  seed: util.RandomSeed;
  probabilities: [Cadence, util.WeightedItem<Cadence>[]][];
  duration: number;
};

export function generate(tune: Tune, params: Parameters, ctx: GenerateContext = defaultContext): util.Timeline<Cadence> {
  const times = [...util.rangeIterator(0, tune.time_measure[0] * tune.time_measure[1], params.duration)];
  const prefix: Cadence[] = [Cadence.T];  //TODO parametrize
  const postfix = CadencePostfix.get(tune.resolution.get(0).value);
  util.assertIsDefined(postfix);
  let chain = new markov.MarkovChain_TimeHomo_FiniteState(
    ctx.rngFactory(params.seed, 'cadence'), prefix[prefix.length-1], new Map(params.probabilities)
  );
  let postfixOffset = times.length - postfix.length;
  let items: util.TimelineItem<Cadence>[] = times.map((t,i) => {
    if (i < prefix.length) {
      return {t:t, value: prefix[i]};
    } else if (i < postfixOffset) {
      return {t:t, value: chain.next_conditional({offset: postfixOffset-i, state: postfix[0]})};
    } else {
      return {t:t, value: postfix[i-postfixOffset]};
    }
  });
  return util.Timeline.fromItems(items);
}
