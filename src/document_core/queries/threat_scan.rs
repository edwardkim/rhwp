//! 무기화 문서 구조 위협 탐지 — **읽기 전용 안전 에어락**.
//!
//! 에이전트가 신뢰할 수 없는 HWP/HWPX 를 열기 **전에** 컨테이너·레코드 구조를
//! 훑어 무기화 신호를 열거한다. APT 방어 맥락의 도구다 — 위장 채용 메일 →
//! 악성 첨부 문서 → 익스플로잇 사슬에서, 문서를 파서·렌더러에 넣기 전에
//! "이 문서는 공격용으로 만들어진 흔적이 있는가"를 사람·에이전트에게 알린다.
//!
//! ## ⚠️ 이것은 휴리스틱이지 안티바이러스가 아니다 — 보증하지 않는다
//!
//! 이 모듈은 **신호(signal)** 를 신고할 뿐 **증거(proof)** 를 내지 않는다. 결정론적
//! 구조 규칙이라, 규칙을 아는 공격자는 우회할 수 있다. 깨끗하다는 판정(`clean:true`)은
//! "이 탐지기가 아는 신호가 없다"는 뜻이지 "안전하다"는 보증이 아니다.
//!
//! **rhwp 의 진짜 방어는 이 탐지기가 아니다.** rhwp 의 실질 방어선은 두 축이다.
//!
//! 1. **메모리 안전** — Rust 로 작성돼, 상용 뷰어를 노리는 메모리 손상 RCE 부류를
//!    언어 차원에서 배제한다(오버플로·UAF·범위 밖 접근이 성립하지 않는다).
//! 2. **DoS 하드닝** — 압축 폭탄·확장 크기 오버플로·순환 참조 등을 파서·리더가
//!    상한과 방문 집합으로 이미 막는다(`cfb_reader`·`record`·`hwpx::reader`).
//!
//! `threat-scan` 은 그 방어 **위에 가시성(visibility)** 을 얹는다 — 파서가 조용히
//! 견뎌 낸 위협을 사람이 볼 수 있게 목록으로 신고한다. 이 도구가 **막을 수 없는 것**:
//! 트로이 목마가 심긴 뷰어 바이너리, OS 수준 익스플로잇, 이 탐지기가 모르는 신형
//! 구조 — 그것들은 안티바이러스·OS·EDR 의 몫이지 문서 엔진의 몫이 아니다.
//!
//! ## 다른 보안 축과의 경계 (중복하지 않는다)
//!
//! - **텍스트 주입 스캐너**(`inspect injection`) — 본문 **문자열**의 프롬프트 주입.
//! - **은닉·유니코드 기만**(`inspect hidden-text`/`unicode`) — 조판·코드포인트 층.
//! - **이 모듈** — **컨테이너·레코드 구조** 층. 본문 텍스트를 판정하지 않는다.
//!
//! ## 탐지하는 신호(구현됨)
//!
//! | kind | severity | 무엇을 잡는가 |
//! | --- | --- | --- |
//! | `embedded_executable` | high | BinData/내장 OLE 스트림이 실행 파일 매직(MZ/PE·ELF·Mach-O)으로 시작 |
//! | `ole_package` | high | 내장 OLE 개체에 `Ole10Native`(임의 파일·실행체를 감싸는 OLE 패키지) |
//! | `malformed_record` | high | 레코드가 스트림 밖을 가리키는 크기를 선언 — 파서 메모리 안전을 노리는 모양 |
//! | `macro_script` | medium | FileHeader 스크립트 플래그(한글 자체 표지) · HWPX `Scripts/` 엔트리 |
//! | `external_reference` | medium | 원격/UNC 로의 OLE 링크·외부 자원 참조(자동 로드 유도) |
//!
//! ## 아직 탐지하지 못하는 것(후속 과제 — 정직한 공백)
//!
//! - HWPX XML 엔티티 확장(billion-laughs)·XXE 구조. (레코드 오버런은 HWP5 전용이다.)
//! - 압축 팽창비 기반 폭탄 신고 — 리더가 이미 상한으로 **막고** 있어 가시성만 남은 후속.
//! - 위험 CLSID 전수 목록·내장 OLE 심층(1단계 초과) 재귀.
//! - 내장 OOXML 안의 VBA 매크로 정적 분석.
//! - HWP5 `Scripts/DefaultJScript` 내용 정적 분석 — 한글은 빈 문서에도 이 스텁을 늘
//!   담아 저장소 존재는 신호가 못 되고, 지금은 FileHeader 스크립트 플래그만 본다.
//!   플래그 없이 스텁에 코드를 숨긴 우회는 이 축이 놓친다.
//! - 암호화·배포용 문서 내부(암호문이라 구조를 읽을 수 없다 — 스캔 범위를 봉투가 밝힌다).
//!
//! ## 왜 문서 코어(IR)가 아니라 바이트에서 도는가
//!
//! 위협은 파싱 **이전**에, 파서가 만나기도 전의 바이트에 있다. 그래서 이 모듈은
//! `DocumentCore` 를 만들지 않고 `CfbReader`·`Record`·`HwpxReader`·`ole_container` 같은
//! 저수준 리더를 직접 쓴다 — 손상·악성 입력에서 완전 파싱이 실패해도 스캔은 돈다.

