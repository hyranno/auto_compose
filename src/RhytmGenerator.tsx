import type { Component } from 'solid-js';
import {Random} from 'reliable-random';

import * as util from './util';
import * as helper from './solid_helper';

import {Tune, NoteTreeNode} from './tune';


export type Parameters = {
  seed: util.RandomSeed;
  max_beat_division_depth: number;
  pBranch: util.SmoothstepParameters;
  pNoteOn: util.SmoothstepParameters;
};
export class ParametersUiAdapter extends helper.UiAdapter<Parameters> {
  private seed = new helper.RandomSeed(4649,459);
  private max_beat_division_depth = new helper.InputBoundNumber(3);
  private pBranch = new helper.Smoothstep(1/2, 3);
  private pNoteOn = new helper.Smoothstep(0, 1.2);
  get(): Parameters {
    return {
      seed: this.seed.get(),
      max_beat_division_depth: this.max_beat_division_depth.get(),
      pBranch: this.pBranch.get(),
      pNoteOn: this.pNoteOn.get()
    };
  }
  set(params: Parameters): Parameters {
    this.seed.set(params.seed);
    this.max_beat_division_depth.set(params.max_beat_division_depth);
    this.pBranch.set(params.pBranch);
    this.pNoteOn.set(params.pNoteOn);
    return params;
  }
  ui: Component = () => {
    return <details><summary>RhythmGenerator</summary>
      <label>seeds
        <helper.ClassUI instance={this.seed} />
      </label>
      <label>max beat division depth
        <helper.ClassUI instance={this.max_beat_division_depth} />
      </label>
      <label>pBranchEdges
        <helper.ClassUI instance={this.pBranch} />
      </label>
      <label>pNoteOnEdges
        <helper.ClassUI instance={this.pNoteOn} />
      </label>
    </details>;
  };
}

export function generateSubTree(
  node: NoteTreeNode, rand: Random, depthcount: number, params: Parameters
) {
  node.children = [];
  let pBranch = util.smoothstep(params.pBranch.edge0, params.pBranch.edge1, node.duration);
  let isBranch =  (0 < depthcount) && (rand.random() < pBranch);
  if (isBranch) {
    const numChildren = 2;
    const subDuration = node.duration / numChildren;
    node.children.push(
      ...[...util.rangeIterator(0,numChildren)].map(i => new NoteTreeNode(
        {t: node.t+i*subDuration, duration: subDuration, isNoteOn: true}
      ))
    );
    node.children.forEach(n => generateSubTree(n, rand, depthcount-1, params));
  } else {
    const pNoteOn = util.smoothstep(params.pNoteOn.edge0, params.pNoteOn.edge1, node.duration);
    node.rhythm.isNoteOn = rand.random() < pNoteOn;
  }
}
export function mergeTopBranches(root: NoteTreeNode, rand: Random, params: Parameters) {
  for (let i=0; i < root.children.length-1; i++) {
    const duration = root.children[i].duration + root.children[i+1].duration;
    const pMerge = 1 - util.smoothstep(params.pBranch.edge0, params.pBranch.edge1, duration);
    if (rand.random() < pMerge) {
      root.children[i].rhythm.duration = duration;
      root.children[i].children = [];
      root.children[i+1].rhythm.duration = 0;
      root.children[i+1].children = [];
      i++;
    }
  }
  root.children = root.children.filter(n => n.duration > 0);
}
export function generate(tune: Tune, params: Parameters): NoteTreeNode {
  const numBeats = tune.time_measure[0];
  const numBars = tune.time_measure[1];
  let rand = new Random(params.seed.state, params.seed.sequence);
  let phrase = new NoteTreeNode({t:0, duration: numBeats*numBars, isNoteOn: true});
  let bars = [...util.rangeIterator(0, numBars)].map(i => new NoteTreeNode({
    t: i*numBeats, duration: numBeats, isNoteOn: true,
  }));
  bars.forEach(bar => {
    let beats = [...util.rangeIterator(0, numBeats)].map(i => new NoteTreeNode({
      t: bar.t+i*1, duration: 1, isNoteOn: true,
    }));
    beats.forEach(beat => generateSubTree(beat, rand, tune.max_beat_division_depth, params));
    bar.push(...beats);
    mergeTopBranches(bar, rand, params);
  });
  phrase.push(...bars);
  return phrase;
}
