//! [#2864] 폰트 조달 경로를 한 곳에서 결정한다.
//!
//! 종전에는 `renderer/pdf.rs`·`renderer/svg.rs`·`renderer/skia/image_conv.rs`·
//! `renderer/skia/renderer.rs` 네 곳이 각자 탐색 경로를 하드코딩했고, 그중
//! `ttfs/hwp`·`ttfs/windows`(로컬 전용 저작권 폰트, `.gitignore`)와
//! `/mnt/c/Windows/Fonts`(WSL2 개발 PC 전용)는 **특정 환경에만 존재**했다.
//!
//! 서버·컨테이너에서 CLI 로 대량 변환할 때 이 경로들은 아무 값도 주지 못하면서
//! 경로 검사만 발생시키고, WSL2 에서는 `/mnt/c` 가 9p 파일시스템이라 간헐적
//! 극단 지연으로 `export-pdf` 가 통째로 멎었다(#2268, 누적 4회 관측).
//!
//! 폰트는 호출자가 `--font-path`(`options.font_paths`) 또는 `RHWP_FONT_PATH`
//! 로 지정하거나 시스템에 설치해 쓴다. 이 모듈은 그 순서만 정의한다.
//!
//! # 조달 순서
//!
//! 1. **호출자 지정** — `--font-path` / `options.font_paths` (최우선)
//! 2. **`RHWP_FONT_PATH`** — 환경변수. 백엔드 대량 변환에서 호출마다 인자를
//!    붙이는 대신 한 번만 설정한다. 복수 경로는 OS 관례 구분자로 나눈다
//!    (유닉스 `:`, Windows `;`).
//! 3. **시스템 기본** — OS별 표준 사용자·시스템 폰트 디렉터리
//! 4. **`ttfs/opensource`** — 최후 폴백. `.gitignore` 대상이 아닌 **저장소 자산**
//!    (NotoSansKR 2종 + OFL)이라 모든 체크아웃에서 확보된다. 폰트 미설치 환경
//!    (CI headless·컨테이너)에서 한국어가 드롭되는 것을 막는다(#2293).

use std::path::{Path, PathBuf};

/// 폰트 탐색 경로 환경변수. 복수 경로는 OS 관례 구분자로 나눈다.
pub const FONT_PATH_ENV: &str = "RHWP_FONT_PATH";

/// 저장소 번들 오픈소스 폰트 — 최후 폴백(#2293 한국어 드롭 방지).
pub const BUNDLED_OPENSOURCE_DIR: &str = "ttfs/opensource";
/// Native 글꼴의 **힙 복사본** 총량 상한. 바이트로 직접 공급한 snapshot
/// (`TypefaceCatalog::try_from_data`)과 CoreText 가 거부한 TTC face 를 따로
/// 꾸린 복사본에 적용한다. 경로로 받은 파일은 복사하지 않고 파일 매핑을
/// face 끼리 공유하므로 이 총량에 들어가지 않는다(#8597).
pub(crate) const MAX_NATIVE_CUSTOM_FONT_BYTES: u64 = 64 * 1024 * 1024;
/// 경로 적재에서 받아들이는 글꼴 파일 수 상한.
pub(crate) const MAX_NATIVE_CUSTOM_FONT_FILES: usize = 256;
/// 경로 적재에서 받아들이는 글꼴 파일 하나의 크기 상한.
pub(crate) const MAX_NATIVE_CUSTOM_FONT_FILE_BYTES: u64 = 32 * 1024 * 1024;

/// 경로 적재가 받아들이지 않은 글꼴 파일과 그 이유.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SkippedNativeFontFile {
    pub path: PathBuf,
    pub reason: String,
}

/// 경로 적재가 실제로 읽을 글꼴 파일 목록. 공급 경로 순서를 보존하며
/// (`sources[i]` ↔ `files[i]`), 같은 파일은 처음 나온 곳에서만 읽는다.
#[derive(Debug, Default)]
pub(crate) struct NativeCustomFontScan {
    pub files: Vec<Vec<PathBuf>>,
    pub skipped: Vec<SkippedNativeFontFile>,
}

