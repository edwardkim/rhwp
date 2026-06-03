# Task #301 최종 보고서: z-table 셀 수식 이중 렌더링 수정

- **이슈**: https://github.com/edwardkim/rhwp/issues/301
- **브랜치**: `local/task301`
- **마일스톤**: 미지정
- **작업일**: 2026-04-24
- **수정 LoC**: src 1파일 +6 -1 / tests 1파일 신규(+45)

## 1. 배경

`samples/exam_math.hwp` 페이지 12 좌측 컬럼 #29 문제 안의 정규분포표(z-table, 5×2)에서 모든 셀의 숫자/헤더 텍스트가 SVG 출력 시 **두 번 그려져 시각적으로 겹침**.

작업지시자 제보로 시작. 한컴 PDF는 z-table을 정상 1회 렌더.

## 2. 근본 원인

### 셀 데이터 구조
z-table 10개 셀 각각:
- `text=""` (빈 텍스트)
- 1 paragraph + 1 control = `Equation` (수식)
- 셀에 보이는 숫자/헤더(`0.5`, `0.1915`, `z`, `P(0≤Z≤z)` 등)는 모두 **수식 스크립트로 저장**됨

### 두 경로 중복 emit
빈 runs를 가진 셀 paragraph의 TAC(treat-as-char) 수식이 두 곳에서 EquationNode로 emit:

1. **`paragraph_layout.rs:1996-2057`** — Task #287에서 추가된 빈-runs TAC 수식 인라인 처리.
   - `comp_line.runs.is_empty() && !tac_offsets_px.is_empty()` 조건에서 EquationNode push
   - 후속 `tree.set_inline_shape_position(...)` 호출

2. **`table_layout.rs:1602-1648`** — Task #287 이전부터 존재하던 셀 Equation 직접 렌더 분기.
   - `has_text_in_para = false` (빈 텍스트)일 때 EquationNode 직접 push
   - paragraph_layout이 그렸는지 검사하지 않음

→ **Task #287 도입 후 발생한 회귀 버그**. 두 경로의 좌표 계산 미세 차이(Δx≈2.5/28.9, Δy≈-1.31)로 살짝 어긋난 두 텍스트가 겹쳐 보임.

## 3. 수정 내용

### `src/renderer/layout/table_layout.rs:1602-1620`

`Control::Equation` 분기에 가드 추가:

```rust
let already_rendered_inline = tree
    .get_inline_shape_position(section_index, cp_idx, ctrl_idx)
    .is_some();
if has_text_in_para || already_rendered_inline {
    inline_x += eq_w;  // paragraph_layout에서 이미 렌더됨
} else {
    // 기존 fallback: paragraph_layout이 그리지 않은 코너 케이스
    ...
}
```

`tree.get_inline_shape_position`이 `Some`이면 paragraph_layout이 이미 렌더하고 위치를 등록한 것이므로 직접 emit 스킵. paragraph_layout의 두 경로(line 1689 일반, 2052 빈-runs Task #287) 모두 `set_inline_shape_position`을 호출하므로 일관되게 검출된다.

기존 `has_text_in_para` 검사는 보조 가드로 유지. paragraph_layout이 부분적으로만 렌더하는 코너 케이스를 보호한다.

## 4. 검증

### 회귀 테스트 (신규)

`tests/issue_301.rs`: SVG에서 z-table 값(`0.1915`, `0.3413`, `0.4332`)이 각 1회만 출현하는지 검증. `0.4772`는 본문에도 등장하므로 2회 기대.

수정 전: FAIL (`0.1915` found 2)
수정 후: PASS

### 전체 테스트 무회귀

```
cargo test --release: 1039 passed / 0 failed / 1 ignored
  - lib (992) / hwpx_roundtrip (14) / hwpx_to_hwp_adapter (25)
  - issue_301 (1, 신규) / svg_snapshot (6) / tab_cross_run (1)
cargo clippy --release: clean
```

기존 SVG 골든 6건 무회귀 → 본 수정은 빈-텍스트 셀의 TAC 수식 케이스에만 영향.

### 다른 샘플 회귀

`exam_math.hwp` 20쪽 + `exam_math_no.hwp` 20쪽 + `exam_math_8.hwp` 1쪽, 셀 내 transform 그룹 2+ 건 휴리스틱 검출 결과 모두 0건.

### 시각 확인

PNG 변환(`inkscape`) 비교: 수정 전 z-table 텍스트 겹침 → 수정 후 깔끔한 1회 렌더링.

## 5. 영향 범위

- **수정 영향**: 빈 텍스트 셀 + TAC 수식 컨트롤 가진 모든 표
- **WASM canvas 경로**: 동일 RenderTree 사용하므로 본 수정으로 동시 해결
- **다른 컨트롤(Picture, Shape) 유사 패턴**: 본 타스크 범위 외. Picture는 `table_layout.rs:1437`의 `will_render_inline` 검사가 있으나 빈-runs 케이스 미처리 가능 → 별도 이슈 후보.

## 6. 산출물

- `mydocs/plans/task_301.md` — 수행계획서
- `mydocs/plans/task_301_impl.md` — 구현계획서
- `mydocs/working/task_301_stage{1,2,3}.md` — 단계별 보고서
- `mydocs/report/task_301_report.md` — 본 문서
- `src/renderer/layout/table_layout.rs` — Equation 분기 가드 추가
- `tests/issue_301.rs` — 회귀 테스트

## 7. 교훈

1. **양방향 렌더 경로의 동기화 검증**: Task #287에서 paragraph_layout에 빈-runs TAC 수식 처리를 추가할 때 table_layout의 기존 fallback과의 중복 가능성이 검토되지 않았다. 두 경로가 같은 노드를 emit할 수 있는 구조에서는 한쪽 추가 시 반드시 다른쪽 가드를 함께 점검해야 한다.
2. **`set_inline_shape_position`의 활용**: 이미 렌더한 inline 객체의 좌표를 등록하는 기존 메커니즘이 중복 회피용 `is_rendered` 플래그로도 자연스럽게 기능. 별도 상태 추가 없이 가드 구현.

## 8. 후속 작업

- **이슈 close**: 작업지시자 승인 후 수행
- **devel merge**: 작업지시자 지시 후 수행
- **별도 이슈 후보**: Picture/Shape의 빈-runs 케이스 동일 패턴 점검
