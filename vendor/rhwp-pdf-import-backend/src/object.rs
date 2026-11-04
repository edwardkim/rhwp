//! PDF object types.

use crate::error::{Error, Result};

/// PDF object representation.
#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    /// Null object
    Null,
    /// Boolean value
    Boolean(bool),
    /// Integer value
    Integer(i64),
    /// Real (floating-point) value
    Real(f64),
    /// String (byte array)
    String(Vec<u8>),
    /// Name (starting with /)
    Name(String),
    /// Array of objects
    Array(Vec<Object>),
    /// Dictionary (key-value pairs)
    Dictionary(std::collections::HashMap<String, Object>),
    /// Stream (dictionary + data)
    Stream {
        /// Stream dictionary
        dict: std::collections::HashMap<String, Object>,
        /// Stream data
        data: crate::import_limits::StreamData,
    },
    /// Indirect object reference
    Reference(ObjectRef),
}

/// Reference to an indirect object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
pub struct ObjectRef {
    /// Object number
    pub id: u32,
    /// Generation number
    pub gen: u16,
}

impl ObjectRef {
    /// Create a new object reference.
    pub fn new(id: u32, gen: u16) -> Self {
        Self { id, gen }
    }
}

impl std::fmt::Display for ObjectRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} R", self.id, self.gen)
    }
}

/// Encode a Rust `&str` as a PDF text string byte sequence.
///
/// Uses PDFDocEncoding (direct code-point byte) for strings whose characters
/// all fall within U+0000–U+00FF.  Falls back to UTF-16BE with leading BOM
/// (`0xFE 0xFF`) for any string that contains a code point above U+00FF.
///
/// This is the encoding required by ISO 32000-2 §7.9.2 for all PDF text
/// strings (metadata fields, annotation text, bookmark titles, form field
/// values, page-label prefixes, etc.).
pub fn encode_pdf_text_string(s: &str) -> Vec<u8> {
    if s.chars().all(|c| (c as u32) <= 0x00FF) {
        s.chars().map(|c| c as u8).collect()
    } else {
        let mut buf = vec![0xFE_u8, 0xFF];
        for unit in s.encode_utf16() {
            buf.push((unit >> 8) as u8);
            buf.push((unit & 0xFF) as u8);
        }
        buf
    }
}

impl Object {
    /// Create a PDF text string object from a Rust string.
    ///
    /// Accepts `&str`, `String`, or any type that implements `AsRef<str>`.
    /// Encodes using PDFDocEncoding for characters within U+0000–U+00FF,
    /// or UTF-16BE with BOM for strings that contain characters above U+00FF.
    pub fn text_string(s: impl AsRef<str>) -> Self {
        Object::String(encode_pdf_text_string(s.as_ref()))
    }

