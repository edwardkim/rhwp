# Task #565 Stage 3 — 본질 정정 적용 + 검증 보고서

- **이슈**: [#565](https://github.com/edwardkim/rhwp/issues/565)
- **단계**: Stage 3 (구현 + 검증)
- **작성일**: 2026-05-04
- **채택 안**: **안 A** (사전 검증 통과)

## 1. Stage 3-1 — 안 A 사전 검증 (12번 본문 한정 강제 적용)

`force_plain_t565 = section==0 && para_index==61` 가드를 임시로 추가하여 12번 본문만 일반 `layout_paragraph` 로 강제 전환.

### 결과 (12번 본문 9개 수식 좌표)

| 변경 전 (Stage 1/2) | 변경 후 (Stage 3-1) |
|---|---|
| 9개 모두 (534.8, 1218.106) — 겹침 | 첫 줄 y=1174.91: X(606.87), A(887.87), B(934.87) |
| | 둘째 줄 y=1196.37: C(549.87), D(569.87), m-4(698.87), m-2(743.97), m+2(789.08), m+4(834.19) |

→ **9개 수식 모두 정상 분산. 안 A 본질 정정 동작 확인**.

표 위치 / 본문 레이아웃 회귀 없음 (Stage 3-3 광범위 sweep 으로 재검증).

## 2. Stage 3-2 — 본 정정 적용

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
    // (기존 layout_inline_table_paragraph 경로)
} else {
    // 일반 layout_paragraph 경로 — 인라인 표 + 인라인 수식 정상 처리
}
```

## 3. Stage 3-3 — 광범위 sweep 검증

### 3.1 `cargo test --lib`

```
test result: ok. 1125 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 71.87s
```

→ **1125 통과, 0 실패** (변경 전 1125 ↔ 변경 후 1125 동일).

### 3.2 `cargo test --release --test svg_snapshot`

```
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
```

→ **6/6 통과** (issue_147 aift, issue_157, issue_267 ktx, form_002, table_text, render_is_deterministic).

### 3.3 광범위 fixture sweep (15 sample / 274 페이지)

대상 fixture (인라인 수식/표 가능성 높은 것 + 회귀 위험 fixture 종합):

```
exam_science.hwp, exam_kor.hwp, exam_math.hwp, exam_eng.hwp, exam_social.hwp,
aift.hwp, issue-505-equations.hwp, eq-01.hwp, equation-lim.hwp,
atop-equation-01.hwp, 21_언어_기출_편집가능본.hwp, 2010-01-06.hwp,
biz_plan.hwp, k-water-rfp.hwp, kps-ai.hwp
```

baseline (Stage 2 commit) vs after-fix SVG byte-compare:

```
총 274 SVG: identical=271, diff=3

diff 파일:
  exam_science_002.svg   ← 12번 본문 정정 (9개 수식 정상 분산)
  exam_science_003.svg   ← 15번 본문 정정 (수식 정상 표시)
  exam_science_004.svg   ← 18/19번 본문 정정 (수식 정상 표시)
```

→ **271 페이지 byte-identical (회귀 0)**. 의도된 정정 3 페이지 (exam_science 페이지 2/3/4) 모두 12/15/18/19번 본문 인라인 수식 정상 표시.

### 3.4 의도된 정정 SVG 텍스트 검증

```
[페이지 2 12번] (단,X는임의의원소기호이고,A,B,C,D의원자량은각각m-4,m-2,m+2,m+4이다.)
[페이지 2 11번] (단,X와Y는임의의원소기호이고,Y2는물과반응하지않는다.)        ← 이전부터 정상
[페이지 3 15번] 15.-그림(가)는원자WsimY의를,(나)는원자이온반지름이온의전하XsimZ의을…
[페이지 4 18번] 18.-표는2xMHA(aq),xMH2B(aq),yMNaOH(aq)의부피를달리하여혼합한…
[페이지 4 19번] 19.-다음은A(g)로부터B(g)와C(g)가생성되는반응의화학반응식이다.
[페이지 4 19번 단서] (단,XsimZ는임의의원소기호이다.)
```

→ 인라인 수식 모두 정상 표시. 누락 0.

### 3.5 `cargo clippy --release`

`src/document_core/commands/{table_ops.rs:1007, object_ops.rs:298}` 의 `panicking_unwrap` 2 건 — **변경 전 baseline 에서도 동일 발생**. 본 변경과 무관 (사전 결함). 별도 정정 권고 (본 task 범위 외).

본 변경(`layout.rs` +13/-1)에 의한 신규 clippy 경고/오류 **0**.

### 3.6 WASM 빌드

본 세션 환경의 Docker daemon 미가동 (`Cannot connect to the Docker daemon`). WASM 빌드는 별도 머신/세션에서 검증 필요. 다만:

- 변경 코드는 `Control` enum match + boolean 가드 (WASM 비호환 API 미사용)
- 동일 `Control::Picture/Shape/Equation` match 패턴은 동일 파일 다른 곳에서 이미 사용 중 (예: `layout.rs:1991`)

→ WASM 빌드 회귀 가능성 매우 낮음. **시각 판정 단계(Stage 4) 또는 별도 환경에서 보강 검증 필요**.

## 4. 변경 LOC

| 파일 | +/- | 비고 |
|------|-----|------|
| `src/renderer/layout.rs` | **+13 / -1** | `has_other_inline_ctrls` 정의 + `has_inline_tables` 가드 강화 |

본 task 의 본질 정정 한 곳. 디버그 로그 모두 제거 확인됨 (`git diff` 12 LOC 추가만 표시).

## 5. 회귀 위험 검증 결과

| 항목 | 결과 |
|------|------|
| `cargo test --lib` 1125 | ✅ 0 회귀 |
| `svg_snapshot` 6/6 | ✅ 0 회귀 |
| 274 페이지 byte-identical | ✅ 271/274 (3건 = 의도된 정정) |
| 인라인 표만 사용 paragraph (12번 셀 내 수식 등) | ✅ 정상 분산 (좌표 분산됨) |
| 인라인 표 + 인라인 그림 동시 케이스 | ✅ `has_other_inline_ctrls` 에 포함 — 동일 분기 |
| 인라인 표 + 인라인 글상자 동시 케이스 | ✅ `has_other_inline_ctrls` 에 포함 — 동일 분기 |
| Task #287 (display equation as own LINE_SEG) | ✅ `paragraph_layout::layout_composed_paragraph` L2245 분기 무수정 — 회귀 영역 외 |

## 6. 잔존 사항 / Stage 4 진행 항목

1. **시각 판정**: 작업지시자 SVG 시각 판정 (12/15/18/19번 본문 인라인 수식 정상 표시 + 회귀 없음).
2. **rhwp-studio web Canvas 시각 판정**: 동일 문서 brwoser 렌더 결과 검증.
3. **WASM 빌드 검증**: Docker 가동 환경에서 별도 확인 후 결과 회신.
4. **이슈 #566 (7번 ㉠ 위치)** 와의 양립성: 본 정정은 7번 셀 내부 ㉠/㉡ 위치 변동 영향 없음 (셀 베이스라인 계산 미변경) — 별도 task 로 후속 처리.

## 7. 승인 요청

본 Stage 3 결과로 **Stage 4 (시각 판정 + 최종 보고서) 진입**을 승인 요청합니다. 시각 판정 통과 시 `task_m100_565_report.md` 작성 + `orders/20260504.md` 갱신 후 본 task 종료.