use serde::Serialize;

use crate::parser::cfb_reader::{decompress_stream_limited, CfbReader};
use crate::parser::detect_format;

/// 스트림 하나를 훑을 때의 바이트 상한. 매직·구조 판정에는 이 정도면 충분하고,
/// 이를 넘으면 깊은 스캔을 접어 이 탐지기 자신이 DoS 로 열리지 않게 한다.
const STREAM_SCAN_CAP: usize = 32 * 1024 * 1024;

/// 레코드 오버런 스캔의 레코드 수 상한 — 손상 입력에서 무한 순회를 막는다.
const MAX_RECORDS_SCANNED: usize = 500_000;

/// 봉투가 싣는 최대 발견 수 — 적대적 입력이 봉투를 무한히 부풀리지 못하게 한다.
const MAX_FINDINGS: usize = 2_000;

/// 위협 신호의 심각도. 규칙별 고정값이다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// 정상 문서에도 나타날 수 있어 다른 맥락과 함께 봐야 한다.
    Low,
    /// 의심스럽지만 단독으로 단정하지 않는다.
    Medium,
    /// 정상 문서에 나타날 이유가 사실상 없다.
    High,
}

impl Severity {
    /// 봉투용 안정 식별자.
    pub fn label(self) -> &'static str {
        match self {
            Severity::Low => "low",
            Severity::Medium => "medium",
            Severity::High => "high",
        }
    }
}

/// 위협 신호 1건 — 봉투에 그대로 실린다.
#[derive(Debug, Clone, Serialize)]
pub struct ThreatFinding {
    /// 신호 종류 (`embedded_executable` 등) — 엔진 라벨.
    pub kind: &'static str,
    /// 심각도 (`high`/`medium`/`low`) — 엔진 판정.
    pub severity: &'static str,
    /// 발견된 **구조적 주소** (스트림 경로·레코드 색인 등) — 엔진값.
    pub location: String,
    /// 문서가 정한 문자열 조각(외부 참조 URL·경로 등) — **문서 파생**, 있을 때만 실린다.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// 사람이 읽고 판단할 근거 — 엔진이 작성한 설명(엔진 수치를 포함할 수 있다).
    pub rationale: String,
}

/// 스캔 결과.
#[derive(Debug, Clone)]
pub struct ThreatReport {
    /// 호출자가 준 입력 경로.
    pub source: String,
    /// 판정한 컨테이너 형식 (`hwp5`/`hwpx`/`unknown`).
    pub format: &'static str,
    /// 실제로 훑은 구조 영역 이름 — 여기 없는 영역은 "깨끗함"이 아니라 "검사 안 함"이다.
    pub scopes: Vec<&'static str>,
    /// 발견된 위협 신호.
    pub findings: Vec<ThreatFinding>,
    /// 스캔 자체가 만난 비치명적 한계(암호화로 못 읽음 등) — 사람용 참고.
    pub notes: Vec<String>,
    /// 발견 수가 상한에 걸려 잘렸는가.
    pub truncated: bool,
}

