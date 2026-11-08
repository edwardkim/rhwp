//! PDF Layers (Optional Content Groups) support.
//!
//! This module provides functionality to create and manage PDF layers,
//! also known as Optional Content Groups (OCGs) per PDF specification.
//!
//! ## Overview
//!
//! PDF layers allow content to be selectively shown or hidden in viewers.
//! Common uses include:
//! - CAD drawings with multiple views
//! - Multi-language documents
//! - Print vs. screen versions
//! - Watermarks that can be toggled
//!
//! ## PDF Structure
//!
//! Layers are implemented using:
//! - `/OCProperties` dictionary in the catalog
//! - `/OCGs` array listing all Optional Content Groups
//! - `/D` (default) dictionary with visibility settings
//! - `/OC` key in content streams/XObjects to associate with layers
//!
//! ## Example
//!
//! ```ignore
//! use pdf_oxide::writer::layers::{LayerBuilder, LayerVisibility};
//!
//! // Create layers
//! let mut layer_builder = LayerBuilder::new();
//! let watermark_layer = layer_builder.add_layer("Watermark")
//!     .visible_on_screen(true)
//!     .printable(false);
//! let annotations_layer = layer_builder.add_layer("Annotations")
//!     .visible(true);
//!
//! // Build layer configuration
//! let layers = layer_builder.build();
//! ```
//!
//! ## Standards Reference
//!
//! - PDF Reference 1.7: Section 4.10 "Optional Content"
//! - ISO 32000-1:2008: Section 8.11 "Optional Content"

use crate::object::{Object, ObjectRef};
use std::collections::HashMap;

/// Represents a PDF layer (Optional Content Group).
#[derive(Debug, Clone)]
pub struct Layer {
    /// Unique name for the layer.
    pub name: String,
    /// Display name shown in PDF viewer UI.
    pub display_name: Option<String>,
    /// Whether the layer is initially visible.
    pub visible: bool,
    /// Whether the layer is visible on screen.
    pub visible_on_screen: bool,
    /// Whether the layer is visible when printing.
    pub visible_on_print: bool,
    /// Whether the layer is exportable.
    pub exportable: bool,
    /// Intent of the layer (View, Design, or both).
    pub intent: LayerIntent,
    /// Layer order within groups (lower = shown first).
    pub order: Option<u32>,
}

impl Default for Layer {
    fn default() -> Self {
        Self {
            name: String::new(),
            display_name: None,
            visible: true,
            visible_on_screen: true,
            visible_on_print: true,
            exportable: true,
            intent: LayerIntent::View,
            order: None,
        }
    }
}

