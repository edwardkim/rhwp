# Task #1151 v3 구현계획서

수행계획서: [task_m100_1151_v3.md](task_m100_1151_v3.md) · v3 root cause 분석: [topandbottom_table_inline_picture_layout.md](../../tech/investigations/issue-1151/topandbottom_table_inline_picture_layout.md)

## 0. 설계 결정

### 0-1. 진입점 — `paragraph_layout.rs` 의 tac picture y 결정

Fix #2 (`mydocs/tech/investigations/issue-1151/topandbottom_table_inline_picture_layout.md` §5) 채택. 본 fix 는 `paragraph_layout.rs` 의 inline tac picture 자리 결정 직전에 sibling TopAndBottom 표 reserved 영역을 picture y 에 가산.

진입점 후보 (Stage 3 의 첫 작업으로 확정):
- `paragraph_layout.rs:994-1003` (`tac_offsets_px` 수집 직후) — picture y 자체에 가산하기엔 너무 이름.
- `paragraph_layout.rs:1540-1553` (run 내 tac 위치 계산 루프) — picture 의 글리프 자리 결정. y 가산 적용 위치 후보.
- `paragraph_layout.rs:3019-3145` (빈 paragraph 의 tac 처리) — paragraph.text 가 비어있을 때 tac picture 처리. 본 task 의 핵심 케이스 (paragraph.text="" 의 sibling 표 + picture).

**결정**: Stage 3-1 에서 진입점 정확 확정 후 진행. 가장 유력한 후보는 빈 paragraph 의 tac 처리 path (paragraph.text="" 인 한컴 산출물 case 에 직접 해당).

### 0-2. 신규 helper — `calc_sibling_topandbottom_table_reserved_hu`

```rust
/// [Task #1151 v3] paragraph 의 sibling controls 중 wrap=TopAndBottom +
/// treat_as_char=false 인 표가 차지하는 vertical 영역 (HU) 합산.
///
/// 한컴 layout 정합: 같은 paragraph 의 inline tac picture 가 표 아래
/// 영역에 그려지도록 picture 의 y 위치 보정값을 계산한다.
fn calc_sibling_topandbottom_table_reserved_hu(controls: &[Control]) -> i32 {
    use crate::model::shape::TextWrap;
    controls.iter().map(|c| match c {
        Control::Table(t) if matches!(t.common.text_wrap, TextWrap::TopAndBottom)
            && !t.common.treat_as_char =>
        {
            // 표 자체 height + outer_margin (top + bottom)
            t.common.height as i32
                + t.outer_margin.top as i32
                + t.outer_margin.bottom as i32
        }
        _ => 0,
    }).sum()
}
```

비고:
- Table 의 outer_margin 필드 존재 확인 필요 (Stage 3 첫 작업). 만약 다른 이름이면 정정.
- 표가 여러 개 있는 경우 모두 합산 (드물지만 가능).

### 0-3. 보정 정책

paragraph 의 ls[0].vpos 가 이미 13064 (한컴 산출물) 인 경우와 표가 그 위치에서 시작하는 것을 고려:

- **선택 A — picture y 에만 가산**: tac picture 의 px y 위치만 helper 결과 만큼 추가. 표 layout 은 기존 path 그대로 (paginate_table_control 의 y_offset). 가장 안전.
- **선택 B — paragraph 의 line offset 보정**: paragraph 의 line 시작 y 자체를 보정 → 표 아래로 정렬. 위험 ↑ (다른 line 결정 path 영향).

**채택**: 선택 A. Stage 3-2 에서 구현.

### 0-4. 단위 테스트 패턴

`#[cfg(test)] mod issue_1151_v3_topandbottom_inline_picture_tests` — paragraph_layout 의 layout 결과 (예: ComposedLine 또는 직접 layout 호출 후 picture 의 PageItem 좌표) 단언. 또는 export-svg 결과를 grep 으로 검증.

가장 단순: 한컴 산출물 (`scenario-a-after.hwp`) 을 통합 테스트로 파싱 + render → SVG 좌표 단언:

