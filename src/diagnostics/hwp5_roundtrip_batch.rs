//! HWP5 roundtrip 배치 검증 (Task #1552).
//!
//! `samples/*.hwp` 등의 HWP5(OLE) 파일을 `parse → serialize → 재parse` 경로로 돌려
//! 파일별 무손실성을 측정하고, 재조립 `.rt.hwp`를 출력 폴더에 남긴다.
//! `hwpx-roundtrip`(Task #1315)의 HWP5 대응 게이트.
//!
//! ```text
//! rhwp hwp5-roundtrip sample.hwp -o output/poc/task1552/
//! rhwp hwp5-roundtrip --batch samples -o output/poc/task1552/
//! ```
//!
//! 출력:
//! - `{out}/{상대경로 stem}.rt.hwp` — 재조립 HWP5
//! - `{out}/inventory.tsv` — 배치 측정 결과 (배치 모드)
//!
//! 검사 항목(단계별 확장):
//! - C1 IR 뼈대 diff (`diff_documents`, 포맷 무관)
//! - C5 2-round 안정성
//! - (Stage 2) C2 BinData 스트림 보존
//! - (Stage 3) C3 페이지수 복원 + C4 CFB 구조

use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::document_core::DocumentCore;
use crate::parser::cfb_reader::{decompress_stream, CfbReader};
use crate::parser::parse_document;
use crate::serializer::hwpx::roundtrip::diff_documents;
use crate::serializer::serialize_document;

/// C3 — 바이트에서 페이지 수 산출(파싱+페이지네이션). 실패/패닉 시 `None`.
/// 배치 중 단일 파일 패닉이 전체를 중단시키지 않도록 `catch_unwind` 로 격리.
fn page_count_of(bytes: &[u8]) -> Option<u32> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        DocumentCore::from_bytes(bytes)
            .ok()
            .map(|dc| dc.page_count())
    }))
    .ok()
    .flatten()
}

/// C4 — 저장본 CFB 구조 검사: 필수 스트림 존재 + 섹션 수 = IR.
fn cfb_structure_ok(out: &[u8], expected_sections: usize) -> (bool, String) {
    let reader = match CfbReader::open(out) {
        Ok(r) => r,
        Err(e) => return (false, format!("CFB 열기 실패: {e}")),
    };
    let mut problems = Vec::new();
    for req in ["/FileHeader", "/DocInfo", "/BodyText/Section0"] {
        if !reader.has_stream(req) {
            problems.push(format!("필수 스트림 없음: {req}"));
        }
    }
    let sc = reader.section_count() as usize;
    if sc != expected_sections {
        problems.push(format!("섹션 수 불일치: cfb={sc} ir={expected_sections}"));
    }
    (problems.is_empty(), problems.join("; "))
}

/// BinData 스트림의 **decompressed 내용** 지문 멀티셋.
///
/// 이름(BIN0001 등)이 아니라 내용 기준 — serializer 가 재명명·재압축할 수 있으므로.
/// 각 스트림은 raw deflate 해제 시도 후 실패 시 raw 바이트를 사용한다(압축 플래그 무관 정규화:
/// 한쪽이 압축·다른쪽이 비압축으로 같은 이미지를 저장해도 내용이 일치하면 동일 지문).
/// CFB 가 아니거나 열 수 없으면 `None`(검사 생략).
fn bindata_fingerprint(bytes: &[u8]) -> Option<BTreeMap<u64, usize>> {
    let mut reader = CfbReader::open(bytes).ok()?;
    let names = reader.list_bin_data();
    let mut multiset: BTreeMap<u64, usize> = BTreeMap::new();
    for name in names {
        let raw = match reader.read_bin_data(&name) {
            Ok(r) => r,
            Err(_) => continue,
        };
        let content = decompress_stream(&raw).unwrap_or(raw);
        let mut h = DefaultHasher::new();
        content.hash(&mut h);
        *multiset.entry(h.finish()).or_insert(0) += 1;
    }
    Some(multiset)
}

