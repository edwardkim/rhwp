//! Issue #1882 (C1c, #1431 Track C): 차트 스타일 4갭 보정 회귀 가드.
//!
//! 한컴 2022 정답지(`pdf/chart/`) 실측 기준 4갭:
//!   ① 자동 제목 "차트 제목" (c:title 요소 존재 + autoTitleDeleted=0 + 텍스트 없음)
//!   ② 팔레트 파랑(#6183D7)→주황(#FE813B)→회색(#B0B0B0)→노랑(#FCD801) — 실측값
//!   ③ 범례 우측 세로 스택 (`c:legendPos val="r"` — 코퍼스 전 샘플)
//!   ④ Y축 headroom + step 기반 눈금 (막대 max 5.0 → 축 0~6 라벨 0,2,4,6)
//!
//! 대표 샘플 × (hwp, hwpx)의 page 0 SVG substring으로 검증.

use std::fs;
use std::path::Path;

fn render_page0_svg(rel: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", rel, e));
    let mut doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {:?}", rel, e));
    doc.render_page_svg(0)
        .unwrap_or_else(|e| panic!("render {}: {:?}", rel, e))
}

fn for_both_exts(stem: &str, f: impl Fn(&str, &str)) {
    for ext in ["hwpx", "hwp"] {
        let rel = format!("samples/chart/{stem}.{ext}");
        f(&rel, &render_page0_svg(&rel));
    }
}

#[test]
fn chart_auto_title_rendered() {
    // ① 다계열: 제목 텍스트가 없으면 한컴처럼 "차트 제목"을 렌더 (regular weight).
    for stem in ["세로막대형/묶은세로막대형", "라인/꺽은선형"] {
        for_both_exts(stem, |rel, svg| {
            assert!(svg.contains("차트 제목"), "{rel}: 자동 제목 미렌더");
            assert!(
                !svg.contains("font-weight=\"600\""),
                "{rel}: 제목이 bold(600) — 한컴은 regular",
            );
        });
    }
    // ① v2: 단일 시리즈는 자동 제목 = 시리즈 이름 (한컴 실측 — 원형 5종 "판매",
    //    단일 시리즈 가로막대 "계열 1"; 차트 종류 불문 시리즈 수 기준).
    for (stem, name) in [
        ("원형/2차원원형", "판매"),
        ("원형/원형대원형", "판매"),
        (
            "특이케이스/가로막대형_하나만있을떄_단일시리즈제목",
            "계열 1",
        ),
    ] {
        for_both_exts(stem, |rel, svg| {
            assert!(
                svg.contains(&format!(">{name}<")),
                "{rel}: 단일 시리즈 이름({name}) 제목 미렌더"
            );
            assert!(
                !svg.contains("차트 제목"),
                "{rel}: 단일 시리즈인데 placeholder 렌더"
            );
        });
    }
}

#[test]
fn chart_hancom_palette_applied() {
    // ② 3시리즈 막대 → 파랑/주황/회색 (구 녹색-우선 #70ad47 미사용).
    for_both_exts("세로막대형/묶은세로막대형", |rel, svg| {
        for color in ["#6183d7", "#fe813b", "#b0b0b0"] {
            assert!(svg.contains(color), "{rel}: 실측 팔레트 {color} 미사용");
        }
        assert!(
            !svg.contains("#70ad47"),
            "{rel}: 구 녹색-우선 팔레트가 여전히 사용됨",
        );
    });
}