```rust
let svg = render_to_svg("samples/tac-verify/scenario-a-after.hwp");
let pic_y = extract_image_y(&svg);
let table_y = extract_table_y(&svg);
let table_h = extract_table_height(&svg);
assert!(pic_y > table_y + table_h, "picture y should be below table");
```

render_to_svg helper 의 시그니처는 Stage 3-3 에서 결정.

### 0-5. 시각 검증

- WASM 재빌드 + dev server 재시연 (사용자 직접 확인)
- `rhwp export-svg` 의 좌표 자동 단언

### 0-6. 회귀 방지

기존 v2 통합 테스트 4 (`integration_tac_toggle_matches_hancom_scenario_{a,b,c,d}`) 유지. v3 fix 가 model 을 변경하지 않으므로 v2 단언은 그대로 PASS.

---

## Stage 3 — Layout fix 구현

### Stage 3-1. 진입점 정확 확정

1. `src/renderer/layout/paragraph_layout.rs:3019-3145` 의 빈 paragraph 의 tac 처리 path 읽기.
2. `scenario-a-after.hwp` 를 디버그 로그로 trace → picture y 결정 시점에서 paragraph.controls / line_segs / tac_offsets_px 의 값 확인.
3. fix 적용 위치 결정 (가장 적합한 단일 진입점).

### Stage 3-2. helper 신설 + 진입점 갱신

1. `calc_sibling_topandbottom_table_reserved_hu` helper 추가 (paragraph_layout.rs 또는 layout 공용 모듈).
2. Stage 3-1 에서 결정된 진입점에서 helper 호출 → tac picture 의 y 결정에 가산.
3. Table 의 outer_margin 필드명 확정 (model/table.rs 또는 model/shape.rs 참조).

### Stage 3-3. 단위 + 통합 테스트

#### 단위 테스트 (`#[cfg(test)] mod issue_1151_v3_tests`)

| 테스트 | 시나리오 | 단언 |
|--------|---------|------|
| `sibling_topandbottom_table_reserved_calc_single` | controls 에 TopAndBottom 표 1개 (h=10000, outer_margin top=200, bottom=200) | helper 결과 = 10400 |
| `sibling_topandbottom_table_reserved_calc_none` | controls 에 표 없음 | helper 결과 = 0 |
| `sibling_topandbottom_table_reserved_excludes_tac_table` | TopAndBottom but tac=true 표 | helper 결과 = 0 |
| `sibling_topandbottom_table_reserved_excludes_square_wrap` | wrap=Square 표 | helper 결과 = 0 |

#### 통합 시각 테스트

`tests/` 폴더 또는 `paragraph_layout.rs` 내 단위 테스트로:

```rust
#[test]
fn v3_scenario_a_picture_below_table_in_svg() {
    let bytes = std::fs::read("samples/tac-verify/scenario-a-after.hwp").unwrap();
    let doc = parse_hwp(&bytes).unwrap();
    let svg = render_section_to_svg(&doc, 0); // helper TBD
    // <image> y > <rect>(table) y + height
    let (pic_y, table_y, table_h) = parse_svg_coords(&svg);
    assert!(pic_y >= table_y + table_h, "picture y={} should be below table bottom y+h={}", pic_y, table_y + table_h);
}
```

또는 시각 검증을 export-svg CLI 호출 + 단언 script 로 검증.

### Stage 3-4. 검증

```bash
cargo test --lib issue_1151_v3
cargo test --lib              # 전수 회귀
cargo clippy --lib -- -D warnings
cargo fmt --all -- --check
```

GREEN 확인 후 Stage 3 commit:
```
Task #1151 v3 Stage 3: paragraph_layout sibling TopAndBottom 표 + inline picture 위치 보정

- calc_sibling_topandbottom_table_reserved_hu helper 신설
- paragraph_layout.rs 의 inline tac picture y 결정에 sibling 표 영역 가산
- 단위 테스트 4 + 통합 시각 테스트
```

`mydocs/working/task_m100_1151_v3_stage3.md` 작성.

---

## Stage 4 — WASM 재빌드 + 브라우저 시각 재시연

### Stage 4-1. WASM 재빌드

```bash
docker compose --env-file .env.docker run --rm wasm
```