/// Native custom 글꼴 경로 적재의 파일 선택 규칙 — skia 카탈로그와 웹폰트
/// 공급 판정이 함께 쓴다. 디렉터리는 한 단계만 읽고 파일명 순으로 정렬하며,
/// `ttf`·`otf`·`ttc` 만 받는다. 파일 수·파일 크기 상한을 넘는 파일은 이유와
/// 함께 `skipped` 에 남긴다(#8597: 예전에는 말없이 건너뛰었다).
pub(crate) fn scan_native_custom_font_files(sources: &[PathBuf]) -> NativeCustomFontScan {
    let mut scan = NativeCustomFontScan::default();
    let mut seen = std::collections::HashSet::new();
    let mut accepted = 0usize;
    for source in sources {
        let mut paths = if source.is_file() {
            vec![source.clone()]
        } else {
            std::fs::read_dir(source)
                .into_iter()
                .flatten()
                .flatten()
                .map(|entry| entry.path())
                .collect::<Vec<_>>()
        };
        paths.sort();
        let mut files = Vec::new();
        for path in paths {
            let supported = path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| {
                    matches!(
                        extension.to_ascii_lowercase().as_str(),
                        "ttf" | "otf" | "ttc"
                    )
                });
            if !supported || !seen.insert(path.clone()) {
                continue;
            }
            let Ok(metadata) = path.metadata() else {
                continue;
            };
            if !metadata.is_file() || metadata.len() == 0 {
                continue;
            }
            let reason = if metadata.len() > MAX_NATIVE_CUSTOM_FONT_FILE_BYTES {
                Some(format!(
                    "파일 크기 {} 바이트가 상한 {} 바이트를 넘음",
                    metadata.len(),
                    MAX_NATIVE_CUSTOM_FONT_FILE_BYTES
                ))
            } else if accepted >= MAX_NATIVE_CUSTOM_FONT_FILES {
                Some(format!(
                    "글꼴 파일 수 상한 {MAX_NATIVE_CUSTOM_FONT_FILES} 개를 넘음"
                ))
            } else {
                None
            };
            match reason {
                Some(reason) => scan.skipped.push(SkippedNativeFontFile { path, reason }),
                None => {
                    accepted += 1;
                    files.push(path);
                }
            }
        }
        scan.files.push(files);
    }
    scan
}

/// `RHWP_FONT_PATH` 를 읽어 경로 목록으로 나눈다.
///
/// 미설정이면 빈 벡터를 돌려준다. 존재하지 않는 경로도 그대로 담아 호출부가
/// 기존 `--font-path` 와 동일하게 경고·건너뛰기를 하도록 맡긴다.
pub fn env_font_paths() -> Vec<PathBuf> {
    let Ok(raw) = std::env::var(FONT_PATH_ENV) else {
        return Vec::new();
    };
    // `split` 은 빈 조각도 내므로(예: 연속 구분자, 양끝 구분자) 걸러낸다.
    raw.split(PATH_SEPARATOR)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .collect()
}

#[cfg(target_os = "windows")]
const PATH_SEPARATOR: char = ';';

#[cfg(not(target_os = "windows"))]
const PATH_SEPARATOR: char = ':';