impl ThreatReport {
    /// 위협 신호가 하나도 없는가. **보증이 아니라 판정이다**(모듈 doc 참조).
    pub fn clean(&self) -> bool {
        self.findings.is_empty()
    }

    /// 가장 높은 심각도(있으면).
    pub fn highest_severity(&self) -> Option<&'static str> {
        self.findings
            .iter()
            .min_by_key(|f| severity_rank_of(f.severity))
            .map(|f| f.severity)
    }
}

fn severity_rank_of(label: &str) -> u8 {
    match label {
        "high" => 0,
        "medium" => 1,
        _ => 2,
    }
}

// ── 매직·문자열 판정 ────────────────────────────────────────────────────────

/// 실행 파일 매직이면 그 종류 이름을 돌려준다. 문서는 실행체를 정당하게 내장하지 않는다.
fn executable_magic(bytes: &[u8]) -> Option<&'static str> {
    if bytes.len() < 4 {
        return None;
    }
    if bytes[0] == 0x4D && bytes[1] == 0x5A {
        // "MZ" — DOS/PE 실행 파일.
        return Some("PE(MZ)");
    }
    if bytes.starts_with(&[0x7F, 0x45, 0x4C, 0x46]) {
        // ELF.
        return Some("ELF");
    }
    if bytes.starts_with(&[0xFE, 0xED, 0xFA, 0xCE])
        || bytes.starts_with(&[0xFE, 0xED, 0xFA, 0xCF])
        || bytes.starts_with(&[0xCF, 0xFA, 0xED, 0xFE])
        || bytes.starts_with(&[0xCE, 0xFA, 0xED, 0xFE])
    {
        // Mach-O.
        return Some("Mach-O");
    }
    None
}

/// CFB/OLE 컨테이너 매직인가 (`D0 CF 11 E0 A1 B1 1A E1`).
fn is_ole_compound(bytes: &[u8]) -> bool {
    bytes.len() >= 8 && bytes[..8] == [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]
}

/// 원격/UNC 로의 참조로 보이는가 — 원격 스킴(`://`)·UNC(`\\host`)·`file:` 만 신호로 본다.
///
/// 로컬 상대 경로 링크(정상 문서의 흔한 이미지 링크)는 걸러 오탐을 막는다.
fn looks_remote(target: &str) -> bool {
    let t = target.trim();
    if t.is_empty() {
        return false;
    }
    let lower = t.to_ascii_lowercase();
    lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("ftp://")
        || lower.starts_with("ftps://")
        || lower.starts_with("smb://")
        || lower.starts_with("file://")
        || lower.starts_with("\\\\") // UNC
        || lower.starts_with("//") // UNC (슬래시)
        || lower.contains("://")
}

/// BinData 원본에서 스캔 대상 바이트 후보를 만든다: 원본 + (압축돼 있으면) 해제본.
///
/// HWP5 BinData 는 항목별로 압축될 수 있어(DocInfo 압축 플래그) 매직이 해제 뒤에만
/// 보인다. 원본과 해제본 양쪽을 상한 안에서 본다. 폭탄은 `decompress_stream_limited`
/// 가 막는다.
fn materialize_candidates(raw: Vec<u8>) -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    let raw_looks_structured = executable_magic(&raw).is_some()
        || is_ole_compound(&raw)
        || raw.starts_with(b"BM")
        || raw.starts_with(&[0x89, 0x50, 0x4E, 0x47]); // PNG
    if !raw_looks_structured {
        if let Ok(decoded) = decompress_stream_limited(&raw, STREAM_SCAN_CAP) {
            if decoded != raw && !decoded.is_empty() {
                out.push(decoded);
            }
        }
    }
    out.push(raw);
    out
}