### Stage 4-2. rhwp-studio dev server 재시연

```bash
cd rhwp-studio && npx vite --host 0.0.0.0 --port 7700 &
```

### Stage 4-3. 4 시나리오 시각 재시연

1. 신규 문서 → 1×1 표 → 셀 안 작은 picture 삽입 → 토글 → **표가 위, picture 가 표 아래에 inline** 시각 확인.
2. 동일 + 큰 picture → 토글 → 동일 시각 정합.
3. 3×3 표 중앙 셀 picture 토글 → 동일 시각 정합.
4. 본문 floating picture (셀 없음) 토글 → 기존 동작 그대로 (picture 가 paragraph 의 inline 위치).
5. 회귀: 본문 inline picture 신규 삽입 + v1 의 셀 안 picture 삽입 (floating) 정상.

### Stage 4-4. 한컴 편집기 1:1 비교

각 시나리오의 한컴 편집기 출력 (`samples/tac-verify/scenario-*-after.hwp` 를 한컴에서 열어 시각 캡쳐) 와 rhwp-studio 출력의 동일성 확인.

### Stage 4-5. Stage 4 commit

```
Task #1151 v3 Stage 4: WASM 재빌드 + 브라우저 시각 재시연 통과

- 4 시나리오 시각 한컴 편집기 출력과 정합
- v1 본문 inline / 셀 안 floating 삽입 회귀 0
```

`mydocs/working/task_m100_1151_v3_stage4.md` 작성.

---

## Stage 5 — 통합 PR + 최종 보고서

### Stage 5-1. 자동 회귀

```bash
cargo test --lib
cargo test --tests
cargo clippy --lib -- -D warnings
cargo fmt --all -- --check
cd rhwp-studio && npx tsc --noEmit
```

### Stage 5-2. picture-props-dialog.ts 주석 갱신 (v2 미반영)

`rhwp-studio/src/ui/picture-props-dialog.ts:2156` 의 주석을 `Rust 측 set_picture_properties_native 의 v2 migration + v3 layout 정합 처리` 로 갱신.

### Stage 5-3. 통합 PR 발행

`local/task1151` 브랜치 push:
```bash
git push origin local/task1151
```

신규 PR:
```bash
gh pr create --repo edwardkim/rhwp \
  --base devel \
  --head johndoekim:local/task1151 \
  --title "Task #1151: 표 + picture 한컴 정합 (삽입 + 글자처럼 취급 토글 + 시각 layout)" \
  --body "..."
```

PR body 에 포함:
- closes #1151
- v1 + v2 + v3 scope 묶음 정리
- 한컴 정합 산출물 (`samples/tac-verify/`) 와 model + 시각 양방향 정합 증거
- 단위/통합 테스트 합산 (v2 11 + v3 4 + 시각 통합)
- 시각 시연 결과 (스크린샷 또는 SVG 좌표 비교)

### Stage 5-4. 최종 보고서

`mydocs/report/task_m100_1151_v3_report.md` (Task #1151 전체 통합):
- v1 / v2 / v3 의 결함 분석, fix, 검증 통합
- 한컴 정합 검증 (model + 시각)
- 검증 결과 (단위 / 통합 / 시각 / 회귀)
- 산출 문서 / 자료 / sample 목록

또는 v1, v2 의 별도 보고서 (`task_m100_1151_report.md`, `task_m100_1151_v2_report.md`) 와 v3 보고서를 모두 유지하고 v3 보고서가 합산 결과 참조.

### Stage 5-5. 최종 Stage 5 commit + push

```
Task #1151 v3 Stage 5 + 최종 보고서: 통합 PR 발행

- cargo test --lib / clippy / fmt 모두 GREEN
- WASM 빌드 + npx tsc --noEmit clean
- picture-props-dialog.ts 주석 갱신 (v2 + v3 책임)
- 신규 PR 발행: closes #1151
- 최종 보고서 mydocs/report/task_m100_1151_v3_report.md
```

---

## 단계별 보고서 위치

각 Stage 완료 시 `mydocs/working/task_m100_1151_v3_stage{N}.md` 와 함께 커밋. CLAUDE.md 규칙 정합.