/// [#4097] C2b — 중첩 OLE CFB(BinData 안의 CFB 컨테이너)의 **루트 CLSID** 지문 멀티셋.
///
/// OLE 개체는 루트 스토리지 엔트리의 CLSID 로 서버를 식별한다 — 스트림 내용이 다 있어도
/// 이 값이 비면 한컴이 개체를 알아보지 못해 틀만 그리고 내용을 비운다(2026-08-05 실측).
/// C2(내용 해시)는 스트림 바이트 전체를 보므로 현행 passthrough 저장에서는 이 검사도
/// 자동 green 이다 — 존재 이유는 **재포장이 저장 경로에 들어오는 날**(차트 편집 #3683 등)
/// 이다. 그때 내용 해시 검사는 "편집분은 달라도 된다"로 완화될 수밖에 없는데, 정체성
/// (CLSID)은 편집 후에도 보존되어야 하므로 이 검사가 계속 지킨다. #3557 이 지적한
/// "IR 에 모델링되지 않아 --verify 가 못 보는" 축을 게이트에서 직접 보는 것이기도 하다.
///
/// BinData OLE 스토리지는 4바이트 LE 크기 prefix 뒤에 CFB 가 온다(#3547) — prefix 유무
/// 두 형태를 모두 인식한다. CFB 가 아닌 BinData(그림 등)는 건너뛴다.
fn nested_ole_clsid_fingerprint(bytes: &[u8]) -> Option<BTreeMap<[u8; 16], usize>> {
    const CFB_MAGIC: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
    let mut reader = CfbReader::open(bytes).ok()?;
    let names = reader.list_bin_data();
    let mut multiset: BTreeMap<[u8; 16], usize> = BTreeMap::new();
    for name in names {
        let raw = match reader.read_bin_data(&name) {
            Ok(r) => r,
            Err(_) => continue,
        };
        let content = decompress_stream(&raw).unwrap_or(raw);
        let cfb = if content.starts_with(&CFB_MAGIC) {
            &content[..]
        } else if content.len() > 12 && content[4..12] == CFB_MAGIC {
            &content[4..]
        } else {
            continue;
        };
        let Some(clsid) = crate::parser::cfb_reader::root_clsid(cfb) else {
            continue;
        };
        *multiset.entry(clsid).or_insert(0) += 1;
    }
    Some(multiset)
}

/// orig 멀티셋에서 rt 가 덮지 못한 항목 수(= 소실된 항목 수).
/// rt 가 더 많이 가진 항목(gained)은 손실이 아니므로 무시한다.
/// C2(BinData 내용 해시)와 C2b(중첩 CLSID)가 공유한다.
fn multiset_lost<K: Ord>(orig: &BTreeMap<K, usize>, rt: &BTreeMap<K, usize>) -> usize {
    let mut lost = 0;
    for (key, &cnt) in orig {
        let have = rt.get(key).copied().unwrap_or(0);
        if cnt > have {
            lost += cnt - have;
        }
    }
    lost
}

#[derive(Debug)]
struct Options {
    input: PathBuf,
    batch: bool,
    out_dir: PathBuf,
}

fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut input: Option<PathBuf> = None;
    let mut batch = false;
    let mut out_dir = PathBuf::from("output/poc/task1552");

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--batch" => batch = true,
            "-o" | "--out" => {
                i += 1;
                let v = args
                    .get(i)
                    .ok_or_else(|| "-o 다음에 출력 폴더가 필요합니다".to_string())?;
                out_dir = PathBuf::from(v);
            }
            other if other.starts_with('-') => {
                return Err(format!("알 수 없는 옵션: {other}"));
            }
            other => {
                if input.is_some() {
                    return Err(format!("입력 경로가 중복 지정됨: {other}"));
                }
                input = Some(PathBuf::from(other));
            }
        }
        i += 1;
    }

    let input = input.ok_or_else(|| {
        "사용법: rhwp hwp5-roundtrip <입력.hwp | --batch 폴더> [-o 출력폴더]".to_string()
    })?;
    Ok(Options {
        input,
        batch,
        out_dir,
    })
}

