//! [#4097] `mini_cfb` 루트 CLSID 보존 — 프로덕션 API 와 독립 오라클의 대조.
//!
//! 자족형이다(`tests/support/issue_4055_chart_probe.rs` 를 쓰지 않는다). support 를 `#[path]`
//! 로 두 번 포함하면 이 파일이 안 쓰는 helper 가 `dead_code` 경고를 내 `clippy -D warnings`
//! 가 깨진다.

use std::io::Read as _;
use std::path::{Path, PathBuf};

use rhwp::parser::ole_container::ole_root_clsid;
use rhwp::serializer::mini_cfb;

fn manifest(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

/// 독립 오라클 — 우리 오프셋 산술이 아니라 `cfb` 크레이트가 읽은 값이다.
/// 읽기와 쓰기가 같은 오해를 공유해도 통과하는 사태를 막는다.
fn clsid_via_cfb_crate(bytes: &[u8]) -> [u8; 16] {
    cfb::CompoundFile::open(std::io::Cursor::new(bytes.to_vec()))
        .expect("CFB 열기")
        .root_entry()
        .clsid()
        .to_bytes_le()
}

/// `samples/chart/` 코퍼스의 `.hwpx`/`.hwp` 에서 중첩 OLE CFB 를 뽑아 `(라벨, 바이트)` 로.
fn corpus_nested_cfbs() -> Vec<(String, Vec<u8>)> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for e in std::fs::read_dir(dir).expect("samples/chart 읽기") {
            let p = e.expect("dir entry").path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().and_then(|s| s.to_str()) == Some("hwpx") {
                out.push(p);
            }
        }
    }
    let mut hwpx_paths = Vec::new();
    walk(&manifest("samples/chart"), &mut hwpx_paths);
    hwpx_paths.sort();
    assert!(!hwpx_paths.is_empty(), "samples/chart 코퍼스가 비어 있다");

    let mut out = Vec::new();
    for hwpx in hwpx_paths {
        for path in [hwpx.clone(), hwpx.with_extension("hwp")] {
            let label = path.display().to_string();
            let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{label} 읽기: {e}"));
            let doc =
                rhwp::parse_document(&bytes).unwrap_or_else(|e| panic!("{label} 파싱: {e:?}"));
            let nested = doc
                .bin_data_content
                .iter()
                .map(|c| c.data.load())
                .find(|b| b.starts_with(&[0xD0, 0xCF, 0x11, 0xE0]))
                .unwrap_or_else(|| panic!("{label}: 중첩 CFB 가 없다"));
            out.push((label, nested));
        }
    }
    out
}

fn streams_of(cfb_bytes: &[u8]) -> Vec<(String, Vec<u8>)> {
    let mut comp =
        cfb::CompoundFile::open(std::io::Cursor::new(cfb_bytes.to_vec())).expect("CFB 열기");
    let paths: Vec<PathBuf> = comp
        .walk()
        .filter(|e| e.is_stream())
        .map(|e| e.path().to_path_buf())
        .collect();
    let mut out = Vec::new();
    for p in paths {
        let mut buf = Vec::new();
        comp.open_stream(&p)
            .expect("스트림 열기")
            .read_to_end(&mut buf)
            .expect("스트림 읽기");
        out.push((p.to_string_lossy().replace('\\', "/"), buf));
    }
    out
}

