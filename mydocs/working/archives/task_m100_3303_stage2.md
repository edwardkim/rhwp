# Task #3303 Stage 2 — 구현계획서

## Stage 1 이후 추가 실증 (구현 직전 조사)

1. **렌더러는 `line_type == None`이면 그리지 않는다** (`para_border_is_visible`,
   `layout.rs:94`; `border_rendering.rs` 전반). 이슈 가설 (b)(renderer가 id≠0이면
   무조건 그림)는 기각 — 파서에서 선 종류를 None으로 주면 충분하다.
2. **`BorderLineType`의 `#[default]`가 `Solid`** (`model/style.rs:559`). 따라서
   `BorderFill::default()`로 만든 bf의 4방향 선이 이미 Solid이고, bde926e4b의 명시적
   Solid 설정은 사실상 no-op — **bf를 생성한 것 자체가 테두리를 그리게 했다**.
3. **음영 경로도 같은 뿌리로 오염**: hwp3-sample4 4쪽 SVG 실측에서 음영 rect 2개의
   좌·우 모서리에 검은 세로선(#000000, 0.4px) 4개가 그려진다. V3에는 선 종류 필드가
   없으므로 이 선들도 합성 아티팩트다. (#3303의 "skia 우측 세로선"·42쪽 좌우
   세로선과 동일 기전 — 상하선은 연결/스킵 로직으로 생략되어 좌우만 보임)

## 수정 — `src/parser/hwp3/mod.rs` `hwp3_para_shape_border_fill()`

```rust
// [#3303] V3 문단 모양(표 13)에는 테두리 선 종류/굵기/색 필드가 없다(181 offset
// on/off 뿐). 한컴 2022는 border=1을 "선 없음(NONE, 0.1mm, 검정)"으로 매핑한다
// (한컴 자체 변환 SO-SUEOP.hwpx: paraPr/border→borderFill 4방향 type="NONE").
// BorderLineType의 Rust default는 Solid이므로 명시적으로 None을 채워야 하며,
// 음영(shade_ratio) 경로도 같은 이유로 선은 항상 None이다.
fn hwp3_para_shape_border_fill(
    hwp3_ps: &crate::parser::hwp3::records::Hwp3ParaShape,
) -> Option<crate::model::style::BorderFill> {
    if hwp3_ps.shade_ratio == 0 && !hwp3_ps.has_border() {
        return None;
    }
    let mut bf = crate::model::style::BorderFill::default();
    for b in bf.borders.iter_mut() {
        b.line_type = crate::model::style::BorderLineType::None;
    }
    if hwp3_ps.shade_ratio > 0 {
        ...기존 음영 로직 무변경...
    }
    Some(bf)
}
```

- has_border Solid 합성 블록 삭제. bf 생성 조건(`shade>0 || has_border`)·참조 배선·
  `border_connection`(attr1 bit28) 배선은 무변경.
- 함수 위 주석을 #2986 → #3303 근거로 교체.

## 테스트 — `src/parser/hwp3/mod.rs` tests

`test_hwp3_para_shape_border_fill_wires_has_border_flag` 교체:

1. `border=1, shade=0` → `Some(bf)`, **4방향 전부 `line_type == None`**, fill 없음
   (한컴 SO-SUEOP.hwpx 변환 구조와 동형).
2. `border=0, shade=20` → `Some(bf)`, 4방향 None, fill Solid(회색) — 음영 경로에서도
   Solid 선 아티팩트가 없음을 고정.
3. `border=0, shade=0` → `None` (기존 동작 유지).

## errata — `mydocs/tech/hwp_spec_errata.md` 2건 추가

1. 표 13 offset 181 "문단 테두리 0=없음, 1=있음": "있음"은 테두리 **구조** 존재이지
   선을 그린다는 뜻이 아님. V3에는 선 종류 필드가 없고, 한컴 2022는 선 없음
   (NONE/0.1mm/검정)으로 매핑 — Solid 합성 금지. 근거: SO-SUEOP.hwp 원시 바이트 +
   한컴 UI + 한컴 변환 SO-SUEOP.hwpx XML 3중 실측 (#3303).
2. 표 13 offset 182 "선 연결 1=테두리 선 연결 안 함": **극성 반대**. 실측(한컴 변환
   HWPX)은 `border_connection=1 → connect="1"`(연결 ON). rhwp 배선(#2976)은 실측과
   일치하므로 무변경.

## 검증 절차

1. 단위: 위 테스트 3케이스 + `cargo test --lib parser::hwp3`.
2. IR: `rhwp dump samples/SO-SUEOP.hwp -p 1005` → border_fill_id=5 유지 확인(구조 보존).
3. **SVG 백엔드 (정량)**: 수정 전/후 `export-svg`로
   - SO-SUEOP p41: 지문 테두리 stroke 라인 소멸 (전 4+? → 0)
   - hwp3-sample4 p3: 음영 rect 2개 유지 + 좌우 세로선 4개 → 0
4. **4-backend 대조**: svg(3) / canvas2d(studio headless 42쪽 스크린샷 — 상자 소멸) /
   render-tree JSON(`export-render-tree` 테두리 LineNode 소멸) / skia(`cargo build
   --release --features native-skia` 후 `export-png` p41 — 우측 세로선 소멸).
5. 회귀: HWP3 코퍼스에서 이 경로 소비 파일은 SO-SUEOP·sample4 2개뿐(Stage 1 스윕)
   — 그 외 파일 무영향은 구조적으로 보장. `cargo test --tests --profile release-test`
   + `fmt --check` (push 전).
6. **시각 판정 게이트(작업지시자)**: SO-SUEOP 42쪽 + sample4 음영 페이지 after 렌더.

## PR

- 커밋 1개(코드+테스트+errata+working/report 문서), `Closes #3303`, 본문 한국어.
- PR 생성은 별도 승인 후. 0.8.1 PATCH 대상.

## 후속 제안 (범위 밖, 이슈 등록 후보)

- ir-diff ParaShape 비교에 `border_fill_id` + 참조 BorderFill 4방향 선 종류 요약 추가
  — 이번 발산(HWPX NONE vs HWP3 Solid)은 현행 ir-diff 사각지대(비교 항목에 없음).