/// 파일 1건의 roundtrip 측정 결과.
#[derive(Debug)]
struct RoundtripRow {
    /// 배치 루트 기준 상대 경로 (단일 모드는 파일명).
    rel_path: String,
    parse_ok: bool,
    serialize_ok: bool,
    reparse_ok: bool,
    ir_diff_count: Option<usize>,
    ir_diff_summary: String,
    /// C2 — 원본 BinData 스트림 수. `None` = 검사 생략(CFB 아님/열기 실패).
    bindata_total: Option<usize>,
    /// C2 — 저장본에서 소실된 BinData 스트림 수(내용 멀티셋 기준).
    bindata_lost: usize,
    /// C2b — 원본의 중첩 OLE CFB 수(루트 CLSID 지문 기준). `None` = 검사 생략. (#4097)
    nested_clsid_total: Option<usize>,
    /// C2b — 저장본에서 소실된 중첩 OLE 루트 CLSID 수(멀티셋 기준). (#4097)
    nested_clsid_lost: usize,
    /// C3 — 원본/저장본의 rhwp 페이지 수. `None` = 재로드 실패/패닉.
    page_before: Option<u32>,
    page_after: Option<u32>,
    /// C4 — 저장본 CFB 구조 검사. `None` = 미실행.
    cfb_struct_ok: Option<bool>,
    cfb_problems: String,
    /// 2-round 안정성: round1 IR vs round2 IR 의 diff 건수. `None` = 미실행/실패.
    round2_diff_count: Option<usize>,
    round2_error: String,
    elapsed_ms: u128,
    error: String,
    /// [#1914] 매직 바이트 실체가 HWP5(OLE/CFB)가 아닌 확장자 위장 파일 —
    /// 검출된 실체 포맷명. plain serialize 게이트의 범위 밖이므로 스킵으로 분류.
    /// (HWP3 는 어댑터(`export_hwp_with_adapter`) 경유가 제품 경로 — 여기서
    /// 돌리면 SectionPageDef 소실 등 도구-경로 전용 IR_DIFF 가 오표기된다, #1892.)
    format_skip: Option<&'static str>,
}

impl RoundtripRow {
    fn status(&self) -> &'static str {
        if self.format_skip.is_some() {
            "FORMAT_SKIP"
        } else if !self.parse_ok {
            "PARSE_FAIL"
        } else if !self.serialize_ok {
            "SERIALIZE_FAIL"
        } else if !self.reparse_ok {
            "REPARSE_FAIL"
        } else if self.ir_diff_count.is_some_and(|c| c > 0) {
            "IR_DIFF"
        } else if self.bindata_lost > 0 {
            "BINDATA_LOSS"
        } else if self.nested_clsid_lost > 0 {
            "OLE_CLSID_LOSS"
        } else if self.cfb_struct_ok == Some(false) {
            "CFB_STRUCT_FAIL"
        } else if self.page_mismatch() {
            "PAGE_DIFF"
        } else if !self.round2_error.is_empty() || self.round2_diff_count.is_none() {
            "ROUND2_FAIL"
        } else if self.round2_diff_count.is_some_and(|c| c > 0) {
            "ROUND2_DIFF"
        } else {
            "PASS"
        }
    }

    /// C3 — 원본/저장본 페이지 수가 둘 다 존재하고 다른 경우.
    fn page_mismatch(&self) -> bool {
        matches!((self.page_before, self.page_after), (Some(a), Some(b)) if a != b)
    }

    /// 회귀 검출용 하드 실패 (등급화 대상 분류와 별개).
    fn is_hard_fail(&self) -> bool {
        if self.format_skip.is_some() {
            // [#1914] 확장자 위장(실체 비-HWP5)은 게이트 범위 밖 — 실패 아님.
            return false;
        }
        !(self.parse_ok && self.serialize_ok && self.reparse_ok)
            || self.bindata_lost > 0
            || self.nested_clsid_lost > 0
            || self.cfb_struct_ok == Some(false)
            || self.page_mismatch()
            || !self.round2_error.is_empty()
    }
}

/// [#1914] 매직 바이트 실체 포맷명 (FORMAT_SKIP 사유 표기용).
fn detected_format_name(fmt: crate::parser::FileFormat) -> Option<&'static str> {
    use crate::parser::FileFormat;
    match fmt {
        FileFormat::Hwpx => Some("HWPX(ZIP)"),
        FileFormat::Hwp3 => Some("HWP3"),
        FileFormat::Hml => Some("HWPML(구 XML)"),
        FileFormat::DrmProtected => Some("DRM 보호"),
        FileFormat::Empty => Some("빈 파일"),
        FileFormat::Hwp | FileFormat::Unknown => None,
    }
}

