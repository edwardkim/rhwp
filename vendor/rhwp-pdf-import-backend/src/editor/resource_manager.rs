//! Resource management for PDF editing.
//!
//! Manages fonts, images, and other resources during content modification.
//! Tracks resource allocation and ensures proper naming/referencing.

use crate::object::ObjectRef;
use std::collections::HashMap;

/// Manages fonts, images, and other resources during editing.
///
/// Provides allocation and tracking of:
/// - Fonts (/F1, /F2, etc.)
/// - Images/XObjects (/Im1, /Im2, etc.)
/// - Graphics states (/GS1, /GS2, etc.)
/// - Patterns, shadings, etc.
#[derive(Debug, Clone)]
pub struct ResourceManager {
    /// Font registry: name → object reference
    fonts: HashMap<String, ObjectRef>,

    /// Font name allocation counter
    next_font_id: u32,

    /// XObject (image) registry: name → object reference
    xobjects: HashMap<String, ObjectRef>,

    /// XObject allocation counter
    next_xobject_id: u32,

    /// Graphics state registry: name → object reference
    graphics_states: HashMap<String, ObjectRef>,

    /// Graphics state allocation counter
    next_graphics_state_id: u32,

    /// Pattern registry: name → object reference
    patterns: HashMap<String, ObjectRef>,

    /// Pattern allocation counter
    next_pattern_id: u32,

    /// Shading registry: name → object reference
    shadings: HashMap<String, ObjectRef>,

    /// Shading allocation counter
    next_shading_id: u32,
}

impl ResourceManager {
    /// Create a new resource manager.
    pub fn new() -> Self {
        Self {
            fonts: HashMap::new(),
            next_font_id: 1,
            xobjects: HashMap::new(),
            next_xobject_id: 1,
            graphics_states: HashMap::new(),
            next_graphics_state_id: 1,
            patterns: HashMap::new(),
            next_pattern_id: 1,
            shadings: HashMap::new(),
            next_shading_id: 1,
        }
    }

    // === Font Management ===

    /// Register or get a font resource name.
    ///
    /// Returns the resource name (e.g., "/F1", "/F2") for use in content streams.
    /// If a font with this name is already registered, returns the existing name.
    ///
    /// # Arguments
    ///
    /// * `font_name` - Font name (e.g., "Helvetica", "Times-Roman")
    ///
    /// # Returns
    ///
    /// Resource name to use in PDF content (e.g., "/F1")
    pub fn register_font(&mut self, font_name: &str) -> String {
        let key = format!("font:{}", font_name);
        if self.fonts.contains_key(&key) {
            // Find existing ID
            for (existing_key, obj_ref) in &self.fonts {
                if existing_key == &key {
                    return format!("/F{}", obj_ref.id);
                }
            }
        }

        let id = self.next_font_id;
        self.next_font_id += 1;

        let resource_name = format!("F{}", id);
        self.fonts.insert(key, ObjectRef::new(id, 0));

        format!("/{}", resource_name)
    }

    /// Get a font resource name by its font name.
    ///
    /// Returns None if the font is not registered.
    pub fn get_font_resource(&self, font_name: &str) -> Option<String> {
        let key = format!("font:{}", font_name);
        self.fonts
            .get(&key)
            .map(|obj_ref| format!("/F{}", obj_ref.id))
    }

    /// List all registered fonts.
    pub fn fonts(&self) -> Vec<(String, String)> {
        self.fonts
            .iter()
            .map(|(key, obj_ref)| {
                let font_name = key.strip_prefix("font:").unwrap_or("").to_string();
                (font_name, format!("F{}", obj_ref.id))
            })
            .collect()
    }

    fn get_font_id(&self, key: &str) -> u32 {
        self.fonts.get(key).map(|obj_ref| obj_ref.id).unwrap_or(0)
    }

    // === Image/XObject Management ===

    /// Register an image/XObject resource.
    ///
    /// Returns the resource name (e.g., "/Im1", "/Im2") for use in content streams.
    pub fn register_image(&mut self) -> String {
        let id = self.next_xobject_id;
        self.next_xobject_id += 1;

        let key = format!("image:{}", id);
        self.xobjects.insert(key, ObjectRef::new(id, 0));

        format!("/Im{}", id)
    }

