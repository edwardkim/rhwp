// 관계형·고정형: 등록한 원본/PDF/전쪽 TSV의 바이트·쪽 대응 계약이다.
// 주석 갱신: 2026-11-22T12:13:07+09:00. 이 helper는 문서나 glyph를 렌더하지 않는다.
// 실제 문서의 시각 근거는 호출 검사 oracle_page_count_baseline의 첫머리에 기록한다.
use sha2::{Digest as _, Sha256};
use std::collections::BTreeMap;
use std::path::{Component, Path};

fn verified_bytes(root: &Path, relative: &str, expected: &str) -> Result<Vec<u8>, String> {
    let name = Path::new(relative);
    if name.is_absolute()
        || name
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
        || !["samples", "pdf", "mydocs/pr/assets"]
            .iter()
            .any(|prefix| name.starts_with(prefix))
    {
        return Err(format!("논리 쪽 증거 경로 오류: {relative}"));
    }
    let path = root.join(name).canonicalize().map_err(|e| e.to_string())?;
    let canonical_root = root.canonicalize().map_err(|e| e.to_string())?;
    if !path.starts_with(canonical_root) {
        return Err(format!("저장소 밖 논리 쪽 증거: {relative}"));
    }
    let data = std::fs::read(path).map_err(|e| e.to_string())?;
    let actual: String = Sha256::digest(&data)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    if actual != expected {
        return Err(format!("논리 쪽 증거 바이트 불일치: {relative}"));
    }
    Ok(data)
}

fn validate_full_page_tsv(data: &[u8], pages: u32) -> Result<(), String> {
    let text = std::str::from_utf8(data).map_err(|e| e.to_string())?;
    let mut lines = text.lines();
    let header: Vec<_> = lines.next().unwrap_or_default().split('\t').collect();
    let page_index = header
        .iter()
        .position(|v| *v == "page")
        .ok_or("쪽 열 누락")?;
    let raw_index = header
        .iter()
        .position(|v| *v == "silhouette_raw_match_percent")
        .ok_or("raw 열 누락")?;
    let mut count = 0;
    for line in lines {
        let values: Vec<_> = line.split('\t').collect();
        let page = values
            .get(page_index)
            .ok_or("쪽 누락")?
            .parse::<u32>()
            .map_err(|e| e.to_string())?;
        let raw = values
            .get(raw_index)
            .ok_or("raw 누락")?
            .parse::<f64>()
            .map_err(|e| e.to_string())?;
        count += 1;
        if page != count || !raw.is_finite() || !(98.0..=100.0).contains(&raw) {
            return Err("논리 쪽 전쪽 TSV의 페이지 또는 98% 선행 조건 미달".into());
        }
    }
    if count != pages || pages == 0 {
        return Err("논리 쪽 전쪽 TSV 쪽수 불일치".into());
    }
    Ok(())
}

fn field<'a>(entry: &'a serde_json::Value, key: &str) -> Result<&'a str, String> {
    entry[key]
        .as_str()
        .ok_or_else(|| format!("논리 쪽 등록 필드 누락: {key}"))
}

pub fn load(root: &Path) -> Result<BTreeMap<String, u32>, String> {
    let bytes =
        std::fs::read(root.join("tests/fixtures/oracle_page_count_verified_references.json"))
            .map_err(|e| e.to_string())?;
    let registry: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    if registry["schema_version"].as_u64() != Some(1) {
        return Err("논리 쪽 등록 스키마 오류".into());
    }
    let entries = registry["references"].as_array().ok_or("등록 목록 누락")?;
    let mut result = BTreeMap::new();
    for entry in entries {
        let Some(proof) = entry.get("logical_page_comparison") else {
            continue;
        };
        if entry["provenance"] != "desktop_hancom_print" {
            return Err("논리 쪽 독립 Print 출처 누락".into());
        }
        let sample = field(entry, "sample")?;
        let pages = entry["pages"]
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .filter(|n| *n > 0)
            .ok_or("논리 쪽수 오류")?;
        verified_bytes(root, sample, field(entry, "source_sha256")?)?;
        verified_bytes(root, field(entry, "pdf")?, field(entry, "pdf_sha256")?)?;
        if proof.as_object().map(|p| p.len()) != Some(2) {
            return Err("Native/fresh WASM 전쪽 증거 누락".into());
        }
        for backend in ["native", "wasm"] {
            let record = &proof[backend];
            let tsv = field(record, "tsv")?;
            if !Path::new(tsv).starts_with("mydocs/pr/assets") {
                return Err("정식 전쪽 TSV 경로 오류".into());
            }
            let data = verified_bytes(root, tsv, field(record, "sha256")?)?;
            validate_full_page_tsv(&data, pages)?;
        }
        if result.insert(sample.to_string(), pages).is_some() {
            return Err(format!("논리 쪽 원본 중복: {sample}"));
        }
    }
    Ok(result)
}

#[test]
fn full_page_evidence_rejects_missing_duplicate_low_or_nonfinite_values() {
    let tsv = |rows: &str| format!("page\tsilhouette_raw_match_percent\n{rows}");
    assert!(validate_full_page_tsv(tsv("1\t100.00\n2\t98.00\n").as_bytes(), 2).is_ok());
    for rows in [
        "1\t100.00\n",
        "1\t100.00\n1\t100.00\n",
        "1\t97.99\n2\t100\n",
        "1\tNaN\n2\t100\n",
        "1\t100.01\n2\t100\n",
    ] {
        assert!(
            validate_full_page_tsv(tsv(rows).as_bytes(), 2).is_err(),
            "{rows}"
        );
    }
}
