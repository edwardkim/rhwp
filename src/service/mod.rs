//! 서비스 연계 표면 — CLI·MCP·WASM 이 **함께 서는** 문서 열기·조회 축.
//!
//! # 왜 이 모듈이 있는가
//!
//! `ROADMAP.md` 의 "rhwp가 공식적으로 맡는 범위" 표는 **서비스 연계 표면**을
//! 업스트림 책임으로 못 박는다 — "자동화와 백엔드 서비스가 사용할 CLI의 기계 판독
//! 출력, MCP 서버와 공개 API 계약". 그런데 그 계약을 담을 공통 모듈이 없어서,
//! 계약의 첫 네 걸음(**열기 · 메타 조회 · 검색 · 텍스트 내보내기**)이 표면마다
//! 다시 쓰여 있다.
//!
//! | 하는 일 | CLI (`src/main.rs`) | MCP (`src/mcp_serve.rs`) | WASM (`src/wasm_api.rs`) |
//! |---|---|---|---|
//! | 바이트 읽고 오류 보고 | `fs::read` + 오류 블록 **45곳** | `session_open` 1곳 | 호출자(JS) 몫 |
//! | 비밀번호 유무 분기 | `load_document` / `load_document_core` (2곳) | `session_open` | `from_bytes` / `from_bytes_with_password` |
//! | 실패를 갈래로 나누기 | `classify_hwp_error` — **한국어 문장 부분일치** | 없음(전부 "파싱 실패") | 없음(`JsValue` 로 통째 전달) |
//! | 형식 재판별 | `detect_format` **24곳** | `detect_format` 1곳 | 코어가 계산 후 폐기 |
//! | 메타 JSON | `info_json_value` | `info_json_value` 재사용 | `DocumentCore::get_document_info` — **필드도 글꼴 규칙도 다름** |
//! | 검색 | `grep`/`grep_with_context` | `grep` | `search_all_text_native` — **엔진 자체가 다름** |
//!
//! 마지막 두 줄이 이 축이 필요한 이유를 가장 잘 보여준다. "이 문서가 쓰는 글꼴은
//! 무엇인가"에 CLI 는 선언 순서 그대로(중복 포함)를, WASM 은 정렬·중복 제거·대체
//! 글꼴 해소본을 준다. "이 단어가 어디 있는가"에 CLI·MCP 는 좌표를 주고 WASM 은
//! 다른 엔진의 다른 모양을 준다. **어느 표면에 물었느냐로 답이 달라지는 값은
//! 계약이 아니다.**
//!
//! # 설계 규약
//!
//! - **결정적·읽기 전용.** LLM·네트워크·전역 상태 없음. 비밀번호도 전역이 아니라
//!   [`OpenOptions`] 인자로 흐른다(현행 CLI 의 `thread_local` 전역과 대비).
//! - **오류는 타입.** [`ServiceError`] 가 갈래를 이름으로 준다. 소비자가 한국어
//!   문장을 부분일치로 판정하는 일이 없어야 한다.
//! - **판정은 오류가 아니다.** 매치 0건, 범위 밖 쪽 요청, 추출 실패한 쪽은 전부
//!   `Ok` 안의 데이터다. 그걸로 무엇을 할지는 표면의 정책이다.
//! - **파싱을 새로 쓰지 않는다.** [`crate::parser`] 와 [`crate::document_core`] 를
//!   그대로 쓴다. 이 축은 그 위에 얹는 **계약**이지 두 번째 파서가 아니다.
//!
//! # 범위 밖
//!
//! 편집·저장·렌더링·MCP 세션 수명·JSON 봉투 껍데기(`schemaVersion`·`source`)는 이
//! 축이 다루지 않는다. 편집이 필요하면 [`OpenedDocument::into_core`] 로 코어를
//! 가져가 기존 경로로 계속한다.
//!
//! # 쓰는 법
//!
//! ```no_run
//! use std::path::Path;
//! use rhwp::service::{DocumentService, OpenOptions, SearchOptions, ServiceError};
//!
//! let service = DocumentService::new();
//! let opened = match service.open_path(Path::new("문서.hwpx"), &OpenOptions::new()) {
//!     Ok(opened) => opened,
//!     // 갈래를 **타입으로** 받는다. 문자열을 다시 읽지 않는다.
//!     Err(error) => {
//!         eprintln!("[{}] {error}", error.code());
//!         std::process::exit(if error.is_usage_error() { 2 } else { 1 });
//!     }
//! };
//!
//! let info = opened.info();
//! println!("{} · {}쪽", info.format, info.page_count);
//!
//! // 매치 0건은 오류가 아니라 답이다.
//! let found = opened.search("계약", &SearchOptions::new().with_limit(20));
//! println!("{}건 중 {}건", found.total_match_count, found.match_count);
//! # let _: fn(ServiceError) = |_| ();
//! ```