/// OS별 시스템 폰트 디렉터리.
///
/// `usvg::fontdb::Database::load_system_fonts()` 를 쓰는 경로에서는 필요 없고,
/// 파일을 직접 훑는 경로(`svg.rs::find_font_file`, `skia` typeface 로딩)에서 쓴다.
pub fn system_font_dirs() -> Vec<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        // macOS의 사용자 설치 위치도 표준 경로다. 누락하면 Full 임베딩이
        // 설치된 굵은 글꼴을 찾지 못해 Chrome의 local() 비트맵 폰트로 돌아간다.
        let mut dirs = Vec::new();
        if let Some(home) = std::env::var_os("HOME").filter(|home| !home.is_empty()) {
            dirs.push(PathBuf::from(home).join("Library/Fonts"));
        }
        dirs.extend([
            PathBuf::from("/Library/Fonts"),
            PathBuf::from("/System/Library/Fonts"),
            PathBuf::from("/System/Library/Fonts/Supplemental"),
        ]);
        dirs
    }
    #[cfg(target_os = "linux")]
    {
        // Fontconfig이 사용하는 사용자 설치 위치도 복구 검색에 포함한다.
        // Skia가 영문 대표 이름만 열거한 글꼴의 현지화 family를 찾을 때 필요하다.
        let mut dirs = Vec::new();
        let home = std::env::var_os("HOME")
            .filter(|home| !home.is_empty())
            .map(PathBuf::from);
        let data_home = std::env::var_os("XDG_DATA_HOME")
            .filter(|path| !path.is_empty())
            .map(PathBuf::from)
            .or_else(|| home.as_ref().map(|home| home.join(".local/share")));
        if let Some(data_home) = data_home {
            dirs.push(data_home.join("fonts"));
        }
        if let Some(home) = home {
            dirs.push(home.join(".fonts"));
        }
        dirs.extend([
            PathBuf::from("/usr/share/fonts"),
            PathBuf::from("/usr/local/share/fonts"),
        ]);
        dirs
    }
    #[cfg(target_os = "windows")]
    {
        vec![PathBuf::from("C:\\Windows\\Fonts")]
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Vec::new()
    }
}

/// 파일을 직접 훑는 경로용 탐색 디렉터리 전체를 조달 순서대로 돌려준다.
///
/// `extra` 는 호출자 지정 경로(`--font-path` 등)이며 최우선이다.
pub fn search_dirs(extra: &[PathBuf]) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = extra.to_vec();
    dirs.extend(env_font_paths());
    dirs.extend(system_font_dirs());
    // 최후 폴백 — 저장소 자산이라 항상 존재한다(#2293).
    dirs.push(PathBuf::from(BUNDLED_OPENSOURCE_DIR));
    dirs
}

/// custom typeface 로더(skia `with_font_paths`)용 디렉터리 — **시스템·번들 제외**.
///
/// custom 로더는 이제 collection 의 모든 bounded face와 family별 스타일
/// 후보를 보존하지만, 해석 체인에서 시스템 매칭보다 앞선다는 계약은 같다.
/// 그래서 여기에는 사용자가 의도한 폰트(호출자 지정 → 환경 변수)만 담는다:
/// - 시스템 폰트는 `FontMgr::match_family_style`이 담당한다. 시스템 디렉터리를
///   custom inventory로 다시 읽어 명시적인 사용자 폰트처럼 우선하지 않는다.
///   예전 single-face 로더의 regular 선점 문제(#3300)도 이 경계의 근거다.
/// - 번들 opensource 를 넣으면 깊은 폴백(Noto Sans KR)이 시스템 1순위를
///   제치고 선점된다(#3300, r23 발산 −6.9pp) — 번들은 별도 최후-폴백 로더
///   (`bundled_font_dirs`)로 체인 말미에만 선다.
pub fn custom_font_dirs(extra: &[PathBuf]) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = extra.to_vec();
    dirs.extend(env_font_paths());
    dirs
}

/// 번들 최후-폴백 로더용 디렉터리 — 폰트 미설치 환경(CI headless·컨테이너)
/// 에서 한국어 드롭을 막는 저장소 자산(#2293). 해석 체인에서는 custom·시스템
/// 뒤(최후)에만 선다(#3300).
pub fn bundled_font_dirs() -> Vec<PathBuf> {
    vec![PathBuf::from(BUNDLED_OPENSOURCE_DIR)]
}

