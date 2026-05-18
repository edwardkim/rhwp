//! Implementation of the definitions in Section 2.3.6 of the WMF
//! specifications.

mod abort_doc;
mod begin_path;
mod check_jpeg_format;
mod check_png_format;
mod clip_to_path;
mod close_channel;
mod download_face;
mod download_header;
mod draw_pattern_rect;
mod encapsulated_postscript;
mod end_doc;
mod end_path;
mod eps_printing;
mod ext_text_out;
mod get_color_table;
mod get_device_units;
mod get_extended_text_metrics;
mod get_facename;
mod get_pair_kern_table;
mod get_phys_page_size;
mod get_printing_offset;
mod get_ps_feature_setting;
mod get_scaling_factor;
mod meta_escape_enhanced_metafile;
mod metafile_driver;
mod new_frame;
mod next_band;
mod open_channel;
mod passthrough;
mod postscript_data;
mod postscript_identify;
mod postscript_ignore;
mod postscript_injection;
mod postscript_passthrough;
mod query_dib_support;
mod query_esc_support;
mod set_color_table;
mod set_copy_count;
mod set_line_cap;
mod set_line_join;
mod set_miter_limit;
mod spcl_passthrough2;
mod startdoc;

use crate::wmf::imports::*;

/// The META_ESCAPE Record specifies extensions to WMF functionality that are
/// not directly available through other records defined in the RecordType
/// Enumeration. The MetafileEscapes Enumeration lists these extensions.
#[derive(Clone, Debug)]
pub enum META_ESCAPE {
    /// The ABORTDOC Record stops processing the current document and erases
    /// everything drawn since the last STARTDOC Record was processed.
    ABORTDOC {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// the WMF record.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// this record type. The lower byte MUST match the lower byte of the
        /// RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that MUST be 0x0000.
        byte_count: u16,
    },
    /// The BEGIN_PATH Record opens a path.
    BEGIN_PATH {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// the WMF record.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// this record type. The lower byte MUST match the lower byte of the
        /// RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that MUST be 0x0000.
        byte_count: u16,
    },
    /// The CHECKJPEGFORMAT Record specifies whether the printer driver
    /// supports JPEG image output.
    CHECKJPEGFORMAT {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// the record.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// this record type. The lower byte MUST match the lower byte of the
        /// RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that specifies the
        /// size, in bytes, of the JPEGBuffer field.
        byte_count: u16,
        /// JPEGBuffer (variable): A buffer of JPEG image data.
        jpeg_buffer: Vec<u8>,
    },
    /// The CHECKPNGFORMAT Record queries the driver to see if it can handle
    /// the given PNG image and parses the PNG image to determine whether the
    /// driver can support it.
    CHECKPNGFORMAT {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// the record.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// this record type. The lower byte MUST match the lower byte of the
        /// RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that specifies the
        /// size, in bytes, of the PNGBuffer field.
        byte_count: u16,
        /// PNGBuffer (variable): A buffer of PNG image data.
        png_buffer: Vec<u8>,
    },
    /// The CLIP_TO_PATH Record applies a function to the current PostScript
    /// clipping path.
    CLIP_TO_PATH {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// this record. This value MUST be 0x0000000E.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// the record type. The lower byte MUST match the lower byte of the
        /// RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that specifies the
        /// size, in bytes, of the record data that follows. This value MUST be
        /// 0x0004.
        byte_count: u16,
        /// ClipFunction (2 bytes): A 16-bit unsigned integer that defines the
        /// function to apply to the PostScript clipping path. This value MUST
        /// be a PostScriptClipping Enumeration table value.
        clip_function: crate::wmf::parser::PostScriptClipping,
        /// Reserved1 (2 bytes): This value SHOULD be zero and SHOULD be
        /// ignored by the client. (Windows 95, Windows 98, and Windows
        /// Millennium Edition implementations set this field to the fill mode
        /// value.)
        reserved1: u16,
    },
    /// The CLOSECHANNEL Record notifies the printer driver that the current
    /// print job is ending. This is the same function as the ENDDOC Record. A
    /// CLOSECHANNEL MUST be preceded by an OPENCHANNEL Record.
    CLOSECHANNEL {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// the WMF record.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// this record type. The low-order byte MUST match the low-order byte
        /// of the RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that MUST be 0x0000.
        byte_count: u16,
    },
    /// The DOWNLOADFACE Record sends the font face.
    DOWNLOADFACE {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// the WMF record.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// this record type. The low-order byte MUST match the low-order byte
        /// of the RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that MUST be 0x0000.
        byte_count: u16,
    },
    /// The DOWNLOADHEADER Record instructs the driver to download all sets of
    /// PostScript procedures.
    DOWNLOADHEADER {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// the WMF record.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// this record type. The lower byte MUST match the lower byte of the
        /// RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that MUST be 0x0000.
        byte_count: u16,
    },
    /// The DRAWPATTERNRECT Record draws a rectangle with a defined pattern.
    DRAWPATTERNRECT {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// the record.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// this record type. The lower byte MUST match the lower byte of the
        /// RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that specifies the
        /// size, in bytes, of the record data that follows. This MUST be
        /// 0x0014.
        byte_count: u16,
        /// Position (8 bytes): A PointL Object that defines the position of
        /// the rectangle.
        position: crate::wmf::parser::PointL,
        /// Size (8 bytes): A PointL Object that defines the dimensions of the
        /// rectangle.
        size: crate::wmf::parser::PointL,
        /// Style (2 bytes): A 16-bit unsigned integer that defines the style.
        style: u16,
        /// Pattern (2 bytes): A 16-bit unsigned integer that defines the
        /// pattern.
        pattern: u16,
    },
    /// The ENCAPSULATED_POSTSCRIPT Record sends arbitrary PostScript data
    /// directly to a printer driver.
    ENCAPSULATED_POSTSCRIPT {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// the WMF record.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// this record type. The low-order byte MUST match the low-order byte
        /// of the RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that specifies the
        /// size, in bytes, of the record data that follows. This value SHOULD
        /// be greater than or equal to the value of the Size field. (Any bytes
        /// that exceed the ByteCount field are ignored by the client.)
        byte_count: u16,
        /// Size (4 bytes): A 32-bit unsigned integer that specifies the total
        /// size, in bytes, of the Size, Version, Points, and Data fields.
        size: u32,
        /// Version (4 bytes): A 32-bit unsigned integer that defines the
        /// PostScript language level.
        version: u32,
        /// Points (24 bytes): An array of three PointL Objects that define the
        /// output parallelogram in 28.4 FIX device coordinates.
        points: crate::wmf::parser::PointL,
        /// Data (variable): The PostScript data.
        data: Vec<u8>,
    },
    /// The END_PATH Record specifies the end of a path.
    END_PATH {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// the WMF record.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// this record type. The lower byte MUST match the lower byte of the
        /// RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that MUST be 0x0000.
        byte_count: u16,
    },
    /// The ENDDOC Record notifies the printer driver that the current print
    /// job is ending.
    ENDDOC {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// the WMF record.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// this record type. The lower byte MUST match the lower byte of the
        /// RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that MUST be 0x0000.
        byte_count: u16,
    },
    /// The EPSPRINTING Record indicates the start or end of Encapsulated
    /// PostScript (EPS) printing.
    EPSPRINTING {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// the record.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// this record type. The lower byte MUST match the lower byte of the
        /// RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that specifies the
        /// size, in bytes, of the SetEpsPrinting field. This MUST be 0x0002.
        byte_count: u16,
        /// SetEpsPrinting (2 bytes): A 16-bit unsigned integer that indicates
        /// the start or end of EPS printing. If the value is nonzero, the
        /// start of EPS printing is indicated; otherwise, the end is
        /// indicated.
        set_eps_printing: u16,
    },
    /// The EXTTEXTOUT Record draws text using the currently selected font,
    /// background color, and text color.
    EXTTEXTOUT {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// the WMF record.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// this record type. The lower byte MUST match the lower byte of the
        /// RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that MUST be 0x0000.
        byte_count: u16,
    },
    /// The GETCOLORTABLE Record gets color table values from the printer
    /// driver.
    GETCOLORTABLE {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// this record.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// the record type. The lower byte MUST match the lower byte of the
        /// RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that specifies the
        /// size, in bytes, of the record data that follows.
        byte_count: u16,
        /// Start (2 bytes): A 16-bit unsigned integer that defines the offset
        /// from the beginning of the record to the start of the color table
        /// data in the ColorTable field.
        start: u16,
        /// ColorTableBuffer (variable): A buffer containing the color table
        /// that is obtained from the printer driver, which is not required to
        /// be contiguous with the static part of the record.
        color_table_buffer: Vec<u8>,
    },
    /// The GETDEVICEUNITS Record gets the current device units.
    GETDEVICEUNITS {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// the WMF record.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// this record type. The lower byte MUST match the lower byte of the
        /// RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that MUST be 0x0000.
        byte_count: u16,
    },
    /// The GETEXTENDEDTEXTMETRICS Record gets the extended text metrics that
    /// are currently configured on the printer driver and applies them to the
    /// playback device context.
    GETEXTENDEDTEXTMETRICS {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// the WMF record.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// the record type. The lower byte MUST match the lower byte of the
        /// RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that MUST be 0x0000.
        byte_count: u16,
    },
    /// The GETFACENAME Record gets the font face name.
    GETFACENAME {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// the WMF record.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// this record type. The lower byte MUST match the lower byte of the
        /// RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that MUST be 0x0000.
        byte_count: u16,
    },
    /// The GETPAIRKERNTABLE Record gets the font kern table.
    GETPAIRKERNTABLE {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// the WMF record.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// this record type. The lower byte MUST match the lower byte of the
        /// RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that MUST be 0x0000.
        byte_count: u16,
    },
    /// The GETPHYSPAGESIZE Record retrieves the physical page size and copies
    /// it to a specified location.
    GETPHYSPAGESIZE {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// the WMF record.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// this record type. The lower byte MUST match the lower byte of the
        /// RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that MUST be 0x0000.
        byte_count: u16,
    },
    /// The GETPRINTINGOFFSET Record retrieves the offset from the upper-left
    /// corner of the physical page where the actual printing or drawing
    /// begins.
    GETPRINTINGOFFSET {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// the WMF record.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// this record type. The lower byte MUST match the lower byte of the
        /// RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that MUST be 0x0000.
        byte_count: u16,
    },
    /// The GET_PS_FEATURESETTING Record is used to query the driver concerning
    /// PostScript features.
    GET_PS_FEATURESETTING {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// the record.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// this record type. The lower byte MUST match the lower byte of the
        /// RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that specifies the
        /// size, in bytes, of the Feature field. This MUST be 0x0004.
        byte_count: u16,
        /// Feature (4 bytes): A 32-bit signed integer that identifies the
        /// feature setting being queried. Possible values are defined in the
        /// PostScriptFeatureSetting Enumeration.
        feature: crate::wmf::parser::PostScriptFeatureSetting,
    },
    /// The GETSCALINGFACTOR Record retrieves the scaling factors for the
    /// x-axis and the y-axis of a printer.
    GETSCALINGFACTOR {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// the WMF record.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// this record type. The lower byte MUST match the lower byte of the
        /// RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that MUST be 0x0000.
        byte_count: u16,
    },
    /// The META_ESCAPE_ENHANCED_METAFILE Record is used to embed an EMF
    /// metafile within a WMF metafile. The EMF metafile is broken up into
    /// sections, each represented by one META_ESCAPE_ENHANCED_METAFILE.
    META_ESCAPE_ENHANCED_METAFILE {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// the record.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// this record type. The lower byte MUST match the lower byte of the
        /// RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that specifies the
        /// size, in bytes, of the record data that follows. This value MUST be
        /// 34 plus the value of the EnhancedMetafileDataSize field.
        byte_count: u16,
        /// CommentIdentifier (4 bytes): A 32-bit unsigned integer that defines
        /// this record as a WMF Comment record. This value MUST be 0x43464D57.
        comment_identifier: u32,
        /// CommentType (4 bytes): A 32-bit unsigned integer that identifies
        /// the type of comment in this record. This value MUST be 0x00000001.
        comment_type: u32,
        /// Version (4 bytes): A 32-bit unsigned integer that specifies EMF
        /// metafile interoperability. This SHOULD be 0x00010000. (Windows does
        /// not check this value.)
        version: u32,
        /// Checksum (2 bytes): A 16-bit unsigned integer used to validate the
        /// correctness of the embedded EMF stream. This value MUST be the
        /// one's-complement of the result of applying an XOR operation to all
        /// WORD structures, defined in [MS-DTYP] section 2.2.61, in the EMF
        /// stream.
        checksum: u16,
        /// Flags (4 bytes): This 32-bit unsigned integer is unused and MUST be
        /// set to zero.
        flags: u32,
        /// CommentRecordCount (4 bytes): A 32-bit unsigned integer that
        /// specifies the total number of consecutive
        /// META_ESCAPE_ENHANCED_METAFILE records that contain the embedded EMF
        /// metafile.
        comment_record_count: u32,
        /// CurrentRecordSize (4 bytes): A 32-bit unsigned integer that
        /// specifies the size, in bytes, of the EnhancedMetafileData field.
        /// This value MUST be less than or equal to 8,192.
        current_record_size: u32,
        /// RemainingBytes (4 bytes): A 32-bit unsigned integer that specifies
        /// the number of bytes in the EMF stream that remain to be processed
        /// after this record. Those additional EMF bytes MUST follow in the
        /// EnhancedMetafileData fields of subsequent
        /// META_ESCAPE_ENHANDED_METAFILE escape records.
        remaining_bytes: u32,
        /// EnhancedMetafileDataSize (4 bytes): A 32-bit unsigned integer that
        /// specifies the total size of the EMF stream embedded in this
        /// sequence of META_ESCAPE_ENHANCED_METAFILE records.
        enhanced_metafile_data_size: u32,
        /// EnhancedMetafileData (variable): A segment of an EMF file. The
        /// bytes in consecutive META_ESCAPE_ENHANCED_METAFILE records MUST be
        /// concatenated to represent the entire embedded EMF file.
        enhanced_metafile_data: Vec<u8>,
    },
    /// The METAFILE_DRIVER Record queries the printer driver about its support
    /// for metafiles on the output device.
    METAFILE_DRIVER {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// this record.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// the record type. The lower byte MUST match the lower byte of the
        /// RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that MUST be 0x0000.
        byte_count: u16,
    },
    /// The NEWFRAME Record informs the printer that the application has
    /// finished writing to a page.
    NEWFRAME {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// the WMF record.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// this record type. The lower byte MUST match the lower byte of the
        /// RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that MUST be 0x0000.
        byte_count: u16,
    },
    /// The NEXTBAND Record informs the printer that the application has
    /// finished writing to a band. Band information is no longer used.
    NEXTBAND {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// the WMF record.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// this record type. The lower byte MUST match the lower byte of the
        /// RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that MUST be 0x0000.
        byte_count: u16,
    },
    /// The PASSTHROUGH Record passes through arbitrary data to the printer
    /// driver.
    PASSTHROUGH {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// the record.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// this record type. The lower byte MUST match the lower byte of the
        /// RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that specifies the
        /// size, in bytes, of the Data field.
        byte_count: u16,
        /// Data (variable): An array of bytes of size ByteCount.
        data: Vec<u8>,
    },
    /// The POSTSCRIPT_DATA Record sends arbitrary PostScript data to the
    /// printer driver.
    POSTSCRIPT_DATA {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// the record.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// this record type. The lower byte MUST match the lower byte of the
        /// RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that specifies the
        /// size, in bytes, of the Data field.
        byte_count: u16,
        /// Data (variable): An array of bytes of size ByteCount.
        data: Vec<u8>,
    },
    /// The POSTSCRIPT_IDENTIFY Record sets the printer driver to either
    /// PostScript-centric or GDI-centric mode.
    ///
    /// Note This record MUST be processed before the STARTDOC Record.
    POSTSCRIPT_IDENTIFY {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// the record.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// this record type. The lower byte MUST match the lower byte of the
        /// RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that specifies the
        /// size, in bytes, of the Data field.
        byte_count: u16,
        /// Data (variable): An array of bytes of size ByteCount.
        data: Vec<u8>,
    },
    /// The POSTSCRIPT_IGNORE Record informs the device to ignore the
    /// PostScript data.
    POSTSCRIPT_IGNORE {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// the WMF record.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// this record type. The lower byte MUST match the lower byte of the
        /// RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that MUST be 0x0000.
        byte_count: u16,
    },
    /// The POSTSCRIPT_INJECTION Record inserts a block of raw data into a
    /// PostScript stream. The input MUST be a 32-bit quantity specifying the
    /// number of bytes to inject, a 16-bit quantity specifying the injection
    /// point, and a 16-bit quantity specifying the page number, followed by
    /// the bytes to inject.
    ///
    /// Note This record MUST be processed before a STARTDOC Record.
    POSTSCRIPT_INJECTION {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// the record.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// this record type. The lower byte MUST match the lower byte of the
        /// RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that specifies the
        /// size, in bytes, of the Data field.
        byte_count: u16,
        /// Data (variable): An array of bytes of size ByteCount.
        data: Vec<u8>,
    },
    /// The POSTSCRIPT_PASSTHROUGH Record sends arbitrary data directly to the
    /// driver. The driver is expected to only process this data when in
    /// PostScript mode. For more information, see the POSTSCRIPT_IDENTIFY
    /// escape record.
    POSTSCRIPT_PASSTHROUGH {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// the record.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// this record type. The lower byte MUST match the lower byte of the
        /// RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that specifies the
        /// size, in bytes, of the Data field.
        byte_count: u16,
        /// Data (variable): An array of bytes of size ByteCount.
        data: Vec<u8>,
    },
    /// The OPENCHANNEL RECORD notifies the printer driver that a new print job
    /// is starting. This is the same function as a STARTDOC Record specified
    /// with a NULL document and output file name, data in raw mode, and a type
    /// of zero.
    OPENCHANNEL {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// the WMF record.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// this record type. The lower byte MUST match the lower byte of the
        /// RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that MUST be 0x0000.
        byte_count: u16,
    },
    /// The QUERYDIBSUPPORT Record queries the driver about its support for DIB
    /// Objects.
    QUERYDIBSUPPORT {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// the WMF record.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// this record type. The lower byte MUST match the lower byte of the
        /// RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that MUST be 0x0000.
        byte_count: u16,
    },
    /// The QUERYESCSUPPORT Record queries the printer driver to determine
    /// whether a specific WMF escape function is supported on the output
    /// device.
    QUERYESCSUPPORT {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// the record.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// this record type. The lower byte MUST match the lower byte of the
        /// RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that specifies the
        /// size, in bytes, of the Query field. This MUST be 0x0002.
        byte_count: u16,
        /// Query (2 bytes): A 16-bit unsigned integer that MUST be a value
        /// from the MetafileEscapes Enumeraton. This record specifies a query
        /// of whether this escape is supported.
        query: crate::wmf::parser::MetafileEscapes,
    },
    /// The SETCOLORTABLE Record sets the color table.
    SETCOLORTABLE {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// the record.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// this record type. The lower byte MUST match the lower byte of the
        /// RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that specifies the
        /// size, in bytes, of the ColorTable field.
        byte_count: u16,
        /// ColorTable (variable): A ByteCount length byte array containing the
        /// color table.
        color_table: Vec<u8>,
    },
    /// The SETCOPYCOUNT Record sets the number of copies.
    SETCOPYCOUNT {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// the record.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// this record type. The lower byte MUST match the lower byte of the
        /// RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that specifies the
        /// size, in bytes, of the CopyCount field. This MUST be 0x0002.
        byte_count: u16,
        /// CopyCount (2 bytes): A 16-bit unsigned integer that specifies the
        /// number of copies to print.
        copy_count: u16,
    },
    /// The SETLINECAP Record specifies the type of line-ending to use in
    /// subsequent graphics operations.
    SETLINECAP {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// the record.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// this record type. The low-order byte MUST match the low-order byte
        /// of the RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that specifies the
        /// size, in bytes, of the Cap field. This MUST be 0x0004.
        byte_count: u16,
        /// Cap (4 bytes): A 32-bit signed integer that defines the type of
        /// line cap. Possible values are specified in the PostScriptCap
        /// Enumeration table.
        cap: crate::wmf::parser::PostScriptCap,
    },
    /// The SETLINEJOIN Record specifies the type of line-joining to use in
    /// subsequent graphics operations.
    SETLINEJOIN {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// the record.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// this record type. The lower byte MUST match the lower byte of the
        /// RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that specifies the
        /// size, in bytes, of the Join field. This MUST be 0x0004.
        byte_count: u16,
        /// Join (4 bytes): A 32-bit signed integer that specifies the type of
        /// line join. Possible values are specified in PostScriptJoin
        /// Enumeration table.
        join: crate::wmf::parser::PostScriptJoin,
    },
    /// The SETMITERLIMIT Record sets the limit for the length of miter joins
    /// to use in subsequent graphics operations.
    SETMITERLIMIT {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// the record.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// this record type. The lower byte MUST match the lower byte of the
        /// RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that specifies the
        /// size, in bytes, of the MiterLimit field. This MUST be 0x0004.
        byte_count: u16,
        /// MiterLimit (4 bytes): A 32-bit signed integer that specifies the
        /// miter limit.
        miter_limit: i32,
    },
    /// The SPCLPASSTHROUGH2 Record enables documents to include private
    /// procedures and other resources to send to the printer driver.
    SPCLPASSTHROUGH2 {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// the record.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// this record type. The lower byte MUST match the lower byte of the
        /// RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that specifies the
        /// size, in bytes, of the record data that follows.
        byte_count: u16,
        /// Reserved (4 bytes): A 32-bit unsigned integer that is not used and
        /// MUST be ignored.
        reserved: u32,
        /// Size (2 bytes): A 16-bit unsigned integer that specifies the size,
        /// in bytes, of the RawData field.
        size: u16,
        /// RawData (variable): The Size-length byte array of unprocessed
        /// private data to send to the printer driver.
        raw_data: Vec<u8>,
    },
    /// The STARTDOC Record informs the printer driver that a new print job is
    /// starting.
    STARTDOC {
        /// RecordSize (4 bytes): A 32-bit unsigned integer that defines the
        /// number of WORD structures, defined in [MS-DTYP] section 2.2.61, in
        /// the record.
        record_size: crate::wmf::parser::RecordSize,
        /// RecordFunction (2 bytes): A 16-bit unsigned integer that defines
        /// this record type. The lower byte MUST match the lower byte of the
        /// RecordType Enumeration table value META_ESCAPE.
        record_function: u16,
        /// ByteCount (2 bytes): A 16-bit unsigned integer that specifies the
        /// size, in bytes, of the DocName field. This size MUST be less than
        /// 260.
        byte_count: u16,
        /// DocName (variable): A string of ByteCount 8-bit characters that
        /// contains the name of the document.
        doc_name: Vec<u8>,
    },
}

