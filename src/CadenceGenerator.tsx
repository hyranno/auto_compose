import type { Component } from 'solid-js';

import * as util from './util';
import * as helper from './solid_helper';

import {Tune, Cadence} from './tune';


export type CadenceGeneratorParameters = {
  seed: util.RandomSeed;
  probabilities: [Cadence, util.WeightedItem<Cadence>[]][];
  duration: number;
};
export class CadenceGenerator {
  private seed = new helper.RandomSeed(4649,459);
  private probabilities = new Map<Cadence, util.WeightedItem<Cadence>[]>([
    [Cadence.T, [{value: Cadence.T, weight: 0.1}, {value: Cadence.D, weight: 0.6}, {value: Cadence.S, weight: 0.3}] ],
    [Cadence.D, [{value: Cadence.T, weight: 0.6}, {value: Cadence.D, weight: 0.1}, {value: Cadence.S, weight: 0.3}] ],
    [Cadence.S, [{value: Cadence.T, weight: 0.9}, {value: Cadence.D, weight: 0.0}, {value: Cadence.S, weight: 0.1}] ],
  ]);
  private duration = new helper.InputBoundNumber(2);
  //how it ends?
  generate(tune: Tune): util.Timeline<Cadence> {
    let times = [...util.rangeIterator(0, tune.time_measure[0] * tune.time_measure[1], this.duration.get())];
    let cadences: Cadence[] = [Cadence.T, Cadence.D, ]; // how it end  //TODO: can change
    let chain = new util.MarkovChain_TimeHomo_FiniteState(this.seed.createRandom(), cadences[cadences.length-1], this.probabilities);
    while (cadences.length < times.length) {
      cadences.push(chain.next());
    }
    let events: util.TimelineItem<Cadence>[] = cadences.reverse().map((v,i)=>{return {t:times[i], value:v}});
    return util.Timeline.fromItems(events);
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
  setParameters(params: CadenceGeneratorParameters) {
    this.duration.set(params.duration);
    this.seed.set(params.seed);
    this.probabilities = new Map<Cadence, util.WeightedItem<Cadence>[]>(params.probabilities);
  }
}
