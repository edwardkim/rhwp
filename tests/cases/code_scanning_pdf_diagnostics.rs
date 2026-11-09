//! 관계형: 실제 PDF 내용은 보존하고 debug 로그에는 본문을 내보내지 않는다.
//! 주석 갱신: 2026-11-09T21:32:06+09:00.
//! 시각 측정 비해당: 로그 비노출과 원문 바이트 보존 검사이며 조판 좌표·쪽수 핀은 없다.

#![cfg(feature = "pdf-import")]

use std::cell::RefCell;
use std::sync::Once;

thread_local! {
    static CAPTURE: RefCell<Option<Vec<String>>> = const { RefCell::new(None) };
}

struct PdfDiagnosticLogger;

impl log::Log for PdfDiagnosticLogger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.target().starts_with("pdf_oxide::document")
            && CAPTURE.with(|capture| capture.borrow().is_some())
    }

    fn log(&self, record: &log::Record<'_>) {
        if self.enabled(record.metadata()) {
            CAPTURE.with(|capture| {
                if let Some(messages) = capture.borrow_mut().as_mut() {
                    messages.push(record.args().to_string());
                }
            });
        }
    }

    fn flush(&self) {}
}

static LOGGER: PdfDiagnosticLogger = PdfDiagnosticLogger;
static INSTALL_LOGGER: Once = Once::new();

fn capture<T>(action: impl FnOnce() -> T) -> (T, Vec<String>) {
    INSTALL_LOGGER.call_once(|| {
        log::set_logger(&LOGGER).expect("install test logger");
        log::set_max_level(log::LevelFilter::Debug);
    });
    CAPTURE.with(|capture| {
        assert!(capture.borrow().is_none(), "nested capture");
        *capture.borrow_mut() = Some(Vec::new());
    });
    struct ResetCapture;
    impl Drop for ResetCapture {
        fn drop(&mut self) {
            CAPTURE.with(|capture| *capture.borrow_mut() = None);
        }
    }
    let reset = ResetCapture;
    let result = action();
    let messages = CAPTURE.with(|capture| capture.borrow_mut().take().unwrap());
    drop(reset);
    (result, messages)
}

fn public_pdf(content: &[u8]) -> Vec<u8> {
    let objects = [
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(),
        b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] /Contents 4 0 R >>".to_vec(),
        [
            format!("<< /Length {} >>\nstream\n", content.len()).into_bytes(),
            content.to_vec(),
            b"\nendstream".to_vec(),
        ]
        .concat(),
    ];
    let mut pdf = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::new();
    for (index, object) in objects.iter().enumerate() {
        offsets.push(pdf.len());
        pdf.extend(format!("{} 0 obj\n", index + 1).as_bytes());
        pdf.extend(object);
        pdf.extend(b"\nendobj\n");
    }
    let xref = pdf.len();
    pdf.extend(format!("xref\n0 {}\n0000000000 65535 f \n", offsets.len() + 1).as_bytes());
    for offset in offsets {
        pdf.extend(format!("{offset:010} 00000 n \n").as_bytes());
    }
    pdf.extend(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    pdf
}

#[test]
fn decoded_pdf_content_keeps_its_bytes_without_entering_debug_logs() {
    let content = b"BT (PUBLIC-SECURITY-BODY-SENTINEL) Tj ET";
    let document = pdf_oxide::PdfDocument::from_bytes(public_pdf(content)).unwrap();
    let ((first, cached), messages) = capture(|| {
        (
            document.get_page_content_data(0).unwrap(),
            document.get_page_content_data(0).unwrap(),
        )
    });
    assert_eq!(first, content);
    assert_eq!(cached, first);
    assert!(messages.iter().any(|message| message.contains("Retrieved")));
    assert!(messages.iter().all(|message| {
        !message.contains("PUBLIC-SECURITY-BODY-SENTINEL") && !message.contains("BT (")
    }));
}

#[test]
fn encrypted_empty_password_pdf_keeps_decoded_content_out_of_debug_logs() {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("pdf/issue8415/authentication");
    let plain = pdf_oxide::PdfDocument::from_bytes(
        std::fs::read(root.join("password-control.pdf")).unwrap(),
    )
    .unwrap();
    let expected = plain.get_page_content_data(0).unwrap();
    assert!(!expected.is_empty());
    let encrypted =
        pdf_oxide::PdfDocument::from_bytes(std::fs::read(root.join("password-empty.pdf")).unwrap())
            .unwrap();
    assert!(encrypted.is_encrypted());
    let (decoded, messages) = capture(|| encrypted.get_page_content_data(0).unwrap());
    assert_eq!(decoded, expected);
    let body = String::from_utf8(expected).expect("public control uses a textual content stream");
    assert!(messages.iter().any(|message| message.contains("Retrieved")));
    let debug_body = format!("{body:?}");
    assert!(messages
        .iter()
        .all(|message| !message.contains(&debug_body)));
}
