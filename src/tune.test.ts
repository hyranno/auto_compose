import * as vectorious from 'vectorious';
import {Random} from 'reliable-random';

import * as util from './util';
import * as tunes from './tune';


describe("chord", ()=>{
  test("rot", ()=>{
    let scale = tunes.Scale.major(69);
    let org = new tunes.Chord(scale, 4, [0,2,4]);
    expect(org.getNotenums()).toEqual([76,80,83]);
    expect(org.rot(scale,1).getNotenums()).toEqual([88,80,83]);
    expect(org.rot(scale,2).getNotenums()).toEqual([88,92,83]);
    expect(org.rot(scale,3).getNotenums()).toEqual([88,92,95]);
    expect(org.rot(scale,4).getNotenums()).toEqual([100,92,95]);
    expect(org.rot(scale,5).getNotenums()).toEqual([100,104,95]);
    expect(org.rot(scale,6).getNotenums()).toEqual([100,104,107]);
    expect(org.rot(scale,-1).getNotenums()).toEqual([76,80,71]);
    expect(org.rot(scale,-2).getNotenums()).toEqual([76,68,71]);
    expect(org.rot(scale,-3).getNotenums()).toEqual([64,68,71]);
    expect(org.rot(scale,-4).getNotenums()).toEqual([64,68,59]);
    expect(org.rot(scale,-5).getNotenums()).toEqual([64,56,59]);
    expect(org.rot(scale,-6).getNotenums()).toEqual([52,56,59]);
  });
});
