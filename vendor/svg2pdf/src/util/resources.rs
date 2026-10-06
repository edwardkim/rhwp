use std::collections::HashMap;
use std::rc::Rc;

use crate::util::allocate::NameAllocator;
use pdf_writer::types::ProcSet;
use pdf_writer::writers::{ColorSpace, Resources};
use pdf_writer::{Dict, Ref};

use super::helper::NameExt;

#[derive(Clone, Copy, Eq, PartialEq)]
enum PendingResourceType {
    XObject,
    Pattern,
    GraphicsState,
    Shading,
    Font,
    ColorSpace,
}

impl PendingResourceType {
    fn get_dict<'a>(&'a self, resources: &'a mut Resources) -> Dict<'a> {
        match *self {
            PendingResourceType::XObject => resources.x_objects(),
            PendingResourceType::Pattern => resources.patterns(),
            PendingResourceType::GraphicsState => resources.ext_g_states(),
            PendingResourceType::Shading => resources.shadings(),
            PendingResourceType::Font => resources.fonts(),
            PendingResourceType::ColorSpace => resources.color_spaces(),
        }
    }

    pub fn iterator() -> impl Iterator<Item = PendingResourceType> {
        [
            PendingResourceType::XObject,
            PendingResourceType::Pattern,
            PendingResourceType::GraphicsState,
            PendingResourceType::Shading,
            PendingResourceType::Font,
            PendingResourceType::ColorSpace,
        ]
        .iter()
        .copied()
    }
}

#[derive(Clone, Eq, PartialEq)]
struct PendingResource {
    object_type: PendingResourceType,
    name: Rc<String>,
    reference: Ref,
}

impl PendingResource {
    fn serialize(&self, dict: &mut Dict) {
        match self.object_type {
            PendingResourceType::ColorSpace => {
                dict.insert(self.name.to_pdf_name())
                    .start::<ColorSpace>()
                    // TODO: Allow other color spaces than ICC-based
                    .icc_based(self.reference);
            }
            _ => {
                dict.pair(self.name.to_pdf_name(), self.reference);
            }
        }
    }
}

/// Holds all resources for an XObject or a page.
/// NOTE: References for distinct objects are assumed to be distinct,
/// as a consequence, two same references are assumed to always point
/// to the same object and thus will be deduplicated.
#[derive(Clone, Eq, PartialEq)]
pub struct ResourceContainer {
    name_allocator: NameAllocator,
    pending_resources: HashMap<Ref, PendingResource>,
}

impl ResourceContainer {
    fn add_resource_entry(
        &mut self,
        reference: Ref,
        object_type: PendingResourceType,
    ) -> Rc<String> {
        // Only insert if reference has not been assigned yet to deduplicate.
        self.pending_resources
            .entry(reference)
            .or_insert_with(|| {
                let name = match object_type {
                    PendingResourceType::XObject => {
                        self.name_allocator.alloc_x_object_name()
                    }
                    PendingResourceType::Pattern => {
                        self.name_allocator.alloc_pattern_name()
                    }
                    PendingResourceType::GraphicsState => {
                        self.name_allocator.alloc_graphics_state_name()
                    }
                    PendingResourceType::Shading => {
                        self.name_allocator.alloc_shading_name()
                    }
                    PendingResourceType::Font => self.name_allocator.alloc_font_name(),
                    PendingResourceType::ColorSpace => {
                        self.name_allocator.alloc_color_space_name()
                    }
                };

                let name = Rc::new(name);
                PendingResource { object_type, reference, name: name.clone() }
            })
            .name
            .clone()
    }

    pub fn new() -> Self {
        Self {
            name_allocator: NameAllocator::default(),
            pending_resources: HashMap::new(),
        }
    }

    /// Add a new XObject as a resource. Returns the name of the XObject.
    pub fn add_x_object(&mut self, reference: Ref) -> Rc<String> {
        self.add_resource_entry(reference, PendingResourceType::XObject)
    }

    /// Add a new Shading as a resource. Returns the name of the Shading.
    pub fn add_shading(&mut self, reference: Ref) -> Rc<String> {
        self.add_resource_entry(reference, PendingResourceType::Shading)
    }

    /// Add a new Pattern as a resource. Returns the name of the Pattern.
    pub fn add_pattern(&mut self, reference: Ref) -> Rc<String> {
        self.add_resource_entry(reference, PendingResourceType::Pattern)
    }

    /// Add a new GraphicsState as a resource. Returns the name of the GraphicsState.
    pub fn add_graphics_state(&mut self, reference: Ref) -> Rc<String> {
        self.add_resource_entry(reference, PendingResourceType::GraphicsState)
    }

    /// Add a new Font as a resource. Returns the name of the Font.
    #[cfg(feature = "text")]
    pub fn add_font(&mut self, reference: Ref) -> Rc<String> {
        self.add_resource_entry(reference, PendingResourceType::Font)
    }

    /// Add a new ColorSpace as a resource. Returns the name of the ColorSpace.
    pub fn add_color_space(&mut self, reference: Ref) -> Rc<String> {
        self.add_resource_entry(reference, PendingResourceType::ColorSpace)
    }

    /// Dump all pending resources into a resources dictionary.
    pub fn finish(self, resources: &mut Resources) {
        for object_type in PendingResourceType::iterator() {
            let mut entries: Vec<_> = self
                .pending_resources
                .values()
                .filter(|e| e.object_type == object_type)
                .collect();
            // Sort by object reference so that dictionary entry order does
            // not depend on `HashMap` iteration order (reproducible output).
            entries.sort_by_key(|e| e.reference);

            if !entries.is_empty() {
                let mut dict = object_type.get_dict(resources);

                for entry in entries {
                    entry.serialize(&mut dict);
                }
            }
        }

        resources.proc_sets([
            ProcSet::Pdf,
            ProcSet::Text,
            ProcSet::ImageColor,
            ProcSet::ImageGrayscale,
        ]);
    }
}