    /// List all registered images.
    pub fn images(&self) -> Vec<(String, String)> {
        self.xobjects
            .values()
            .map(|obj_ref| {
                let key = format!("image:{}", obj_ref.id);
                (key, format!("Im{}", obj_ref.id))
            })
            .collect()
    }

    // === Graphics State Management ===

    /// Register a graphics state resource.
    ///
    /// Returns the resource name (e.g., "/GS1", "/GS2").
    pub fn register_graphics_state(&mut self) -> String {
        let id = self.next_graphics_state_id;
        self.next_graphics_state_id += 1;

        let key = format!("gs:{}", id);
        self.graphics_states.insert(key, ObjectRef::new(id, 0));

        format!("/GS{}", id)
    }

    /// List all registered graphics states.
    pub fn graphics_states(&self) -> Vec<(String, String)> {
        self.graphics_states
            .values()
            .map(|obj_ref| {
                let key = format!("gs:{}", obj_ref.id);
                (key, format!("GS{}", obj_ref.id))
            })
            .collect()
    }

    // === Pattern Management ===

    /// Register a pattern resource.
    ///
    /// Returns the resource name (e.g., "/Pat1", "/Pat2").
    pub fn register_pattern(&mut self) -> String {
        let id = self.next_pattern_id;
        self.next_pattern_id += 1;

        let key = format!("pattern:{}", id);
        self.patterns.insert(key, ObjectRef::new(id, 0));

        format!("/Pat{}", id)
    }

    /// List all registered patterns.
    pub fn patterns(&self) -> Vec<(String, String)> {
        self.patterns
            .values()
            .map(|obj_ref| {
                let key = format!("pattern:{}", obj_ref.id);
                (key, format!("Pat{}", obj_ref.id))
            })
            .collect()
    }

    // === Shading Management ===

    /// Register a shading (gradient) resource.
    ///
    /// Returns the resource name (e.g., "/Sh1", "/Sh2").
    pub fn register_shading(&mut self) -> String {
        let id = self.next_shading_id;
        self.next_shading_id += 1;

        let key = format!("shading:{}", id);
        self.shadings.insert(key, ObjectRef::new(id, 0));

        format!("/Sh{}", id)
    }

    /// List all registered shadings.
    pub fn shadings(&self) -> Vec<(String, String)> {
        self.shadings
            .values()
            .map(|obj_ref| {
                let key = format!("shading:{}", obj_ref.id);
                (key, format!("Sh{}", obj_ref.id))
            })
            .collect()
    }

    // === General ===

    /// Get a count of all registered resources.
    pub fn resource_count(&self) -> usize {
        self.fonts.len()
            + self.xobjects.len()
            + self.graphics_states.len()
            + self.patterns.len()
            + self.shadings.len()
    }

