/// The MetafileEscapes Enumeration specifies printer driver functionality that
/// might not be directly accessible through WMF records defined in the
/// RecordType Enumeration.
///
/// These values are used by Escape Record Types.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, strum::FromRepr, strum::EnumIter)]
#[repr(u16)]
pub enum MetafileEscapes {
    /// Notifies the printer driver that the application has finished writing
    /// to a page.
    NEWFRAME = 0x0001,
    /// Stops processing the current document.
    ABORTDOC = 0x0002,
    /// Notifies the printer driver that the application has finished writing
    /// to a band.
    NEXTBAND = 0x0003,
    /// Sets color table values.
    SETCOLORTABLE = 0x0004,
    /// Gets color table values.
    GETCOLORTABLE = 0x0005,
    /// Causes all pending output to be flushed to the output device.
    FLUSHOUT = 0x0006,
    /// Indicates that the printer driver SHOULD print text only, and no
    /// graphics.
    DRAFTMODE = 0x0007,
    /// Queries a printer driver to determine whether a specific escape
    /// function is supported on the output device it drives.
    QUERYESCSUPPORT = 0x0008,
    /// Sets the application-defined function that allows a print job to be
    /// canceled during printing.
    SETABORTPROC = 0x0009,
    /// Notifies the printer driver that a new print job is starting.
    STARTDOC = 0x000A,
    /// Notifies the printer driver that the current print job is ending.
    ENDDOC = 0x000B,
    /// Retrieves the physical page size currently selected on an output
    /// device.
    GETPHYSPAGESIZE = 0x000C,
    /// Retrieves the offset from the upper-left corner of the physical page
    /// where the actual printing or drawing begins.
    GETPRINTINGOFFSET = 0x000D,
    /// Retrieves the scaling factors for the x-axis and the y-axis of a
    /// printer.
    GETSCALINGFACTOR = 0x000E,
    /// Used to embed an enhanced metafile format (EMF) metafile within a WMF
    /// metafile.
    META_ESCAPE_ENHANCED_METAFILE = 0x000F,
    /// Sets the width of a pen in pixels.
    SETPENWIDTH = 0x0010,
    /// Sets the number of copies.
    SETCOPYCOUNT = 0x0011,
    /// Sets the source, such as a particular paper tray or bin on a printer,
    /// for output forms.
    SETPAPERSOURCE = 0x0012,
    /// This record passes through arbitrary data.
    PASSTHROUGH = 0x0013,
    /// Gets information concerning graphics technology that is supported on a
    /// device.
    GETTECHNOLOGY = 0x0014,
    /// Specifies the line-drawing mode to use in output to a device.
    SETLINECAP = 0x0015,
    /// Specifies the line-joining mode to use in output to a device.
    SETLINEJOIN = 0x0016,
    /// Sets the limit for the length of miter joins to use in output to a
    /// device.
    SETMITERLIMIT = 0x0017,
    /// Retrieves or specifies settings concerning banding on a device, such as
    /// the number ofbands.
    BANDINFO = 0x0018,
    /// Draws a rectangle with a defined pattern.
    DRAWPATTERNRECT = 0x0019,
    /// Retrieves the physical pen size currently defined on a device.
    GETVECTORPENSIZE = 0x001A,
    /// Retrieves the physical brush size currently defined on a device.
    GETVECTORBRUSHSIZE = 0x001B,
    /// Enables or disables double-sided (duplex) printing on a device.
    ENABLEDUPLEX = 0x001C,
    /// Retrieves or specifies the source of output forms on a device.
    GETSETPAPERBINS = 0x001D,
    /// Retrieves or specifies the paper orientation on a device.
    GETSETPRINTORIENT = 0x001E,
    /// Retrieves information concerning the sources of different forms on an
    /// output device.
    ENUMPAPERBINS = 0x001F,
    /// Specifies the scaling of device-independent bitmaps (DIBs).
    SETDIBSCALING = 0x0020,
    /// Indicates the start and end of an encapsulated PostScript (EPS)
    /// section.
    EPSPRINTING = 0x0021,
    /// Queries a printer driver for paper dimensions and other forms data.
    ENUMPAPERMETRICS = 0x0022,
    /// Retrieves or specifies paper dimensions and other forms data on an
    /// output device.
    GETSETPAPERMETRICS = 0x0023,
    /// Sends arbitrary PostScript data to an output device.
    POSTSCRIPT_DATA = 0x0025,
    /// Notifies an output device to ignore PostScript data.
    POSTSCRIPT_IGNORE = 0x0026,
    /// Gets the device units currently configured on an output device.
    GETDEVICEUNITS = 0x002A,
    /// Gets extended text metrics currently configured on an output device.
    GETEXTENDEDTEXTMETRICS = 0x0100,
    /// Gets the font kern table currently defined on an output device.
    GETPAIRKERNTABLE = 0x0102,
    /// Draws text using the currently selected font, background color, and
    /// text color.
    EXTTEXTOUT = 0x0200,
    /// Gets the font face name currently configured on a device.
    GETFACENAME = 0x0201,
    /// Sets the font face name on a device.
    DOWNLOADFACE = 0x0202,
    /// Queries a printer driver about the support for metafiles on an output
    /// device.
    METAFILE_DRIVER = 0x0801,
    /// Queries the printer driver about its support for DIBs on an output
    /// device.
    QUERYDIBSUPPORT = 0x0C01,
    /// Opens a path.
    BEGIN_PATH = 0x1000,
    /// Defines a clip region that is bounded by a path. The input MUST be a
    /// 16-bit quantity that defines the action to take.
    CLIP_TO_PATH = 0x1001,
    /// Ends a path.
    END_PATH = 0x1002,
    /// The same as STARTDOC specified with a NULL document and output
    /// filename, data in raw mode, and a type of zero.
    OPENCHANNEL = 0x100E,
    /// Instructs the printer driver to download sets of PostScript procedures.
    DOWNLOADHEADER = 0x100F,
    /// The same as ENDDOC. See OPENCHANNEL.
    CLOSECHANNEL = 0x1010,
    /// Sends arbitrary data directly to a printer driver, which is expected to
    /// process this data only when in PostScript mode. See
    /// POSTSCRIPT_IDENTIFY. (Windows NT 3.1, Windows NT 3.5, Windows NT 3.51,
    /// Windows 95, Windows NT 4.0, Windows 98, and Windows Millennium Edition:
    /// This functionality is not supported.)
    POSTSCRIPT_PASSTHROUGH = 0x1013,
    /// Sends arbitrary data directly to the printer driver.
    ENCAPSULATED_POSTSCRIPT = 0x1014,
    /// Sets the printer driver to either PostScript or GDI mode. (Windows NT
    /// 3.1, Windows NT 3.5, Windows NT 3.51, Windows 95, Windows NT 4.0,
    /// Windows 98, and Windows Millennium Edition: This functionality is not
    /// supported.)
    POSTSCRIPT_IDENTIFY = 0x1015,
    /// Inserts a block of raw data into a PostScript stream. The input MUST be
    /// a 32-bit quantity specifying the number of bytes to inject, a 16-bit
    /// quantity specifying the injection point, and a 16-bit quantity
    /// specifying the page number, followed by the bytes to inject. (Windows
    /// NT 3.1, Windows NT 3.5, Windows NT 3.51, Windows 95, Windows NT 4.0,
    /// Windows 98, and Windows Millennium Edition: This functionality is not
    /// supported.)
    POSTSCRIPT_INJECTION = 0x1016,
    /// Checks whether the printer supports a JPEG image. (Windows NT 3.1,
    /// Windows NT 3.5, Windows NT 3.51, Windows 95, Windows NT 4.0, Windows
    /// 98, and Windows Millennium Edition: This functionality is not
    /// supported.)
    CHECKJPEGFORMAT = 0x1017,
    /// Checks whether the printer supports a PNG image. (Windows NT 3.1,
    /// Windows NT 3.5, Windows NT 3.51, Windows 95, Windows NT 4.0, Windows
    /// 98, and Windows Millennium Edition: This functionality is not
    /// supported.)
    CHECKPNGFORMAT = 0x1018,
    /// Gets information on a specified feature setting for a PostScript
    /// printer driver. (Windows NT 3.1, Windows NT 3.5, Windows NT 3.51,
    /// Windows 95, Windows NT 4.0, Windows 98, and Windows Millennium Edition:
    /// This functionality is not supported.)
    GET_PS_FEATURESETTING = 0x1019,
    /// Enables applications to write documents to a file or to a printer in
    /// XML Paper Specification (XPS) format. (Windows NT 3.1, Windows NT 3.5,
    /// Windows NT 3.51, Windows 95, Windows NT 4.0, Windows 98, Windows
    /// Millennium Edition, Windows 2000, Windows XP, and Windows Server 2003:
    /// This functionality is not supported.)
    MXDC_ESCAPE = 0x101A,
    /// Enables applications to include private procedures and other arbitrary
    /// data in documents. (Windows NT 3.1, Windows NT 3.5, Windows NT 3.51,
    /// Windows 95, Windows NT 4.0, Windows 98, and Windows Millennium Edition:
    /// This functionality is not supported.)
    SPCLPASSTHROUGH2 = 0x11D8,
}

crate::wmf::parser::constants::impl_parser!(MetafileEscapes, u16);
