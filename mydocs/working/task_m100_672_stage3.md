# Task #672 Stage 3 단계별 보고서 — 광범위 회귀 sweep + 최종 검증

## 1. 광범위 페이지네이션 회귀 sweep

`samples/` 폴더 전체 187 fixture 페이지 수 BEFORE/AFTER 비교:

| 영역 | 결과 |
|------|------|
| BEFORE (task671) | 187 fixtures / **2013 pages** |
| AFTER (task672) | 187 fixtures / **2013 pages** |
| **차이** | **0** ✅ |

**회귀 위험 영역 완전 좁힘** — 본 task 정정 (TAC 표 비례 축소 임계값 강화) 이 다른 fixture 의 페이지네이션에 영향 0.

## 2. 결정적 검증 (최종)

| 검증 영역 | 결과 |
|----------|------|
| `cargo build --release` | ✅ |
| `cargo test --lib --release` | ✅ **1155 passed** (회귀 0) |
| `cargo test --release --test svg_snapshot` | ✅ **6/6** |
| `cargo test --release --test issue_546` | ✅ **1/1** |
| `cargo test --release --test issue_554` | ✅ **12/12** |
| `cargo clippy --release` | ✅ 0 warnings |

## 3. 본 task 본질 영역 정정 결과

| 영역 | 결과 |
|------|------|
| TAC 표 비례 축소 발동 영역 (≤2% 차이) | 면제 → 측정값 row_heights 보존 ✅ |
| 발동 영역 (≥5% 차이, 의도적 압축) | 그대로 동작 ✅ |
| 셀 [21] row_heights[5] | 66.88 (비례 축소) → **67.76 (측정값 보존)** ✅ |
| 셀 [21] cell_h | 66.88 → **67.76** ✅ |
| layout 단 recompose 3줄 분할 | ✅ |
| SVG 단 3줄 baseline 그려짐 | ✅ |

## 4. 시각 판정 게이트웨이 (작업지시자)

본 task #672 본질 영역:
- ✅ TAC 표 비례 축소 면제 정정 완료
- ✅ row_heights 측정값 보존
- ✅ 광범위 sweep 회귀 0

잔존 결함 (별도 Issue):
- ⚠️ 셀 [21] / [52] 마지막 줄 PNG 시각 표시 안 됨
- → **Issue #674** 별도 등록 ([링크](https://github.com/edwardkim/rhwp/issues/674))

본 task 본질 영역 (TAC 표 비례 축소) 과 다른 본질 영역 (paragraph_layout 줄 위치 vs row_heights) 으로 분리.

## 5. 코드 변경 사항 정리

### 단일 분기 정정

`src/renderer/height_measurer.rs:805-822` — TAC 표 비례 축소 임계값 가드 추가.

- 임계값: `(common_h * 0.02).max(1.0)` — 2% + 절대값 1px 보장
- 작은 차이 (≤2%): 측정값 우선 (raw_table_height) 사용 → row_heights 보존
- 큰 차이 (≥2%): 비례 축소 발동 (기존 동작 유지)

### 회귀 위험 영역 좁힘

- 단일 분기 가드 추가 — 다른 영역 무영향
- 광범위 sweep 차이 0 입증
- 의도적 큰 압축 (5%+ 차이) 그대로 동작

## 6. 진단 도구

- `examples/inspect_task672.rs` — TAC 표 비례 축소 발동 영역 sweep 도구 (187 fixture 분포 분석)

## 7. Stage 3 완료 — 최종 보고서 작성 영역 진입

본 단계별 보고서 + 결정적 검증 + 광범위 sweep 회귀 0 + 잔존 결함 별도 Issue 분리 → **최종 결과 보고서** (`mydocs/report/task_m100_672_report.md`) 작성 진입.