impl META_ESCAPE {
    #[cfg_attr(feature = "tracing", tracing::instrument(
        level = tracing::Level::TRACE,
        skip_all,
        fields(
            %record_size,
            record_function = %format!("{record_function:#06X}"),
        ),
        err(level = tracing::Level::ERROR, Display),
    ))]
    #[allow(clippy::too_many_lines)]
    pub fn parse<R: crate::wmf::Read>(
        buf: &mut R,
        mut record_size: crate::wmf::parser::RecordSize,
        record_function: u16,
    ) -> Result<Self, crate::wmf::parser::ParseError> {
        crate::wmf::parser::records::check_lower_byte_matches(
            record_function,
            crate::wmf::parser::RecordType::META_ESCAPE,
        )?;

        let (escape, escape_bytes) = crate::wmf::parser::MetafileEscapes::parse(buf)?;
        record_size.consume(escape_bytes);

        let record = match escape {
            crate::wmf::parser::MetafileEscapes::ABORTDOC => {
                Self::parse_as_ABORTDOC(buf, record_size, record_function)?
            }
            crate::wmf::parser::MetafileEscapes::BEGIN_PATH => {
                Self::parse_as_BEGIN_PATH(buf, record_size, record_function)?
            }
            crate::wmf::parser::MetafileEscapes::CHECKJPEGFORMAT => {
                Self::parse_as_CHECKJPEGFORMAT(buf, record_size, record_function)?
            }
            crate::wmf::parser::MetafileEscapes::CHECKPNGFORMAT => {
                Self::parse_as_CHECKPNGFORMAT(buf, record_size, record_function)?
            }
            crate::wmf::parser::MetafileEscapes::CLIP_TO_PATH => {
                Self::parse_as_CLIP_TO_PATH(buf, record_size, record_function)?
            }
            crate::wmf::parser::MetafileEscapes::CLOSECHANNEL => {
                Self::parse_as_CLOSECHANNEL(buf, record_size, record_function)?
            }
            crate::wmf::parser::MetafileEscapes::DOWNLOADFACE => {
                Self::parse_as_DOWNLOADFACE(buf, record_size, record_function)?
            }
            crate::wmf::parser::MetafileEscapes::DOWNLOADHEADER => {
                Self::parse_as_DOWNLOADHEADER(buf, record_size, record_function)?
            }
            crate::wmf::parser::MetafileEscapes::DRAWPATTERNRECT => {
                Self::parse_as_DRAWPATTERNRECT(buf, record_size, record_function)?
            }
            crate::wmf::parser::MetafileEscapes::ENCAPSULATED_POSTSCRIPT => {
                Self::parse_as_ENCAPSULATED_POSTSCRIPT(buf, record_size, record_function)?
            }
            crate::wmf::parser::MetafileEscapes::END_PATH => {
                Self::parse_as_END_PATH(buf, record_size, record_function)?
            }
            crate::wmf::parser::MetafileEscapes::ENDDOC => {
                Self::parse_as_ENDDOC(buf, record_size, record_function)?
            }
            crate::wmf::parser::MetafileEscapes::EPSPRINTING => {
                Self::parse_as_EPSPRINTING(buf, record_size, record_function)?
            }
            crate::wmf::parser::MetafileEscapes::EXTTEXTOUT => {
                Self::parse_as_EXTTEXTOUT(buf, record_size, record_function)?
            }
            crate::wmf::parser::MetafileEscapes::GETCOLORTABLE => {
                Self::parse_as_GETCOLORTABLE(buf, record_size, record_function)?
            }
            crate::wmf::parser::MetafileEscapes::GETDEVICEUNITS => {
                Self::parse_as_GETDEVICEUNITS(buf, record_size, record_function)?
            }
            crate::wmf::parser::MetafileEscapes::GETEXTENDEDTEXTMETRICS => {
                Self::parse_as_GETEXTENDEDTEXTMETRICS(buf, record_size, record_function)?
            }
            crate::wmf::parser::MetafileEscapes::GETFACENAME => {
                Self::parse_as_GETFACENAME(buf, record_size, record_function)?
            }
            crate::wmf::parser::MetafileEscapes::GETPAIRKERNTABLE => {
                Self::parse_as_GETPAIRKERNTABLE(buf, record_size, record_function)?
            }
            crate::wmf::parser::MetafileEscapes::GETPHYSPAGESIZE => {
                Self::parse_as_GETPHYSPAGESIZE(buf, record_size, record_function)?
            }
            crate::wmf::parser::MetafileEscapes::GETPRINTINGOFFSET => {
                Self::parse_as_GETPRINTINGOFFSET(buf, record_size, record_function)?
            }
            crate::wmf::parser::MetafileEscapes::GET_PS_FEATURESETTING => {
                Self::parse_as_GET_PS_FEATURESETTING(buf, record_size, record_function)?
            }
            crate::wmf::parser::MetafileEscapes::GETSCALINGFACTOR => {
                Self::parse_as_GETSCALINGFACTOR(buf, record_size, record_function)?
            }
            crate::wmf::parser::MetafileEscapes::META_ESCAPE_ENHANCED_METAFILE => {
                Self::parse_as_META_ESCAPE_ENHANCED_METAFILE(buf, record_size, record_function)?
            }
            crate::wmf::parser::MetafileEscapes::METAFILE_DRIVER => {
                Self::parse_as_METAFILE_DRIVER(buf, record_size, record_function)?
            }
            crate::wmf::parser::MetafileEscapes::NEWFRAME => {
                Self::parse_as_NEWFRAME(buf, record_size, record_function)?
            }
            crate::wmf::parser::MetafileEscapes::NEXTBAND => {
                Self::parse_as_NEXTBAND(buf, record_size, record_function)?
            }
            crate::wmf::parser::MetafileEscapes::PASSTHROUGH => {
                Self::parse_as_PASSTHROUGH(buf, record_size, record_function)?
            }
            crate::wmf::parser::MetafileEscapes::POSTSCRIPT_DATA => {
                Self::parse_as_POSTSCRIPT_DATA(buf, record_size, record_function)?
            }
            crate::wmf::parser::MetafileEscapes::POSTSCRIPT_IDENTIFY => {
                Self::parse_as_POSTSCRIPT_IDENTIFY(buf, record_size, record_function)?
            }
            crate::wmf::parser::MetafileEscapes::POSTSCRIPT_IGNORE => {
                Self::parse_as_POSTSCRIPT_IGNORE(buf, record_size, record_function)?
            }
            crate::wmf::parser::MetafileEscapes::POSTSCRIPT_INJECTION => {
                Self::parse_as_POSTSCRIPT_INJECTION(buf, record_size, record_function)?
            }
            crate::wmf::parser::MetafileEscapes::POSTSCRIPT_PASSTHROUGH => {
                Self::parse_as_POSTSCRIPT_PASSTHROUGH(buf, record_size, record_function)?
            }
            crate::wmf::parser::MetafileEscapes::OPENCHANNEL => {
                Self::parse_as_OPENCHANNEL(buf, record_size, record_function)?
            }
            crate::wmf::parser::MetafileEscapes::QUERYDIBSUPPORT => {
                Self::parse_as_QUERYDIBSUPPORT(buf, record_size, record_function)?
            }
            crate::wmf::parser::MetafileEscapes::QUERYESCSUPPORT => {
                Self::parse_as_QUERYESCSUPPORT(buf, record_size, record_function)?
            }
            crate::wmf::parser::MetafileEscapes::SETCOLORTABLE => {
                Self::parse_as_SETCOLORTABLE(buf, record_size, record_function)?
            }
            crate::wmf::parser::MetafileEscapes::SETCOPYCOUNT => {
                Self::parse_as_SETCOPYCOUNT(buf, record_size, record_function)?
            }
            crate::wmf::parser::MetafileEscapes::SETLINECAP => {
                Self::parse_as_SETLINECAP(buf, record_size, record_function)?
            }
            crate::wmf::parser::MetafileEscapes::SETLINEJOIN => {
                Self::parse_as_SETLINEJOIN(buf, record_size, record_function)?
            }
            crate::wmf::parser::MetafileEscapes::SETMITERLIMIT => {
                Self::parse_as_SETMITERLIMIT(buf, record_size, record_function)?
            }
            crate::wmf::parser::MetafileEscapes::SPCLPASSTHROUGH2 => {
                Self::parse_as_SPCLPASSTHROUGH2(buf, record_size, record_function)?
            }
            crate::wmf::parser::MetafileEscapes::STARTDOC => {
                Self::parse_as_STARTDOC(buf, record_size, record_function)?
            }
            v => {
                return Err(crate::wmf::parser::ParseError::NotSupported {
                    cause: format!("Metafile Escapes `{v:?}` is not supported"),
                });
            }
        };

        Ok(record)
    }
}