    /// Get the type name of this object (without data).
    ///
    /// Returns a human-readable type name like "String", "Array", "Dictionary", etc.
    /// without including the actual data content.
    pub fn type_name(&self) -> &'static str {
        match self {
            Object::Null => "Null",
            Object::Boolean(_) => "Boolean",
            Object::Integer(_) => "Integer",
            Object::Real(_) => "Real",
            Object::String(_) => "String",
            Object::Name(_) => "Name",
            Object::Array(_) => "Array",
            Object::Dictionary(_) => "Dictionary",
            Object::Stream { .. } => "Stream",
            Object::Reference(_) => "Reference",
        }
    }

    /// Try to cast to integer.
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Object::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Try to cast to name.
    pub fn as_name(&self) -> Option<&str> {
        match self {
            Object::Name(s) => Some(s),
            _ => None,
        }
    }

    /// Try to cast to dictionary. Works for both Dictionary and Stream objects.
    pub fn as_dict(&self) -> Option<&std::collections::HashMap<String, Object>> {
        match self {
            Object::Dictionary(d) => Some(d),
            Object::Stream { dict, .. } => Some(dict),
            _ => None,
        }
    }

    /// Try to cast to array.
    pub fn as_array(&self) -> Option<&Vec<Object>> {
        match self {
            Object::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// Try to cast to reference.
    pub fn as_reference(&self) -> Option<ObjectRef> {
        match self {
            Object::Reference(r) => Some(*r),
            _ => None,
        }
    }

    /// Try to cast to boolean.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Object::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Try to cast to real number.
    pub fn as_real(&self) -> Option<f64> {
        match self {
            Object::Real(r) => Some(*r),
            _ => None,
        }
    }

    /// Try to cast to string (bytes).
    pub fn as_string(&self) -> Option<&[u8]> {
        match self {
            Object::String(s) => Some(s),
            _ => None,
        }
    }

    /// Check if object is null.
    pub fn is_null(&self) -> bool {
        matches!(self, Object::Null)
    }

    /// Decode stream data using filters specified in the stream dictionary.
    ///
    /// This is a convenience method that calls `decode_stream_data_with_decryption`
    /// with no encryption parameters.
    ///
    /// # Returns
    ///
    /// The decoded stream data, or an error if this is not a stream object
    /// or if decoding fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use pdf_oxide::object::Object;
    ///
    /// # fn example(stream_obj: Object) -> Result<(), Box<dyn std::error::Error>> {
    /// let decoded_data = stream_obj.decode_stream_data()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn decode_stream_data(&self) -> Result<Vec<u8>> {
        self.decode_stream_data_with_decryption(None, 0, 0)
    }

    /// Decode bounded stream bytes; the document loader decrypts authenticated
    /// encrypted streams before passing them to this filter decoder.
    pub fn decode_stream_data_with_budget(
        &self,
        budget: &crate::import_limits::ImportBudget,
    ) -> Result<Vec<u8>> {
        match self {
            Object::Stream { dict, data } => {
                crate::stream_parameters_limited::decode_resolved(dict, data, budget)
            },
            Object::Dictionary(_) | Object::Null => {
                budget.check()?;
                Ok(Vec::new())
            },
            _ => Err(Error::InvalidObjectType {
                expected: "Stream".into(),
                found: self.type_name().into(),
            }),
        }
    }

    /// Decode stream data with optional decryption.
    ///
    /// PDF Spec: Section 7.6.2 - General Encryption Algorithm states that streams
    /// must be decrypted BEFORE applying filters (decompression).
    ///
    /// # Arguments
    ///
    /// * `decryption_fn` - Optional decryption function (from EncryptionHandler)
    /// * `obj_num` - Object number (for encryption key derivation)
    /// * `gen_num` - Generation number (for encryption key derivation)
    ///
    /// # Returns
    ///
    /// The decoded stream data, or an error if decoding/decryption fails.
    pub fn decode_stream_data_with_decryption(
        &self,
        decryption_fn: Option<&dyn Fn(&[u8]) -> Result<Vec<u8>>>,
        obj_num: u32,
        gen_num: u32,
    ) -> Result<Vec<u8>> {
        match self {
            Object::Stream { dict, data } => {
                if let Some(budget) = data.budget() {
                    if decryption_fn.is_some() {
                        return Err(Error::Unsupported("encrypted limited import".into()));
                    }
                    return self.decode_stream_data_with_budget(budget);
                }
                // Step 1: Decrypt stream data BEFORE applying filters
                // PDF Spec: Section 7.6.2 - Encryption must be applied before compression
                //
                // IMPORTANT: For encrypted streams, we must NOT trim whitespace before decryption
                // because encrypted data is binary and trimming might corrupt it (especially for AES
                // where the first 16 bytes are the IV). We only trim for unencrypted streams.
                let decrypted_data = if let Some(decrypt) = decryption_fn {
                    log::debug!(
                        "Decrypting stream for object {} {} (length: {} bytes)",
                        obj_num,
                        gen_num,
                        data.len()
                    );
                    // For encrypted streams, pass raw data without trimming
                    match decrypt(data) {
                        Ok(data) => {
                            log::debug!("Decryption successful: {} bytes", data.len());
                            data
                        },
                        Err(e) => {
                            log::error!(
                                "Decryption failed for object {} {}: {}",
                                obj_num,
                                gen_num,
                                e
                            );
                            return Err(e);
                        },
                    }
                } else {
                    // `parse_stream_data` in the parser already consumes
                    // exactly one EOL after the `stream` keyword per
                    // ISO 32000-1:2008 §7.3.8.1, so `data` begins with the
                    // first byte of actual stream content. Re-trimming
                    // CR/LF here would corrupt binary streams that
                    // legitimately start with 0x0A or 0x0D — e.g. an
                    // Indexed palette whose first CMYK byte is 0x0D.
                    data.to_vec()
                };

                // Step 2: Apply filters (decompression)
                // PDF Spec: ISO 32000-1:2008, Section 7.3.8.2 - Stream Objects
                let filters = dict
                    .get("Filter")
                    .map(extract_filter_names)
                    .unwrap_or_default();

                if filters.is_empty() {
                    // No filters, return decrypted data
                    Ok(decrypted_data)
                } else {
                    // Get decode parameters if present
                    // PDF Spec: ISO 32000-1:2008, Section 7.4.2 - DecodeParms
                    let decode_params = extract_decode_params(dict.get("DecodeParms"));

                    // Decode using filter pipeline with parameters
                    crate::decoders::decode_stream_with_params(
                        &decrypted_data,
                        &filters,
                        decode_params.as_ref(),
                    )
                }
            },
            Object::Dictionary(dict) => {
                // Per ISO 32000, every stream is a dictionary. Some PDFs (e.g.,
                // SafeDocs Dialect-StreamIsDict.pdf) store objects as plain
                // dictionaries where a stream is expected. Treat as empty stream.
                log::warn!("Dictionary used where Stream expected, treating as empty stream");
                let filters = dict
                    .get("Filter")
                    .map(extract_filter_names)
                    .unwrap_or_default();
                if filters.is_empty() {
                    Ok(Vec::new())
                } else {
                    let decode_params = extract_decode_params(dict.get("DecodeParms"));
                    crate::decoders::decode_stream_with_params(
                        &[],
                        &filters,
                        decode_params.as_ref(),
                    )
                }
            },
            _ => Err(Error::InvalidObjectType {
                expected: "Stream".to_string(),
                found: self.type_name().to_string(),
            }),
        }
    }
}

/// Extract filter names from a Filter object.
///
/// The Filter entry can be either:
/// - A single Name (e.g., /FlateDecode)
/// - An Array of Names (e.g., [/ASCII85Decode /FlateDecode])
fn extract_filter_names(filter_obj: &Object) -> Vec<String> {
    match filter_obj {
        Object::Name(name) => vec![name.clone()],
        Object::Array(arr) => arr
            .iter()
            .filter_map(|obj| obj.as_name().map(|s| s.to_string()))
            .collect(),
        _ => vec![],
    }
}