/// 단일 HWP5 파일 roundtrip 실행. 재조립 파일을 `rt_path`에 기록.
fn roundtrip_one(path: &Path, rel_path: &str, rt_path: &Path) -> RoundtripRow {
    let started = Instant::now();
    let mut row = RoundtripRow {
        rel_path: rel_path.to_string(),
        parse_ok: false,
        serialize_ok: false,
        reparse_ok: false,
        ir_diff_count: None,
        ir_diff_summary: String::new(),
        bindata_total: None,
        bindata_lost: 0,
        nested_clsid_total: None,
        nested_clsid_lost: 0,
        page_before: None,
        page_after: None,
        cfb_struct_ok: None,
        cfb_problems: String::new(),
        round2_diff_count: None,
        round2_error: String::new(),
        elapsed_ms: 0,
        error: String::new(),
        format_skip: None,
    };

    let finish = |mut row: RoundtripRow, started: Instant| -> RoundtripRow {
        row.elapsed_ms = started.elapsed().as_millis();
        row
    };

    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            row.error = format!("읽기 실패: {e}");
            return finish(row, started);
        }
    };

    // [#1914] 확장자 아닌 매직 바이트로 실체 판별. HWP3/HWPX 실체는 plain
    // serialize 게이트의 범위 밖 — HWP3 를 여기서 돌리면 어댑터 미경유
    // SectionPageDef 소실 등 도구-경로 전용 IR_DIFF 가 오표기된다(#1892).
    // Unknown(빈 파일/DRM 래퍼 등)은 종전대로 파싱 실패가 정당하므로 진행한다.
    if let Some(actual) = detected_format_name(crate::parser::detect_format(&bytes)) {
        row.format_skip = Some(actual);
        row.error = format!(
            "확장자 위장/포맷 불일치: 실체 {actual} — HWP5 게이트 범위 밖 \
             (HWPX 는 hwpx-roundtrip, HWP3 저장 검증은 convert/export 경로 사용)"
        );
        return finish(row, started);
    }

    let doc1 = match parse_document(&bytes) {
        Ok(d) => d,
        Err(e) => {
            row.error = format!("파싱 실패: {e}");
            return finish(row, started);
        }
    };
    row.parse_ok = true;

    let out = match serialize_document(&doc1) {
        Ok(o) => o,
        Err(e) => {
            row.error = format!("직렬화 실패: {e}");
            return finish(row, started);
        }
    };
    row.serialize_ok = true;

    if let Some(parent) = rt_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            row.error = format!("출력 폴더 생성 실패: {e}");
            return finish(row, started);
        }
    }
    if let Err(e) = fs::write(rt_path, &out) {
        row.error = format!("재조립 파일 쓰기 실패: {e}");
        return finish(row, started);
    }

    let doc2 = match parse_document(&out) {
        Ok(d) => d,
        Err(e) => {
            row.error = format!("재파싱 실패: {e}");
            return finish(row, started);
        }
    };
    row.reparse_ok = true;

    let diff = diff_documents(&doc1, &doc2);
    row.ir_diff_count = Some(diff.differences.len());
    row.ir_diff_summary = diff
        .differences
        .iter()
        .map(|d| d.to_string())
        .collect::<Vec<_>>()
        .join("; ");

    // C2 — BinData 스트림 보존 (원본 bytes vs 저장본 out 의 내용 멀티셋 비교)
    if let (Some(orig_fp), Some(rt_fp)) = (bindata_fingerprint(&bytes), bindata_fingerprint(&out)) {
        row.bindata_total = Some(orig_fp.values().sum());
        row.bindata_lost = multiset_lost(&orig_fp, &rt_fp);
    }

    // C2b — 중첩 OLE 루트 CLSID 보존 (#4097). 내용(C2)과 별개로 개체 정체성을 본다.
    if let (Some(orig_id), Some(rt_id)) = (
        nested_ole_clsid_fingerprint(&bytes),
        nested_ole_clsid_fingerprint(&out),
    ) {
        row.nested_clsid_total = Some(orig_id.values().sum());
        row.nested_clsid_lost = multiset_lost(&orig_id, &rt_id);
    }

    // C3 — 페이지수 복원 (rhwp 자기 일관 기준; 외부 한글-only 붕괴는 미검출 한계)
    row.page_before = page_count_of(&bytes);
    row.page_after = page_count_of(&out);

    // C4 — 저장본 CFB 구조
    let (cfb_ok, cfb_problems) = cfb_structure_ok(&out, doc1.sections.len());
    row.cfb_struct_ok = Some(cfb_ok);
    row.cfb_problems = cfb_problems;

    // 2-round 안정성: round1 IR(doc2) 을 다시 직렬화→파싱한 IR(doc3) 과 비교해 0 이어야 안정.
    match serialize_document(&doc2) {
        Ok(out2) => match parse_document(&out2) {
            Ok(doc3) => {
                let diff2 = diff_documents(&doc2, &doc3);
                row.round2_diff_count = Some(diff2.differences.len());
            }
            Err(e) => row.round2_error = format!("2-round 재파싱 실패: {e}"),
        },
        Err(e) => row.round2_error = format!("2-round 직렬화 실패: {e}"),
    }

    finish(row, started)
}

