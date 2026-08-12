//! [#4558] 감사 표준 — 10년 축(감사 보고·리콜·적합성) 코어.
//!
//! ## 사다리의 마지막 계단 (설계서 §1)
//!
//! 1~9년 축이 만든 검증 재료(영수증·감사·계보·서명·앵커·게이트·번들·공개·
//! 원장)를 **감사인이 읽는 언어**로 묶는다. 보고서의 요건은 하나다:
//! **"감사 보고서를 감사할 수 있다"** — 전 수치는 기존 축의 검증을 그대로
//! 재실행해 얻은 기계 합산이고, 보고서 자체가 4년 사이드카로 서명된다.
//!
//! ## 리콜 = 후손 폐쇄집합 (설계서 §2.2)
//!
//! "부모 P 가 오염으로 판명됐다 — 무엇을 회수하나"의 답은 계보 그래프에서
//! 오염 노드의 후손 전건이다. 캡슐은 부모 링크(자식→부모)만 들고 있으므로,
//! 폴더의 각 캡슐에서 조상 사슬을 걸어 오염 노드가 나오면 영향으로 분류한다.
//! 9년 원장이 있으면 영향 캡슐의 정산 청구 좌표까지 짚는다(리콜의 회계 연결).
//!
//! ## 적합성 = 기존 검증의 재사용 (설계서 §2.3)
//!
//! L1~L5 등급 검사에 새 판정기를 발명하지 않는다 — 영수증 존재(1년)·계보
//! 유효(3년)·서명 판정(4년)·앵커 포함(5년)·게이트 통과(6년)·원장 무결(9년)
//! 전부 기존 코드 경로다. 8년(선택적 공개 운영)은 기계 판정 밖임을 보고
//! 필드로 정직하게 명시한다.

use sha2::{Digest, Sha256};

/// 감사 보고서의 `kind`.
pub const REPORT_KIND: &str = "agentLaborAuditReport";

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(64);
    for b in digest {
        use std::fmt::Write as _;
        let _ = write!(hex, "{b:02x}");
    }
    hex
}

/// 폴더의 캡슐 하나 — 경로·파싱 값·파일 바이트 해시.
pub struct CapsuleNode {
    pub name: String,
    pub path: std::path::PathBuf,
    pub value: serde_json::Value,
    pub file_sha256: String,
}

/// `*.capsule.json` 을 비재귀로 모은다(audit 와 같은 폴더 규약). 이름 정렬.
pub fn collect(dir: &str) -> Result<Vec<CapsuleNode>, String> {
    let mut out = Vec::new();
    let entries =
        std::fs::read_dir(dir).map_err(|e| format!("폴더를 읽을 수 없습니다 - {dir}: {e}"))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("폴더 항목 오류: {e}"))?;
        let path = entry.path();
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        if !name.ends_with(".capsule.json") {
            continue;
        }
        let bytes =
            std::fs::read(&path).map_err(|e| format!("캡슐을 읽을 수 없습니다 - {name}: {e}"))?;
        let value: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|e| format!("{name}: 파싱 실패 - {e}"))?;
        out.push(CapsuleNode {
            file_sha256: sha256_hex(&bytes),
            name,
            path,
            value,
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

/// 부모 링크의 경로를 캡슐 파일 기준으로 해석한다(계보·번들과 같은 규약).
pub fn resolve_parent(capsule_path: &std::path::Path, parent: &str) -> std::path::PathBuf {
    let pp = std::path::PathBuf::from(parent);
    if pp.is_absolute() {
        pp
    } else {
        capsule_path
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .join(pp)
    }
}

/// 한 캡슐의 조상 사슬을 걸어 (조상 파일 해시 목록, 파손 지점) 을 얻는다.
///
/// 링크 유효성 = 부모 파일 바이트 해시 == 기록된 sha256, 그리고 계보 불변식
/// (부모 산출 해시 == 자식 입력 해시). 파손이 나오면 그 지점에서 멈춘다 —
/// 파손 이후의 "조상"은 신뢰할 수 없는 주장이다.
pub struct Ancestry {
    /// 자기 자신을 제외한 조상들의 (이름, 파일 해시) — 가까운 순.
    pub ancestors: Vec<(String, String)>,
    /// 파손 지점 설명. `None` 이면 뿌리까지 유효.
    pub broken_at: Option<String>,
}

pub fn walk_ancestry(node_path: &std::path::Path, node_value: &serde_json::Value) -> Ancestry {
    let mut ancestors = Vec::new();
    let mut current_path = node_path.to_path_buf();
    let mut current = node_value.clone();
    for _ in 0..1000 {
        let parent = current["parent"].clone();
        if parent.is_null() {
            return Ancestry {
                ancestors,
                broken_at: None,
            };
        }
        let (Some(pp), Some(psha)) = (parent["capsule"].as_str(), parent["sha256"].as_str()) else {
            return Ancestry {
                ancestors,
                broken_at: Some(format!(
                    "{}: parent 형식 오류",
                    current_path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                )),
            };
        };
        let parent_path = resolve_parent(&current_path, pp);
        let parent_name = parent_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let Ok(bytes) = std::fs::read(&parent_path) else {
            return Ancestry {
                ancestors,
                broken_at: Some(format!("{parent_name}: 부모 파일 없음")),
            };
        };
        let actual = sha256_hex(&bytes);
        if actual != psha {
            return Ancestry {
                ancestors,
                broken_at: Some(format!("{parent_name}: 부모 해시 불일치")),
            };
        }
        let Ok(parent_value) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
            return Ancestry {
                ancestors,
                broken_at: Some(format!("{parent_name}: 부모 파싱 실패")),
            };
        };
        // 계보 불변식 — 부모의 산출이 자식의 입력이어야 한다.
        let parent_out = parent_value["receipt"]["outputSha256"]
            .as_str()
            .unwrap_or("");
        let child_in = current["receipt"]["inputSha256"].as_str().unwrap_or("");
        if !parent_out.is_empty() && !child_in.is_empty() && parent_out != child_in {
            return Ancestry {
                ancestors,
                broken_at: Some(format!("{parent_name}: 계보 불변식 위반(산출≠입력)")),
            };
        }
        ancestors.push((parent_name, actual));
        current_path = parent_path;
        current = parent_value;
    }
    Ancestry {
        ancestors,
        broken_at: Some("체인 길이 1000 초과 — 순환 의심".to_string()),
    }
}

/// 폴더 그래프의 머리(어느 캡슐의 부모도 아닌 노드) 인덱스 목록과 뿌리 수.
///
/// 부모 참조는 파일 해시로 대조한다 — 경로 표기가 달라도 같은 파일이면 같은
/// 노드다(해시가 정체성, 이 저장소의 결).
pub fn heads_and_roots(nodes: &[CapsuleNode]) -> (Vec<usize>, usize) {
    let mut referenced: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for node in nodes {
        let parent = &node.value["parent"];
        if let Some(psha) = parent["sha256"].as_str() {
            referenced.insert(psha.to_string());
        }
    }
    let heads: Vec<usize> = nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| !referenced.contains(&n.file_sha256))
        .map(|(i, _)| i)
        .collect();
    let roots = nodes.iter().filter(|n| n.value["parent"].is_null()).count();
    (heads, roots)
}