/// Extract decode parameters from a DecodeParms object.
///
/// PDF Spec: ISO 32000-1:2008, Section 7.4.2 - LZWDecode and FlateDecode Parameters
///
/// The DecodeParms entry can be:
/// - A dictionary (for single filter)
/// - An array of dictionaries (for multiple filters)
/// - Null or absent (no parameters)
///
/// This function extracts predictor parameters used for PNG/TIFF encoding.
fn extract_decode_params(params_obj: Option<&Object>) -> Option<crate::decoders::DecodeParams> {
    let dict = match params_obj? {
        Object::Dictionary(d) => d,
        Object::Array(arr) => {
            // For array, take the first non-null dictionary
            arr.iter().filter_map(|obj| obj.as_dict()).next()?
        },
        _ => return None,
    };

    // Extract predictor parameters per PDF Spec Table 3.7
    let predictor = dict
        .get("Predictor")
        .and_then(|obj| obj.as_integer())
        .unwrap_or(1); // Default: no prediction

    let columns = dict
        .get("Columns")
        .and_then(|obj| obj.as_integer())
        .unwrap_or(1) as usize;

    let colors = dict
        .get("Colors")
        .and_then(|obj| obj.as_integer())
        .unwrap_or(1) as usize;

    let bits_per_component = dict
        .get("BitsPerComponent")
        .and_then(|obj| obj.as_integer())
        .unwrap_or(8) as usize;

    Some(crate::decoders::DecodeParams {
        predictor,
        columns,
        colors,
        bits_per_component,
    })
}

/// Extract CCITT-specific decode parameters from a DecodeParms object.
///
/// PDF Spec: ISO 32000-1:2008, Section 7.4.6 - CCITTFaxDecode Filter Parameters
///
/// The DecodeParms entry can be:
/// - A dictionary (for single filter)
/// - An array of dictionaries (for multiple filters)
/// - Null or absent (no parameters, use defaults)
///
/// CCITT parameters:
/// - /K: Group indicator (-1=Group 4, 0=Group 3 1-D, >0=Group 3 2-D)
/// - /Columns: Image width in pixels
/// - /Rows: Image height in pixels (optional)
/// - /BlackIs1: Pixel interpretation (false=white is 0, true=white is 1)
/// - /EndOfLine: Include EOL code (default false)
/// - /EncodedByteAlign: Byte-aligned encoding (default false)
/// - /EndOfBlock: Include RTC code (default true)
pub fn extract_ccitt_params(params_obj: Option<&Object>) -> Option<crate::decoders::CcittParams> {
    extract_ccitt_params_with_width(params_obj, None)
}

