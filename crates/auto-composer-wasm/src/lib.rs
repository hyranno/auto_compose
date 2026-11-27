//! WebAssembly bindings of [`auto_composer`].
//!
//! The values crossing the boundary are plain JavaScript objects of the same shape as the JSON
//! of the parameters (`resources/*.json`) and of the golden fixtures ([`TuneSnapshot`]).
//! Their TypeScript declarations are generated from the Rust types by `tsify`.
//! Invalid parameters are reported as a thrown `Error`.

use auto_composer::generator::{tune as tune_generator, variant};
use auto_composer::snapshot::TuneSnapshot;
use auto_composer::tune::Tune;
use serde::Serialize;
use serde::de::DeserializeOwned;
use wasm_bindgen::prelude::*;

/// The default parameters of a tune (`resources/TuneGenParams_default.json`).
#[wasm_bindgen(js_name = defaultTuneParameters, unchecked_return_type = "TuneParameters")]
pub fn default_tune_parameters() -> Result<JsValue, JsError> {
    to_js(&tune_generator::Parameters::default())
}

/// The default parameters of a variant (`resources/VariantGenParams_default.json`).
#[wasm_bindgen(js_name = defaultVariantParameters, unchecked_return_type = "VariantParameters")]
pub fn default_variant_parameters() -> Result<JsValue, JsError> {
    to_js(&variant::Parameters::default())
}

/// Generates a tune.
#[wasm_bindgen(js_name = generateTune, unchecked_return_type = "TuneSnapshot")]
pub fn generate_tune(
    #[wasm_bindgen(unchecked_param_type = "TuneParameters")] params: JsValue,
) -> Result<JsValue, JsError> {
    to_js(&TuneSnapshot::from(&tune(params)?))
}

/// Generates the variant of the tune of `base`.
#[wasm_bindgen(js_name = generateVariant, unchecked_return_type = "TuneSnapshot")]
pub fn generate_variant(
    #[wasm_bindgen(unchecked_param_type = "VariantParameters")] params: JsValue,
    #[wasm_bindgen(unchecked_param_type = "TuneParameters")] base: JsValue,
) -> Result<JsValue, JsError> {
    to_js(&TuneSnapshot::from(&variant_of(params, base)?))
}

/// The tune of `params` followed by its variant, as the example page plays it.
#[wasm_bindgen(unchecked_return_type = "TuneSnapshot")]
pub fn compose(
    #[wasm_bindgen(unchecked_param_type = "TuneParameters")] params: JsValue,
    #[wasm_bindgen(unchecked_param_type = "VariantParameters")] variant: JsValue,
) -> Result<JsValue, JsError> {
    let base = tune(params.clone())?;
    let variant = variant_of(variant, params)?;
    to_js(&TuneSnapshot::from(&base.merge(&variant)))
}

fn tune(params: JsValue) -> Result<Tune, JsError> {
    let params: tune_generator::Parameters = from_js(params, "tune parameters")?;
    Ok(tune_generator::generate(&params)?)
}

fn variant_of(params: JsValue, base: JsValue) -> Result<Tune, JsError> {
    let params: variant::Parameters = from_js(params, "variant parameters")?;
    let base: tune_generator::Parameters = from_js(base, "tune parameters")?;
    Ok(variant::generate(&params, &base)?)
}

fn from_js<T: DeserializeOwned>(value: JsValue, what: &str) -> Result<T, JsError> {
    serde_wasm_bindgen::from_value(value).map_err(|e| JsError::new(&format!("invalid {what}: {e}")))
}

fn to_js<T: Serialize>(value: &T) -> Result<JsValue, JsError> {
    // Plain objects rather than `Map`s, so that the values can be passed to `JSON.stringify`.
    let serializer = serde_wasm_bindgen::Serializer::json_compatible();
    Ok(value.serialize(&serializer)?)
}
