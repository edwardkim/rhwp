//! Opt-in object and Text IR v2 exports. Schema-v1 JSON remains the canonical
//! serialization path; these adapters do not maintain a second paint writer.

use std::collections::HashSet;

use js_sys::{Array, Reflect, Uint8Array};
use wasm_bindgen::prelude::*;

use super::HwpDocument;
use crate::paint::{LayerJsonOptions, PageLayerTree, RenderProfile};

#[derive(Clone, Copy)]
pub(crate) enum TextExport {
    V1,
    Compat,
    StrictOutline,
    StrictRun,
}

fn serialize(tree: &PageLayerTree, format: TextExport) -> Result<String, JsValue> {
    let result = match format {
        TextExport::V1 => return Ok(tree.to_json()),
        TextExport::Compat => tree.to_json_v2_compat(),
        TextExport::StrictOutline => tree.to_json_v2_strict_glyph_outline(),
        TextExport::StrictRun => tree.to_json_v2_strict_glyph_run(),
    };
    result.map_err(|issues| {
        serde_json::to_string(&issues)
            .ok()
            .and_then(|json| js_sys::JSON::parse(&json).ok())
            .unwrap_or_else(|| JsValue::from_str("invalid Text IR v2 export"))
    })
}

fn resource_hints(value: &JsValue) -> Result<HashSet<String>, JsValue> {
    if value.is_null() || value.is_undefined() {
        return Ok(HashSet::new());
    }
    if !Array::is_array(value) {
        return Err(JsValue::from_str("resource keys must be an array"));
    }
    let values = value.unchecked_ref::<Array>();
    if values.length() > 4096 {
        return Err(JsValue::from_str("resource key hint limit exceeded"));
    }
    let mut hints = HashSet::new();
    for key in values.iter() {
        let key = key
            .as_string()
            .ok_or_else(|| JsValue::from_str("resource keys must be strings"))?;
        if key.len() > 1024 {
            return Err(JsValue::from_str("resource key hint is too long"));
        }
        hints.insert(key);
    }
    Ok(hints)
}

fn object_value(
    tree: &PageLayerTree,
    json: &str,
    known_images: &HashSet<String>,
    known_svgs: &HashSet<String>,
    omit_font_bytes: bool,
) -> Result<JsValue, JsValue> {
    // Reuse every current source/visual/resource field from the canonical writer.
    // This is intentionally not a claim of the old hand-written JS writer's speed.
    let value = js_sys::JSON::parse(json)?;
    let resources = Reflect::get(&value, &JsValue::from_str("resources"))?;
    let images = Array::new_with_length(tree.resources.image_count() as u32);
    for (id, bytes) in tree.resources.image_resources() {
        if !tree
            .resources
            .image_resource_key(id)
            .is_some_and(|key| known_images.contains(key))
        {
            images.set(id.0 as u32, Uint8Array::from(bytes).into());
        }
    }
    let svgs = Array::new_with_length(tree.resources.svg_count() as u32);
    for (id, fragment) in tree.resources.svg_resources() {
        if !tree
            .resources
            .svg_resource_key(id)
            .is_some_and(|key| known_svgs.contains(key))
        {
            svgs.set(id.0 as u32, JsValue::from_str(fragment));
        }
    }
    let fonts = Array::new();
    if !omit_font_bytes {
        for (id, bytes) in tree.resources.font_blob_resources() {
            fonts.set(id.0 as u32, Uint8Array::from(bytes).into());
        }
    }
    Reflect::set(&resources, &JsValue::from_str("images"), &images)?;
    Reflect::set(&resources, &JsValue::from_str("svgFragments"), &svgs)?;
    Reflect::set(&resources, &JsValue::from_str("fontBlobs"), &fonts)?;
    Ok(value)
}

impl HwpDocument {
    fn export_text_tree(&self, page: u32, profile: &str) -> Result<PageLayerTree, JsValue> {
        let profile = RenderProfile::parse(profile)
            .ok_or_else(|| JsValue::from_str(&format!("unsupported render profile: {profile}")))?;
        self.build_page_layer_tree_with_profile(page, profile)
            .map_err(JsValue::from)
    }

    fn export_text_json(
        &self,
        page: u32,
        profile: &str,
        format: TextExport,
    ) -> Result<String, JsValue> {
        super::render_patch_boundary::export_text_json(self, page, profile, format)
    }

    fn export_text_value(
        &self,
        page: u32,
        profile: &str,
        format: TextExport,
        known_images: JsValue,
        known_svgs: JsValue,
    ) -> Result<JsValue, JsValue> {
        super::render_patch_boundary::export_text_value(
            self,
            page,
            profile,
            format,
            known_images,
            known_svgs,
            LayerJsonOptions::default(),
        )
    }
}

