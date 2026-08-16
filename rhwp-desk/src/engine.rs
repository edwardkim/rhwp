//! rhwp.exe 탐색 — 설정 > 환경변수 > PATH > 저장소 관례 위치 순.
//!
//! 후보 생성(`candidate_paths`)과 존재 판정(`first_existing`)을 분리해
//! 파일시스템 없이도 우선순위 로직을 단위 테스트할 수 있게 한다.

use std::path::{Path, PathBuf};

/// 탐색 후보 1건 — 경로와 그 출처 라벨.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub path: PathBuf,
    pub source: String,
}

fn exe_name() -> &'static str {
    if cfg!(windows) {
        "rhwp.exe"
    } else {
        "rhwp"
    }
}

/// 우선순위대로 후보 경로를 생성한다. 존재 검사는 하지 않는다.
///
/// 1. 사용자가 설정 화면에서 지정한 경로
/// 2. `RHWP_DESK_ENGINE` 환경변수
/// 3. `PATH` 의 각 디렉토리
/// 4. 실행 파일 위치·작업 디렉토리의 조상들 아래 `target/{debug,release,release-test}`
///    (저장소 안에서 개발 빌드를 돌릴 때의 관례 위치)
pub fn candidate_paths(
    configured: Option<&str>,
    env_override: Option<&str>,
    path_var: Option<&str>,
    exe_dir: Option<&Path>,
    cwd: Option<&Path>,
) -> Vec<Candidate> {
    let mut out: Vec<Candidate> = Vec::new();
    let mut push = |path: PathBuf, source: &str| {
        if !out.iter().any(|c| c.path == path) {
            out.push(Candidate {
                path,
                source: source.to_string(),
            });
        }
    };

    if let Some(p) = configured.filter(|s| !s.trim().is_empty()) {
        push(PathBuf::from(p.trim()), "설정");
    }
    if let Some(p) = env_override.filter(|s| !s.trim().is_empty()) {
        push(PathBuf::from(p.trim()), "환경변수 RHWP_DESK_ENGINE");
    }
    if let Some(pv) = path_var {
        let sep = if cfg!(windows) { ';' } else { ':' };
        for dir in pv.split(sep).filter(|d| !d.is_empty()) {
            push(Path::new(dir).join(exe_name()), "PATH");
        }
    }
    for base in [exe_dir, cwd].into_iter().flatten() {
        let mut anc = Some(base);
        for _ in 0..6 {
            let Some(dir) = anc else { break };
            for profile in ["debug", "release", "release-test"] {
                push(
                    dir.join("target").join(profile).join(exe_name()),
                    "저장소 탐색",
                );
            }
            push(dir.join(exe_name()), "저장소 탐색");
            anc = dir.parent();
        }
    }
    out
}

/// 후보 중 판정 함수를 통과하는 첫 항목을 고른다.
pub fn first_existing<F>(candidates: &[Candidate], exists: F) -> Option<Candidate>
where
    F: Fn(&Path) -> bool,
{
    candidates.iter().find(|c| exists(&c.path)).cloned()
}

/// 실제 파일시스템으로 탐색한다.
pub fn discover(configured: Option<&str>) -> Option<Candidate> {
    let env_override = std::env::var("RHWP_DESK_ENGINE").ok();
    let path_var = std::env::var("PATH").ok();
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf));
    let cwd = std::env::current_dir().ok();
    let cands = candidate_paths(
        configured,
        env_override.as_deref(),
        path_var.as_deref(),
        exe_dir.as_deref(),
        cwd.as_deref(),
    );
    first_existing(&cands, |p| p.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 설정_경로가_최우선이다() {
        let cands = candidate_paths(
            Some("C:/tools/rhwp.exe"),
            Some("C:/env/rhwp.exe"),
            Some("C:/bin"),
            None,
            None,
        );
        assert_eq!(cands[0].path, PathBuf::from("C:/tools/rhwp.exe"));
        assert_eq!(cands[0].source, "설정");
        assert_eq!(cands[1].source, "환경변수 RHWP_DESK_ENGINE");
        assert_eq!(cands[2].source, "PATH");
    }

    #[test]
    fn 빈_설정은_건너뛰고_중복은_한_번만() {
        let cands = candidate_paths(Some("  "), Some("C:/env/rhwp.exe"), None, None, None);
        assert_eq!(cands.len(), 1);
        assert_eq!(cands[0].source, "환경변수 RHWP_DESK_ENGINE");

        // 설정과 환경변수가 같은 경로면 첫 출처만 남는다.
        let dup = candidate_paths(
            Some("C:/x/rhwp.exe"),
            Some("C:/x/rhwp.exe"),
            None,
            None,
            None,
        );
        assert_eq!(dup.len(), 1);
        assert_eq!(dup[0].source, "설정");
    }

    #[test]
    fn 저장소_관례_위치를_조상_방향으로_훑는다() {
        let cands = candidate_paths(
            None,
            None,
            None,
            Some(Path::new("C:/repo/rhwp-desk/target/debug")),
            None,
        );
        // exe 가 rhwp-desk/target/debug 에 있으면 조상 중 저장소 루트의
        // target/{debug,release,release-test} 가 후보에 포함되어야 한다.
        let expect = PathBuf::from("C:/repo/target/debug").join(super::exe_name());
        assert!(cands.iter().any(|c| c.path == expect), "{cands:?}");
    }

    #[test]
    fn first_existing_은_우선순위를_지킨다() {
        let cands = vec![
            Candidate {
                path: PathBuf::from("a"),
                source: "설정".into(),
            },
            Candidate {
                path: PathBuf::from("b"),
                source: "PATH".into(),
            },
        ];
        let hit = first_existing(&cands, |p| p == Path::new("b")).unwrap();
        assert_eq!(hit.path, PathBuf::from("b"));
        assert!(first_existing(&cands, |_| false).is_none());
    }
}