// ── 발견 수집기 ─────────────────────────────────────────────────────────────

struct Collector {
    findings: Vec<ThreatFinding>,
    notes: Vec<String>,
    truncated: bool,
}

impl Collector {
    fn new() -> Self {
        Collector {
            findings: Vec::new(),
            notes: Vec::new(),
            truncated: false,
        }
    }

    fn push(
        &mut self,
        kind: &'static str,
        severity: Severity,
        location: String,
        detail: Option<String>,
        rationale: String,
    ) {
        if self.findings.len() >= MAX_FINDINGS {
            self.truncated = true;
            return;
        }
        self.findings.push(ThreatFinding {
            kind,
            severity: severity.label(),
            location,
            detail,
            rationale,
        });
    }

    fn note(&mut self, message: String) {
        if self.notes.len() < 64 {
            self.notes.push(message);
        }
    }
}

// ── 내장 스트림(실행체·OLE 패키지) ──────────────────────────────────────────

/// 한 내장 스트림의 바이트를 훑어 실행체·OLE 패키지를 신고한다.
fn scan_embedded_bytes(location: &str, raw: Vec<u8>, col: &mut Collector) {
    for bytes in materialize_candidates(raw) {
        if let Some(kind) = executable_magic(&bytes) {
            col.push(
                "embedded_executable",
                Severity::High,
                location.to_string(),
                None,
                format!(
                    "내장 스트림이 실행 파일 매직({kind})으로 시작합니다 — 문서는 실행체를 \
                     정당하게 내장하지 않습니다. 무기화 첨부의 전형적 형태입니다."
                ),
            );
            return;
        }
        if is_ole_compound(&bytes) {
            scan_nested_ole(location, &bytes, col);
            return;
        }
    }
}

/// 내장 OLE 개체(중첩 CFB)의 내부 스트림을 훑는다 — 실행체 payload·OLE 패키지 신호.
fn scan_nested_ole(location: &str, ole_bytes: &[u8], col: &mut Collector) {
    let Some(streams) = crate::parser::ole_container::all_ole_streams(ole_bytes) else {
        return;
    };
    let mut flagged_package = false;
    let mut flagged_exec = false;
    for (name, data) in &streams {
        if !flagged_exec {
            if let Some(kind) = executable_magic(data) {
                col.push(
                    "embedded_executable",
                    Severity::High,
                    format!("{location}»{name}"),
                    None,
                    format!(
                        "내장 OLE 개체 안의 스트림이 실행 파일 매직({kind})을 담고 있습니다 — \
                         문서에 실행체가 포장돼 있습니다."
                    ),
                );
                flagged_exec = true;
            }
        }
        if !flagged_package && (name == "\u{0001}Ole10Native" || name.ends_with("Ole10Native")) {
            // Ole10Native = OLE 패키지(packager) — 임의 파일·스크립트·실행체를 감싼다.
            // payload 선두의 실행 매직은 위 축이 이미 잡으므로 여기선 패키지 존재만 신고한다.
            let payload_exec =
                ole10native_payload(data).and_then(|p| executable_magic(&p).map(|k| k.to_string()));
            let sev = if payload_exec.is_some() {
                Severity::High
            } else {
                Severity::Medium
            };
            let extra = match &payload_exec {
                Some(k) => format!(" 감싼 payload 가 실행 파일 매직({k})입니다."),
                None => String::new(),
            };
            col.push(
                "ole_package",
                sev,
                format!("{location}»Ole10Native"),
                None,
                format!(
                    "내장 OLE 개체가 OLE 패키지(Ole10Native)입니다 — 임의 파일·스크립트·실행체를 \
                     문서에 감싸 넣는 고전적 통로입니다.{extra}"
                ),
            );
            flagged_package = true;
        }
    }
}

