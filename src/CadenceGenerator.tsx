import type { Component } from 'solid-js';
import { Random } from 'reliable-random';

import * as util from './util';
import * as helper from './solid_helper';
import * as markov from './markov';

import {Tune, Cadence, CadencePostfix} from './tune';


export type CadenceGeneratorParameters = {
  seed: util.RandomSeed;
  probabilities: [Cadence, util.WeightedItem<Cadence>[]][];
  duration: number;
};
export class CadenceGeneratorParametersUiAdapter extends helper.UiAdapter<CadenceGeneratorParameters> {
  private seed = new helper.RandomSeed(4649,459);
  private probabilities: [Cadence, util.WeightedItem<Cadence>[]][] = ([
    [Cadence.T, [{value: Cadence.T, weight: 0.1}, {value: Cadence.D, weight: 0.3}, {value: Cadence.S, weight: 0.6}] ],
    [Cadence.D, [{value: Cadence.T, weight: 0.9}, {value: Cadence.D, weight: 0.1}, {value: Cadence.S, weight: 0.0}] ],
    [Cadence.S, [{value: Cadence.T, weight: 0.4}, {value: Cadence.D, weight: 0.5}, {value: Cadence.S, weight: 0.1}] ],
  ]);
  private duration = new helper.InputBoundNumber(2);
  get(): CadenceGeneratorParameters {
    return {
      seed: this.seed.get(),
      probabilities: this.probabilities,
      duration: this.duration.get()
    };
  }
  set(params: CadenceGeneratorParameters): CadenceGeneratorParameters {
    this.duration.set(params.duration);
    this.seed.set(params.seed);
    this.probabilities = params.probabilities;
    return params;
  }
  ui: Component = () => {
    return <details>
      <summary>Cadence Generator</summary>
      <label>seeds
        <helper.ClassUI instance={this.seed} />
      </label>
      <label>duration
        <helper.ClassUI instance={this.duration} />
      </label>
    </details>;
  };
}

export class CadenceGenerator {
  generate(tune: Tune, params: CadenceGeneratorParameters): util.Timeline<Cadence> {
    const times = [...util.rangeIterator(0, tune.time_measure[0] * tune.time_measure[1], params.duration)];
    const prefix: Cadence[] = [Cadence.T];  //TODO parametrize
    const postfix = CadencePostfix.get(tune.resolution.get(0).value);
    util.assertIsDefined(postfix);
    let chain = new markov.MarkovChain_TimeHomo_FiniteState(
      new Random(params.seed.state, params.seed.sequence), prefix[prefix.length-1], new Map(params.probabilities)
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
}
