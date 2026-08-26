//! 성공한 직렬화가 산출물에 담지 못한 사용자 내용을 보고한다 (#4430).
//!
//! 직렬화 실패는 [`super::SerializeError`]이고, 성공했지만 일부 내용을 빈 값으로
//! 대체하거나 생략한 경우만 이 보고서에 들어간다. 보고서는 문서 세션에 저장하지
//! 않고 [`SerializedDocument`]가 바이트와 함께 소유한다. 따라서 이전 저장의 보고서가
//! 다음 저장 실패 뒤에 남거나, 서로 다른 저장 바이트와 뒤섞일 수 없다.

use std::fmt;

/// content-loss JSON 계약 버전.
pub const CONTENT_LOSS_SCHEMA_VERSION: u32 = 1;

/// 이번 직렬화의 출력 형식.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SerializedFormat {
    Hwp,
    Hwpx,
}

impl fmt::Display for SerializedFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Hwp => f.write_str("HWP"),
            Self::Hwpx => f.write_str("HWPX"),
        }
    }
}

/// 산출물에서 일어난 손실의 안정 식별자.
///
/// 후속 생산자는 같은 보고서 경계를 재사용한다. 예를 들어 표현 불가 컨트롤은
/// `ControlOmitted`, 필드 파라미터 축소는 `MetadataReduced`를 사용한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ContentLossCode {
    BinaryContentEmptied,
    ControlOmitted,
    MetadataReduced,
}

/// 잃은 내용의 도메인 종류.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ContentLossSubject {
    BinaryData,
    Control,
    FieldParameters,
}

/// 손실이 필요해진 원인.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ContentLossReason {
    /// 제한 안에서 자원을 읽을 수 없었다.
    ResourceReadFailedOrLimitExceeded,
    /// 원본 저장 바이트는 있지만 대상 스트림의 압축 상태와 달라 통과시킬 수 없었다.
    StoredCompressionMismatch,
    /// 제한 안에서 읽지 못했고 원본 저장 바이트도 제공되지 않았다.
    RawPassthroughUnavailable,
    /// 대상 형식에 대응 표현이 없다. 후속 컨트롤/메타데이터 생산자용이다.
    UnsupportedByOutputFormat,
}

impl fmt::Display for ContentLossReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ResourceReadFailedOrLimitExceeded => {
                f.write_str("제한 안에서 자원을 읽지 못했거나 크기 상한을 넘음")
            }
            Self::StoredCompressionMismatch => {
                f.write_str("원본 저장 압축 상태가 대상 스트림과 다름")
            }
            Self::RawPassthroughUnavailable => {
                f.write_str("안전하게 통과시킬 원본 저장 바이트가 없음")
            }
            Self::UnsupportedByOutputFormat => f.write_str("출력 형식에 대응 표현이 없음"),
        }
    }
}

/// 산출물에서 잃은 내용 한 건.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentLoss {
    pub code: ContentLossCode,
    pub subject: ContentLossSubject,
    /// 대상 산출물 안의 경로 또는 공통 IR 경로.
    pub path: String,
    pub reason: ContentLossReason,
    /// BinData 손실일 때의 공통 IR 자원 ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<u16>,
}

impl ContentLoss {
    pub fn binary_content_emptied(
        resource_id: u16,
        path: impl Into<String>,
        reason: ContentLossReason,
    ) -> Self {
        Self {
            code: ContentLossCode::BinaryContentEmptied,
            subject: ContentLossSubject::BinaryData,
            path: path.into(),
            reason,
            resource_id: Some(resource_id),
        }
    }
}

impl fmt::Display for ContentLoss {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.code {
            ContentLossCode::BinaryContentEmptied => write!(
                f,
                "{} (BinData ID {}) 내용을 읽을 수 없어 빈 내용으로 저장했습니다 ({})",
                self.path,
                self.resource_id.unwrap_or(0),
                self.reason
            ),
            ContentLossCode::ControlOmitted => {
                write!(f, "{} 컨트롤을 산출물에 담지 못했습니다", self.path)
            }
            ContentLossCode::MetadataReduced => {
                write!(
                    f,
                    "{} 메타데이터 일부를 산출물에 담지 못했습니다",
                    self.path
                )
            }
        }
    }
}

/// 한 번의 직렬화에서 발생한 사용자 내용 손실.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentLossReport {
    output_format: SerializedFormat,
    losses: Vec<ContentLoss>,
}

impl ContentLossReport {
    pub fn new(output_format: SerializedFormat) -> Self {
        Self {
            output_format,
            losses: Vec::new(),
        }
    }

    pub fn output_format(&self) -> SerializedFormat {
        self.output_format
    }

    pub fn is_empty(&self) -> bool {
        self.losses.is_empty()
    }

    pub fn len(&self) -> usize {
        self.losses.len()
    }

    pub fn losses(&self) -> &[ContentLoss] {
        &self.losses
    }

    pub fn record(&mut self, loss: ContentLoss) {
        self.losses.push(loss);
    }

    /// WASM/Studio 경계를 건너는 canonical JSON DTO.
    pub fn to_json(&self) -> String {
        #[derive(serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Dto<'a> {
            schema_version: u32,
            output_format: SerializedFormat,
            count: usize,
            losses: &'a [ContentLoss],
        }

        serde_json::to_string(&Dto {
            schema_version: CONTENT_LOSS_SCHEMA_VERSION,
            output_format: self.output_format,
            count: self.losses.len(),
            losses: &self.losses,
        })
        .expect("content-loss DTO contains only serializable fields")
    }

    /// byte-only 네이티브 API의 기존 경고 채널.
    ///
    /// 브라우저는 stderr를 받을 수 없으므로 reported API의 값을 소비해야 한다.
    pub fn write_warnings_to_stderr(&self) {
        for loss in &self.losses {
            eprintln!("경고: {} 저장 중 {loss}", self.output_format);
        }
    }
}

/// 직렬화된 바이트와 바로 그 바이트에서 발생한 손실을 한 생명주기로 묶는다.
#[derive(Debug)]
#[must_use = "산출 바이트와 content-loss 보고서를 같은 저장 흐름에서 함께 소비해야 합니다"]
pub struct SerializedDocument {
    bytes: Vec<u8>,
    content_loss: ContentLossReport,
}

impl SerializedDocument {
    pub fn new(bytes: Vec<u8>, content_loss: ContentLossReport) -> Self {
        Self {
            bytes,
            content_loss,
        }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn content_loss(&self) -> &ContentLossReport {
        &self.content_loss
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    pub fn into_parts(self) -> (Vec<u8>, ContentLossReport) {
        (self.bytes, self.content_loss)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_json_has_stable_schema_and_exact_resource_location() {
        let mut report = ContentLossReport::new(SerializedFormat::Hwpx);
        report.record(ContentLoss::binary_content_emptied(
            7,
            "BinData/image7.png",
            ContentLossReason::RawPassthroughUnavailable,
        ));

        let value: serde_json::Value = serde_json::from_str(&report.to_json()).unwrap();
        assert_eq!(value["schemaVersion"], 1);
        assert_eq!(value["outputFormat"], "hwpx");
        assert_eq!(value["count"], 1);
        assert_eq!(value["losses"][0]["code"], "binaryContentEmptied");
        assert_eq!(value["losses"][0]["subject"], "binaryData");
        assert_eq!(value["losses"][0]["path"], "BinData/image7.png");
        assert_eq!(value["losses"][0]["resourceId"], 7);
        assert_eq!(value["losses"][0]["reason"], "rawPassthroughUnavailable");
    }
}