/// Ole10Native payload 를 떼어 낸다: `[u32 LE 전체길이][2B 플래그][ANSI 라벨\0][ANSI 파일명\0][ANSI 경로\0][u32 데이터길이][데이터]`.
///
/// 형식이 어긋나면 `None`. 완전 파싱이 아니라 payload 선두를 얻어 실행 매직만 본다.
fn ole10native_payload(data: &[u8]) -> Option<Vec<u8>> {
    if data.len() < 6 {
        return None;
    }
    // 선두 u32 전체 길이 다음 2바이트 플래그, 이후 널종단 ANSI 3필드를 건너뛴다.
    let mut pos = 6usize;
    for _ in 0..3 {
        let start = pos;
        while pos < data.len() && data[pos] != 0 {
            pos += 1;
        }
        if pos >= data.len() {
            return None;
        }
        pos += 1; // 널 종단 건너뛰기
        let _ = start;
    }
    // 데이터 길이(u32 LE) 다음이 실제 payload.
    if pos + 4 > data.len() {
        return None;
    }
    pos += 4;
    if pos >= data.len() {
        return None;
    }
    Some(data[pos..].to_vec())
}

// ── 레코드 오버런(익스플로잇 모양) ──────────────────────────────────────────

/// 레코드 스트림을 헤더만 따라 걸으며, 스트림 밖을 가리키는 크기를 선언한 레코드를 신고한다.
///
/// `record::Record::read_all` 과 같은 헤더 해독(태그 10b·레벨 10b·크기 12b, 크기==0xFFF 면
/// 확장 4바이트)을 쓰되, 첫 오버런에서 파싱을 포기하는 대신 **신고**한다. 데이터는 읽지
/// 않아 할당이 없고, 레코드 수를 상한으로 묶어 이 스캔 자신이 DoS 로 열리지 않게 한다.
fn scan_records_for_overrun(stream_label: &str, data: &[u8], col: &mut Collector) {
    let mut pos = 0usize;
    let mut idx = 0usize;
    while pos + 4 <= data.len() && idx < MAX_RECORDS_SCANNED {
        let header = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
        pos += 4;
        let tag_id = (header & 0x3FF) as u16;
        let mut size = (header >> 20) as u32;
        if size == 0xFFF {
            if pos + 4 > data.len() {
                col.push(
                    "malformed_record",
                    Severity::High,
                    format!("{stream_label}/record[{idx}]"),
                    None,
                    format!(
                        "레코드(tag={tag_id})가 확장 크기 헤더를 선언했지만 스트림이 그 4바이트 \
                         전에 끝납니다 — 파서 경계를 노리는 잘린 헤더 모양입니다."
                    ),
                );
                return;
            }
            size = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
            pos += 4;
        }
        let available = data.len() - pos;
        match pos.checked_add(size as usize) {
            None => {
                col.push(
                    "malformed_record",
                    Severity::High,
                    format!("{stream_label}/record[{idx}]"),
                    None,
                    format!(
                        "레코드(tag={tag_id})가 선언한 크기 {size}바이트가 usize 를 넘겨 오프셋 \
                         계산이 오버플로합니다 — wasm32 랩어라운드로 경계 검사를 무력화하려는 모양입니다."
                    ),
                );
                return;
            }
            Some(end) if end > data.len() => {
                col.push(
                    "malformed_record",
                    Severity::High,
                    format!("{stream_label}/record[{idx}]"),
                    None,
                    format!(
                        "레코드(tag={tag_id})가 {size}바이트를 선언했지만 스트림에는 {available}바이트만 \
                         남았습니다 — 선언 크기가 스트림 밖을 가리키는, 파서 메모리 안전을 노리는 모양입니다."
                    ),
                );
                return;
            }
            Some(end) => {
                pos = end;
            }
        }
        idx += 1;
    }
}

// ── HWP5 ────────────────────────────────────────────────────────────────────