    /// Clear all registered resources.
    pub fn clear(&mut self) {
        self.fonts.clear();
        self.xobjects.clear();
        self.graphics_states.clear();
        self.patterns.clear();
        self.shadings.clear();
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_manager_creation() {
        let _manager = ResourceManager::new();
    }

    #[test]
    fn test_font_registration() {
        let mut manager = ResourceManager::new();

        let name1 = manager.register_font("Helvetica");
        assert_eq!(name1, "/F1");

        let name2 = manager.register_font("Times");
        assert_eq!(name2, "/F2");

        let name3 = manager.register_font("Helvetica");
        assert_eq!(name3, "/F1");
    }

    #[test]
    fn test_image_registration() {
        let mut manager = ResourceManager::new();

        let img1 = manager.register_image();
        assert_eq!(img1, "/Im1");

        let img2 = manager.register_image();
        assert_eq!(img2, "/Im2");
    }

    #[test]
    fn test_graphics_state_registration() {
        let mut manager = ResourceManager::new();

        let gs1 = manager.register_graphics_state();
        assert_eq!(gs1, "/GS1");

        let gs2 = manager.register_graphics_state();
        assert_eq!(gs2, "/GS2");
    }

    #[test]
    fn test_resource_count() {
        let mut manager = ResourceManager::new();

        assert_eq!(manager.resource_count(), 0);

        manager.register_font("Helvetica");
        assert_eq!(manager.resource_count(), 1);

        manager.register_image();
        assert_eq!(manager.resource_count(), 2);

        manager.register_graphics_state();
        assert_eq!(manager.resource_count(), 3);
    }

    #[test]
    fn test_clear() {
        let mut manager = ResourceManager::new();

        manager.register_font("Helvetica");
        manager.register_image();
        assert_eq!(manager.resource_count(), 2);

        manager.clear();
        assert_eq!(manager.resource_count(), 0);
    }

    #[test]
    fn test_default() {
        let manager = ResourceManager::default();
        assert_eq!(manager.resource_count(), 0);
    }

    #[test]
    fn test_get_font_resource() {
        let mut manager = ResourceManager::new();
        assert!(manager.get_font_resource("Helvetica").is_none());

        manager.register_font("Helvetica");
        let resource = manager.get_font_resource("Helvetica");
        assert!(resource.is_some());
        assert_eq!(resource.unwrap(), "/F1");
    }

    #[test]
    fn test_fonts_list() {
        let mut manager = ResourceManager::new();
        manager.register_font("Helvetica");
        manager.register_font("Times");

        let fonts = manager.fonts();
        assert_eq!(fonts.len(), 2);
    }

    #[test]
    fn test_images_list() {
        let mut manager = ResourceManager::new();
        manager.register_image();
        manager.register_image();

        let images = manager.images();
        assert_eq!(images.len(), 2);
    }

    #[test]
    fn test_graphics_states_list() {
        let mut manager = ResourceManager::new();
        manager.register_graphics_state();

        let states = manager.graphics_states();
        assert_eq!(states.len(), 1);
    }

    #[test]
    fn test_pattern_registration() {
        let mut manager = ResourceManager::new();

        let pat1 = manager.register_pattern();
        assert_eq!(pat1, "/Pat1");

        let pat2 = manager.register_pattern();
        assert_eq!(pat2, "/Pat2");
    }

    #[test]
    fn test_patterns_list() {
        let mut manager = ResourceManager::new();
        manager.register_pattern();
        manager.register_pattern();

        let patterns = manager.patterns();
        assert_eq!(patterns.len(), 2);
    }

    #[test]
    fn test_shading_registration() {
        let mut manager = ResourceManager::new();

        let sh1 = manager.register_shading();
        assert_eq!(sh1, "/Sh1");

        let sh2 = manager.register_shading();
        assert_eq!(sh2, "/Sh2");
    }

    #[test]
    fn test_shadings_list() {
        let mut manager = ResourceManager::new();
        manager.register_shading();

        let shadings = manager.shadings();
        assert_eq!(shadings.len(), 1);
    }

    #[test]
    fn test_resource_count_all_types() {
        let mut manager = ResourceManager::new();

        manager.register_font("Courier");
        manager.register_image();
        manager.register_graphics_state();
        manager.register_pattern();
        manager.register_shading();

        assert_eq!(manager.resource_count(), 5);
    }

    #[test]
    fn test_clear_all_types() {
        let mut manager = ResourceManager::new();

        manager.register_font("Courier");
        manager.register_image();
        manager.register_graphics_state();
        manager.register_pattern();
        manager.register_shading();
        assert_eq!(manager.resource_count(), 5);

        manager.clear();
        assert_eq!(manager.resource_count(), 0);
        assert!(manager.fonts().is_empty());
        assert!(manager.images().is_empty());
        assert!(manager.graphics_states().is_empty());
        assert!(manager.patterns().is_empty());
        assert!(manager.shadings().is_empty());
    }

    #[test]
    fn test_clone() {
        let mut manager = ResourceManager::new();
        manager.register_font("Helvetica");
        manager.register_image();

        let cloned = manager.clone();
        assert_eq!(cloned.resource_count(), 2);
        assert!(cloned.get_font_resource("Helvetica").is_some());
    }

    #[test]
    fn test_debug() {
        let manager = ResourceManager::new();
        let debug = format!("{:?}", manager);
        assert!(debug.contains("ResourceManager"));
    }

    #[test]
    fn test_get_font_id() {
        let mut manager = ResourceManager::new();
        assert_eq!(manager.get_font_id("font:Helvetica"), 0); // not registered

        manager.register_font("Helvetica");
        assert_eq!(manager.get_font_id("font:Helvetica"), 1);
    }
}