impl Layer {
    /// Create a new layer with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            name: name.clone(),
            display_name: Some(name),
            ..Default::default()
        }
    }

    /// Set the display name.
    pub fn display_name(&mut self, name: impl Into<String>) -> &mut Self {
        self.display_name = Some(name.into());
        self
    }

    /// Set initial visibility.
    pub fn visible(&mut self, visible: bool) -> &mut Self {
        self.visible = visible;
        self
    }

    /// Set screen visibility.
    pub fn visible_on_screen(&mut self, visible: bool) -> &mut Self {
        self.visible_on_screen = visible;
        self
    }

    /// Set print visibility.
    pub fn visible_on_print(&mut self, visible: bool) -> &mut Self {
        self.visible_on_print = visible;
        self
    }

    /// Set whether exportable.
    pub fn exportable(&mut self, exportable: bool) -> &mut Self {
        self.exportable = exportable;
        self
    }

    /// Set the layer intent.
    pub fn intent(&mut self, intent: LayerIntent) -> &mut Self {
        self.intent = intent;
        self
    }

    /// Set the layer order.
    pub fn order(&mut self, order: u32) -> &mut Self {
        self.order = Some(order);
        self
    }

    /// Build the OCG dictionary.
    pub fn build_ocg_dict(&self) -> HashMap<String, Object> {
        let mut dict = HashMap::new();
        dict.insert("Type".to_string(), Object::Name("OCG".to_string()));
        dict.insert(
            "Name".to_string(),
            Object::String(
                self.display_name
                    .as_ref()
                    .unwrap_or(&self.name)
                    .as_bytes()
                    .to_vec(),
            ),
        );

        // Intent
        match self.intent {
            LayerIntent::View => {
                dict.insert("Intent".to_string(), Object::Name("View".to_string()));
            },
            LayerIntent::Design => {
                dict.insert("Intent".to_string(), Object::Name("Design".to_string()));
            },
            LayerIntent::Both => {
                dict.insert(
                    "Intent".to_string(),
                    Object::Array(vec![
                        Object::Name("View".to_string()),
                        Object::Name("Design".to_string()),
                    ]),
                );
            },
        }

        dict
    }

    /// Build the usage dictionary for this layer.
    pub fn build_usage_dict(&self) -> Option<HashMap<String, Object>> {
        // Only create usage dict if we have non-default settings
        if self.visible_on_screen && self.visible_on_print && self.exportable {
            return None;
        }

        let mut usage = HashMap::new();

        // Print usage
        if !self.visible_on_print {
            let mut print_dict = HashMap::new();
            print_dict.insert("PrintState".to_string(), Object::Name("OFF".to_string()));
            usage.insert("Print".to_string(), Object::Dictionary(print_dict));
        }

        // View usage
        if !self.visible_on_screen {
            let mut view_dict = HashMap::new();
            view_dict.insert("ViewState".to_string(), Object::Name("OFF".to_string()));
            usage.insert("View".to_string(), Object::Dictionary(view_dict));
        }

        // Export usage
        if !self.exportable {
            let mut export_dict = HashMap::new();
            export_dict.insert("ExportState".to_string(), Object::Name("OFF".to_string()));
            usage.insert("Export".to_string(), Object::Dictionary(export_dict));
        }

        if usage.is_empty() {
            None
        } else {
            Some(usage)
        }
    }
}

/// Layer intent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LayerIntent {
    /// Layer is for viewing purposes.
    #[default]
    View,
    /// Layer is for design purposes.
    Design,
    /// Layer is for both viewing and design.
    Both,
}

/// Layer visibility state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LayerVisibility {
    /// Layer is visible by default.
    #[default]
    On,
    /// Layer is hidden by default.
    Off,
}

/// Builder for creating PDF layer configurations.
#[derive(Debug, Default)]
pub struct LayerBuilder {
    layers: Vec<Layer>,
    base_state: LayerVisibility,
    creator: Option<String>,
    application_name: Option<String>,
}

impl LayerBuilder {
    /// Create a new layer builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new layer.
    pub fn add_layer(&mut self, name: impl Into<String>) -> &mut Layer {
        let layer = Layer::new(name);
        self.layers.push(layer);
        self.layers.last_mut().unwrap()
    }

    /// Set the base visibility state for layers.
    pub fn base_state(mut self, state: LayerVisibility) -> Self {
        self.base_state = state;
        self
    }

    /// Set the creator application name.
    pub fn creator(mut self, creator: impl Into<String>) -> Self {
        self.creator = Some(creator.into());
        self
    }

    /// Set the application name.
    pub fn application_name(mut self, name: impl Into<String>) -> Self {
        self.application_name = Some(name.into());
        self
    }

    /// Get the layers.
    pub fn layers(&self) -> &[Layer] {
        &self.layers
    }

    /// Get mutable access to layers.
    pub fn layers_mut(&mut self) -> &mut Vec<Layer> {
        &mut self.layers
    }

