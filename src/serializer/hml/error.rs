use crate::parser::FileFormat;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HmlSaveBlocker {
    pub code: &'static str,
    pub xml_path: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HmlExportError {
    UnsupportedSourceFormat {
        actual: FileFormat,
        blockers: Vec<HmlSaveBlocker>,
    },
    LossyImport {
        blockers: Vec<HmlSaveBlocker>,
    },
    LossyImportAndUnsupportedIr {
        blockers: Vec<HmlSaveBlocker>,
    },
    UnsupportedIr {
        blockers: Vec<HmlSaveBlocker>,
    },
}

impl HmlExportError {
    pub fn blockers(&self) -> &[HmlSaveBlocker] {
        match self {
            Self::UnsupportedSourceFormat { blockers, .. }
            | Self::LossyImport { blockers }
            | Self::LossyImportAndUnsupportedIr { blockers }
            | Self::UnsupportedIr { blockers } => blockers,
        }
    }
}

impl std::fmt::Display for HmlExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let blockers = self.blockers();
        write!(f, "HML 저장이 차단되었습니다")?;
        for blocker in blockers {
            write!(f, ": {} ({})", blocker.message, blocker.xml_path)?;
        }
        Ok(())
    }
}

impl std::error::Error for HmlExportError {}

pub(crate) fn unsupported_ir(path: &str, message: impl Into<String>) -> HmlExportError {
    HmlExportError::UnsupportedIr {
        blockers: vec![HmlSaveBlocker {
            code: "HML_UNSUPPORTED_IR",
            xml_path: path.to_string(),
            message: message.into(),
        }],
    }
}
