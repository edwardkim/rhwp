//! [#4549] 연합 번들 — 7년 축(.lineage-bundle) 코어.
//!
//! ## 무엇을 푸나
//!
//! 캡슐이 조직 경계를 넘는 순간: 상대 키를 어떻게 믿고, 상대 로그를 어떻게
//! 검증하고, 조상 전체를 어떻게 받나. 답은 전송 프로토콜이 아니라 **오프라인
//! 파일 하나로 전건 검증이 닫히는 교환 형식**이다 — 이 저장소의 결
//! (서버 없는 검증, 신뢰 기관 없는 해시) 그대로.
//!
//! ## 컨테이너 규약 (zip — 신규 포맷 발명 없음, HWPX 선례)
//!
//! ```text
//! manifest.json            {kind:"lineageBundle", head, files:[{path, sha256}], domain}
//! capsules/<이름>          머리 + 조상 폐쇄집합 전체
//! signatures/<이름>.sig.json  각 캡슐의 분리 서명 (있으면)
//! anchor/proofs.json       캡슐별 머클 경로 + 체크포인트 (있으면)
//! domain.json              발신 도메인 자기 소개 (있으면 — 참고용)
//! ```
//!
//! ## 신뢰의 방향 (F2 방어)
//!
//! 검증은 번들에 **동봉된** keyring 을 절대 쓰지 않는다 — 위장 도메인이 자기
//! 키로 전부 통과시키는 구멍이다. 서명 판정 기준은 수신자가 **자기 경로로
//! 받은** trust-domain 파일의 keyring 뿐이며, 그 파일의 수령 경로가 신뢰
//! 뿌리다(도구 밖 거버넌스 — 4년 축과 같은 문장).

use std::io::{Read as _, Write as _};

/// 매니페스트의 `kind`.
pub const BUNDLE_KIND: &str = "lineageBundle";
/// 신뢰 도메인 파일의 `kind`.
pub const DOMAIN_KIND: &str = "trustDomain";

/// 계보 폐쇄집합 — 머리에서 부모 링크를 따라 모은 (파일명, 절대경로) 목록.
///
/// 부모 경로는 캡슐 파일 기준 상대 해석(계보 규약 그대로). 파일명 충돌
/// (다른 폴더의 같은 이름)은 번들 평면 구조에서 모호하므로 오류다.
pub fn closure(head: &str) -> Result<Vec<(String, std::path::PathBuf)>, String> {
    let mut out: Vec<(String, std::path::PathBuf)> = Vec::new();
    let mut current = std::path::PathBuf::from(head);
    for _ in 0..1000 {
        let name = current
            .file_name()
            .ok_or_else(|| format!("캡슐 경로가 이상합니다: {}", current.display()))?
            .to_string_lossy()
            .into_owned();
        if out.iter().any(|(n, _)| *n == name) {
            return Err(format!(
                "폐쇄집합 파일명 충돌: {name} — 번들은 평면 구조라 같은 이름의 조상을 담을 수 없다"
            ));
        }
        let bytes = std::fs::read(&current)
            .map_err(|e| format!("캡슐을 읽을 수 없습니다 - {}: {e}", current.display()))?;
        let capsule: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|e| format!("{name}: 캡슐 파싱 실패 - {e}"))?;
        out.push((name.clone(), current.clone()));
        let parent = &capsule["parent"];
        if parent.is_null() {
            return Ok(out);
        }
        let pp = parent["capsule"]
            .as_str()
            .ok_or_else(|| format!("{name}: parent.capsule 없음"))?;
        let pp_path = std::path::PathBuf::from(pp);
        current = if pp_path.is_absolute() {
            pp_path
        } else {
            current
                .parent()
                .unwrap_or(std::path::Path::new("."))
                .join(pp_path)
        };
    }
    Err("체인 길이 1000 초과 — 순환 의심".to_string())
}

/// zip 에 파일 하나를 쓴다.
pub fn zip_put<W: std::io::Write + std::io::Seek>(
    zip: &mut zip::ZipWriter<W>,
    path: &str,
    bytes: &[u8],
) -> Result<(), String> {
    zip.start_file::<_, ()>(path, zip::write::FileOptions::default())
        .map_err(|e| format!("zip 항목 시작 실패 - {path}: {e}"))?;
    zip.write_all(bytes)
        .map_err(|e| format!("zip 쓰기 실패 - {path}: {e}"))
}

/// 번들의 전 항목을 (경로 → 바이트) 로 읽는다.
pub fn read_all(bundle_path: &str) -> Result<std::collections::BTreeMap<String, Vec<u8>>, String> {
    let file = std::fs::File::open(bundle_path)
        .map_err(|e| format!("번들을 열 수 없습니다 - {bundle_path}: {e}"))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("zip 파싱 실패 - {bundle_path}: {e}"))?;
    let mut map = std::collections::BTreeMap::new();
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("zip 항목 읽기 실패: {e}"))?;
        if entry.is_dir() {
            continue;
        }
        let name = entry.name().replace('\\', "/");
        let mut bytes = Vec::with_capacity(entry.size() as usize);
        entry
            .read_to_end(&mut bytes)
            .map_err(|e| format!("zip 본문 읽기 실패 - {name}: {e}"))?;
        map.insert(name, bytes);
    }
    Ok(map)
}

/// trust-domain 파일 파싱 — (도메인 이름, keyring 값, 체크포인트 배열).
pub fn parse_trust_domain(
    text: &str,
) -> Result<(String, serde_json::Value, Vec<serde_json::Value>), String> {
    let v: serde_json::Value =
        serde_json::from_str(text).map_err(|e| format!("trust-domain 파싱 실패 - {e}"))?;
    if v["kind"] != DOMAIN_KIND {
        return Err(format!("kind 가 {DOMAIN_KIND} 가 아닙니다"));
    }
    let domain = v["domain"].as_str().unwrap_or("(이름 없음)").to_string();
    let keyring = v["keyring"].clone();
    if keyring.is_null() {
        return Err(
            "trust-domain 에 keyring 이 필요합니다 (동봉 keyring 은 불신 — F2)".to_string(),
        );
    }
    let checkpoints = v["checkpoints"].as_array().cloned().unwrap_or_default();
    Ok((domain, keyring, checkpoints))
}