// Build, serialize and adapt resources in one hot call: otherwise a cold builder
// can populate caches with old paint before a hot JSON writer reads them.
pub(crate) fn export_text_json_impl(
    document: &HwpDocument,
    page: u32,
    profile: &str,
    format: TextExport,
) -> Result<String, JsValue> {
    serialize(&document.export_text_tree(page, profile)?, format)
}

pub(crate) fn export_text_value_impl(
    document: &HwpDocument,
    page: u32,
    profile: &str,
    format: TextExport,
    known_images: JsValue,
    known_svgs: JsValue,
    options: LayerJsonOptions,
) -> Result<JsValue, JsValue> {
    let known_images = resource_hints(&known_images)?;
    let known_svgs = resource_hints(&known_svgs)?;
    let tree = document.export_text_tree(page, profile)?;
    let json = match format {
        TextExport::V1 => super::get_page_layer_tree_with_profile_impl(
            document,
            page,
            profile,
            options.omit_image_bytes,
            options.omit_font_bytes,
        )?,
        _ => serialize(&tree, format)?,
    };
    object_value(
        &tree,
        &json,
        &known_images,
        &known_svgs,
        options.omit_font_bytes,
    )
}

#[wasm_bindgen]
impl HwpDocument {
    /// Opt-in schema-v2 export; the existing schema-v1 API is unchanged.
    #[wasm_bindgen(js_name = getPageLayerTreeV2Compat)]
    pub fn get_page_layer_tree_v2_compat(&self, page: u32) -> Result<String, JsValue> {
        self.get_page_layer_tree_v2_compat_with_profile(page, "screen")
    }

    #[wasm_bindgen(js_name = getPageLayerTreeV2CompatWithProfile)]
    pub fn get_page_layer_tree_v2_compat_with_profile(
        &self,
        page: u32,
        profile: &str,
    ) -> Result<String, JsValue> {
        self.export_text_json(page, profile, TextExport::Compat)
    }

    /// Opt-in schema-v2 export; the existing schema-v1 API is unchanged.
    #[wasm_bindgen(js_name = getPageLayerTreeV2StrictGlyphOutline)]
    pub fn get_page_layer_tree_v2_strict_glyph_outline(
        &self,
        page: u32,
    ) -> Result<String, JsValue> {
        self.get_page_layer_tree_v2_strict_glyph_outline_with_profile(page, "screen")
    }

    #[wasm_bindgen(js_name = getPageLayerTreeV2StrictGlyphOutlineWithProfile)]
    pub fn get_page_layer_tree_v2_strict_glyph_outline_with_profile(
        &self,
        page: u32,
        profile: &str,
    ) -> Result<String, JsValue> {
        self.export_text_json(page, profile, TextExport::StrictOutline)
    }

    /// Opt-in schema-v2 export; the existing schema-v1 API is unchanged.
    #[wasm_bindgen(js_name = getPageLayerTreeV2StrictGlyphRun)]
    pub fn get_page_layer_tree_v2_strict_glyph_run(&self, page: u32) -> Result<String, JsValue> {
        self.get_page_layer_tree_v2_strict_glyph_run_with_profile(page, "screen")
    }

    #[wasm_bindgen(js_name = getPageLayerTreeV2StrictGlyphRunWithProfile)]
    pub fn get_page_layer_tree_v2_strict_glyph_run_with_profile(
        &self,
        page: u32,
        profile: &str,
    ) -> Result<String, JsValue> {
        self.export_text_json(page, profile, TextExport::StrictRun)
    }

    /// Object adapter with owned binary resources. No WASM-memory views escape.
    #[wasm_bindgen(js_name = getPageLayerTreeValue)]
    pub fn get_page_layer_tree_value(&self, page: u32) -> Result<JsValue, JsValue> {
        self.get_page_layer_tree_value_with_profile(page, "screen")
    }

    #[wasm_bindgen(js_name = getPageLayerTreeValueWithProfile)]
    pub fn get_page_layer_tree_value_with_profile(
        &self,
        page: u32,
        profile: &str,
    ) -> Result<JsValue, JsValue> {
        self.export_text_value(
            page,
            profile,
            TextExport::V1,
            JsValue::UNDEFINED,
            JsValue::UNDEFINED,
        )
    }

    /// Only content-addressed keys supplied by the caller omit an array slot.
    /// Optional byte-omission flags preserve the current source-image/font key
    /// APIs; omitted flags retain the original four-argument inline contract.
    #[wasm_bindgen(js_name = getPageLayerTreeValueWithProfileAndResourceKeys)]
    pub fn get_page_layer_tree_value_with_profile_and_resource_keys(
        &self,
        page: u32,
        profile: &str,
        known_image_keys: JsValue,
        known_svg_keys: JsValue,
        omit_font_bytes: Option<bool>,
        omit_image_bytes: Option<bool>,
    ) -> Result<JsValue, JsValue> {
        super::render_patch_boundary::export_text_value(
            self,
            page,
            profile,
            TextExport::V1,
            known_image_keys,
            known_svg_keys,
            LayerJsonOptions {
                omit_font_bytes: omit_font_bytes.unwrap_or(false),
                omit_image_bytes: omit_image_bytes.unwrap_or(false),
            },
        )
    }