fn scan_hwp5(data: &[u8], col: &mut Collector) -> (&'static str, Vec<&'static str>) {
    let scopes = vec![
        "binDataStreams",
        "oleObjects",
        "docInfoRecords",
        "bodyTextRecords",
        "scriptFlag",
        "externalLinks",
    ];

    let mut cfb = match CfbReader::open(data) {
        Ok(c) => c,
        Err(e) => {
            col.note(format!(
                "CFB 컨테이너를 열 수 없어 HWP5 구조 스캔을 건너뜁니다: {e}"
            ));
            return ("hwp5", scopes);
        }
    };

    // FileHeader 플래그 — 압축·암호화·배포·스크립트.
    let (compressed, encrypted, distribution, script_flag) = match cfb.read_file_header() {
        Ok(hdr) => match crate::parser::header::parse_file_header(&hdr) {
            Ok(fh) => (
                fh.flags.compressed,
                fh.flags.encrypted,
                fh.flags.distribution,
                fh.flags.script,
            ),
            Err(_) => (true, false, false, false),
        },
        Err(_) => (true, false, false, false),
    };

    // ① 스크립트/매크로 — FileHeader 의 script 플래그가 **권위 신호**다.
    //
    // [실측] 한글이 저장하는 거의 모든 HWP5 는 빈 `Scripts/DefaultJScript`(16~136B 기본
    // 스텁)를 늘 담는다. 그래서 저장소 존재 자체는 신호가 못 된다(정상 공문서 전부가
    // 걸린다). 한글은 문서가 **실제로 스크립트를 저장할 때만** FileHeader 의 script
    // 비트를 켠다 — 그 플래그를 신호로 삼아 오탐을 없앤다. 플래그 없이 DefaultJScript 에
    // 코드를 숨긴 우회는 이 축이 놓친다(JScript 내용 정적 분석은 후속 과제).
    if script_flag {
        col.push(
            "macro_script",
            Severity::Medium,
            "FileHeader.flags.script".to_string(),
            None,
            "FileHeader 가 스크립트 저장 플래그를 선언했습니다 — 문서가 실행 가능한 \
             스크립트(매크로)를 실제로 담고 있다는 한글 자체의 표지입니다."
                .to_string(),
        );
    }

    // ② BinData 내장 스트림 — 실행체·OLE 패키지.
    for name in cfb.list_bin_data() {
        match cfb.read_bin_data_limited(&name, STREAM_SCAN_CAP) {
            Ok(bytes) => scan_embedded_bytes(&format!("BinData/{name}"), bytes, col),
            Err(e) => col.note(format!("BinData/{name} 를 깊이 스캔할 수 없습니다: {e}")),
        }
    }

    // ③ 외부 참조 — DocInfo 의 Link BinData 가 원격/UNC 를 가리키는가.
    //    ④ 레코드 오버런 — DocInfo·BodyText 레코드 스트림.
    // 암호화·배포용은 본문이 암호문이라 레코드로 해석하면 오탐이 난다 — 레코드 축을 끈다.
    if encrypted || distribution {
        col.note(
            "암호화/배포용 문서라 DocInfo·BodyText 내부를 레코드로 읽지 않았습니다(암호문 오탐 방지)."
                .to_string(),
        );
    }

    match cfb.read_doc_info_limited(compressed, STREAM_SCAN_CAP) {
        Ok(doc_info) => {
            if !(encrypted || distribution) {
                scan_records_for_overrun("DocInfo", &doc_info, col);
            }
            // DocInfo 를 구조 파싱해 Link BinData 의 원격 참조를 본다(best-effort).
            if let Ok((di, _)) = crate::parser::doc_info::parse_doc_info(&doc_info) {
                for bd in &di.bin_data_list {
                    if bd.data_type != crate::model::bin_data::BinDataType::Link {
                        continue;
                    }
                    let target = bd
                        .abs_path
                        .as_deref()
                        .filter(|s| looks_remote(s))
                        .or_else(|| bd.rel_path.as_deref().filter(|s| looks_remote(s)));
                    if let Some(t) = target {
                        col.push(
                            "external_reference",
                            Severity::Medium,
                            "DocInfo/BinData[Link]".to_string(),
                            Some(t.to_string()),
                            "문서가 원격/UNC 위치의 외부 파일을 링크로 참조합니다 — 열람 시 \
                             자동으로 외부 자원을 불러오도록 유도하는 형태일 수 있습니다."
                                .to_string(),
                        );
                    }
                }
            }
        }
        Err(e) => col.note(format!(
            "DocInfo 를 읽을 수 없어 레코드/링크 축을 건너뜁니다: {e}"
        )),
    }

    if !(encrypted || distribution) {
        let sections = cfb.section_count();
        for i in 0..sections {
            match cfb.read_body_text_section_limited(i, compressed, STREAM_SCAN_CAP) {
                Ok(sec) => scan_records_for_overrun(&format!("BodyText/Section{i}"), &sec, col),
                Err(e) => col.note(format!("BodyText/Section{i} 를 읽을 수 없습니다: {e}")),
            }
        }
    }

    ("hwp5", scopes)
}