pub mod error;
pub mod open;
pub mod query;

pub use error::ServiceError;
pub use open::{
    format_token, DocumentInfo, DocumentService, DocumentSource, OpenOptions, OpenedDocument,
    DEFAULT_MAX_BYTES,
};
pub use query::{PageText, SearchOptions, SearchOutcome, TextExport, TextExportOptions};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::FileFormat;
    use std::path::{Path, PathBuf};

    /// 저장소 루트 기준 샘플 경로. 작업 디렉터리에 의존하지 않는다.
    fn sample(relative: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
    }

    fn hwpx_sample() -> PathBuf {
        sample("samples/hwpx/143E433F503322BD33.hwpx")
    }

    fn hwp_sample() -> PathBuf {
        sample("samples/143E433F503322BD33.hwp")
    }

    /// 상한 없는 서비스 — 샘플 크기와 무관하게 열리도록.
    fn service() -> DocumentService {
        DocumentService::new().with_max_bytes(None)
    }

    #[test]
    fn open_path_reads_and_parses_hwpx_sample() {
        let path = hwpx_sample();
        let expected_size = std::fs::metadata(&path).expect("샘플 메타").len() as usize;
        let opened = service()
            .open_path(&path, &OpenOptions::new())
            .expect("HWPX 샘플은 열려야 한다");
        assert_eq!(opened.format(), FileFormat::Hwpx);
        // 원본 크기를 **버리지 않는다** — 소비자가 파일을 다시 stat 하지 않아도 된다.
        assert_eq!(opened.size_bytes(), expected_size);
        assert_eq!(
            opened.source(),
            &DocumentSource::Path(path),
            "경로에서 열었으면 출처가 경로여야 한다"
        );
        assert!(opened.page_count() >= 1);
    }

    #[test]
    fn format_is_auto_detected_from_bytes_alone() {
        // 같은 문서의 두 형식이 확장자 없이 **바이트만으로** 갈려야 한다.
        let hwpx = std::fs::read(hwpx_sample()).expect("HWPX 샘플 읽기");
        let hwp = std::fs::read(hwp_sample()).expect("HWP 샘플 읽기");

        let opened_hwpx = service()
            .open_bytes(&hwpx, &OpenOptions::new())
            .expect("HWPX 바이트");
        let opened_hwp = service()
            .open_bytes(&hwp, &OpenOptions::new())
            .expect("HWP 바이트");

        assert_eq!(opened_hwpx.format(), FileFormat::Hwpx);
        assert_eq!(opened_hwp.format(), FileFormat::Hwp);
        assert_eq!(opened_hwpx.info().format, "hwpx");
        assert_eq!(opened_hwp.info().format, "hwp5");
        assert_eq!(opened_hwpx.source(), &DocumentSource::Bytes);
    }

    #[test]
    fn missing_path_is_not_found_not_parse_failure() {
        let missing = sample("samples/이런파일은없다_87f3.hwpx");
        let error = service()
            .open_path(&missing, &OpenOptions::new())
            .expect_err("없는 파일은 실패해야 한다");
        assert_eq!(
            error,
            ServiceError::NotFound {
                path: missing.clone()
            }
        );
        assert_eq!(error.code(), "NOT_FOUND");
        // "경로를 고치면 되는" 실패다 — 소비자는 이걸로 EXIT_USAGE 를 고른다.
        assert!(error.is_usage_error());
        assert!(!error.needs_password());
    }

    #[test]
    fn unrecognized_bytes_are_unsupported_format() {
        let service = service();
        let junk = service
            .open_bytes(b"this is not a hwp document at all", &OpenOptions::new())
            .expect_err("미상 바이트는 실패해야 한다");
        assert_eq!(
            junk,
            ServiceError::UnsupportedFormat {
                detected: FileFormat::Unknown
            }
        );
        assert_eq!(junk.code(), "UNSUPPORTED_FORMAT");

        // 빈 입력은 "알 수 없음"과 **다른 사실**이다 — 감지 결과를 그대로 싣는다.
        let empty = service
            .open_bytes(&[], &OpenOptions::new())
            .expect_err("빈 입력은 실패해야 한다");
        assert_eq!(
            empty,
            ServiceError::UnsupportedFormat {
                detected: FileFormat::Empty
            }
        );
    }

    #[test]
    fn size_limit_rejects_before_parsing() {
        let bytes = std::fs::read(hwpx_sample()).expect("HWPX 샘플 읽기");
        let limit = 1024;
        assert!(
            bytes.len() > limit,
            "샘플이 상한보다 커야 의미 있는 검증이다"
        );

        let error = DocumentService::new()
            .open_bytes(&bytes, &OpenOptions::new().with_max_bytes(limit))
            .expect_err("상한 초과는 실패해야 한다");
        assert_eq!(
            error,
            ServiceError::TooLarge {
                size_bytes: bytes.len(),
                limit_bytes: limit,
            }
        );
        assert_eq!(error.code(), "TOO_LARGE");

        // 호출 단위 상한이 서비스 기본값을 덮어쓴다.
        let tight = DocumentService::new().with_max_bytes(Some(limit));
        assert!(tight.open_bytes(&bytes, &OpenOptions::new()).is_err());
        assert!(tight
            .open_bytes(&bytes, &OpenOptions::new().with_max_bytes(bytes.len()))
            .is_ok());
    }

    #[test]
    fn info_reports_format_size_and_counts() {
        let path = hwpx_sample();
        let expected_size = std::fs::metadata(&path).expect("샘플 메타").len() as usize;
        let opened = service()
            .open_path(&path, &OpenOptions::new())
            .expect("HWPX 샘플");
        let info = opened.info();

        assert_eq!(info.format, "hwpx");
        assert_eq!(info.size_bytes, expected_size);
        assert!(info.sections >= 1, "구역이 최소 하나는 있어야 한다");
        assert_eq!(info.page_count, opened.page_count());
        assert!(info.para_count >= 1, "문단이 최소 하나는 있어야 한다");
        assert!(!info.encrypted, "평문 샘플이다");
        assert!(info.version.is_some(), "HWPX 는 버전 문자열을 갖는다");

        // 직렬화 어휘가 봉투 계약(camelCase)과 같아야 한다.
        let json = serde_json::to_value(&info).expect("직렬화");
        for key in ["format", "sizeBytes", "pageCount", "paraCount", "fonts"] {
            assert!(json.get(key).is_some(), "{key} 필드가 있어야 한다");
        }
    }

    #[test]
    fn title_scan_can_be_disabled() {
        let path = hwpx_sample();
        let scanned = service()
            .open_path(&path, &OpenOptions::new())
            .expect("HWPX 샘플")
            .info();
        let unscanned = service()
            .with_title_scan(false)
            .open_path(&path, &OpenOptions::new())
            .expect("HWPX 샘플")
            .info();

        assert!(
            unscanned.title.is_none(),
            "제목 추정을 끄면 언제나 None 이다"
        );
        // 나머지 메타는 옵션과 무관하게 같아야 한다(결정성).
        assert_eq!(scanned.page_count, unscanned.page_count);
        assert_eq!(scanned.fonts, unscanned.fonts);
    }

    #[test]
    fn export_text_keeps_one_entry_per_page() {
        let opened = service()
            .open_path(&hwpx_sample(), &OpenOptions::new())
            .expect("HWPX 샘플");
        let export = opened.export_text(&TextExportOptions::new());

        assert_eq!(
            export.page_count as u32,
            opened.page_count(),
            "쪽 항목을 빼면 pageCount 가 문서를 실제보다 짧게 말한다"
        );
        assert_eq!(export.pages.len(), export.page_count);
        for (index, page) in export.pages.iter().enumerate() {
            assert_eq!(page.page, index as u32, "쪽 번호는 0부터 연속이다");
        }
        assert!(!export.truncated);
        assert_eq!(export.omitted_count, 0);
        assert_eq!(export.next_offset, None, "무제한이면 더 읽을 것이 없다");
        assert!(export.out_of_range.is_empty());
        assert!(!export.has_failures());
        assert!(
            !export.concatenated().trim().is_empty(),
            "샘플에는 본문 텍스트가 있다"
        );
    }

    #[test]
    fn export_text_truncation_reports_omission_and_next_offset() {
        let opened = service()
            .open_path(&hwpx_sample(), &OpenOptions::new())
            .expect("HWPX 샘플");
        let full = opened.export_text(&TextExportOptions::new());
        let full_chars: usize = full.pages.iter().map(|p| p.text.chars().count()).sum();
        assert!(full_chars > 40, "절단을 검증하려면 본문이 충분해야 한다");

        let cut = opened.export_text(&TextExportOptions::new().with_max_chars(20));
        let cut_chars: usize = cut.pages.iter().map(|p| p.text.chars().count()).sum();

        assert_eq!(cut_chars, 20);
        assert!(cut.truncated);
        assert_eq!(cut.omitted_count, full_chars - 20);
        assert_eq!(cut.next_offset, Some(20), "이어읽기 지점을 명시한다");
        assert_eq!(
            cut.page_count, full.page_count,
            "예산이 떨어져도 쪽 주소는 남는다"
        );
    }

    #[test]
    fn export_text_out_of_range_page_is_data_not_error() {
        let opened = service()
            .open_path(&hwpx_sample(), &OpenOptions::new())
            .expect("HWPX 샘플");
        let beyond = opened.page_count() + 100;
        let export = opened.export_text(&TextExportOptions::new().with_pages(vec![0, beyond]));

        assert_eq!(export.pages.len(), 1, "유효한 쪽만 실린다");
        assert_eq!(export.pages[0].page, 0);
        assert_eq!(
            export.out_of_range,
            vec![beyond],
            "건너뛴 쪽을 조용히 숨기지 않는다"
        );
    }

    #[test]
    fn search_returns_matches_with_coordinates() {
        let opened = service()
            .open_path(&hwpx_sample(), &OpenOptions::new())
            .expect("HWPX 샘플");
        // 문서 내용을 하드코딩하지 않는다 — 문서에서 실제 문자열을 뽑아 그걸 찾는다.
        //
        // 조판된 쪽 텍스트는 줄바꿈이 낱말 사이에 끼어들 수 있어 그대로 검색어로
        // 쓰면 안 된다. 그래서 두 단계로 간다: 한 글자로 문단을 찾고, 그 **문단
        // 텍스트**에서 이어진 두 글자를 뽑는다(문단 안이면 반드시 연속이다).
        let text = opened.export_text(&TextExportOptions::new()).concatenated();
        let seed = text
            .chars()
            .find(|c| {
                !c.is_whitespace()
                    && !opened
                        .search(&c.to_string(), &SearchOptions::new())
                        .is_empty()
            })
            .expect("본문 글자가 문단에서도 찾혀야 한다");
        let seeded = opened.search(&seed.to_string(), &SearchOptions::new());
        let anchor = &seeded.matches[0];
        let needle: String = anchor
            .text
            .chars()
            .skip(anchor.char_offset)
            .take(2)
            .collect();

        let found = opened.search(&needle, &SearchOptions::new());
        assert!(!found.is_empty(), "문단에서 뽑은 문자열은 찾혀야 한다");
        assert_eq!(found.query, needle);
        assert_eq!(found.match_count, found.matches.len());
        assert_eq!(found.match_count, found.total_match_count);
        assert!(!found.truncated);
        assert_eq!(found.omitted_count, 0);

        let first = &found.matches[0];
        assert_eq!(first.length, needle.chars().count());
        assert!(
            first.text.contains(&needle),
            "매치가 속한 문단 텍스트가 함께 온다"
        );
        // 좌표가 붙는다는 것이 이 축의 요점이다 — 조판된 문단은 쪽 번호를 갖는다.
        assert!(
            found.matches.iter().any(|m| m.page.is_some()),
            "매치에 쪽 좌표가 붙어야 한다"
        );
    }

    #[test]
    fn search_without_match_is_ok_not_error() {
        let opened = service()
            .open_path(&hwpx_sample(), &OpenOptions::new())
            .expect("HWPX 샘플");

        let none = opened.search("존재하지않는문자열_9f2c1b7e", &SearchOptions::new());
        assert!(none.is_empty());
        assert_eq!(none.total_match_count, 0);
        assert_eq!(none.match_count, 0);
        assert!(none.matches.is_empty());
        assert!(!none.truncated, "0건은 절단이 아니다");
        assert_eq!(none.next_offset, None);

        // 빈 검색어도 판정(0건)이지 실패가 아니다.
        let empty = opened.search("", &SearchOptions::new());
        assert!(empty.is_empty());
    }

    #[test]
    fn search_window_walks_all_matches_without_moving_the_total() {
        let opened = service()
            .open_path(&hwpx_sample(), &OpenOptions::new())
            .expect("HWPX 샘플");
        let text = opened.export_text(&TextExportOptions::new()).concatenated();
        // 여러 번 나오는 글자를 고른다 — 창 이동을 검증하려면 매치가 2건 이상이어야 한다.
        let needle = text
            .chars()
            .find(|c| {
                !c.is_whitespace()
                    && opened
                        .search(&c.to_string(), &SearchOptions::new())
                        .total_match_count
                        >= 3
            })
            .map(|c| c.to_string())
            .expect("3건 이상 나오는 글자가 있어야 한다");

        let all = opened.search(&needle, &SearchOptions::new());
        let total = all.total_match_count;

        let first = opened.search(&needle, &SearchOptions::new().with_limit(2));
        assert_eq!(first.match_count, 2);
        assert_eq!(
            first.total_match_count, total,
            "총량은 창과 무관하게 고정이다"
        );
        assert!(first.truncated);
        assert_eq!(first.omitted_count, total - 2);
        assert_eq!(first.next_offset, Some(2));

        let second = opened.search(&needle, &SearchOptions::new().with_limit(2).with_offset(2));
        assert_eq!(second.total_match_count, total);
        assert_eq!(second.offset, 2);
        // 창이 겹치지 않는다.
        assert_ne!(
            first.matches[0].char_offset.to_string() + &first.matches[0].paragraph.to_string(),
            second.matches[0].char_offset.to_string() + &second.matches[0].paragraph.to_string(),
        );

        // 마지막 창을 지나면 더 없음이다.
        let past_end = opened.search(&needle, &SearchOptions::new().with_offset(total));
        assert_eq!(past_end.match_count, 0);
        assert_eq!(past_end.total_match_count, total);
        assert_eq!(past_end.next_offset, None);
    }

    #[test]
    fn error_codes_and_severity_are_stable() {
        // 소비자가 exit code·JSON code 로 매핑하는 축이다. 토큰이 바뀌면 계약이 깨진다.
        let cases: [(ServiceError, &str, bool, bool); 7] = [
            (
                ServiceError::NotFound {
                    path: PathBuf::from("x"),
                },
                "NOT_FOUND",
                true,
                false,
            ),
            (
                ServiceError::Io {
                    path: PathBuf::from("x"),
                    kind: std::io::ErrorKind::PermissionDenied,
                },
                "IO",
                false,
                false,
            ),
            (
                ServiceError::UnsupportedFormat {
                    detected: FileFormat::DrmProtected,
                },
                "UNSUPPORTED_FORMAT",
                true,
                false,
            ),
            (
                ServiceError::PasswordRequired,
                "PASSWORD_REQUIRED",
                true,
                true,
            ),
            (
                ServiceError::PasswordMismatch,
                "PASSWORD_MISMATCH",
                false,
                true,
            ),
            (
                ServiceError::TooLarge {
                    size_bytes: 2,
                    limit_bytes: 1,
                },
                "TOO_LARGE",
                true,
                false,
            ),
            (ServiceError::Parse("깨짐".into()), "PARSE", false, false),
        ];
        for (error, code, usage, needs_password) in cases {
            assert_eq!(error.code(), code);
            assert_eq!(error.is_usage_error(), usage, "{code}");
            assert_eq!(error.needs_password(), needs_password, "{code}");
            assert!(
                !error.to_string().is_empty(),
                "{code} 는 사람용 문장도 준다"
            );
        }
    }

    #[test]
    fn open_failure_classification_derives_needles_from_types() {
        use crate::parser::crypto::CryptoError;
        use crate::parser::ParseError;
        use crate::HwpError;

        // 바늘을 한국어 상수로 박지 않고 **타입에서** 만든다는 사실 자체를 잠근다.
        let encrypted = HwpError::from(ParseError::EncryptedDocument);
        assert_eq!(
            ServiceError::from_open_failure(&encrypted, false),
            ServiceError::PasswordRequired
        );
        // 비밀번호를 주었는데도 암호 오류면 "주세요"가 아니라 "틀렸다"이다.
        assert_eq!(
            ServiceError::from_open_failure(&encrypted, true),
            ServiceError::PasswordMismatch
        );

        let wrong = HwpError::from(ParseError::CryptoError(CryptoError::WrongPassword));
        assert_eq!(
            ServiceError::from_open_failure(&wrong, true),
            ServiceError::PasswordMismatch
        );

        // 그 밖의 파싱 실패는 이름 없는 갈래로 뭉치되, 원문은 사람용으로 보존한다.
        let other = HwpError::InvalidFile("레코드 손상".into());
        assert_eq!(
            ServiceError::from_open_failure(&other, false),
            ServiceError::Parse("레코드 손상".into())
        );
    }

    #[test]
    fn service_is_deterministic_and_read_only() {
        // 같은 입력을 두 번 열면 두 번 다 같은 답이 나와야 한다.
        let bytes = std::fs::read(hwpx_sample()).expect("HWPX 샘플 읽기");
        let service = service();
        let first = service
            .open_bytes(&bytes, &OpenOptions::new())
            .expect("첫 열기");
        let second = service
            .open_bytes(&bytes, &OpenOptions::new())
            .expect("둘째 열기");

        assert_eq!(first.info(), second.info());
        assert_eq!(
            first.export_text(&TextExportOptions::new()),
            second.export_text(&TextExportOptions::new())
        );
        // 검색 결과는 직렬화 값으로 비교한다 — 그쪽이 소비자가 실제로 받는 계약이다.
        assert_eq!(
            serde_json::to_value(first.search("가", &SearchOptions::new())).expect("직렬화"),
            serde_json::to_value(second.search("가", &SearchOptions::new())).expect("직렬화")
        );
        // 원본 바이트는 그대로다 — 이 축은 읽기만 한다.
        assert_eq!(bytes, std::fs::read(hwpx_sample()).expect("재읽기"));
    }
}
