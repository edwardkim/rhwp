# 최종 결과보고 — #2063 초대형 표 성능 + 과분할

브랜치 `fix/2063-cellunits-quadratic-scan` (base: devel). 결정: **성능 확정 + 과분할은 #1937/#1842 트랙 인계**.

## 1. 성능(O(n²) 제거) — 완료 ✅

### 근본 원인
`cell_units_uncached`(table_layout.rs)가 표-불변량 `has_visible_text_with_nested_table`
(전체 셀 스캔)를 **셀별로** 계산 → `cell_units` 캐시를 채우는 동안 52,694² ≈ **28억 회**.

### 수정 (commit f3e1ad30)
표 포인터 키 캐시 `table_nested_text_flag_cache` 신설, 표 단위 1회 계산으로 hoist. O(셀²)→O(셀).
`src/renderer/layout.rs`(+7), `src/renderer/layout/table_layout.rs`(+23/−6).

### 검증 (21914299, 52,694셀)
| 항목 | before | after |
|---|---|---|
| dump-pages(213p) | ~47s→timeout | **2s** |
| export-pdf | hang | 167s |
| render-diff(--via hwp) | **>420s TIMEOUT** | **283s** (배치 임계 이내) |
| 페이지 수 / 판정 | — | **213 불변, 0.00px, PASS** |
| lib 테스트 | — | **2143 passed / 0 failed** |

→ **#2063 1차 증상(렌더 타임아웃/hang) 해소.** clippy clean, fmt(변경 파일).

## 2. 과분할(+51) — 진단 완료, #1937/#1842 인계

### 계측 (결정적)
- rhwp 표 높이 **128,437px** vs 한글 2022 **96,498px** → **rhwp +33% 과대**.
- 행높이 분포: rhwp 단일행 ~**20px** × 5,036행 vs 한글 median **17.08px**(= authored cell.height 1282HU).
- 다중행 wrap 문제 아님(rhwp 26px+ 21행뿐). **단일행 ~3px/행 계통 과대**가 지배.

### 메커니즘
`resolve_row_heights`가 `required = content_height + pad_top + pad_bottom` 로 계산 후
`max(authored cell.height, required)`. rhwp의 (줄높이+패딩)≈20px가 authored 17.1px를 초과 →
대부분 행이 authored 높이를 무시하고 팽창 → 표 +33% → +51쪽.

### 판정
국소 버그 아님. **저장 authored 높이 신뢰 vs content 측정**의 계통 긴장 = #1937/#1842/#1658
(RHWP_TABLE_DRIFT) 행/줄높이 드리프트 클래스. 안전 수정은 **코퍼스 전역 줄높이 재캘리브레이션**
(golden 스냅샷 대량 변동 + 전역 시각 재판정) = 전용 트랙. 본 타스크(무회귀)에서 미수행.
→ 정밀 계측 결과를 #1937/#1842에 인계(코멘트).

## 3. 산출
- Stage1 커밋 f3e1ad30. 계획 `task_m100_2063.md`/`_impl.md`, 보고 `_stage1.md`/`_report.md`.
- Fork PR → devel (성능 전용).
- #2063 갱신(성능 해소 + 과분할 인계), #1937 코멘트(정밀 계측).