// ── HWPX ────────────────────────────────────────────────────────────────────

fn scan_hwpx(data: &[u8], col: &mut Collector) -> (&'static str, Vec<&'static str>) {
    let scopes = vec![
        "binDataEntries",
        "oleObjects",
        "scriptEntries",
        "manifestExternalRefs",
    ];

    let mut reader = match crate::parser::hwpx::reader::HwpxReader::open(data) {
        Ok(r) => r,
        Err(e) => {
            col.note(format!("HWPX ZIP 을 열 수 없어 스캔을 건너뜁니다: {e}"));
            return ("hwpx", scopes);
        }
    };

    let names = reader.file_names();

    // ① 스크립트 엔트리.
    for name in &names {
        let norm = name.trim_start_matches('/');
        if norm.starts_with("Scripts/") && !norm.ends_with('/') {
            col.push(
                "macro_script",
                Severity::Medium,
                name.clone(),
                None,
                "HWPX 스크립트 엔트리입니다 — 패키지에 스크립트/매크로가 들어 있습니다."
                    .to_string(),
            );
        }
    }

    // ② BinData 내장 엔트리 — 실행체·OLE 패키지.
    for name in &names {
        let norm = name.trim_start_matches('/');
        if norm.starts_with("BinData/") && !norm.ends_with('/') {
            match reader.read_file_bytes_limited(name, STREAM_SCAN_CAP) {
                Ok(bytes) => scan_embedded_bytes(name, bytes, col),
                Err(e) => col.note(format!("{name} 를 깊이 스캔할 수 없습니다: {e}")),
            }
        }
    }

    // ③ 외부 참조 — content.hpf 매니페스트의 BinData href 가 원격을 가리키는가.
    if let Ok(hpf) = reader.read_file("Contents/content.hpf") {
        if let Ok(info) = crate::parser::hwpx::content::parse_content_hpf(&hpf) {
            for item in &info.bin_data_items {
                if looks_remote(&item.href) || (!item.is_embedded && item.href.contains("://")) {
                    col.push(
                        "external_reference",
                        Severity::Medium,
                        "Contents/content.hpf/manifest".to_string(),
                        Some(item.href.clone()),
                        "HWPX 매니페스트가 원격 위치의 외부 자원을 참조합니다 — 열람 시 \
                         자동으로 외부 자원을 불러오도록 유도하는 형태일 수 있습니다."
                            .to_string(),
                    );
                }
            }
        }
    }

    ("hwpx", scopes)
}

// ── 진입점 ──────────────────────────────────────────────────────────────────