#[test]
fn chart_axis_headroom_and_sparse_ticks() {
    // ④ 막대 데이터 max 5.0(step 경계) → 축 0~6, step 재계산 → 라벨 0,2,4,6.
    for_both_exts("세로막대형/묶은세로막대형", |rel, svg| {
        for want in [">0<", ">2<", ">4<", ">6<"] {
            assert!(
                svg.contains(want),
                "{rel}: 축 라벨 {want} 없음 (0~6 step 2)"
            );
        }
        for absent in [">3<", ">5<"] {
            assert!(
                !svg.contains(absent),
                "{rel}: 축 라벨 {absent} 존재 (성긴 라벨이어야)",
            );
        }
    });
    // ④ scatter Y max 4.0 → 0~5 (headroom), X max 2.6 → 0~3 step 0.5 유지.
    for_both_exts("분산형/표식만있는분산형", |rel, svg| {
        for want in [">5<", ">0.5<", ">2.5<", ">3<"] {
            assert!(svg.contains(want), "{rel}: 축 라벨 {want} 없음");
        }
    });
    // ④ 방향별 눈금 밀도 (한컴 실측): 같은 합(12.3)인데
    //    누적'세로' → 0~15 step 5 / 누적'가로' → 0~14 step 2.
    for_both_exts("세로막대형/누적세로막대형", |rel, svg| {
        for want in [">10<", ">15<"] {
            assert!(
                svg.contains(want),
                "{rel}: 세로 누적 라벨 {want} 없음 (0~15 step 5)"
            );
        }
        assert!(
            !svg.contains(">14<"),
            "{rel}: 세로 누적에 14 라벨 (step 2 회귀)"
        );
    });
    for_both_exts("가로막대형/누적가로막대형", |rel, svg| {
        assert!(
            svg.contains(">14<"),
            "{rel}: 가로 누적 라벨 14 없음 (0~14 step 2)"
        );
        assert!(!svg.contains(">15<"), "{rel}: 가로 누적에 15 라벨");
    });
    // ④ 3D 축 정책 (한컴 실측): 묶은 3D는 세로·가로 모두 0~5(무헤드룸),
    //    누적 3D 세로는 0~20(2D 15 + 1 step), 누적 3D 가로는 2D와 동일 0~14.
    for stem in [
        "세로막대형/3차원묶은세로막대형",
        "가로막대형/3차원묶은가로막대형",
    ] {
        for_both_exts(stem, |rel, svg| {
            assert!(svg.contains(">5<"), "{rel}: 3D 묶은 라벨 5 없음 (0~5)");
            assert!(!svg.contains(">6<"), "{rel}: 3D 묶은에 headroom(6) 적용됨");
        });
    }
    for_both_exts(
        "세로막대형/3차원누적세로막대형",
        |rel, svg| {
            assert!(
                svg.contains(">20<"),
                "{rel}: 3D 누적세로 라벨 20 없음 (0~20)"
            );
        },
    );
    for_both_exts(
        "가로막대형/3차원누적가로막대형",
        |rel, svg| {
            assert!(
                svg.contains(">14<"),
                "{rel}: 3D 누적가로 라벨 14 없음 (0~14)"
            );
            assert!(
                !svg.contains(">20<"),
                "{rel}: 3D 누적가로에 세로용 과헤드룸 적용됨"
            );
        },
    );
}

#[test]
fn chart_legend_on_right() {
    // ③ legendPos=r → 범례 그룹이 존재하고, 범례 텍스트 x가 모든 데이터 막대의
    //    우측 끝보다 오른쪽 (하단 가로 배치였다면 x가 플롯 좌측부터 시작).
    for_both_exts("세로막대형/묶은세로막대형", |rel, svg| {
        // 페이지 SVG에는 페이지 배경 등 차트 밖 rect도 있으므로 차트 그룹 내부로 한정.
        let chart_start = svg
            .find("class=\"hwp-ooxml-chart\"")
            .unwrap_or_else(|| panic!("{rel}: 차트 그룹 없음"));
        let chart = &svg[chart_start..];
        let legend_start = chart
            .find("class=\"hwp-chart-legend\"")
            .unwrap_or_else(|| panic!("{rel}: hwp-chart-legend 그룹 없음"));
        let (plot, legend) = chart.split_at(legend_start);
        let legend = &legend[..legend.find("</g>").unwrap_or(legend.len())];

        let legend_text_x = first_attr_f64(legend, "<text ", "x=\"")
            .unwrap_or_else(|| panic!("{rel}: 범례 텍스트 없음"));
        let max_bar_right = plot
            .split("<rect ")
            .skip(1)
            .filter(|c| {
                let tag = &c[..c.find('>').unwrap_or(c.len())];
                // 데이터 막대: fill=#... + stroke 없음 + 범례 스와치(10×10) 제외
                !tag.contains("stroke")
                    && tag.contains("fill=\"#")
                    && !tag.contains("width=\"10\" height=\"10\"")
            })
            .filter_map(|c| {
                let tag = &c[..c.find('>').unwrap_or(c.len())];
                Some(attr_f64(tag, "x=\"")? + attr_f64(tag, "width=\"")?)
            })
            .fold(f64::NEG_INFINITY, f64::max);
        assert!(
            legend_text_x > max_bar_right,
            "{rel}: 범례 텍스트 x={legend_text_x}가 막대 우측 끝 {max_bar_right}보다 왼쪽 (우측 배치 아님)",
        );
    });
}

fn attr_f64(tag: &str, pat: &str) -> Option<f64> {
    let s = tag.find(pat)? + pat.len();
    let e = s + tag[s..].find('"')?;
    tag[s..e].parse().ok()
}

fn first_attr_f64(fragment: &str, elem: &str, pat: &str) -> Option<f64> {
    let chunk = fragment.split(elem).nth(1)?;
    attr_f64(&chunk[..chunk.find('>').unwrap_or(chunk.len())], pat)
}
