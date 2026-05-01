# Task #431 최종 결과보고서 — 분할 표 셀 내 문단 미출력 (단위 mismatch 정합화) + dump-pages 진단 도구 보강

## 결과 요약

`samples/synam-001.hwp` 15페이지 (인덱스 14) 가 빈 채로 출력되던 결함 정정. 페이지 14, 15, 16 에 동일 fragment `PartialTable rows=6..7 cont=true` 가 반복되지만 페이지 15 만 SVG 빈 페이지였던 정황.

**본질**: `compute_cell_line_ranges` 의 `content_limit` 비교 단위 mismatch — `line_end_pos` (절대 좌표) 와 `content_limit` (상대 길이) 비교가 잘못되어 `content_offset > content_limit` 케이스에서 즉시 break.

**부수 작업**: dump-pages 에 PartialTable 의 split_start/end 정보 추가 — 본 결함 진단 + 향후 분할 표 결함 진단 도구.

## 회귀 origin 확정

| 항목 | 정합 |
|------|------|
| commit | **`af6753f5`** "Task #362: kps-ai PartialTable + Square wrap 처리 (8항목 누적 수정)" |
| 머지 시점 | 2026-04-27 13:10 (메인테이너 직접 정정) |

**bisect 영역**: 4/13 (VSCode 확장 빌드 시점, 정상) ~ 4/30 (현재, 결함). 확장 WASM 시점 비교로 영역 확정:
- VSCode 확장 (4/13): 3,491,947 bytes — 정상
- 크롬 확장 (4/21): 3,901,445 bytes — 정상
- 현재 (4/30): 4,204,778 bytes — 결함

bisect 자동화로 Task #362 commit 확정.

## 정정 본질

### 단위 mismatch (라인 ~2168)

```rust
// 정정 전
if has_limit && line_end_pos > content_limit {  // ← 단위 mismatch
    break;
}
```

- `line_end_pos = cum + line_h` = **절대 좌표** (셀 시작부터의 누적 px)
- `content_limit = split_end_limit = avail_content` = **상대 길이** (현재 페이지에서 표시할 px)

```rust
// 정정 후
let abs_limit = if has_limit { content_offset + content_limit } else { 0.0 };

if has_limit && line_end_pos > abs_limit {
    break;
}
```

### 케이스별 정합

| 페이지 | content_offset | content_limit | abs_limit | 정정 효과 |
|--------|---------------|---------------|-----------|----------|
| 14 | 315.1 | 965.4 | 1280.5 | 출력량 증가 (이전 cap 정합화) |
| **15** | **1280.6** | **965.4** | **2246.0** | **빈 페이지 → 정상 출력** |
| 16 | 2246.0 | 0.0 | 0.0 | 변화 없음 (`content_limit=0` 케이스) |

## 변경 영역 (2 파일 — 5개소)

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/table_layout.rs::compute_cell_line_ranges` | `abs_limit` 변수 도입 + atomic 분기 + line 분기에서 `abs_limit` 사용 (3개소) |
| `src/document_core/queries/rendering.rs::dump_page_items` | PartialTable 출력에 split_start/end 정보 추가 (1개소 + struct 패턴 destructure) |

## Stage 별 결과

| Stage | 내용 | 결과 |
|-------|------|------|
| 1 | dump-pages 보강 + 베이스라인 측정 | 단위 mismatch 결함 식별 |
| 2 | 정정 적용 (`abs_limit`) | 페이지 15 빈 페이지 해소 |
| 3 | 광범위 회귀 검증 | cargo test 1080 passed, kps-ai 정합 영역 (p56/p67-68/p73) 보존 |
| 4 | 작업지시자 시각 검증 | synam-001 14/15/16 정상 + kps-ai 정상 |
| 5 | 최종 결과보고서 + 오늘할일 갱신 | 본 문서 |

## 검증 게이트

| 검증 | 결과 |
|------|------|
| `cargo test --lib` | **1080 passed** ✅ |
| `cargo test --test svg_snapshot` | 6/6 ✅ |
| `cargo test --test issue_418` | 1/1 ✅ |
| `cargo clippy --lib -- -D warnings` | 0건 ✅ |
| WASM 빌드 | 4,204,778 bytes ✅ |
| 작업지시자 시각 검증 | synam-001 + kps-ai 정상 |

## 잔여 결함 — 이슈 #485 등록

페이지 15 의 마지막 줄 시각 클립핑 잔여 결함 — 본 정정의 추가 결함이 아닌 Task #362 의 `split_end_limit = avail_content` (typeset 추정) 와 layout 의 실제 line height 가 미세하게 어긋난 정합 문제.

## 진단 도구 — dump-pages 보강

PartialTable 출력에 split 정보 추가:

```
PartialTable   pi=140 ci=0  rows=6..7  cont=true  8x2  vpos=9340  split_start=1280.6 split_end=965.4
```

본 결함 진단의 결정적 도구. 향후 분할 표 결함 진단에 동일하게 활용 가능.

## 작업지시자 통찰 (보존)

본 작업의 본질 진단은 작업지시자의 통찰로 도출:

1. *"VSCode 확장에서도 이 문제는 발생되지 않습니다"* + *"크롬 확장으로 보면 이 현상이 없고 정상적으로 조판됩니다"* — 회귀 origin 가설 (4/13~4/30)
2. *"v0.7.3 버전에서는 이 문제가 없었습니다"* — bisect 시작점 확정
3. *"분할된 표의 셀내 문단이 출력되지 않는 문제로 접근해야 합니다"* — 본질 명확화
4. *"이번 기회에 dump-pages 에 분할 표에 대한 정보도 추가해서 진행하는게 좋을 것 같습니다"* — 진단 도구 보강

## 메모리 원칙 정합

- **`feedback_visual_regression_grows`**: dump-pages 결과 (typeset 단계) 와 SVG 출력 (layout 단계) 이 다를 수 있음 — 자동 검증 만으로 시각 회귀 검출 불가, 작업지시자 시각 판정 게이트 필수
- **`feedback_search_troubleshootings_first`**: 트러블슈팅 사전 검색 — `typeset_partial_table_wrap_around.md` (Task #362 트러블슈팅), `typeset_layout_drift_analysis.md` 정합
- **`feedback_hancom_compat_specific_over_general`**: Task #362 의 일반화 정정 (8항목 누적) 이 단위 mismatch 결함을 도입 → 케이스별 명시 가드 우선

## 다음 단계

- 이슈 #431 close
- `local/task431` → `local/devel` 머지
- 오늘할일 갱신
- 잔여 결함은 이슈 #485 로 추후 처리
