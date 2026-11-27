//! Golden cases through the JavaScript boundary: `wasm-pack test --node crates/auto-composer-wasm`.
//!
//! The values are converted from JSON to JavaScript objects with `JSON.parse`, as the page does
//! when it loads the parameters, so that the numbers arrive as JavaScript numbers.

#![cfg(target_arch = "wasm32")]

use auto_composer::snapshot::TuneSnapshot;
use auto_composer_wasm::{
    compose, default_tune_parameters, default_variant_parameters, generate_tune, generate_variant,
};
use serde_json::Value;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::wasm_bindgen_test;

/// The seed of this case is the largest one.
const TUNE_CASE: &str = include_str!("../../../fixtures/golden/tune/default_seed-large.json");
const VARIANT_CASE: &str =
    include_str!("../../../fixtures/golden/variant/default_layer-rhythm_span-6-11.json");

fn case(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

fn to_js(v: &Value) -> JsValue {
    js_sys::JSON::parse(&v.to_string()).unwrap()
}

fn snapshot(v: JsValue) -> TuneSnapshot {
    serde_wasm_bindgen::from_value(v).unwrap()
}

fn expected(case: &Value) -> TuneSnapshot {
    serde_json::from_value(case["expected"].clone()).unwrap()
}

#[wasm_bindgen_test]
fn generates_a_golden_tune() {
    let case = case(TUNE_CASE);
    let actual = generate_tune(to_js(&case["params"])).unwrap();
    assert_eq!(snapshot(actual), expected(&case));
}

#[wasm_bindgen_test]
fn generates_a_golden_variant() {
    let case = case(VARIANT_CASE);
    let actual = generate_variant(to_js(&case["params"]), to_js(&case["base"])).unwrap();
    assert_eq!(snapshot(actual), expected(&case));
}

#[wasm_bindgen_test]
fn composes_the_tune_and_its_variant() {
    let case = case(VARIANT_CASE);
    let (params, base) = (to_js(&case["params"]), to_js(&case["base"]));
    let tune = snapshot(generate_tune(base.clone()).unwrap());
    let variant = expected(&case);
    let composed = snapshot(compose(base, params).unwrap());
    assert_eq!(composed.length, tune.length + variant.length);
    assert_eq!(composed.notes.len(), tune.notes.len() + variant.notes.len());
    assert_eq!(
        composed.notes[tune.notes.len()].t,
        variant.notes[0].t + tune.length
    );
}

#[wasm_bindgen_test]
fn the_defaults_are_the_resource_files() {
    let tune: Value = serde_wasm_bindgen::from_value(default_tune_parameters().unwrap()).unwrap();
    let variant: Value =
        serde_wasm_bindgen::from_value(default_variant_parameters().unwrap()).unwrap();
    // The defaults are plain objects that survive `JSON.stringify`.
    let text = js_sys::JSON::stringify(&default_tune_parameters().unwrap()).unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(&String::from(text)).unwrap(),
        tune
    );
    assert_eq!(
        tune["time_measure"],
        serde_json::json!({"beats_per_bar": 4, "bars": 4})
    );
    assert_eq!(variant["layer"][0]["value"], "rhythm");
    // They generate without errors.
    let tune = generate_tune(default_tune_parameters().unwrap());
    assert!(tune.is_ok());
}

#[wasm_bindgen_test]
fn invalid_parameters_throw() {
    let mut params = case(TUNE_CASE)["params"].clone();
    params["cadence"]["duration"] = 0.into();
    assert!(generate_tune(to_js(&params)).is_err());
    params["resolution"] = "Half".into();
    assert!(generate_tune(to_js(&params)).is_err());
    assert!(generate_tune(JsValue::NULL).is_err());
}