/// Extract CCITT decompression parameters from a PDF object with optional width override.
///
/// This function extracts CCITT Group 3 or Group 4 decompression parameters from a PDF
/// /DecodeParms dictionary. If image_width is provided, it will be used as the /Columns
/// parameter, overriding any value in the dictionary.
///
/// # Arguments
/// * `params_obj` - Optional PDF object containing CCITT parameters (Dictionary or Array)
/// * `image_width` - Optional width override to use as /Columns parameter
///
/// # Returns
/// Some(CcittParams) if valid parameters are found, None otherwise
pub fn extract_ccitt_params_with_width(
    params_obj: Option<&Object>,
    image_width: Option<u32>,
) -> Option<crate::decoders::CcittParams> {
    let dict = match params_obj? {
        Object::Dictionary(d) => d,
        Object::Array(arr) => {
            // For array, take the first non-null dictionary
            arr.iter().filter_map(|obj| obj.as_dict()).next()?
        },
        _ => return None,
    };

    // Extract CCITT parameters with PDF defaults.
    // ISO 32000-1:2008 Table 11: /K absent defaults to 0 (pure 1-D Group 3),
    // not Group 4 (-2) or mixed Group 3/4 (K > 0).
    let k = dict.get("K").and_then(|obj| obj.as_integer()).unwrap_or(0); // Default: Group 3, 1-D

    let columns = dict
        .get("Columns")
        .and_then(|obj| obj.as_integer())
        .map(|v| v as u32)
        .or(image_width)
        .unwrap_or(1);

    let rows = dict
        .get("Rows")
        .and_then(|obj| obj.as_integer())
        .map(|v| v as u32);

    let black_is_1 = dict
        .get("BlackIs1")
        .and_then(|obj| obj.as_bool())
        .unwrap_or(false); // PDF default: white=0, black=1

    let end_of_line = dict
        .get("EndOfLine")
        .and_then(|obj| obj.as_bool())
        .unwrap_or(false); // PDF default: no EOL

    let encoded_byte_align = dict
        .get("EncodedByteAlign")
        .and_then(|obj| obj.as_bool())
        .unwrap_or(false); // PDF default: no alignment

    let end_of_block = dict
        .get("EndOfBlock")
        .and_then(|obj| obj.as_bool())
        .unwrap_or(true); // PDF default: RTC code present

    Some(crate::decoders::CcittParams {
        k,
        columns,
        rows,
        black_is_1,
        end_of_line,
        encoded_byte_align,
        end_of_block,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_object_integer() {
        let obj = Object::Integer(42);
        assert_eq!(obj.as_integer(), Some(42));
        assert!(obj.as_name().is_none());
        assert!(!obj.is_null());
    }

    #[test]
    fn test_object_name() {
        let obj = Object::Name("Type".to_string());
        assert_eq!(obj.as_name(), Some("Type"));
        assert!(obj.as_integer().is_none());
    }

    #[test]
    fn test_object_bool() {
        let obj = Object::Boolean(true);
        assert_eq!(obj.as_bool(), Some(true));
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_object_real() {
        let obj = Object::Real(3.14);
        assert_eq!(obj.as_real(), Some(3.14));
    }

    #[test]
    fn test_object_string() {
        let obj = Object::String(b"Hello".to_vec());
        assert_eq!(obj.as_string(), Some(&b"Hello"[..]));
    }

    #[test]
    fn test_object_null() {
        let obj = Object::Null;
        assert!(obj.is_null());
        assert!(obj.as_integer().is_none());
    }

    #[test]
    fn test_object_array() {
        let obj = Object::Array(vec![Object::Integer(1), Object::Integer(2)]);
        let arr = obj.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_integer(), Some(1));
    }

    #[test]
    fn test_object_dictionary() {
        let mut dict = HashMap::new();
        dict.insert("Type".to_string(), Object::Name("Page".to_string()));
        let obj = Object::Dictionary(dict);

        let d = obj.as_dict().unwrap();
        assert_eq!(d.get("Type").unwrap().as_name(), Some("Page"));
    }

    #[test]
    fn test_object_stream_dict_access() {
        let mut dict = HashMap::new();
        dict.insert("Length".to_string(), Object::Integer(100));
        let obj = Object::Stream {
            dict,
            data: crate::import_limits::StreamData::from_static(b"stream data"),
        };

        // Stream objects should also be accessible as dictionaries
        let d = obj.as_dict().unwrap();
        assert_eq!(d.get("Length").unwrap().as_integer(), Some(100));
    }

    #[test]
    fn test_object_reference() {
        let obj_ref = ObjectRef::new(10, 0);
        let obj = Object::Reference(obj_ref);

        assert_eq!(obj.as_reference(), Some(obj_ref));
        assert_eq!(obj_ref.id, 10);
        assert_eq!(obj_ref.gen, 0);
    }

    #[test]
    fn test_object_ref_display() {
        let obj_ref = ObjectRef::new(10, 0);
        assert_eq!(format!("{}", obj_ref), "10 0 R");
    }

    #[test]
    fn test_object_clone() {
        let obj = Object::Integer(42);
        let cloned = obj.clone();
        assert_eq!(obj, cloned);
    }

    #[test]
    fn test_object_ref_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(ObjectRef::new(1, 0));
        set.insert(ObjectRef::new(2, 0));
        set.insert(ObjectRef::new(1, 0)); // Duplicate

        assert_eq!(set.len(), 2); // Should only have 2 unique refs
    }

    #[test]
    fn test_decode_stream_no_filter() {
        let mut dict = HashMap::new();
        dict.insert("Length".to_string(), Object::Integer(5));
        let obj = Object::Stream {
            dict,
            data: crate::import_limits::StreamData::from_static(b"Hello"),
        };

        let decoded = obj.decode_stream_data().unwrap();
        assert_eq!(decoded, b"Hello");
    }

    #[test]
    fn test_decode_stream_single_filter() {
        let mut dict = HashMap::new();
        dict.insert("Filter".to_string(), Object::Name("ASCIIHexDecode".to_string()));
        let obj = Object::Stream {
            dict,
            data: crate::import_limits::StreamData::from_static(b"48656C6C6F"), // "Hello" in hex
        };

        let decoded = obj.decode_stream_data().unwrap();
        assert_eq!(decoded, b"Hello");
    }

    #[test]
    fn test_decode_stream_filter_array() {
        let mut dict = HashMap::new();
        // Note: filters are applied in order - first ASCII85, then what it produces
        dict.insert(
            "Filter".to_string(),
            Object::Array(vec![Object::Name("ASCIIHexDecode".to_string())]),
        );
        let obj = Object::Stream {
            dict,
            data: crate::import_limits::StreamData::from_static(b"48656C6C6F"),
        };

        let decoded = obj.decode_stream_data().unwrap();
        assert_eq!(decoded, b"Hello");
    }

    #[test]
    fn test_decode_stream_not_a_stream() {
        let obj = Object::Integer(42);
        let result = obj.decode_stream_data();
        assert!(result.is_err());
        match result {
            Err(Error::InvalidObjectType { expected, found }) => {
                assert_eq!(expected, "Stream");
                assert_eq!(found, "Integer");
            },
            _ => panic!("Expected InvalidObjectType error"),
        }
    }

    #[test]
    fn test_decode_dictionary_as_stream() {
        let mut dict = HashMap::new();
        dict.insert("Length".to_string(), Object::Integer(0));
        let obj = Object::Dictionary(dict);

        let decoded = obj.decode_stream_data().unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn test_decode_dictionary_as_stream_with_filter() {
        let mut dict = HashMap::new();
        dict.insert("Filter".to_string(), Object::Name("ASCIIHexDecode".to_string()));
        let obj = Object::Dictionary(dict);

        // ASCIIHexDecode on empty data should produce empty output
        let decoded = obj.decode_stream_data().unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn test_extract_filter_names_single() {
        let filter = Object::Name("FlateDecode".to_string());
        let names = extract_filter_names(&filter);
        assert_eq!(names, vec!["FlateDecode"]);
    }

    #[test]
    fn test_extract_filter_names_array() {
        let filter = Object::Array(vec![
            Object::Name("ASCII85Decode".to_string()),
            Object::Name("FlateDecode".to_string()),
        ]);
        let names = extract_filter_names(&filter);
        assert_eq!(names, vec!["ASCII85Decode", "FlateDecode"]);
    }

    #[test]
    fn test_extract_filter_names_invalid() {
        let filter = Object::Integer(42);
        let names = extract_filter_names(&filter);
        assert!(names.is_empty());
    }

    // ---- Tests for type_name ----

    #[test]
    fn test_type_name_all_variants() {
        assert_eq!(Object::Null.type_name(), "Null");
        assert_eq!(Object::Boolean(true).type_name(), "Boolean");
        assert_eq!(Object::Integer(0).type_name(), "Integer");
        assert_eq!(Object::Real(0.0).type_name(), "Real");
        assert_eq!(Object::String(vec![]).type_name(), "String");
        assert_eq!(Object::Name("X".to_string()).type_name(), "Name");
        assert_eq!(Object::Array(vec![]).type_name(), "Array");
        assert_eq!(Object::Dictionary(HashMap::new()).type_name(), "Dictionary");
        assert_eq!(
            Object::Stream {
                dict: HashMap::new(),
                data: crate::import_limits::StreamData::new()
            }
            .type_name(),
            "Stream"
        );
        assert_eq!(Object::Reference(ObjectRef::new(1, 0)).type_name(), "Reference");
    }

    // ---- Tests for as_* methods returning None on wrong type ----

    #[test]
    fn test_as_integer_returns_none_for_non_integer() {
        assert!(Object::Null.as_integer().is_none());
        assert!(Object::Boolean(true).as_integer().is_none());
        assert!(Object::Real(1.0).as_integer().is_none());
        assert!(Object::String(vec![]).as_integer().is_none());
        assert!(Object::Name("X".to_string()).as_integer().is_none());
        assert!(Object::Array(vec![]).as_integer().is_none());
        assert!(Object::Dictionary(HashMap::new()).as_integer().is_none());
        assert!(Object::Reference(ObjectRef::new(1, 0))
            .as_integer()
            .is_none());
    }

    #[test]
    fn test_as_name_returns_none_for_non_name() {
        assert!(Object::Null.as_name().is_none());
        assert!(Object::Integer(1).as_name().is_none());
        assert!(Object::Boolean(true).as_name().is_none());
        assert!(Object::Real(1.0).as_name().is_none());
        assert!(Object::String(vec![]).as_name().is_none());
        assert!(Object::Array(vec![]).as_name().is_none());
    }

    #[test]
    fn test_as_dict_returns_none_for_non_dict() {
        assert!(Object::Null.as_dict().is_none());
        assert!(Object::Integer(1).as_dict().is_none());
        assert!(Object::Boolean(true).as_dict().is_none());
        assert!(Object::Real(1.0).as_dict().is_none());
        assert!(Object::String(vec![]).as_dict().is_none());
        assert!(Object::Name("X".to_string()).as_dict().is_none());
        assert!(Object::Array(vec![]).as_dict().is_none());
        assert!(Object::Reference(ObjectRef::new(1, 0)).as_dict().is_none());
    }

    #[test]
    fn test_as_array_returns_none_for_non_array() {
        assert!(Object::Null.as_array().is_none());
        assert!(Object::Integer(1).as_array().is_none());
        assert!(Object::Dictionary(HashMap::new()).as_array().is_none());
        assert!(Object::Name("X".to_string()).as_array().is_none());
    }

    #[test]
    fn test_as_reference_returns_none_for_non_reference() {
        assert!(Object::Null.as_reference().is_none());
        assert!(Object::Integer(1).as_reference().is_none());
        assert!(Object::Name("X".to_string()).as_reference().is_none());
        assert!(Object::Dictionary(HashMap::new()).as_reference().is_none());
    }

    #[test]
    fn test_as_bool_returns_none_for_non_bool() {
        assert!(Object::Null.as_bool().is_none());
        assert!(Object::Integer(1).as_bool().is_none());
        assert!(Object::Real(1.0).as_bool().is_none());
        assert!(Object::String(vec![]).as_bool().is_none());
    }

    #[test]
    fn test_as_real_returns_none_for_non_real() {
        assert!(Object::Null.as_real().is_none());
        assert!(Object::Integer(1).as_real().is_none());
        assert!(Object::Boolean(true).as_real().is_none());
        assert!(Object::String(vec![]).as_real().is_none());
    }

    #[test]
    fn test_as_string_returns_none_for_non_string() {
        assert!(Object::Null.as_string().is_none());
        assert!(Object::Integer(1).as_string().is_none());
        assert!(Object::Boolean(true).as_string().is_none());
        assert!(Object::Real(1.0).as_string().is_none());
        assert!(Object::Name("X".to_string()).as_string().is_none());
    }

    #[test]
    fn test_is_null_returns_false_for_non_null() {
        assert!(!Object::Integer(0).is_null());
        assert!(!Object::Boolean(false).is_null());
        assert!(!Object::Real(0.0).is_null());
        assert!(!Object::String(vec![]).is_null());
        assert!(!Object::Name("".to_string()).is_null());
        assert!(!Object::Array(vec![]).is_null());
        assert!(!Object::Dictionary(HashMap::new()).is_null());
    }

    // ---- Tests for ObjectRef ----

    #[test]
    fn test_object_ref_display_with_non_zero_gen() {
        let obj_ref = ObjectRef::new(42, 3);
        assert_eq!(format!("{}", obj_ref), "42 3 R");
    }

    #[test]
    fn test_object_ref_equality() {
        let a = ObjectRef::new(5, 0);
        let b = ObjectRef::new(5, 0);
        let c = ObjectRef::new(5, 1);
        let d = ObjectRef::new(6, 0);
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_ne!(a, d);
    }

    #[test]
    fn test_object_ref_copy() {
        let a = ObjectRef::new(10, 0);
        let b = a; // Copy
        assert_eq!(a, b);
        // a is still usable after copy
        assert_eq!(a.id, 10);
    }

    // ---- Tests for extract_filter_names edge cases ----

    #[test]
    fn test_extract_filter_names_array_with_non_names() {
        let filter = Object::Array(vec![
            Object::Name("FlateDecode".to_string()),
            Object::Integer(42),
            Object::Name("LZWDecode".to_string()),
        ]);
        let names = extract_filter_names(&filter);
        assert_eq!(names, vec!["FlateDecode", "LZWDecode"]);
    }

    #[test]
    fn test_extract_filter_names_empty_array() {
        let filter = Object::Array(vec![]);
        let names = extract_filter_names(&filter);
        assert!(names.is_empty());
    }

    #[test]
    fn test_extract_filter_names_null() {
        let filter = Object::Null;
        let names = extract_filter_names(&filter);
        assert!(names.is_empty());
    }

    #[test]
    fn test_extract_filter_names_boolean() {
        let filter = Object::Boolean(true);
        let names = extract_filter_names(&filter);
        assert!(names.is_empty());
    }

    // ---- Tests for extract_decode_params ----

    #[test]
    fn test_extract_decode_params_none() {
        let result = extract_decode_params(None);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_decode_params_defaults() {
        // Empty dictionary should yield default params
        let dict = Object::Dictionary(HashMap::new());
        let result = extract_decode_params(Some(&dict)).unwrap();
        assert_eq!(result.predictor, 1);
        assert_eq!(result.columns, 1);
        assert_eq!(result.colors, 1);
        assert_eq!(result.bits_per_component, 8);
    }

    #[test]
    fn test_extract_decode_params_custom_values() {
        let mut d = HashMap::new();
        d.insert("Predictor".to_string(), Object::Integer(12));
        d.insert("Columns".to_string(), Object::Integer(800));
        d.insert("Colors".to_string(), Object::Integer(3));
        d.insert("BitsPerComponent".to_string(), Object::Integer(16));
        let dict = Object::Dictionary(d);
        let result = extract_decode_params(Some(&dict)).unwrap();
        assert_eq!(result.predictor, 12);
        assert_eq!(result.columns, 800);
        assert_eq!(result.colors, 3);
        assert_eq!(result.bits_per_component, 16);
    }

    #[test]
    fn test_extract_decode_params_from_array() {
        // Array of dictionaries - should take the first non-null
        let mut d = HashMap::new();
        d.insert("Predictor".to_string(), Object::Integer(15));
        d.insert("Columns".to_string(), Object::Integer(640));
        let dict = Object::Dictionary(d);
        let arr = Object::Array(vec![dict]);
        let result = extract_decode_params(Some(&arr)).unwrap();
        assert_eq!(result.predictor, 15);
        assert_eq!(result.columns, 640);
    }

    #[test]
    fn test_extract_decode_params_invalid_type() {
        let obj = Object::Integer(42);
        let result = extract_decode_params(Some(&obj));
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_decode_params_null_object() {
        let obj = Object::Null;
        let result = extract_decode_params(Some(&obj));
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_decode_params_empty_array() {
        let arr = Object::Array(vec![]);
        let result = extract_decode_params(Some(&arr));
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_decode_params_array_with_only_non_dicts() {
        let arr = Object::Array(vec![Object::Integer(1), Object::Null]);
        let result = extract_decode_params(Some(&arr));
        assert!(result.is_none());
    }

    // ---- Tests for extract_ccitt_params ----

    #[test]
    fn test_extract_ccitt_params_none() {
        let result = extract_ccitt_params(None);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_ccitt_params_defaults() {
        let dict = Object::Dictionary(HashMap::new());
        let result = extract_ccitt_params(Some(&dict)).unwrap();
        assert_eq!(result.k, 0); // ISO 32000-1 Table 11 default: Group 3, 1-D
        assert_eq!(result.columns, 1);
        assert!(result.rows.is_none());
        assert!(!result.black_is_1);
        assert!(!result.end_of_line);
        assert!(!result.encoded_byte_align);
        assert!(result.end_of_block);
    }

    #[test]
    fn test_extract_ccitt_params_custom_values() {
        let mut d = HashMap::new();
        d.insert("K".to_string(), Object::Integer(0));
        d.insert("Columns".to_string(), Object::Integer(1728));
        d.insert("Rows".to_string(), Object::Integer(2376));
        d.insert("BlackIs1".to_string(), Object::Boolean(true));
        d.insert("EndOfLine".to_string(), Object::Boolean(true));
        d.insert("EncodedByteAlign".to_string(), Object::Boolean(true));
        d.insert("EndOfBlock".to_string(), Object::Boolean(false));
        let dict = Object::Dictionary(d);
        let result = extract_ccitt_params(Some(&dict)).unwrap();
        assert_eq!(result.k, 0);
        assert_eq!(result.columns, 1728);
        assert_eq!(result.rows, Some(2376));
        assert!(result.black_is_1);
        assert!(result.end_of_line);
        assert!(result.encoded_byte_align);
        assert!(!result.end_of_block);
    }

    #[test]
    fn test_extract_ccitt_params_invalid_type() {
        let obj = Object::Integer(42);
        let result = extract_ccitt_params(Some(&obj));
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_ccitt_params_from_array() {
        let mut d = HashMap::new();
        d.insert("K".to_string(), Object::Integer(-1));
        d.insert("Columns".to_string(), Object::Integer(612));
        let dict = Object::Dictionary(d);
        let arr = Object::Array(vec![dict]);
        let result = extract_ccitt_params(Some(&arr)).unwrap();
        assert_eq!(result.k, -1);
        assert_eq!(result.columns, 612);
    }

    // ---- Tests for extract_ccitt_params_with_width ----

    #[test]
    fn test_extract_ccitt_params_with_width_override() {
        // Width override should be used when Columns is absent
        let dict = Object::Dictionary(HashMap::new());
        let result = extract_ccitt_params_with_width(Some(&dict), Some(2550)).unwrap();
        assert_eq!(result.columns, 2550);
    }

    #[test]
    fn test_extract_ccitt_params_with_width_columns_takes_precedence() {
        // Columns in dictionary should take precedence over image_width
        let mut d = HashMap::new();
        d.insert("Columns".to_string(), Object::Integer(1000));
        let dict = Object::Dictionary(d);
        let result = extract_ccitt_params_with_width(Some(&dict), Some(2550)).unwrap();
        assert_eq!(result.columns, 1000);
    }

    #[test]
    fn test_extract_ccitt_params_with_width_none_params() {
        let result = extract_ccitt_params_with_width(None, Some(200));
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_ccitt_params_with_width_no_override_no_columns() {
        // Neither Columns nor width override: should default to 1
        let dict = Object::Dictionary(HashMap::new());
        let result = extract_ccitt_params_with_width(Some(&dict), None).unwrap();
        assert_eq!(result.columns, 1);
    }

    // ---- Tests for decode_stream_data_with_decryption ----

    #[test]
    fn test_decode_stream_with_decryption_no_decrypt() {
        let mut dict = HashMap::new();
        dict.insert("Length".to_string(), Object::Integer(5));
        let obj = Object::Stream {
            dict,
            data: crate::import_limits::StreamData::from_static(b"Hello"),
        };
        let decoded = obj.decode_stream_data_with_decryption(None, 0, 0).unwrap();
        assert_eq!(decoded, b"Hello");
    }

    #[test]
    fn test_decode_stream_with_decryption_fn() {
        let mut dict = HashMap::new();
        dict.insert("Length".to_string(), Object::Integer(5));
        let obj = Object::Stream {
            dict,
            data: crate::import_limits::StreamData::from_static(b"Hello"),
        };
        // Simple identity "decryption" function
        let decrypt_fn = |data: &[u8]| -> Result<Vec<u8>> { Ok(data.to_vec()) };
        let decoded = obj
            .decode_stream_data_with_decryption(Some(&decrypt_fn), 1, 0)
            .unwrap();
        assert_eq!(decoded, b"Hello");
    }

    #[test]
    fn test_decode_stream_with_decryption_fn_that_transforms() {
        let mut dict = HashMap::new();
        dict.insert("Length".to_string(), Object::Integer(5));
        let obj = Object::Stream {
            dict,
            data: crate::import_limits::StreamData::from_static(b"\x01\x02\x03"),
        };
        // "Decryption" that XOR-s with 0xFF
        let decrypt_fn =
            |data: &[u8]| -> Result<Vec<u8>> { Ok(data.iter().map(|b| b ^ 0xFF).collect()) };
        let decoded = obj
            .decode_stream_data_with_decryption(Some(&decrypt_fn), 1, 0)
            .unwrap();
        assert_eq!(decoded, vec![0xFE, 0xFD, 0xFC]);
    }

    #[test]
    fn test_decode_stream_with_decryption_fn_error() {
        let mut dict = HashMap::new();
        dict.insert("Length".to_string(), Object::Integer(5));
        let obj = Object::Stream {
            dict,
            data: crate::import_limits::StreamData::from_static(b"Hello"),
        };
        let decrypt_fn =
            |_data: &[u8]| -> Result<Vec<u8>> { Err(Error::InvalidPdf("decrypt fail".into())) };
        let result = obj.decode_stream_data_with_decryption(Some(&decrypt_fn), 1, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_stream_not_a_stream_with_decryption() {
        let obj = Object::Name("NotAStream".to_string());
        let result = obj.decode_stream_data_with_decryption(None, 0, 0);
        assert!(result.is_err());
        if let Err(Error::InvalidObjectType { expected, found }) = result {
            assert_eq!(expected, "Stream");
            assert_eq!(found, "Name");
        } else {
            panic!("Expected InvalidObjectType error");
        }
    }

    /// Stream data must be returned verbatim. The parser already
    /// consumes the single EOL that follows the `stream` keyword per
    /// ISO 32000-1:2008 §7.3.8.1, so `decode_stream_data` must not
    /// further strip leading CR/LF bytes — that would corrupt binary
    /// streams whose first byte is legitimately 0x0A or 0x0D.
    #[test]
    fn test_decode_stream_preserves_leading_cr_lf() {
        let mut dict = HashMap::new();
        dict.insert("Length".to_string(), Object::Integer(7));
        let obj = Object::Stream {
            dict,
            data: crate::import_limits::StreamData::from_static(b"\r\nHello"),
        };
        let decoded = obj.decode_stream_data().unwrap();
        assert_eq!(decoded, b"\r\nHello");
    }

    #[test]
    fn test_decode_dictionary_as_stream_empty() {
        let dict = HashMap::new();
        let obj = Object::Dictionary(dict);
        let decoded = obj.decode_stream_data().unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn test_decode_stream_with_decode_params() {
        // Test stream with ASCIIHexDecode and DecodeParms (params should be ignored for ASCIIHex)
        let mut dict = HashMap::new();
        dict.insert("Filter".to_string(), Object::Name("ASCIIHexDecode".to_string()));
        let mut decode_params = HashMap::new();
        decode_params.insert("Predictor".to_string(), Object::Integer(1));
        dict.insert("DecodeParms".to_string(), Object::Dictionary(decode_params));
        let obj = Object::Stream {
            dict,
            data: crate::import_limits::StreamData::from_static(b"48656C6C6F"),
        };
        let decoded = obj.decode_stream_data().unwrap();
        assert_eq!(decoded, b"Hello");
    }

    // ---- Tests for Object equality ----

    #[test]
    fn test_object_equality() {
        assert_eq!(Object::Null, Object::Null);
        assert_eq!(Object::Boolean(true), Object::Boolean(true));
        assert_ne!(Object::Boolean(true), Object::Boolean(false));
        assert_eq!(Object::Integer(42), Object::Integer(42));
        assert_ne!(Object::Integer(42), Object::Integer(43));
        assert_eq!(Object::String(b"abc".to_vec()), Object::String(b"abc".to_vec()));
        assert_ne!(Object::String(b"abc".to_vec()), Object::String(b"def".to_vec()));
        assert_ne!(Object::Null, Object::Integer(0));
    }

    // ---- Tests for Object::Boolean values ----

    #[test]
    fn test_as_bool_false() {
        assert_eq!(Object::Boolean(false).as_bool(), Some(false));
    }

    // ---- Tests for as_dict on Stream ----

    #[test]
    fn test_as_dict_on_stream_returns_stream_dict() {
        let mut dict = HashMap::new();
        dict.insert("Type".to_string(), Object::Name("XObject".to_string()));
        dict.insert("Subtype".to_string(), Object::Name("Image".to_string()));
        let obj = Object::Stream {
            dict: dict.clone(),
            data: crate::import_limits::StreamData::from_static(b"image data"),
        };
        let result_dict = obj.as_dict().unwrap();
        assert_eq!(result_dict.len(), 2);
        assert_eq!(result_dict.get("Type").unwrap().as_name(), Some("XObject"));
        assert_eq!(result_dict.get("Subtype").unwrap().as_name(), Some("Image"));
    }

    // ---- Tests for encode_pdf_text_string / Object::text_string ----

    #[test]
    fn test_encode_pdf_text_string_ascii_unchanged() {
        let bytes = encode_pdf_text_string("Hello, world!");
        assert_eq!(bytes, b"Hello, world!");
    }

    #[test]
    fn test_encode_pdf_text_string_latin1_direct_byte() {
        // é = U+00E9 → byte 0xE9 in PDFDocEncoding (same as Latin-1)
        let bytes = encode_pdf_text_string("é");
        assert_eq!(bytes, vec![0xE9_u8]);
    }

    #[test]
    fn test_encode_pdf_text_string_portuguese_accents() {
        // "Lógico" from issue #402
        let bytes = encode_pdf_text_string("Lógico");
        // L=0x4C  ó=0xF3  g=0x67  i=0x69  c=0x63  o=0x6F
        assert_eq!(bytes, vec![0x4C, 0xF3, 0x67, 0x69, 0x63, 0x6F]);
    }

    #[test]
    fn test_encode_pdf_text_string_all_latin1_supplement() {
        // Verify every character U+0080–U+00FF maps to its code-point byte
        for cp in 0x80_u32..=0xFF {
            let ch = char::from_u32(cp).unwrap();
            let s: String = ch.into();
            let bytes = encode_pdf_text_string(&s);
            assert_eq!(bytes, vec![cp as u8], "failed for U+{:04X}", cp);
        }
    }

    #[test]
    fn test_encode_pdf_text_string_cjk_uses_utf16be_with_bom() {
        // 中 = U+4E2D
        let bytes = encode_pdf_text_string("中");
        // BOM 0xFE 0xFF, then U+4E2D as big-endian: 0x4E 0x2D
        assert_eq!(bytes, vec![0xFE, 0xFF, 0x4E, 0x2D]);
    }

    #[test]
    fn test_encode_pdf_text_string_mixed_triggers_utf16be() {
        // If any char is above U+00FF the whole string goes UTF-16BE
        let bytes = encode_pdf_text_string("aé中");
        assert_eq!(&bytes[..2], &[0xFE, 0xFF], "must start with BOM");
        // a=0x0061, é=0x00E9, 中=0x4E2D
        let expected = vec![0xFE, 0xFF, 0x00, 0x61, 0x00, 0xE9, 0x4E, 0x2D];
        assert_eq!(bytes, expected);
    }

    #[test]
    fn test_encode_pdf_text_string_supplementary_plane_surrogate_pair() {
        // 𝄞 (MUSICAL SYMBOL G CLEF) = U+1D11E, encoded in UTF-16 as surrogate pair
        let bytes = encode_pdf_text_string("𝄞");
        assert_eq!(&bytes[..2], &[0xFE, 0xFF], "must start with BOM");
        // UTF-16BE surrogate pair for U+1D11E: 0xD834 0xDD1E
        assert_eq!(bytes, vec![0xFE, 0xFF, 0xD8, 0x34, 0xDD, 0x1E]);
    }

    #[test]
    fn test_object_text_string_accepts_str() {
        match Object::text_string("hello") {
            Object::String(b) => assert_eq!(b, b"hello"),
            _ => panic!("expected String"),
        }
    }

    #[test]
    fn test_object_text_string_accepts_owned_string() {
        let s = String::from("Ångström");
        match Object::text_string(s) {
            Object::String(b) => {
                // Å=0xC5  n=0x6E  g=0x67  s=0x73  t=0x74  r=0x72  ö=0xF6  m=0x6D
                assert_eq!(b, vec![0xC5, 0x6E, 0x67, 0x73, 0x74, 0x72, 0xF6, 0x6D]);
            },
            _ => panic!("expected String"),
        }
    }

    #[test]
    fn test_object_text_string_empty() {
        match Object::text_string("") {
            Object::String(b) => assert!(b.is_empty()),
            _ => panic!("expected String"),
        }
    }
}
