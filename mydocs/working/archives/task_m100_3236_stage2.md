---
kind: working
status: active
canonical: mydocs/plans/archives/task_m100_3236.md
last_verified: 2026-08-01
---

# Task #3236 Stage 2 보고 — 옵션 A 구현 + red-check

## 수정 내용 (승인된 옵션 A)

`src/renderer/typeset.rs` — #1891 단일행 선언-신뢰 특례
(`single_row_object_declared_fits_current`)에 **비율 상한** 요건 추가:

```rust
const SINGLE_ROW_DECLARED_TRUST_MAX_RATIO: f64 = 1.5;
// 조건에 추가:
&& table_total <= declared_object_total * SINGLE_ROW_DECLARED_TRUST_MAX_RATIO
```

근거: 폰트 대체 팽창은 인접 가드들 기준 10~20% 수준이라 **1.5배를 넘는 초과는
셀 내용이 진짜로 큰 것**이다. 이 경우 한컴도 쪽 경계에서 셀을 분할한다(정답지 PDF
실측). 상한 안(64px 초과 ~ 1.5배)의 특례는 보존되어 #1891 의도가 유지된다.

diff 규모: `typeset.rs` +11줄(상수 1 + 조건 1 + 주석 9). 로직 변경은 조건 한 줄이다.

## fixture 에서의 효과

| 지표 | 수정 전 | 수정 후 | 한컴 정답지 |
|---|---|---|---|
| pi=17 배치 | Table 통짜 (p1) | **PartialTable p1+p2** | 분할 |
| `LAYOUT_OVERFLOW_CELL` | 23건 (최대 468px) | **0건** | — |
| 쪽수 | 2 | **2 (불변)** | 2 |
| 쪽별 텍스트 분포 | 84% / 16% | **46% / 54%** | 45% / 55% |
| p2 시작 텍스트 | (경계 텍스트 p1 clip 구간에 방출) | **"경과되지 않은 외국인투자기업인 경우, 또는…"** | **글자 단위 일치** |

## 회귀 테스트 + red-check

`tests/issue_3236_split_single_cell_table.rs` 신설 — 공개 API(`HwpDocument` →
`render_page_svg`)만 사용:

1. 쪽수 2 유지 (분할이 새 쪽을 만들지 않음 — 한컴 동일)
2. 경계 텍스트("경과되지")가 **p2 에 존재**하고 **p1 에 부재**
3. p1 텍스트 비중 0.35~0.60 (통짜 배치 시 0.84 로 쏠림)

**red-check 통과**: 상한 조건 한 줄을 제거하고 실행 → "셀 분할 후반부가 p2 로
이어져야 한다" 단언에서 정확히 FAILED → 복원 후 ok.

## 인접 회귀 확인 (focused)

| 게이트 | 결과 |
|---|---|
| `issue_1891`(규제영향분석서 82쪽·157쪽 쪽수 fixture — 특례 도입 이슈) | **3 passed** |
| `issue_1842` | 2 passed |
| **table/split/rowbreak 계열 테스트 전체**(#1073 중첩 분할, #2097/#2105 계열, RowBreak fragment 등) | **118 passed / 0 failed** |
| `cargo fmt --check` | 통과 |

## Stage 3 계획 (승인 필요 게이트 포함)

계획서 §4 검증 기준을 측정법 정정(추적 2차)에 맞춰 갱신해 실행한다:

1. 정답지 PDF **이미지 스왑 대조** (작업지시자 지시 방법)
2. studio CDP 잉크 재측정 (p2 하단 0.02% → 정상 잉크)
3. `cargo test --profile release-test --tests` 전체 + clippy — **PR CI급이므로 별도 승인 후 실행**
4. wasm 재빌드 (studio/확장 반영 확인)
5. 92셋 스모크