/// **2026-08-05 한컴 판정을 통과한 산출물과 프로덕션 산출물이 바이트 동일함을 고정한다.**
///
/// #4055 stage4 는 `build_cfb` 출력의 루트 엔트리 +80 에 CLSID 를 사후 스탬프한 파일들을
/// 한컴에서 판정해 전부 정상 개봉·렌더을 확인했고, CLSID 외의 차이(color flag 0→1, 타임스탬프,
/// 섹터 배치)는 무해 판정을 받았다(`mydocs/working/task_m100_4055_stage4.md:64-68`).
///
/// 이 테스트는 그 레시피를 **동결 사본**으로 인라인해 두고, 프로덕션
/// `build_cfb_with_root_clsid` 가 코퍼스 전건에서 그것과 바이트 동일한 것을 만든다고 단언한다.
/// 두 가지를 동시에 지킨다.
///
/// 1. AC④ 를 "새로 측정할 미지"에서 "이미 측정된 것의 재확인"으로 강등시킨다.
/// 2. `build_cfb` 출력이 CLSID 16바이트 외에 달라지면 즉시 red 다 — 기존 호출자 9곳
///    무영향의 기계적 증명이다.
#[test]
fn production_api_reproduces_the_hancom_validated_bytes() {
    let mut checked = 0usize;
    for (label, nested) in corpus_nested_cfbs() {
        let streams = streams_of(&nested);
        let refs: Vec<(&str, &[u8])> = streams
            .iter()
            .map(|(n, d)| (n.as_str(), d.as_slice()))
            .collect();
        let clsid = clsid_via_cfb_crate(&nested);

        let production =
            mini_cfb::build_cfb_with_root_clsid(&refs, clsid).expect("프로덕션 재포장");

        // #4055 가 한컴에 넘긴 레시피 — 여기서만 재현하는 동결 참조본이다.
        // (`build_cfb` 출력 + 루트 디렉터리 엔트리 +80 사후 스탬프)
        let legacy_recipe = {
            let mut v = mini_cfb::build_cfb(&refs).expect("재포장");
            let shift = u16::from_le_bytes([v[0x1E], v[0x1F]]) as usize;
            let dir_start = u32::from_le_bytes(v[0x30..0x34].try_into().unwrap()) as usize;
            let at = (dir_start + 1) * (1usize << shift) + 80;
            v[at..at + 16].copy_from_slice(&clsid);
            v
        };

        assert_eq!(
            production, legacy_recipe,
            "{label}: 한컴 판정을 통과한 산출물과 바이트가 달라졌다"
        );
        checked += 1;
    }
    assert_eq!(checked, 56, "코퍼스 28종 × 2포맷을 전건 대조해야 한다");
}

/// 프로덕션 리더가 `cfb` 크레이트와 같은 값을 읽는지 — 오프셋 산술 교차검증.
#[test]
fn production_reader_agrees_with_the_cfb_crate() {
    let mut checked = 0usize;
    for (label, nested) in corpus_nested_cfbs() {
        let expected = clsid_via_cfb_crate(&nested);
        assert_ne!(expected, [0u8; 16], "{label}: 코퍼스 차트는 CLSID 를 단다");
        assert_eq!(
            ole_root_clsid(&nested),
            Some(expected),
            "{label}: ole_root_clsid 가 cfb 크레이트와 달라졌다"
        );
        // 위임 래퍼와 유일 구현이 같은 값을 준다.
        assert_eq!(
            ole_root_clsid(&nested),
            rhwp::parser::cfb_reader::root_clsid(&nested),
            "{label}: 위임 래퍼가 유일 구현과 어긋났다"
        );
        checked += 1;
    }
    assert_eq!(checked, 56);
}

/// 리더가 망가진 입력에 패닉하지 않는다.
///
/// 패닉은 WASM 모듈 전체를 죽여 편집 중인 다른 문서까지 잃게 한다 —
/// `LenientCfbReader::open` 이 섹터 지수를 검증하는 것과 같은 근거다.
#[test]
fn reader_returns_none_for_malformed_input() {
    assert_eq!(ole_root_clsid(&[]), None, "빈 입력");
    assert_eq!(ole_root_clsid(&[0u8; 511]), None, "512바이트 미만");
    assert_eq!(ole_root_clsid(&[0u8; 4096]), None, "매직 불일치");

    let sound = mini_cfb::build_cfb(&[("/X", b"y" as &[u8])]).expect("CFB");
    assert_eq!(ole_root_clsid(&sound), Some([0u8; 16]), "정상 입력 기준선");

    // 섹터 지수 65535 — 검증 없이 `1usize << shift` 하면 wasm32 에서 shift overflow.
    let mut bad_shift = sound.clone();
    bad_shift[0x1E] = 0xFF;
    bad_shift[0x1F] = 0xFF;
    assert_eq!(ole_root_clsid(&bad_shift), None, "섹터 지수 범위 밖");

    // 섹터 지수 0 — sector_size 1 로 아래 산술이 무너진다.
    let mut zero_shift = sound.clone();
    zero_shift[0x1E] = 0x00;
    zero_shift[0x1F] = 0x00;
    assert_eq!(ole_root_clsid(&zero_shift), None, "섹터 지수 0");

    // first dir sector = ENDOFCHAIN — checked_mul 이 걸러야 한다.
    let mut eoc = sound.clone();
    eoc[0x30..0x34].copy_from_slice(&0xFFFF_FFFEu32.to_le_bytes());
    assert_eq!(ole_root_clsid(&eoc), None, "dir_start 가 ENDOFCHAIN");

    // 헤더는 멀쩡한데 디렉터리 섹터가 잘려 나간 파일.
    let truncated = &sound[..600];
    assert_eq!(ole_root_clsid(truncated), None, "루트 엔트리 +96 미만 절단");
}