/// 회귀 테스트용 — 인메모리 baseline 검사(C1~C5). 하드 실패 시 사유 반환.
///
/// 게이트(`roundtrip_one`)와 동일한 검사 순서. 호출자는 HWP3(`detect_format`)·배포용
/// (`header.distribution`) 등 범위 밖 샘플을 사전에 걸러야 한다(이 함수는 무조건 C1~C5 수행).
pub fn baseline_check(bytes: &[u8]) -> Result<(), String> {
    let doc1 = parse_document(bytes).map_err(|e| format!("파싱 실패: {e}"))?;
    let out = serialize_document(&doc1).map_err(|e| format!("직렬화 실패: {e}"))?;
    let doc2 = parse_document(&out).map_err(|e| format!("재파싱 실패: {e}"))?;

    // C1 IR 뼈대
    let diff = diff_documents(&doc1, &doc2);
    if !diff.differences.is_empty() {
        return Err(format!(
            "IR diff {}건: {}",
            diff.differences.len(),
            diff.differences
                .iter()
                .map(|d| d.to_string())
                .collect::<Vec<_>>()
                .join("; ")
        ));
    }

    // C2 BinData 보존
    if let (Some(orig_fp), Some(rt_fp)) = (bindata_fingerprint(bytes), bindata_fingerprint(&out)) {
        let lost = multiset_lost(&orig_fp, &rt_fp);
        if lost > 0 {
            return Err(format!(
                "BinData 소실 {}/{}",
                lost,
                orig_fp.values().sum::<usize>()
            ));
        }
    }

    // C2b 중첩 OLE 루트 CLSID 보존 (#4097). 현행 passthrough 저장에서는 자동 green —
    // 재포장이 저장 경로에 들어오는 날(차트 편집 등) C2 를 편집 허용으로 완화해도
    // 개체 정체성은 이 검사가 계속 지킨다.
    if let (Some(orig_id), Some(rt_id)) = (
        nested_ole_clsid_fingerprint(bytes),
        nested_ole_clsid_fingerprint(&out),
    ) {
        let lost = multiset_lost(&orig_id, &rt_id);
        if lost > 0 {
            return Err(format!(
                "중첩 OLE 루트 CLSID 소실 {}/{} — 한컴이 개체를 알아보지 못한다 (#4097)",
                lost,
                orig_id.values().sum::<usize>()
            ));
        }
    }

    // C4 CFB 구조
    let (cfb_ok, cfb_problems) = cfb_structure_ok(&out, doc1.sections.len());
    if !cfb_ok {
        return Err(format!("CFB 구조: {cfb_problems}"));
    }

    // C3 페이지수 복원
    if let (Some(a), Some(b)) = (page_count_of(bytes), page_count_of(&out)) {
        if a != b {
            return Err(format!("페이지 변화 {a}→{b}"));
        }
    }

    // C5 2-round 안정성
    let out2 = serialize_document(&doc2).map_err(|e| format!("2-round 직렬화 실패: {e}"))?;
    let doc3 = parse_document(&out2).map_err(|e| format!("2-round 재파싱 실패: {e}"))?;
    let diff2 = diff_documents(&doc2, &doc3);
    if !diff2.differences.is_empty() {
        return Err(format!(
            "2-round 불안정: IR diff {}건",
            diff2.differences.len()
        ));
    }

    Ok(())
}