/// 바이트를 스캔해 위협 보고서를 만든다. **읽기 전용** — 어떤 입력도 변경하지 않는다.
///
/// 형식(HWP5 CFB / HWPX ZIP)을 매직으로 판정해 알맞은 구조 스캐너를 돌린다. 알 수 없는
/// 형식이면 빈 보고서(스캔 범위 없음)를 돌려준다.
pub fn scan_bytes(source: &str, data: &[u8]) -> ThreatReport {
    use crate::parser::FileFormat;

    let mut col = Collector::new();
    let (format, scopes) = match detect_format(data) {
        FileFormat::Hwp => scan_hwp5(data, &mut col),
        FileFormat::Hwpx => scan_hwpx(data, &mut col),
        FileFormat::Hwp3 => {
            col.note(
                "HWP3(비 CFB) 형식은 이 구조 스캐너의 대상이 아닙니다 — 스캔하지 않았습니다."
                    .to_string(),
            );
            ("unknown", Vec::new())
        }
        other => {
            col.note(format!(
                "HWP/HWPX 컨테이너가 아니라 구조 스캔 대상이 아닙니다(감지: {other:?})."
            ));
            ("unknown", Vec::new())
        }
    };

    // 결정론적 순서: 심각도 내림차순 → 종류 → 주소 → detail.
    col.findings.sort_by(|a, b| {
        severity_rank_of(a.severity)
            .cmp(&severity_rank_of(b.severity))
            .then_with(|| a.kind.cmp(b.kind))
            .then_with(|| a.location.cmp(&b.location))
            .then_with(|| a.detail.cmp(&b.detail))
    });

    ThreatReport {
        source: source.to_string(),
        format,
        scopes,
        findings: col.findings,
        notes: col.notes,
        truncated: col.truncated,
    }
}

/// `--json` 봉투를 만든다(출처 표지는 호출부가 `provenance::marked` 로 붙인다).
pub fn envelope(report: &ThreatReport) -> serde_json::Value {
    serde_json::json!({
        "schemaVersion": crate::schema_registry::ENVELOPE_SCHEMA_VERSION,
        "source": report.source,
        "format": report.format,
        "scanScopes": report.scopes,
        "findings": report.findings,
        "findingCount": report.findings.len(),
        "highestSeverity": report.highest_severity(),
        "clean": report.clean(),
        "truncated": report.truncated,
        "notes": report.notes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn executable_magic_detects_pe_elf_macho() {
        assert_eq!(executable_magic(b"MZ\x90\x00"), Some("PE(MZ)"));
        assert_eq!(executable_magic(&[0x7F, 0x45, 0x4C, 0x46]), Some("ELF"));
        assert_eq!(executable_magic(&[0xFE, 0xED, 0xFA, 0xCE]), Some("Mach-O"));
        assert_eq!(executable_magic(b"BM\x00\x00"), None);
        assert_eq!(executable_magic(b"%PD"), None);
    }

    #[test]
    fn looks_remote_flags_url_and_unc_only() {
        assert!(looks_remote("http://evil.example/x.dll"));
        assert!(looks_remote("https://evil.example/x"));
        assert!(looks_remote("\\\\10.0.0.5\\share\\payload"));
        assert!(looks_remote("smb://host/share"));
        assert!(!looks_remote("images/logo.png"));
        assert!(!looks_remote("..\\rel\\local.bmp"));
        assert!(!looks_remote(""));
    }

    #[test]
    fn record_overrun_is_flagged_and_clean_stream_is_not() {
        // 정상: 크기 2인 레코드 하나(헤더 4B + 데이터 2B).
        let size: u32 = 2;
        let header = 0x10u32 | (size << 20);
        let mut clean = header.to_le_bytes().to_vec();
        clean.extend_from_slice(&[0xAA, 0xBB]);
        let mut col = Collector::new();
        scan_records_for_overrun("DocInfo", &clean, &mut col);
        assert!(col.findings.is_empty(), "정상 레코드는 신고되면 안 된다");

        // 오버런: 크기 9999를 선언하지만 데이터는 2바이트뿐.
        let header = 0x10u32 | (9999u32 << 20);
        let mut bad = header.to_le_bytes().to_vec();
        bad.extend_from_slice(&[0xAA, 0xBB]);
        let mut col = Collector::new();
        scan_records_for_overrun("DocInfo", &bad, &mut col);
        assert_eq!(col.findings.len(), 1);
        assert_eq!(col.findings[0].kind, "malformed_record");
        assert_eq!(col.findings[0].severity, "high");
    }

    #[test]
    fn unknown_format_scans_nothing() {
        let report = scan_bytes("x.bin", b"not a document");
        assert_eq!(report.format, "unknown");
        assert!(report.scopes.is_empty());
        assert!(report.clean());
    }
}