    /// Check if there are any layers.
    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }

    /// Get the number of layers.
    pub fn len(&self) -> usize {
        self.layers.len()
    }

    /// Build the OCProperties dictionary.
    ///
    /// This returns the dictionary that should be added to the catalog.
    pub fn build_oc_properties(&self, ocg_refs: &[ObjectRef]) -> HashMap<String, Object> {
        let mut props = HashMap::new();

        // OCGs array - list of all Optional Content Groups
        let ocgs_array: Vec<Object> = ocg_refs.iter().map(|r| Object::Reference(*r)).collect();
        props.insert("OCGs".to_string(), Object::Array(ocgs_array.clone()));

        // D (default) configuration dictionary
        let mut d_dict = HashMap::new();

        // Name of the configuration
        d_dict.insert("Name".to_string(), Object::text_string("Default"));

        // Creator (application that created the layers)
        if let Some(ref creator) = self.creator {
            d_dict.insert("Creator".to_string(), Object::text_string(creator));
        }

        // Base state
        d_dict.insert(
            "BaseState".to_string(),
            Object::Name(
                match self.base_state {
                    LayerVisibility::On => "ON",
                    LayerVisibility::Off => "OFF",
                }
                .to_string(),
            ),
        );

        // ON array - layers that are visible by default (when BaseState is OFF)
        // OFF array - layers that are hidden by default (when BaseState is ON)
        let mut on_refs: Vec<Object> = Vec::new();
        let mut off_refs: Vec<Object> = Vec::new();

        for (i, layer) in self.layers.iter().enumerate() {
            if i < ocg_refs.len() {
                if layer.visible {
                    on_refs.push(Object::Reference(ocg_refs[i]));
                } else {
                    off_refs.push(Object::Reference(ocg_refs[i]));
                }
            }
        }

        if !on_refs.is_empty() {
            d_dict.insert("ON".to_string(), Object::Array(on_refs));
        }
        if !off_refs.is_empty() {
            d_dict.insert("OFF".to_string(), Object::Array(off_refs));
        }

        // Order array - defines display order in viewer UI
        d_dict.insert("Order".to_string(), Object::Array(ocgs_array));

        props.insert("D".to_string(), Object::Dictionary(d_dict));

        props
    }
}

/// Configuration for layer membership.
#[derive(Debug, Clone)]
pub struct LayerMembership {
    /// The OCGs this content belongs to.
    pub ocgs: Vec<ObjectRef>,
    /// Visibility policy.
    pub policy: VisibilityPolicy,
}

/// Visibility policy for layer membership.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VisibilityPolicy {
    /// Content is visible if ALL OCGs are visible.
    #[default]
    AllOn,
    /// Content is visible if ANY OCG is visible.
    AnyOn,
    /// Content is visible if ALL OCGs are hidden.
    AllOff,
    /// Content is visible if ANY OCG is hidden.
    AnyOff,
}

impl LayerMembership {
    /// Create a new layer membership with a single OCG.
    pub fn new(ocg_ref: ObjectRef) -> Self {
        Self {
            ocgs: vec![ocg_ref],
            policy: VisibilityPolicy::AllOn,
        }
    }

    /// Add an OCG to the membership.
    pub fn add_ocg(&mut self, ocg_ref: ObjectRef) -> &mut Self {
        self.ocgs.push(ocg_ref);
        self
    }