    /// Object adapter with owned binary resources. No WASM-memory views escape.
    #[wasm_bindgen(js_name = getPageLayerTreeValueV2Compat)]
    pub fn get_page_layer_tree_value_v2_compat(&self, page: u32) -> Result<JsValue, JsValue> {
        self.get_page_layer_tree_value_v2_compat_with_profile(page, "screen")
    }

    #[wasm_bindgen(js_name = getPageLayerTreeValueV2CompatWithProfile)]
    pub fn get_page_layer_tree_value_v2_compat_with_profile(
        &self,
        page: u32,
        profile: &str,
    ) -> Result<JsValue, JsValue> {
        self.export_text_value(
            page,
            profile,
            TextExport::Compat,
            JsValue::UNDEFINED,
            JsValue::UNDEFINED,
        )
    }

    /// Only content-addressed keys supplied by the caller omit an array slot.
    #[wasm_bindgen(js_name = getPageLayerTreeValueV2CompatWithProfileAndResourceKeys)]
    pub fn get_page_layer_tree_value_v2_compat_with_profile_and_resource_keys(
        &self,
        page: u32,
        profile: &str,
        known_image_keys: JsValue,
        known_svg_keys: JsValue,
    ) -> Result<JsValue, JsValue> {
        self.export_text_value(
            page,
            profile,
            TextExport::Compat,
            known_image_keys,
            known_svg_keys,
        )
    }

    /// Object adapter with owned binary resources. No WASM-memory views escape.
    #[wasm_bindgen(js_name = getPageLayerTreeValueV2StrictGlyphOutline)]
    pub fn get_page_layer_tree_value_v2_strict_glyph_outline(
        &self,
        page: u32,
    ) -> Result<JsValue, JsValue> {
        self.get_page_layer_tree_value_v2_strict_glyph_outline_with_profile(page, "screen")
    }

    #[wasm_bindgen(js_name = getPageLayerTreeValueV2StrictGlyphOutlineWithProfile)]
    pub fn get_page_layer_tree_value_v2_strict_glyph_outline_with_profile(
        &self,
        page: u32,
        profile: &str,
    ) -> Result<JsValue, JsValue> {
        self.export_text_value(
            page,
            profile,
            TextExport::StrictOutline,
            JsValue::UNDEFINED,
            JsValue::UNDEFINED,
        )
    }

    /// Only content-addressed keys supplied by the caller omit an array slot.
    #[wasm_bindgen(js_name = getPageLayerTreeValueV2StrictGlyphOutlineWithProfileAndResourceKeys)]
    pub fn get_page_layer_tree_value_v2_strict_glyph_outline_with_profile_and_resource_keys(
        &self,
        page: u32,
        profile: &str,
        known_image_keys: JsValue,
        known_svg_keys: JsValue,
    ) -> Result<JsValue, JsValue> {
        self.export_text_value(
            page,
            profile,
            TextExport::StrictOutline,
            known_image_keys,
            known_svg_keys,
        )
    }

    /// Object adapter with owned binary resources. No WASM-memory views escape.
    #[wasm_bindgen(js_name = getPageLayerTreeValueV2StrictGlyphRun)]
    pub fn get_page_layer_tree_value_v2_strict_glyph_run(
        &self,
        page: u32,
    ) -> Result<JsValue, JsValue> {
        self.get_page_layer_tree_value_v2_strict_glyph_run_with_profile(page, "screen")
    }

    #[wasm_bindgen(js_name = getPageLayerTreeValueV2StrictGlyphRunWithProfile)]
    pub fn get_page_layer_tree_value_v2_strict_glyph_run_with_profile(
        &self,
        page: u32,
        profile: &str,
    ) -> Result<JsValue, JsValue> {
        self.export_text_value(
            page,
            profile,
            TextExport::StrictRun,
            JsValue::UNDEFINED,
            JsValue::UNDEFINED,
        )
    }

    /// Only content-addressed keys supplied by the caller omit an array slot.
    #[wasm_bindgen(js_name = getPageLayerTreeValueV2StrictGlyphRunWithProfileAndResourceKeys)]
    pub fn get_page_layer_tree_value_v2_strict_glyph_run_with_profile_and_resource_keys(
        &self,
        page: u32,
        profile: &str,
        known_image_keys: JsValue,
        known_svg_keys: JsValue,
    ) -> Result<JsValue, JsValue> {
        self.export_text_value(
            page,
            profile,
            TextExport::StrictRun,
            known_image_keys,
            known_svg_keys,
        )
    }
}
