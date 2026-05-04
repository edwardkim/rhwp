# Task #565 최종 결과 보고서 — exam_science.hwp 인라인 수식(treat_as_char) 미렌더 정정

- **이슈**: [#565](https://github.com/edwardkim/rhwp/issues/565)
- **마일스톤**: v1.0.0 (M100)
- **브랜치**: `local/task565` (분기: `local/devel`)
- **작성일**: 2026-05-04
- **상태**: **완료 (시각 판정 대기)**

## 1. 결함 요약

`samples/exam_science.hwp` 페이지 2 의 12·15·18·19번 문제 본문 인라인 수식 (원소 기호 `A/B/C/D/X` + 원자량 `m-4/m-2/m+2/m+4` 등) 이 모두 빈 자리만 표시되고 글자가 누락. 한컴 PDF 정답에는 정상 표시.

## 2. 본질 원인

`src/renderer/layout.rs::layout_column_item` 의 `PageItem::FullParagraph` 분기에서:

- `has_inline_tables = TRUE` 인 문단은 `paragraph_layout::layout_inline_table_paragraph` 가 호출됨
- 이 함수는 인라인 표 + 텍스트 세그먼트만 처리하고 **인라인 수식·treat_as_char Picture/Shape 는 무시**
- 결과: `inline_shape_position` 미등록 → paginator 가 등록한 `PageItem::Shape (수식)` 들이 `shape_layout` 에 도달했을 때 `inline_pos.is_none()` → fallback 경로 (`shape_layout.rs:140-181`) 가 모두 동일 좌표 `(col_area.x, para_y)` = SVG `(534.8, 1218.106)` 에 9개 수식을 겹쳐 그림

`paragraph_layout::layout_composed_paragraph` 의 인라인 수식 처리 경로 (run_tacs / inline_x) 자체는 정상 — 0.60 (12번 그림 문단, 인라인 표 없음) 의 8개 수식은 정상 분산되었으며, 0.61 (12번 본문, 인라인 표 + 수식 9) 만 잘못된 분기로 떨어진 것.

## 3. 본질 정정

`src/renderer/layout.rs` (+13 / -1 LOC):

```rust
// [Task #565] 인라인 표 + 다른 인라인 컨트롤(수식/treat_as_char Picture/Shape)
// 이 같이 있는 문단은 layout_inline_table_paragraph 가 인라인 수식 등을
// 처리하지 않아 shape_layout fallback (col_area.x, para_y) 으로 9개 수식이
// 동일 좌표에 겹친다 (exam_science.hwp 12/15/18/19번). 일반
// layout_paragraph 로 보내 인라인 표 + 인라인 수식이 같은 line/x 체계
// (run_tacs / inline_x) 로 정상 배치되도록 한다.
let has_other_inline_ctrls = para.controls.iter().any(|c| match c {
    Control::Equation(_) => true,
    Control::Picture(p) => p.common.treat_as_char,
    Control::Shape(s) => s.common().treat_as_char,
    _ => false,
});

if has_inline_tables && !has_other_inline_ctrls {
    // 기존 layout_inline_table_paragraph 경로
} else {
    // 일반 layout_paragraph 경로 — 인라인 표 + 인라인 수식 정상 처리
}
```

## 4. 검증 결과

| 항목 | 결과 | 비고 |
|------|------|------|
| `cargo test --lib` | ✅ **1125 통과 / 0 실패** | 변경 전 1125 ↔ 변경 후 1125 동일 |
| `cargo test --release --test svg_snapshot` | ✅ **6/6 통과** | issue_147/157/267, form_002, table_text, render_is_deterministic |
| 광범위 sweep (15 fixture, 274 페이지) | ✅ **271 byte-identical, 3 의도 정정** | 회귀 0 |
| `cargo clippy --release` (본 변경) | ✅ **신규 경고/오류 0** | 사전 결함 2건 변경 전후 동일 (별도 정정 권고) |
| WASM 빌드 | ⏳ Docker 미가동 — 미검증 | Stage 4 보강 (별도 환경) |

### 4.1 광범위 sweep 대상 fixture (15개)

```
exam_science.hwp, exam_kor.hwp, exam_math.hwp, exam_eng.hwp, exam_social.hwp,
aift.hwp, issue-505-equations.hwp, eq-01.hwp, equation-lim.hwp,
atop-equation-01.hwp, 21_언어_기출_편집가능본.hwp, 2010-01-06.hwp,
biz_plan.hwp, k-water-rfp.hwp, kps-ai.hwp
```

### 4.2 의도된 정정 3 페이지

- `exam_science_002.svg` — 12번 본문 9개 수식 (X/A/B/C/D + m-4/m-2/m+2/m+4) 정상 분산
- `exam_science_003.svg` — 15번 본문 W~Y, X~Z 정상 표시
- `exam_science_004.svg` — 18/19번 본문 인라인 수식 정상 표시

### 4.3 SVG 텍스트 검증 (의도 정정 영역)

```
[페이지 2 12번] (단,X는임의의원소기호이고,A,B,C,D의원자량은각각m-4,m-2,m+2,m+4이다.)
[페이지 3 15번] 15.-그림(가)는원자WsimY의를,(나)는원자이온반지름이온의전하XsimZ의을…
[페이지 4 18번] 18.-표는2xMHA(aq),xMH2B(aq),yMNaOH(aq)의부피를…
[페이지 4 19번] 19.-다음은A(g)로부터B(g)와C(g)가생성되는반응의화학반응식이다.
[페이지 4 19번] (단,XsimZ는임의의원소기호이다.)
```

→ 인라인 수식 누락 0 건.

### 4.4 변경 전후 좌표 비교 (12번 본문)

| 변경 전 | 변경 후 |
|---|---|
| 9개 수식 모두 (534.8, 1218.106) — 동일 좌표 겹침 | 첫 줄 y=1174.91: X(606.87), A(887.87), B(934.87) |
|  | 둘째 줄 y=1196.37: C(549.87), D(569.87), m-4(698.87), m-2(743.97), m+2(789.08), m+4(834.19) |

## 5. 회귀 위험 검증

| 케이스 | 검증 결과 |
|--------|----------|
| 인라인 표만 사용 paragraph (12번 보기 셀 등) | ✅ 정상 분산 (좌표 분산됨) — 본 정정의 가드 조건 외 |
| 인라인 표 + 인라인 그림 동시 케이스 | ✅ `has_other_inline_ctrls` 에 포함 — 일반 layout_paragraph 처리 |
| 인라인 표 + 인라인 글상자 동시 케이스 | ✅ `has_other_inline_ctrls` 에 포함 — 일반 layout_paragraph 처리 |
| Task #287 (display equation as own LINE_SEG) | ✅ `paragraph_layout` L2245 분기 무수정 — 회귀 영역 외 |
| 광범위 sweep | ✅ 271/274 byte-identical (3건 = 의도 정정) |

## 6. 변경 요약

| 파일 | +/- | 비고 |
|------|-----|------|
| `src/renderer/layout.rs` | **+13 / -1** | `has_inline_tables` 가드 강화 |
| `mydocs/plans/task_m100_565.md` | 신규 | 수행 계획서 |
| `mydocs/plans/task_m100_565_impl.md` | 신규 | 구현 계획서 |
| `mydocs/working/task_m100_565_stage1.md` | 신규 | Stage 1 정밀 진단 |
| `mydocs/working/task_m100_565_stage3.md` | 신규 | Stage 3 적용 + 검증 |
| `mydocs/report/task_m100_565_report.md` | 신규 | 본 최종 보고서 |

## 7. 시각 판정 안내 (작업지시자)

검증용 SVG 4개 파일이 `output/` 에 생성됨:

- `output/exam_science_001.svg` — 페이지 1 (회귀 검증, byte-identical)
- `output/exam_science_002.svg` — 페이지 2 (12번 본문 정정, 11번도 비교 가능)
- `output/exam_science_003.svg` — 페이지 3 (15번 본문 정정)
- `output/exam_science_004.svg` — 페이지 4 (18/19번 본문 정정)

추가 검증 권고:

1. **rhwp-studio web Canvas 시각 판정**: WASM 빌드 후 brwoser 에서 동일 문서 렌더 결과 검증
2. **WASM 빌드**: Docker 가동 환경에서 `docker compose --env-file .env.docker run --rm wasm` 후 크기 확인

## 8. 후속 / 잔존 사항

- **이슈 #566 (7번 ㉠ 위치)**: 본 정정과 별개 결함 (셀 베이스라인). 별도 task 로 후속 처리 (이미 등록).
- **clippy 사전 결함 2건** (`document_core/commands/{table_ops.rs:1007, object_ops.rs:298}` `panicking_unwrap`): 본 task 범위 외 — 별도 issue 등록 권고.
- **이슈 close**: 작업지시자 승인 후 `gh issue close 565` 수행.
- **local/devel merge**: 작업지시자 승인 후 `git checkout local/devel && git merge local/task565 --no-ff` 수행.

## 9. 메모리 정합

- `feedback_essential_fix_regression_risk` ✅ 광범위 sweep + cargo test 1125 + svg_snapshot 6/6 으로 회귀 0 입증
- `feedback_rule_not_heuristic` ✅ 본질 정정 — 분기 조건 정확화 (휴리스틱 fallback 회피)
- `feedback_pdf_not_authoritative` ✅ PDF 는 보조 ref. SVG 텍스트 추출로 인라인 수식 표시 검증