    /// Set the visibility policy.
    pub fn policy(mut self, policy: VisibilityPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// Build the OCMD dictionary.
    pub fn build_ocmd_dict(&self) -> HashMap<String, Object> {
        let mut dict = HashMap::new();
        dict.insert("Type".to_string(), Object::Name("OCMD".to_string()));

        if self.ocgs.len() == 1 {
            dict.insert("OCGs".to_string(), Object::Reference(self.ocgs[0]));
        } else {
            dict.insert(
                "OCGs".to_string(),
                Object::Array(self.ocgs.iter().map(|r| Object::Reference(*r)).collect()),
            );
        }

        // Policy (P key)
        let policy_name = match self.policy {
            VisibilityPolicy::AllOn => "AllOn",
            VisibilityPolicy::AnyOn => "AnyOn",
            VisibilityPolicy::AllOff => "AllOff",
            VisibilityPolicy::AnyOff => "AnyOff",
        };
        dict.insert("P".to_string(), Object::Name(policy_name.to_string()));

        dict
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_creation() {
        let mut layer = Layer::new("Background");
        layer
            .display_name("Background Layer")
            .visible(true)
            .visible_on_print(false);

        assert_eq!(layer.name, "Background");
        assert_eq!(layer.display_name, Some("Background Layer".to_string()));
        assert!(layer.visible);
        assert!(!layer.visible_on_print);
    }

    #[test]
    fn test_layer_builder() {
        let mut builder = LayerBuilder::new();
        builder.add_layer("Layer1").visible(true);
        builder.add_layer("Layer2").visible(false);

        assert_eq!(builder.len(), 2);
        assert!(!builder.is_empty());
        assert!(builder.layers()[0].visible);
        assert!(!builder.layers()[1].visible);
    }

    #[test]
    fn test_layer_builder_base_state() {
        let builder = LayerBuilder::new()
            .base_state(LayerVisibility::Off)
            .creator("pdf_oxide");

        assert_eq!(builder.base_state, LayerVisibility::Off);
        assert_eq!(builder.creator, Some("pdf_oxide".to_string()));
    }

    #[test]
    fn test_layer_ocg_dict() {
        let mut layer = Layer::new("Test");
        layer.intent(LayerIntent::Both);

        let dict = layer.build_ocg_dict();
        assert!(dict.contains_key("Type"));
        assert!(dict.contains_key("Name"));
        assert!(dict.contains_key("Intent"));

        if let Some(Object::Name(type_name)) = dict.get("Type") {
            assert_eq!(type_name, "OCG");
        } else {
            panic!("Type should be a Name");
        }
    }

    #[test]
    fn test_layer_usage_dict() {
        // Default settings - no usage dict needed
        let layer_default = Layer::new("Default");
        assert!(layer_default.build_usage_dict().is_none());

        // Non-printable layer - should have usage dict
        let mut layer_no_print = Layer::new("NoPrint");
        layer_no_print.visible_on_print(false);
        let usage = layer_no_print.build_usage_dict();
        assert!(usage.is_some());
        assert!(usage.unwrap().contains_key("Print"));
    }

    #[test]
    fn test_layer_membership() {
        let ocg_ref = ObjectRef::new(1, 0);
        let membership = LayerMembership::new(ocg_ref).policy(VisibilityPolicy::AnyOn);

        assert_eq!(membership.ocgs.len(), 1);
        assert_eq!(membership.policy, VisibilityPolicy::AnyOn);
    }

    #[test]
    fn test_layer_membership_dict() {
        let ocg_ref = ObjectRef::new(1, 0);
        let membership = LayerMembership::new(ocg_ref);
        let dict = membership.build_ocmd_dict();

        assert!(dict.contains_key("Type"));
        assert!(dict.contains_key("OCGs"));
        assert!(dict.contains_key("P"));

        if let Some(Object::Name(type_name)) = dict.get("Type") {
            assert_eq!(type_name, "OCMD");
        } else {
            panic!("Type should be a Name");
        }
    }

    #[test]
    fn test_layer_intent() {
        assert_eq!(LayerIntent::default(), LayerIntent::View);

        let mut layer = Layer::new("Design");
        layer.intent(LayerIntent::Design);
        assert_eq!(layer.intent, LayerIntent::Design);
    }

    #[test]
    fn test_visibility_policy() {
        assert_eq!(VisibilityPolicy::default(), VisibilityPolicy::AllOn);
    }

    // ---- Tests for Layer defaults ----

    #[test]
    fn test_layer_default() {
        let layer = Layer::default();
        assert_eq!(layer.name, "");
        assert!(layer.display_name.is_none());
        assert!(layer.visible);
        assert!(layer.visible_on_screen);
        assert!(layer.visible_on_print);
        assert!(layer.exportable);
        assert_eq!(layer.intent, LayerIntent::View);
        assert!(layer.order.is_none());
    }

    #[test]
    fn test_layer_new_sets_name_and_display_name() {
        let layer = Layer::new("MyLayer");
        assert_eq!(layer.name, "MyLayer");
        assert_eq!(layer.display_name, Some("MyLayer".to_string()));
    }

    // ---- Tests for Layer setter methods ----

    #[test]
    fn test_layer_visible_on_screen() {
        let mut layer = Layer::new("Test");
        layer.visible_on_screen(false);
        assert!(!layer.visible_on_screen);
    }

    #[test]
    fn test_layer_exportable() {
        let mut layer = Layer::new("Test");
        layer.exportable(false);
        assert!(!layer.exportable);
    }

    #[test]
    fn test_layer_order() {
        let mut layer = Layer::new("Test");
        layer.order(5);
        assert_eq!(layer.order, Some(5));
    }

    // ---- Tests for build_ocg_dict with different intents ----

    #[test]
    fn test_layer_ocg_dict_view_intent() {
        let layer = Layer::new("ViewLayer");
        let dict = layer.build_ocg_dict();
        if let Some(Object::Name(intent)) = dict.get("Intent") {
            assert_eq!(intent, "View");
        } else {
            panic!("Expected View intent");
        }
    }

    #[test]
    fn test_layer_ocg_dict_design_intent() {
        let mut layer = Layer::new("DesignLayer");
        layer.intent(LayerIntent::Design);
        let dict = layer.build_ocg_dict();
        if let Some(Object::Name(intent)) = dict.get("Intent") {
            assert_eq!(intent, "Design");
        } else {
            panic!("Expected Design intent");
        }
    }

    #[test]
    fn test_layer_ocg_dict_both_intent() {
        let mut layer = Layer::new("BothLayer");
        layer.intent(LayerIntent::Both);
        let dict = layer.build_ocg_dict();
        if let Some(Object::Array(arr)) = dict.get("Intent") {
            assert_eq!(arr.len(), 2);
            assert_eq!(arr[0], Object::Name("View".to_string()));
            assert_eq!(arr[1], Object::Name("Design".to_string()));
        } else {
            panic!("Expected Array intent for Both");
        }
    }

    #[test]
    fn test_layer_ocg_dict_uses_display_name() {
        let mut layer = Layer::new("internal_name");
        layer.display_name("Displayed Name");
        let dict = layer.build_ocg_dict();
        if let Some(Object::String(name_bytes)) = dict.get("Name") {
            assert_eq!(name_bytes, b"Displayed Name");
        } else {
            panic!("Expected String for Name");
        }
    }

    #[test]
    fn test_layer_ocg_dict_falls_back_to_name() {
        let layer = Layer {
            name: "fallback_name".to_string(),
            ..Default::default()
        };
        // display_name is None, so build_ocg_dict should use name
        let dict = layer.build_ocg_dict();
        if let Some(Object::String(name_bytes)) = dict.get("Name") {
            assert_eq!(name_bytes, b"fallback_name");
        } else {
            panic!("Expected String for Name");
        }
    }

    // ---- Tests for build_usage_dict ----

    #[test]
    fn test_usage_dict_not_visible_on_screen() {
        let mut layer = Layer::new("Test");
        layer.visible_on_screen(false);
        let usage = layer.build_usage_dict().unwrap();
        assert!(usage.contains_key("View"));
        if let Some(Object::Dictionary(view_dict)) = usage.get("View") {
            assert_eq!(view_dict.get("ViewState"), Some(&Object::Name("OFF".to_string())));
        }
    }

    #[test]
    fn test_usage_dict_not_exportable() {
        let mut layer = Layer::new("Test");
        layer.exportable(false);
        let usage = layer.build_usage_dict().unwrap();
        assert!(usage.contains_key("Export"));
        if let Some(Object::Dictionary(export_dict)) = usage.get("Export") {
            assert_eq!(export_dict.get("ExportState"), Some(&Object::Name("OFF".to_string())));
        }
    }

    #[test]
    fn test_usage_dict_multiple_non_defaults() {
        let mut layer = Layer::new("Test");
        layer
            .visible_on_screen(false)
            .visible_on_print(false)
            .exportable(false);
        let usage = layer.build_usage_dict().unwrap();
        assert!(usage.contains_key("View"));
        assert!(usage.contains_key("Print"));
        assert!(usage.contains_key("Export"));
    }

    // ---- Tests for LayerBuilder ----

    #[test]
    fn test_layer_builder_empty() {
        let builder = LayerBuilder::new();
        assert!(builder.is_empty());
        assert_eq!(builder.len(), 0);
        assert!(builder.layers().is_empty());
    }

    #[test]
    fn test_layer_builder_application_name() {
        let builder = LayerBuilder::new().application_name("MyApp");
        assert_eq!(builder.application_name, Some("MyApp".to_string()));
    }

    #[test]
    fn test_layer_builder_layers_mut() {
        let mut builder = LayerBuilder::new();
        builder.add_layer("Layer1");
        builder.layers_mut().push(Layer::new("Layer2"));
        assert_eq!(builder.len(), 2);
    }

    // ---- Tests for build_oc_properties ----

    #[test]
    fn test_build_oc_properties_basic() {
        let mut builder = LayerBuilder::new();
        builder.add_layer("Visible").visible(true);
        builder.add_layer("Hidden").visible(false);

        let refs = vec![ObjectRef::new(10, 0), ObjectRef::new(11, 0)];
        let props = builder.build_oc_properties(&refs);

        assert!(props.contains_key("OCGs"));
        assert!(props.contains_key("D"));

        if let Some(Object::Array(ocgs)) = props.get("OCGs") {
            assert_eq!(ocgs.len(), 2);
        }

        if let Some(Object::Dictionary(d_dict)) = props.get("D") {
            assert!(d_dict.contains_key("Name"));
            assert!(d_dict.contains_key("BaseState"));
            assert!(d_dict.contains_key("Order"));

            // Visible layer should be in ON array
            if let Some(Object::Array(on_arr)) = d_dict.get("ON") {
                assert_eq!(on_arr.len(), 1);
            }
            // Hidden layer should be in OFF array
            if let Some(Object::Array(off_arr)) = d_dict.get("OFF") {
                assert_eq!(off_arr.len(), 1);
            }
        }
    }

    #[test]
    fn test_build_oc_properties_with_creator() {
        let mut builder = LayerBuilder::new().creator("pdf_oxide_test");
        builder.add_layer("Layer1");

        let refs = vec![ObjectRef::new(1, 0)];
        let props = builder.build_oc_properties(&refs);

        if let Some(Object::Dictionary(d_dict)) = props.get("D") {
            if let Some(Object::String(creator_bytes)) = d_dict.get("Creator") {
                assert_eq!(creator_bytes, b"pdf_oxide_test");
            } else {
                panic!("Expected Creator string");
            }
        }
    }

    #[test]
    fn test_build_oc_properties_off_base_state() {
        let mut builder = LayerBuilder::new().base_state(LayerVisibility::Off);
        builder.add_layer("Layer1");

        let refs = vec![ObjectRef::new(1, 0)];
        let props = builder.build_oc_properties(&refs);

        if let Some(Object::Dictionary(d_dict)) = props.get("D") {
            assert_eq!(d_dict.get("BaseState"), Some(&Object::Name("OFF".to_string())));
        }
    }

    #[test]
    fn test_build_oc_properties_all_visible() {
        let mut builder = LayerBuilder::new();
        builder.add_layer("Layer1").visible(true);
        builder.add_layer("Layer2").visible(true);

        let refs = vec![ObjectRef::new(1, 0), ObjectRef::new(2, 0)];
        let props = builder.build_oc_properties(&refs);

        if let Some(Object::Dictionary(d_dict)) = props.get("D") {
            // All visible = all in ON, no OFF
            if let Some(Object::Array(on_arr)) = d_dict.get("ON") {
                assert_eq!(on_arr.len(), 2);
            }
            assert!(!d_dict.contains_key("OFF"));
        }
    }

    #[test]
    fn test_build_oc_properties_all_hidden() {
        let mut builder = LayerBuilder::new();
        builder.add_layer("Layer1").visible(false);
        builder.add_layer("Layer2").visible(false);

        let refs = vec![ObjectRef::new(1, 0), ObjectRef::new(2, 0)];
        let props = builder.build_oc_properties(&refs);

        if let Some(Object::Dictionary(d_dict)) = props.get("D") {
            assert!(!d_dict.contains_key("ON"));
            if let Some(Object::Array(off_arr)) = d_dict.get("OFF") {
                assert_eq!(off_arr.len(), 2);
            }
        }
    }

    #[test]
    fn test_build_oc_properties_more_layers_than_refs() {
        // When there are more layers than refs, extra layers are skipped
        let mut builder = LayerBuilder::new();
        builder.add_layer("Layer1").visible(true);
        builder.add_layer("Layer2").visible(true);
        builder.add_layer("Layer3").visible(true);

        let refs = vec![ObjectRef::new(1, 0)]; // Only 1 ref for 3 layers
        let props = builder.build_oc_properties(&refs);

        if let Some(Object::Dictionary(d_dict)) = props.get("D") {
            if let Some(Object::Array(on_arr)) = d_dict.get("ON") {
                assert_eq!(on_arr.len(), 1); // Only first layer included
            }
        }
    }

    // ---- Tests for LayerMembership ----

    #[test]
    fn test_layer_membership_add_ocg() {
        let mut membership = LayerMembership::new(ObjectRef::new(1, 0));
        membership.add_ocg(ObjectRef::new(2, 0));
        membership.add_ocg(ObjectRef::new(3, 0));
        assert_eq!(membership.ocgs.len(), 3);
    }

    #[test]
    fn test_layer_membership_policy_method() {
        let membership =
            LayerMembership::new(ObjectRef::new(1, 0)).policy(VisibilityPolicy::AllOff);
        assert_eq!(membership.policy, VisibilityPolicy::AllOff);
    }

    #[test]
    fn test_layer_membership_ocmd_single_ocg() {
        let membership = LayerMembership::new(ObjectRef::new(5, 0));
        let dict = membership.build_ocmd_dict();

        // Single OCG should be a Reference, not an Array
        if let Some(Object::Reference(r)) = dict.get("OCGs") {
            assert_eq!(r.id, 5);
            assert_eq!(r.gen, 0);
        } else {
            panic!("Expected single Reference for single OCG");
        }
    }

    #[test]
    fn test_layer_membership_ocmd_multiple_ocgs() {
        let mut membership = LayerMembership::new(ObjectRef::new(5, 0));
        membership.add_ocg(ObjectRef::new(6, 0));
        let dict = membership.build_ocmd_dict();

        // Multiple OCGs should be an Array
        if let Some(Object::Array(arr)) = dict.get("OCGs") {
            assert_eq!(arr.len(), 2);
        } else {
            panic!("Expected Array for multiple OCGs");
        }
    }

    #[test]
    fn test_layer_membership_ocmd_all_policies() {
        let policies = vec![
            (VisibilityPolicy::AllOn, "AllOn"),
            (VisibilityPolicy::AnyOn, "AnyOn"),
            (VisibilityPolicy::AllOff, "AllOff"),
            (VisibilityPolicy::AnyOff, "AnyOff"),
        ];

        for (policy, expected_name) in policies {
            let membership = LayerMembership::new(ObjectRef::new(1, 0)).policy(policy);
            let dict = membership.build_ocmd_dict();
            if let Some(Object::Name(name)) = dict.get("P") {
                assert_eq!(name, expected_name);
            } else {
                panic!("Expected Name for P key");
            }
        }
    }

    // ---- Tests for LayerVisibility ----

    #[test]
    fn test_layer_visibility_default() {
        assert_eq!(LayerVisibility::default(), LayerVisibility::On);
    }

    // ---- Tests for LayerIntent ----

    #[test]
    fn test_layer_intent_default() {
        assert_eq!(LayerIntent::default(), LayerIntent::View);
    }

    #[test]
    fn test_layer_intent_equality() {
        assert_eq!(LayerIntent::View, LayerIntent::View);
        assert_eq!(LayerIntent::Design, LayerIntent::Design);
        assert_eq!(LayerIntent::Both, LayerIntent::Both);
        assert_ne!(LayerIntent::View, LayerIntent::Design);
        assert_ne!(LayerIntent::View, LayerIntent::Both);
    }
}
