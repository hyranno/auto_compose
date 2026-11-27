[English](README.md) | 日本語

# これは
作曲補助プログラムです。
各種パラメータに基づき楽節を自動生成します。
西洋の音楽理論をベースにしていますが、簡略化や誤りを含みます。
作曲したいがゼロから作るのは厳しい、という時にどうぞ。

次の2つの形で使えます。

* [Webページ](#webページ) ... ブラウザで[ここ](https://hyranno.github.io/auto_compose/)から試せます。
* [Rustライブラリ](#rustライブラリ)(`auto-composer`) ... 自分のプログラムから曲を生成できます。
  TypeScriptの型定義付きのWebAssemblyバインディングもあります。


# 著作権について
このプログラム自体の著作権はauthor(hyranno)に帰属し、MIT Licenseで利用できます。
このプログラムは機械学習を採用していないので、学習データセットの著作権問題は存在しません。

プログラムが出力したままの楽曲については、著作権が認められない可能性があります。
AIによって生み出される創作物の取扱いに準ずるものと考えられますが、法に詳しい人に確認してください。

プログラムの出力を元に編曲・作曲されたものについては、編曲・作曲した人に著作権があります。


# Webページ
画面上部は各種パラメータ設定、 画面中央はピアノロール、 画面下部は出力パネルになっています。

## パラメータ設定
生成に用いられるパラメータを設定できます。
各パラメータの意味については後述します。各入力欄の横の `?` にも説明が表示されます。
パラメータを変更後、generateボタンを押して(または入力欄でEnterを押して)曲を再生成してください。
読み取れない値の入力欄は赤枠で示され、直すまで曲は生成されません。

設定はsave JSONボタンでファイルに保存し、load JSONボタンで読み込めます。
load defaultボタンでページを開いたときの設定(曲は `resources/TuneGenParams_minor.json`)に戻せます。
ファイルの例はresourcesフォルダにあり、書式(JSON Schema)はschemasフォルダにあります。
同じファイルを[Rustライブラリ](#rustライブラリ)でも読み込めます。
読み込んだファイルに誤りがある場合は、反映されずにエラーが表示されます。

## ピアノロール
縦軸に音高、横軸に時間を取るピアノロールです。
左端の数字はMidiノートナンバーを表します。
上端に表示されている色はCadenceを表し、青がTonic、黄色がSubdominant、赤がDominantを表しています。

ピンク色はキーの高さを表しています。
スケールに含まれる音は白、含まれない音は灰色で表されます。
オレンジ色はコード(和音)に含まれる音を表し、濃い部分はルート(根音)を表しています。
青色はメロディの音を表しています。

## 出力パネル
生成された曲をplayボタンで聞いたり、midiファイルに出力することができます。
midiファイルをDAWやDTMソフトで読み込んで編集することを想定しています。


# Rustライブラリ
生成ロジックはRustのcrate `auto-composer`(`crates/auto-composer`)です。
crates.ioには公開していないので、Git経由で依存してください。

```toml
[dependencies]
auto-composer = { git = "https://github.com/hyranno/auto_compose" }
serde_json = "1"  # パラメータをJSONから読み込む場合
```

Rust 1.85以降(edition 2024)が必要です。
次の例はWebページと同様に曲とその変奏を生成し、メロディを表示します。

```rust
use auto_composer::generator::{tune, variant};
use auto_composer::snapshot::TuneSnapshot;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // パラメータは resources/ と同じJSONの書式です。既定値から始めることもできます。
    let json = std::fs::read_to_string("TuneGenParams_minor.json")?;
    let mut params: tune::Parameters = serde_json::from_str(&json)?;
    params.seed = 42;
    let variant_params = variant::Parameters::default();

    // 曲の後にその変奏を続けます。
    let tune = tune::generate(&params)?;
    let phrase = tune.merge(&variant::generate(&variant_params, &params)?);

    // 時刻と長さは拍、音高はMidiノートナンバーです。
    for note in phrase.notes.list() {
        match note.value.pitch {
            Some(pitch) => println!("{}: {pitch} for {} beats", note.t, note.value.duration),
            None => println!("{}: rest for {} beats", note.t, note.value.duration),
        }
    }

    // 曲全体(cadence、chord、notes)をJSONにすることもできます。
    println!("{}", serde_json::to_string(&TuneSnapshot::from(&phrase))?);
    Ok(())
}
```

主な項目は以下の通りです。

| 項目 | 内容 |
|---|---|
| `generator::tune::Parameters` | 曲のパラメータ(`resources/TuneGenParams_*.json`)。`Default`は`TuneGenParams_default.json`です。 |
| `generator::tune::generate` | パラメータから`Tune`を生成します。 |
| `generator::variant::Parameters` | 変奏のパラメータ(`resources/VariantGenParams_default.json`)。 |
| `generator::variant::generate` | 与えた曲のパラメータによる曲の変奏を生成します。 |
| `tune::Tune` | 生成された曲。スケールと、cadence・chord・notesのタイムラインを持ちます。`merge`で2つの曲をつなげます。 |
| `snapshot::TuneSnapshot` | `Tune`をシリアライズできる単純な形にしたものです。 |
| `error::Error` | 不正なパラメータはpanicせずエラーとして返されます。 |

同じseedとパラメータからは常に同じ曲が生成されます。
`generator`には各層(cadence、chord、rhythm、pitch)の生成器もあり、曲の一部だけを生成・変更することもできます。
詳しくはドキュメント(`cargo doc -p auto-composer --open`)を参照してください。

オプションのfeatureとして、`schemars`(パラメータのJSON Schema)と`tsify`(WebAssemblyバインディング向けのTypeScript型定義)があります。

## WebAssembly
`crates/auto-composer-wasm`はライブラリをJavaScriptから使えるようにします(`generateTune`、`generateVariant`、`compose`など)。
値はJSONと同じ形の素のオブジェクトでやり取りします。
APIとビルド方法は[doc/dev/wasm.md](doc/dev/wasm.md)(英語)を参照してください。


# パラメータ概説
Webページとライブラリに共通する、パラメータの概要を説明します。
名前はWebページの表示に従っています。JSONでのフィールド名はschemasフォルダを参照してください。
このセクションではとりあえず使えることを目標とします。
パラメータの詳細を理解するにはアルゴリズムのセクションを参照してください。

* 共通
    + seed ... 乱数のシード値です。0から4294967295までの整数を入れます。
        これを変えると似た感じの別の曲になります。値自体に意味はありません。
        各Generatorにもseed欄があり、入力するとそのGeneratorだけこの値の代わりに使います。
        ほかの層を保ったまま一つの層だけを変えたいときに使います。
* Scale
    + key ... キー音です。
    + tones ... スケールを、キー音からの相対音高で表します。 Major/minorのボタンで入力できます。
* Resolution ... 4小節がどのように終わるかを設定します。
    曲が続く感じにしたいならHalfやPlagal、終わった感じにしたいならPerfectなどを選ぶとよいでしょう。
* Time Measure ... 拍子です。
* Cadence
    + duration ... コードが切り替わる周期を設定します。4/4拍子なら2か4、3/4拍子なら3が適当です。
    + transitions ... 各Cadenceの次にどのCadenceが来るかの重みです。
* Chord
    + roots ... Cadenceごとの、コードのルートの重みです。スケールの度数で表します。
    + tones ... コードの構成音の重みです。ルートからの度数で表します(`0, 2, 4` で三和音)。
* Rhythm
    + max beat division depth ... 最も短い音の長さを設定します。2^x分音符になります。
    + p_branch edges ... 音をどれだけ分割するかを設定します。
        左の値が小さいと短い音が多く、右の値が小さいと長い音が少なくなります。
    + p_note_on edges ... 音がどれくらい休符になるかを設定します。値が小さいと休符になりやすくなります。
* Pitch
    + abs_pitch_factor edges ... 大きいと、キー音から離れた音が出やすくなります。
    + rel_pitch_factor edges ... 大きいと、直前の音から離れた音が出やすくなります。
    + factor_in_scale ... 大きいと、スケール内の音が出やすくなります。
    + factor_in_chord ... 大きいと、コード内の音が出やすくなります。
    + rhythm_exponent_factor ... 大きいと、音の長さが音高選択に与える影響が大きくなります。
* Variant ... 曲の後に演奏される変奏です。
    + seed, resolution ... 曲のものと同様です。
    + timespan ... 変化させる区間の重みです。拍で表します。
    + layer ... 変化させる層の重みです。


# 構成と開発
* `crates/auto-composer` ... 生成ロジック本体のRustライブラリです。
* `crates/auto-composer-wasm` ... WebAssemblyのバインディングです(wasm-packでビルドします)。
* `web` ... バインディングを使うWebページ(Vite + Solid + TypeScript)です。
* `resources` ... パラメータの既定値と例です。`schemas` はその書式で、Rustの型から生成しています。
* `fixtures` ... 生成結果を固定するテストデータ(golden fixture)です。

開発環境はdevcontainer(`.devcontainer`)にまとめてあります。主なコマンドは以下の通りです。

```sh
cargo test --all-features             # Rustのテスト
cd web && npm run wasm && npm install  # 初回のみ(wasmのパッケージを先に作る)
npm run dev                           # Webページ(http://localhost:5173)
```

詳しくは`doc/dev/`以下(英語)を参照してください。変更履歴は`CHANGELOG.md`にあります。


# アルゴリズム
このセクションでは、プログラムがどのように楽節を生成しているかを説明します。
アルゴリズムやパラメータの詳細な理解を得るには、ソースコードと併せて読んでください。
音楽理論をどのように実装しているかという話にもなりますが、
用語等は誤りを含む可能性があるので注意してください。
マルコフ連鎖など確率過程の知識を要求する箇所があります。

生成される楽節は前半と後半からなり、前半を生成したのち、
それを基に変更を加えて後半(variant)を生成しています。

生成は抽象度の異なる複数の段階からなり、cadence(終止形)、chord(和音)、メロディのrhythm、pitchの順で生成されます。

![Abstraction](./doc/abstraction.svg)

## 前半生成
楽節の前半を生成する部分です。
ソースコードでは`crates/auto-composer/src/generator/tune.rs`がトップレベルです。`generator::tune::Parameters` をJSONから読み込みます。

### Cadence
chordが具体的な音の集合であるのに対し、
chordが持つ機能や性質を表すものを和声機能、
和声機能の連なりをcadence(終止形)と呼びます。
和声機能はTonic/Dominant/Subdominantの3値を取り、
それぞれ次に取りうる値がある程度決まっています。
このプログラムではcadenceを、上記3値を状態とした、斉時的有限状態マルコフ連鎖と考えて生成しています。
ただしcadenceの末尾パターンはresolution(終止)で決定されます。

現在の状態と末尾パターンが与えられたとき、次の各状態の確率は、その状態への遷移確率と、
その状態から残りの歩数で末尾パターンの先頭に到達する確率との積に比例します。
このプログラムでは、この条件付き確率分布を用いて次の状態(Cadence)を決定します。
末尾パターンに到達できない場合は、その条件を外します。

### Chord
chordはroot(根音)とtones(構成音)からなり、cadenceに依存します。
本来なら一部chordはResolutionの影響を受けますが、実装が煩雑になりそうなので無視しています。
構成音の一部についてオクターブを変更する転回についても取り扱っていません。

rootはスケール上でキー音から何度の音高かで表します。
rootはcadenceの和声機能ごとに値がある程度決まるので、和声機能ごとに設定された無記憶情報源から抽選しています。
tonesはスケール上で根音から何度の音高かで表し、例えば(0,2,4)は三和音です。
tonesの理論はよく分からないので無記憶情報源から抽選しています。

### Rhythm
rhythmについては音楽理論が手近で見当たらなかったので独自方式を採用しています。
拍の長さを1として、各拍を再帰的かつ確率的に2分割し、
そこで得られた音の長さそれぞれを確率で休符にします。
分割する確率と休符にしない確率は現在の音の長さに依存し、
それぞれp_branchとp_note_onのエッジによるsmoothstepに従います。
ただし、max beat division depth 以上には分割しません。

### Pitch
rhythmで得られた音の長さそれぞれに対してpitchを設定し、メロディを生成します。休符はpitchを持たず、休符の後の音はその前の音に続きます。
pitchはscale, chord, rhythmに依存するマルコフ連鎖として扱います。
時間信号であるchordとrhythmの影響を受けるので斉時的とは言えないことに注意してください。
本来なら一部pitchはResolutionの影響を受けますが、実装が煩雑になりそうなので無視しています。

pitchは候補の中から、対応付けられた重みに従い抽選され、それぞれの候補の重みは以下で計算されます。

weight = (absFactor * relFactor * scaleFactor * chordFactor) ^ (1 + rhythmExponent)

* absFactor ... キー音からの距離に依存し、abs_pitch_factorのエッジによるsmoothstepで計算されます。
* relFactor ... 直前の音高からの距離に依存し、rel_pitch_factorのエッジによるsmoothstepで計算されます。
* scaleFactor ... スケールに含まれる音ならfactor_in_scale、そうでないなら1です。
* chordFactor ... コードに含まれる音ならfactor_in_chord、そうでないなら1です。
* rhythmExponent ... 短い音では奇抜な音を出しやすく、長い音では無難な音にするための項です。
    rhythmExponent = rhythm_exponent_factor * 音の長さ

## Variant生成
与えられた曲を基にアレンジを加えた曲を生成する部分です。
アレンジを加える時間範囲と、アレンジを加える段階(cadence, chord, pitch, rhythm)を設定しておきます。
ソースコードでは`crates/auto-composer/src/generator/variant.rs`がトップレベルです。`generator::variant::Parameters` をJSONから読み込みます。

### Cadence で変更を加える
アレンジを加える範囲について、その前後の状態を考慮した条件付き確率分布に従うマルコフ連鎖で再生成します。
Cadenceに依存するChord, Pitchも再生成されます。

### Chord で変更を加える
アレンジを加える範囲について、新たな乱数でchordを再生成します。
Chordに依存するPitchも再生成されます。

### Pitch で変更を加える
アレンジを加える範囲について、
その前後の音高を考慮した条件付き確率分布に従うマルコフ連鎖で再生成します。
ただし前述のようにrhythmとchordへの依存により斉時的ではないので、
そのまま条件付き確率分布を求めるのは厳しいです。
よって、rhythmとchordの影響を排した斉時的有限状態マルコフ連鎖によって条件付き確率分布を求めたのち、
直後の分布にのみrhythmとchordの影響を後付けして計算しています。

### Rhythm で変更を加える
アレンジを加える範囲について、
音の長さを確率的に分割・結合、休符か否かを再抽選し、変更があった箇所(休符から音になった箇所を含む)のpitchを再生成します。