/// fontdb가 ID 16 우선으로 생략한 ID 1·21의 실제 가족 이름을 보존한다.
/// 기존 face·스타일·우선순위는 그대로 두고 같은 source/index를 가리키는
/// 이름 뷰만 추가한다. Full Name(ID 4)·PostScript(ID 6)는 가족으로 승격하지 않는다.
pub(crate) fn retain_fontdb_family_aliases(database: &mut usvg::fontdb::Database) {
    let aliases: Vec<_> = database
        .faces()
        .filter_map(|info| {
            let families = database.with_face_data(info.id, |bytes, index| {
                let face = ttf_parser::Face::parse(bytes, index).ok()?;
                let mut families = Vec::new();
                for record in face.names() {
                    if !matches!(
                        record.name_id,
                        ttf_parser::name_id::FAMILY | ttf_parser::name_id::WWS_FAMILY
                    ) {
                        continue;
                    }
                    let Some(name) = record.to_string() else {
                        continue;
                    };
                    let name = name.trim();
                    if !name.is_empty()
                        && !info.families.iter().any(|(known, _)| known == name)
                        && !families.iter().any(|(known, _)| known == name)
                    {
                        families.push((name.to_owned(), record.language()));
                    }
                }
                (!families.is_empty()).then_some(families)
            })??;
            let mut alias = info.clone();
            alias.families = families;
            Some(alias)
        })
        .collect();
    for alias in aliases {
        database.push_face_info(alias);
    }
}

