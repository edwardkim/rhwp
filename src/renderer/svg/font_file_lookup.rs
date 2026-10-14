//! SVG 파일 공급에서 원 face와 대체 face의 우선순위를 보존한다.

use super::{
    font_local_aliases, font_local_bold_aliases, known_bold_font_filenames, known_font_filenames,
    korean_gothic_substitute,
};

/// 이름 테이블과 한컴 Print로 확인된 원 face 설치 파일.
/// 휴먼명조의 일반 face는 HMKMM이며 HCR/Haansoft 바탕은 다른 프로그램이다.
/// 실제 bold 파일은 확인되지 않았으므로 굵은 파일 선택 정책은 유지한다.
fn original_font_files(font_name: &str, bold: bool) -> Vec<FontFileName> {
    if bold || !matches!(font_name, "휴먼명조" | "HumanMyeongJo") {
        return Vec::new();
    }
    let mut files = vec!["HMKMM.TTF".to_string()];
    for name in ["휴먼명조", "HumanMyeongJo"] {
        for ext in ["ttf", "otf", "ttc", "TTF"] {
            files.push(format!("{name}.{ext}"));
        }
    }
    files
        .into_iter()
        .filter_map(FontFileName::from_document_candidate)
        .collect()
}

/// 파일시스템 탐색에 넘기는 하나의 폰트 파일명.
///
/// 문서 메타데이터에서 유래한 후보는 SVG 폰트 해석 단계에서만
/// 이 형식으로 바뀐다. 파일시스템 루프는 이 타입의 후보만
/// 설정된 검색 루트에 결합한다.
#[derive(Clone, Debug)]
pub(super) struct FontFileName(String);

impl FontFileName {
    pub(super) fn from_document_candidate(candidate: String) -> Option<Self> {
        use std::path::Component;

        if candidate.contains(['/', '\\', '\0']) {
            return None;
        }
        let mut components = std::path::Path::new(&candidate).components();
        (matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none())
            .then_some(Self(candidate))
    }

    fn as_path(&self) -> &std::path::Path {
        std::path::Path::new(&self.0)
    }
}

/// SVG 폰트 해석이 선택한 파일명과 탐색 루트.
pub(super) struct FontFileLookupPlan {
    original_candidates: Vec<FontFileName>,
    candidates: Vec<FontFileName>,
    pub(super) search_dirs: Vec<std::path::PathBuf>,
}

/// 문서 폰트명을 SVG 폰트 해석용 파일 후보로 계획한다.
///
/// 별칭·대체 폰트·후보 우선순위는 렌더러 정책이므로 여기서
/// 선택한다. 이후의 파일시스템 조회는 타입으로 검증된 후보를 읽는 기계적 단계다.
pub(super) fn plan_svg_font_file_lookup(
    font_name: &str,
    extra_paths: &[std::path::PathBuf],
    bold: bool,
) -> FontFileLookupPlan {
    let original_candidates = original_font_files(font_name, bold);
    let candidates = {
        // 폰트명 → 파일명 후보 생성
        let known_files = if bold {
            known_bold_font_filenames(font_name)
        } else {
            known_font_filenames(font_name)
        };
        let mut files: Vec<String> = known_files.iter().map(|s| s.to_string()).collect();
        let aliases = if bold {
            font_local_bold_aliases(font_name).to_vec()
        } else {
            font_local_aliases(font_name)
        };
        let mut names = vec![font_name.to_string()];
        for a in &aliases {
            names.push(a.to_string());
        }
        for name in &names {
            let clean = name.replace(' ', "");
            files.push(format!("{}.ttf", name));
            files.push(format!("{}.otf", name));
            files.push(format!("{}.ttc", name));
            files.push(format!("{}.TTF", name));
            if clean != *name {
                files.push(format!("{}.ttf", clean));
                files.push(format!("{}.otf", clean));
                files.push(format!("{}.ttc", clean));
            }
        }
        // Task #1224: 고딕 계열은 오픈소스 대체(Noto Sans KR ExtraLight)를 최후 후보로 추가.
        // 실제 저작권 폰트가 앞선 탐색 경로에 있으면 그쪽이 우선하므로, 대체는
        // 탐색 경로 말단(ttfs/opensource)에서만 매칭된다.
        if !bold {
            if let Some(sub) = korean_gothic_substitute(font_name) {
                files.push(sub.to_string());
            }
        }
        files
            .into_iter()
            .filter_map(FontFileName::from_document_candidate)
            .collect()
    };

    // [#2864] 탐색 경로(우선순위 순)는 renderer::font_paths 가 단일 정의한다.
    // 호출자 지정 → RHWP_FONT_PATH → OS 시스템 경로 → ttfs/opensource(최후 폴백).
    // 종전의 ttfs/hwp·ttfs/windows(로컬 전용)와 /mnt/c/Windows/Fonts(WSL2 전용)는
    // 제거했다. Task #1224 의 고딕 대체(Noto Sans KR ExtraLight)는 최후 탐색인
    // ttfs/opensource 에서 그대로 매칭된다.
    FontFileLookupPlan {
        original_candidates,
        candidates,
        search_dirs: crate::renderer::font_paths::search_dirs(extra_paths),
    }
}

/// 계획된 폰트 파일을 설정된 루트에서 읽는다.
pub(super) fn find_font_file(plan: &FontFileLookupPlan) -> Option<std::path::PathBuf> {
    // 원 face의 직접 파일을 모든 루트에서 확인한 뒤 대체 face를 찾는다.
    // 같은 face가 여러 루트에 있으면 호출자가 지정한 루트 순서를 보존한다.
    for candidates in [&plan.original_candidates, &plan.candidates] {
        for dir in &plan.search_dirs {
            if !dir.exists() {
                continue;
            }
            for candidate in candidates {
                let path = dir.join(candidate.as_path());
                if path.is_file() {
                    return Some(path);
                }
            }
        }
    }
    None
}
