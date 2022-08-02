
import {Tune, Cadence, Chord, Note} from './tune';
import {TuneGenerator} from './TuneGenerator';
import {CadenceGenerator, CadenceGeneratorParameters} from './CadenceGenerator';
import {ChordGenerator, ChordGeneratorParameters} from './ChordGenerator';
import {NoteGenerator, NoteGeneratorParameters} from './NoteGenerator';
import * as util from './util';



export class VariantGenerator {
  private orgGenerator: TuneGenerator;
  generate(): Tune {
    let tune = this.orgGenerator.generate();
    //modify part
    return tune;
  }
}


class CadenceVariantGenerator {
  private orgGenerator: ChordGenerator;
  private nopRand: util.WeightedRandom<boolean>;
  private regenRangeRand: util.WeightedRandom<[number,number]>;
  generate(tune: Tune): util.Timeline<Cadence> {
    if (this.nopRand.get()) {
      return tune.cadence;
    } else {
      let range = this.regenRangeRand.get();
      let [startIndex, endIndex] = range.map((t) => tune.cadence.getIndex(t))
      let li = tune.cadence.list();
      for (let i=startIndex+1; i<endIndex; i++) {
        //regen where li[i] and li[endIndex] are defined
      }
      return util.Timeline.fromItems(li);
    }
  }
}

class ChordVariantGenerator {
  //generate(tune: Tune, part: [number, number]): util.Timeline<number> {}
}
class NoteVariantGenerator {
  //generate(tune: Tune, part: [number, number]): util.Timeline<Note> {}
}
class RhythmVariantGenerator {}