/// `fontdb` 에 조달 순서대로 폰트를 적재한다.
///
/// 시스템 폰트는 `load_system_fonts()` 가 담당하므로 여기서는 호출자 지정 →
/// 환경변수 → 번들 순으로 디렉터리만 추가한다. 존재하지 않는 경로는 건너뛰되,
/// `extra`(사용자가 명시한 경로)만 경고한다 — 환경변수·번들은 없을 수 있다.
pub fn load_into_fontdb(fontdb: &mut usvg::fontdb::Database, extra: &[PathBuf]) {
    for dir in extra {
        if dir.exists() {
            fontdb.load_fonts_dir(dir);
        } else {
            eprintln!(
                "WARN: font path '{}' not found. 해당 경로의 폰트는 로드하지 않습니다.",
                dir.display()
            );
        }
    }
    for dir in env_font_paths() {
        if dir.exists() {
            fontdb.load_fonts_dir(&dir);
        } else {
            eprintln!(
                "WARN: {FONT_PATH_ENV} entry '{}' not found. 해당 경로의 폰트는 로드하지 않습니다.",
                dir.display()
            );
        }
    }
    let bundled = Path::new(BUNDLED_OPENSOURCE_DIR);
    if bundled.exists() {
        fontdb.load_fonts_dir(bundled);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// 환경변수는 프로세스 전역이라 테스트가 병렬로 돌면 서로를 덮는다.
    /// `RHWP_FONT_PATH` 를 만지는 테스트는 이 락으로 직렬화한다 — 락 없이 셋이
    /// 병렬로 돌면 set 과 read 사이에 다른 테스트의 remove_var 가 끼어들어
    /// 간헐 실패한다(실측 플레이크).
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn env_font_paths_parses_and_filters() {
        let _g = ENV_LOCK.lock().unwrap();
        // 미설정
        std::env::remove_var(FONT_PATH_ENV);
        assert!(env_font_paths().is_empty(), "미설정이면 빈 목록이어야 한다");

        // 단일 경로
        std::env::set_var(FONT_PATH_ENV, "/tmp/fonts-a");
        assert_eq!(env_font_paths(), vec![PathBuf::from("/tmp/fonts-a")]);

        // 복수 경로 + 빈 조각(연속·양끝 구분자) 제거
        let joined = format!(
            "{sep}/tmp/fonts-a{sep}{sep}/tmp/fonts-b{sep}",
            sep = PATH_SEPARATOR
        );
        std::env::set_var(FONT_PATH_ENV, joined);
        assert_eq!(
            env_font_paths(),
            vec![PathBuf::from("/tmp/fonts-a"), PathBuf::from("/tmp/fonts-b")],
            "빈 조각은 걸러내고 순서는 보존해야 한다"
        );

        std::env::remove_var(FONT_PATH_ENV);
    }

    #[test]
    fn search_dirs_orders_caller_first_and_bundled_last() {
        let _g = ENV_LOCK.lock().unwrap();
        std::env::remove_var(FONT_PATH_ENV);
        let extra = vec![PathBuf::from("/tmp/caller")];
        let dirs = search_dirs(&extra);

        assert_eq!(
            dirs.first(),
            Some(&PathBuf::from("/tmp/caller")),
            "호출자 지정 경로가 최우선이어야 한다"
        );
        assert_eq!(
            dirs.last(),
            Some(&PathBuf::from(BUNDLED_OPENSOURCE_DIR)),
            "번들 오픈소스 폰트가 최후 폴백이어야 한다"
        );
        #[cfg(target_os = "linux")]
        {
            let home = std::env::var_os("HOME")
                .filter(|value| !value.is_empty())
                .map(PathBuf::from);
            let data_home = std::env::var_os("XDG_DATA_HOME")
                .filter(|value| !value.is_empty())
                .map(PathBuf::from)
                .or_else(|| home.as_ref().map(|path| path.join(".local/share")));
            if let Some(data_home) = data_home {
                assert!(dirs.contains(&data_home.join("fonts")));
            }
            if let Some(home) = home {
                assert!(dirs.contains(&home.join(".fonts")));
            }
        }
        #[cfg(target_os = "macos")]
        if let Some(home) = std::env::var_os("HOME").filter(|home| !home.is_empty()) {
            let user_dir = PathBuf::from(home).join("Library/Fonts");
            let user_index = dirs.iter().position(|dir| dir == &user_dir).unwrap();
            let system_index = dirs
                .iter()
                .position(|dir| dir == Path::new("/Library/Fonts"))
                .unwrap();
            assert!(
                user_index < system_index,
                "표준 사용자 설치 글꼴을 시스템 폴백보다 먼저 찾는다"
            );
        }
    }

    /// [#2864] 환경 종속 경로가 조달 목록에 다시 들어오지 않도록 고정한다.
    /// 이 가드가 없으면 편의를 위해 재추가되어 #2268 간헐 행이 재발할 수 있다.
    #[test]
    fn search_dirs_excludes_environment_specific_paths() {
        let _g = ENV_LOCK.lock().unwrap();
        std::env::remove_var(FONT_PATH_ENV);
        let dirs = search_dirs(&[]);
        let joined: Vec<String> = dirs.iter().map(|p| p.display().to_string()).collect();

        for banned in [
            "/mnt/c/Windows/Fonts", // WSL2 개발 PC 전용 — #2268 간헐 행 원인
            "ttfs/hwp",             // .gitignore 로컬 전용(저작권 폰트)
            "ttfs/windows",
        ] {
            assert!(
                !joined.iter().any(|d| d == banned),
                "환경 종속 경로가 조달 목록에 있다: {banned}\n\
                 폰트가 필요하면 --font-path 또는 {FONT_PATH_ENV} 로 지정한다."
            );
        }
    }

    /// [#3300] custom 로더 목록은 시스템·번들을 포함하면 안 된다.
    /// - 시스템: 사용자 지정 inventory로 승격하지 않고 FontMgr의 스타일 매칭에 위임
    /// - 번들: 깊은 폴백(Noto)이 시스템 1순위를 제치고 본문을 렌더(r23 −6.9pp)
    #[test]
    fn custom_font_dirs_excludes_system_and_bundled() {
        let _g = ENV_LOCK.lock().unwrap();
        std::env::set_var(FONT_PATH_ENV, "/tmp/env-fonts");
        let extra = vec![PathBuf::from("/tmp/caller")];
        let dirs = custom_font_dirs(&extra);
        assert_eq!(
            dirs,
            vec![
                PathBuf::from("/tmp/caller"),
                PathBuf::from("/tmp/env-fonts")
            ],
            "custom 로더는 호출자 지정 → 환경변수만 담는다"
        );
        for sys in system_font_dirs() {
            assert!(!dirs.contains(&sys), "시스템 디렉터리 포함 금지: {sys:?}");
        }
        assert!(
            !dirs.contains(&PathBuf::from(BUNDLED_OPENSOURCE_DIR)),
            "번들은 custom 이 아니라 최후-폴백 로더로 간다"
        );
        std::env::remove_var(FONT_PATH_ENV);
    }

    /// [#3300] 번들 최후-폴백 로더는 저장소 자산만 담는다(#2293 한국어 드롭 방지).
    #[test]
    fn bundled_font_dirs_is_repo_asset_only() {
        assert_eq!(
            bundled_font_dirs(),
            vec![PathBuf::from(BUNDLED_OPENSOURCE_DIR)]
        );
    }
}