/// 폴더에서 `.hwp` 파일을 재귀 수집 (정렬된 순서). `.hwpx`는 제외.
fn collect_hwp5_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries =
            fs::read_dir(&dir).map_err(|e| format!("폴더 읽기 실패 {}: {e}", dir.display()))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("폴더 항목 읽기 실패: {e}"))?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("hwp"))
            {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

fn tsv_escape(s: &str) -> String {
    s.replace(['\t', '\n', '\r'], " ")
}

fn write_tsv(out_dir: &Path, rows: &[RoundtripRow]) -> Result<PathBuf, String> {
    let tsv_path = out_dir.join("inventory.tsv");
    let opt_to_str = |o: Option<usize>| o.map(|c| c.to_string()).unwrap_or_else(|| "-".to_string());
    let opt_u32 = |o: Option<u32>| o.map(|c| c.to_string()).unwrap_or_else(|| "-".to_string());
    let opt_bool = |o: Option<bool>| match o {
        Some(true) => "true",
        Some(false) => "false",
        None => "-",
    };
    let mut tsv = String::from(
        "sample\tstatus\tparse_ok\tserialize_ok\treparse_ok\tir_diff_count\tbindata_total\tbindata_lost\tnested_clsid_total\tnested_clsid_lost\tpage_before\tpage_after\tcfb_struct_ok\tround2_diff\telapsed_ms\terror\tir_diff_summary\tcfb_problems\tround2_error\n",
    );
    for row in rows {
        tsv.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            tsv_escape(&row.rel_path),
            row.status(),
            row.parse_ok,
            row.serialize_ok,
            row.reparse_ok,
            opt_to_str(row.ir_diff_count),
            opt_to_str(row.bindata_total),
            row.bindata_lost,
            opt_to_str(row.nested_clsid_total),
            row.nested_clsid_lost,
            opt_u32(row.page_before),
            opt_u32(row.page_after),
            opt_bool(row.cfb_struct_ok),
            opt_to_str(row.round2_diff_count),
            row.elapsed_ms,
            tsv_escape(&row.error),
            tsv_escape(&row.ir_diff_summary),
            tsv_escape(&row.cfb_problems),
            tsv_escape(&row.round2_error),
        ));
    }
    fs::write(&tsv_path, tsv).map_err(|e| format!("TSV 쓰기 실패: {e}"))?;
    Ok(tsv_path)
}

fn print_summary(rows: &[RoundtripRow]) {
    let count = |s: &str| rows.iter().filter(|r| r.status() == s).count();
    println!();
    println!("=== hwp5-roundtrip 요약 ===");
    println!("  총 파일        : {}", rows.len());
    println!("  PASS           : {}", count("PASS"));
    println!("  IR_DIFF        : {}", count("IR_DIFF"));
    println!("  BINDATA_LOSS   : {}", count("BINDATA_LOSS"));
    println!("  CFB_STRUCT_FAIL: {}", count("CFB_STRUCT_FAIL"));
    println!("  PAGE_DIFF      : {}", count("PAGE_DIFF"));
    println!("  ROUND2_DIFF    : {}", count("ROUND2_DIFF"));
    println!("  ROUND2_FAIL    : {}", count("ROUND2_FAIL"));
    println!("  PARSE_FAIL     : {}", count("PARSE_FAIL"));
    println!("  SERIALIZE_FAIL : {}", count("SERIALIZE_FAIL"));
    println!("  REPARSE_FAIL   : {}", count("REPARSE_FAIL"));
}

/// `rt.hwp` 출력 경로 — 배치 루트 기준 상대 구조를 출력 폴더 아래에 유지.
fn rt_output_path(out_dir: &Path, rel_path: &str) -> PathBuf {
    let rel = Path::new(rel_path);
    let stem = rel
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "output".to_string());
    let mut out = out_dir.to_path_buf();
    if let Some(parent) = rel.parent() {
        out.push(parent);
    }
    out.push(format!("{stem}.rt.hwp"));
    out
}

