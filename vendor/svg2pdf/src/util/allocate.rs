use pdf_writer::Ref;

/// Struct that keeps track of ref allocations in a PDF.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct RefAllocator {
    /// The next free id for indirect object references.
    ref_alloc: Ref,
}

impl RefAllocator {
    /// Create a new allocator with a specific start ID.
    pub fn new() -> Self {
        Self { ref_alloc: Ref::new(1) }
    }

    /// Allocate a new reference ID.
    pub fn alloc_ref(&mut self) -> Ref {
        self.ref_alloc.bump()
    }
}

/// Struct that keeps track name allocations in a XObject/Page.
#[derive(Clone, Copy, Eq, PartialEq, Default)]
pub struct NameAllocator {
    /// The next number that will be used for the name of an XObject in a resource
    /// dictionary, e.g. "xo0".
    next_x_object_num: i32,
    /// The next number that will be used for the name of a graphics state in a resource
    /// dictionary, e.g. "gs0".
    next_graphics_state_num: i32,
    /// The next number that will be used for the name of a pattern in a resource
    /// dictionary, e.g. "po0".
    next_patterns_num: i32,
    /// The next number that will be used for the name of a shading in a resource
    /// dictionary, e.g. "sh0".
    next_shadings_num: i32,
    /// The next number that will be used for the name of a font in a resource
    /// dictionary, e.g. "fo0".
    next_fonts_num: i32,
    /// The next number that will be used for the name of a color space in a resource
    /// dictionary, e.g. "cs0".
    next_color_space_num: i32,
}

impl NameAllocator {
    /// Allocate a new XObject name.
    pub fn alloc_x_object_name(&mut self) -> String {
        let num = self.next_x_object_num;
        self.next_x_object_num += 1;
        format!("xo{}", num)
    }

    /// Allocate a new graphics state name.
    pub fn alloc_graphics_state_name(&mut self) -> String {
        let num = self.next_graphics_state_num;
        self.next_graphics_state_num += 1;
        format!("gs{}", num)
    }

    /// Allocate a new pattern name.
    pub fn alloc_pattern_name(&mut self) -> String {
        let num = self.next_patterns_num;
        self.next_patterns_num += 1;
        format!("po{}", num)
    }

    /// Allocate a new shading name.
    pub fn alloc_shading_name(&mut self) -> String {
        let num = self.next_shadings_num;
        self.next_shadings_num += 1;
        format!("sh{}", num)
    }

    /// Allocate a new shading name.
    pub fn alloc_font_name(&mut self) -> String {
        let num = self.next_fonts_num;
        self.next_fonts_num += 1;
        format!("fo{}", num)
    }

    /// Allocate a new color space name.
    pub fn alloc_color_space_name(&mut self) -> String {
        let num = self.next_color_space_num;
        self.next_color_space_num += 1;
        format!("cs{}", num)
    }
}