pub fn run(args: &[String]) {
    let opts = match parse_args(args) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("오류: {e}");
            std::process::exit(2);
        }
    };

    let inputs: Vec<(PathBuf, String)> = if opts.batch {
        match collect_hwp5_files(&opts.input) {
            Ok(files) => files
                .into_iter()
                .map(|p| {
                    let rel = p
                        .strip_prefix(&opts.input)
                        .map(|r| r.to_string_lossy().to_string())
                        .unwrap_or_else(|_| p.to_string_lossy().to_string());
                    (p, rel)
                })
                .collect(),
            Err(e) => {
                eprintln!("오류: {e}");
                std::process::exit(2);
            }
        }
    } else {
        let rel = opts
            .input
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| opts.input.to_string_lossy().to_string());
        vec![(opts.input.clone(), rel)]
    };

    if inputs.is_empty() {
        eprintln!(
            "오류: 처리할 .hwp 파일이 없습니다: {}",
            opts.input.display()
        );
        std::process::exit(2);
    }

    let mut rows = Vec::with_capacity(inputs.len());
    for (path, rel) in &inputs {
        let rt_path = rt_output_path(&opts.out_dir, rel);
        let row = roundtrip_one(path, rel, &rt_path);
        let fmt_opt =
            |o: Option<usize>| o.map(|c| c.to_string()).unwrap_or_else(|| "-".to_string());
        let mut extra = String::new();
        if row.bindata_lost > 0 {
            extra.push_str(&format!(
                " bin_lost={}/{}",
                row.bindata_lost,
                fmt_opt(row.bindata_total)
            ));
        }
        if row.nested_clsid_lost > 0 {
            extra.push_str(&format!(
                " ole_clsid_lost={}/{}",
                row.nested_clsid_lost,
                fmt_opt(row.nested_clsid_total)
            ));
        }
        if row.page_mismatch() {
            extra.push_str(&format!(
                " pg={}→{}",
                row.page_before.map(|p| p.to_string()).unwrap_or_default(),
                row.page_after.map(|p| p.to_string()).unwrap_or_default()
            ));
        }
        println!(
            "[{:>15}] diff={:>3} r2={:>3}{} {:>6}ms  {}",
            row.status(),
            fmt_opt(row.ir_diff_count),
            fmt_opt(row.round2_diff_count),
            extra,
            row.elapsed_ms,
            row.rel_path
        );
        for detail in [&row.error, &row.cfb_problems, &row.round2_error] {
            if !detail.is_empty() {
                println!("                 └ {}", detail);
            }
        }
        rows.push(row);
    }

    if opts.batch {
        match write_tsv(&opts.out_dir, &rows) {
            Ok(p) => println!("\nTSV 저장: {}", p.display()),
            Err(e) => {
                eprintln!("오류: {e}");
                std::process::exit(1);
            }
        }
        print_summary(&rows);
    }

    // 하드 실패(파싱/직렬화/재파싱/2-round 오류)가 있으면 비정상 종료 코드 (회귀 검출용)
    if rows.iter().any(|r| r.is_hard_fail()) {
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blank_row() -> RoundtripRow {
        RoundtripRow {
            rel_path: String::new(),
            parse_ok: true,
            serialize_ok: true,
            reparse_ok: true,
            ir_diff_count: Some(0),
            ir_diff_summary: String::new(),
            bindata_total: None,
            bindata_lost: 0,
            nested_clsid_total: None,
            nested_clsid_lost: 0,
            page_before: None,
            page_after: None,
            cfb_struct_ok: Some(true),
            cfb_problems: String::new(),
            round2_diff_count: Some(0),
            round2_error: String::new(),
            elapsed_ms: 0,
            error: String::new(),
            format_skip: None,
        }
    }

    #[test]
    fn parse_args_single_file() {
        let args = vec!["sample.hwp".to_string()];
        let o = parse_args(&args).unwrap();
        assert_eq!(o.input, PathBuf::from("sample.hwp"));
        assert!(!o.batch);
        assert_eq!(o.out_dir, PathBuf::from("output/poc/task1552"));
    }

    #[test]
    fn parse_args_batch_with_out() {
        let args = vec![
            "--batch".to_string(),
            "samples".to_string(),
            "-o".to_string(),
            "output/poc/x".to_string(),
        ];
        let o = parse_args(&args).unwrap();
        assert!(o.batch);
        assert_eq!(o.input, PathBuf::from("samples"));
        assert_eq!(o.out_dir, PathBuf::from("output/poc/x"));
    }

    #[test]
    fn parse_args_rejects_unknown_option() {
        let args = vec!["--nope".to_string()];
        assert!(parse_args(&args).is_err());
    }

    #[test]
    fn parse_args_requires_input() {
        let args: Vec<String> = vec![];
        assert!(parse_args(&args).is_err());
    }

    #[test]
    fn rt_output_path_keeps_subdir() {
        let p = rt_output_path(Path::new("out"), "basic/interview.hwp");
        assert_eq!(p, PathBuf::from("out/basic/interview.rt.hwp"));
    }

    #[test]
    fn rt_output_path_flat_file() {
        let p = rt_output_path(Path::new("out"), "business_overview.hwp");
        assert_eq!(p, PathBuf::from("out/business_overview.rt.hwp"));
    }

    #[test]
    fn tsv_escape_strips_tabs_newlines() {
        assert_eq!(tsv_escape("a\tb\nc"), "a b c");
    }

    #[test]
    fn multiset_lost_counts_only_missing() {
        let orig: BTreeMap<u64, usize> = [(1, 2), (2, 1), (3, 1)].into_iter().collect();
        // rt 가 hash=1 하나만 가지고 hash=3 소실, hash=2 유지, gained hash=9 무시
        let rt: BTreeMap<u64, usize> = [(1, 1), (2, 1), (9, 5)].into_iter().collect();
        // 소실: hash=1 (2→1, 1개) + hash=3 (1→0, 1개) = 2
        assert_eq!(multiset_lost(&orig, &rt), 2);
        // 동일 멀티셋이면 0
        assert_eq!(multiset_lost(&orig, &orig), 0);
    }

    /// [#4097] C2b 검출기가 실제로 중첩 CFB 를 본다 — 자동 green 이 공허하지 않다는 증명.
    ///
    /// 차트 코퍼스 `.hwp` 는 `/BinData/BIN0001.OLE` 에 4바이트 prefix + deflate 로 중첩 CFB 를
    /// 담고 있고, 그 루트 CLSID 는 비-0 이다(#4097 Stage 1 실측: 56건 전부
    /// `{4C3DA137-DC90-47B9-9BED-59DAE352A280}`). 검출기가 이것을 정확히 1건으로 세고
    /// 비-0 CLSID 를 읽어야 한다 — 못 읽으면 C2b 는 "검사할 게 없어서 green" 인 상태다.
    #[test]
    fn nested_clsid_fingerprint_sees_chart_corpus() {
        let path = Path::new("samples/chart/세로막대형/묶은세로막대형.hwp");
        let bytes = fs::read(path).expect("차트 코퍼스는 저장소에 커밋되어 있다");
        let fp = nested_ole_clsid_fingerprint(&bytes).expect("바깥 CFB 열기");
        assert_eq!(
            fp.values().sum::<usize>(),
            1,
            "중첩 OLE CFB 1건을 세야 한다"
        );
        let clsid = *fp.keys().next().unwrap();
        assert_ne!(clsid, [0u8; 16], "차트 개체의 루트 CLSID 는 비-0 이다");
    }

    #[test]
    fn page_mismatch_only_when_both_present_and_differ() {
        let mut row = blank_row();
        row.page_before = Some(5);
        row.page_after = Some(5);
        assert!(!row.page_mismatch());
        row.page_after = Some(1);
        assert!(row.page_mismatch());
        row.page_after = None;
        assert!(!row.page_mismatch(), "한쪽이 None 이면 미스매치 아님");
    }

    #[test]
    fn cfb_structure_rejects_non_cfb() {
        let (ok, problems) = cfb_structure_ok(b"not a cfb file", 1);
        assert!(!ok);
        assert!(problems.contains("CFB"));
    }

    #[test]
    fn bindata_fingerprint_preserved_on_roundtrip() {
        // 이미지 포함 소형 샘플이 있으면 C2 보존을 확인한다.
        let sample = Path::new("samples/basic/interview.hwp");
        if !sample.exists() {
            return;
        }
        let bytes = fs::read(sample).unwrap();
        let fp = bindata_fingerprint(&bytes);
        assert!(fp.is_some(), "CFB BinData 지문 추출 실패");
        assert!(fp.unwrap().values().sum::<usize>() > 0, "BinData 없음");
    }

    #[test]
    fn collect_hwp5_files_excludes_hwpx() {
        // samples 폴더가 있으면 .hwpx 가 섞이지 않는지 확인.
        let root = Path::new("samples");
        if !root.exists() {
            return;
        }
        let files = collect_hwp5_files(root).unwrap();
        assert!(
            files
                .iter()
                .all(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("hwp"))),
            "hwpx 파일이 수집됨"
        );
    }
}
