# Stage 1 완료보고 — #2063 성능(O(n²) 제거)

## 근본 원인
`cell_units_uncached`(table_layout.rs)에서 표-불변량 `has_visible_text_with_nested_table`
(전체 셀 스캔)를 **셀별로** 계산. `cell_units`는 셀별 메모이즈되나 캐시를 채우는 과정에서
셀마다 전체 셀 스캔 → 52,694² ≈ **28억 회** = 지배적 O(n²) hot-path.

## 수정
- `LayoutEngine`에 `table_nested_text_flag_cache: RefCell<HashMap<usize,bool>>` 신설
  (표 포인터 키, `cell_units_cache`와 동일 조판 경계에서 clear).
- `table_has_visible_text_with_nested_table(table)` 헬퍼로 표 단위 1회 계산·캐시.
- `cell_units_uncached`는 이 헬퍼 호출로 대체. O(셀²) → O(셀).
- 변경: `src/renderer/layout.rs`(+7), `src/renderer/layout/table_layout.rs`(+23/−6).

## 검증 (21914299, 52,694셀)
| 항목 | before | after |
|---|---|---|
| dump-pages(전체 213p) | ~47s(→timeout) | **2s** |
| export-pdf | hang | 167s |
| render-diff(--via hwp) | **>420s TIMEOUT** | **283s** (배치 임계 이내) |
| 페이지 수 | 213 | **213 (불변)** |
| render-diff 판정 | — | **A=213 B=213, 0.00px, PASS** |

- **산출 불변 확인**: 페이지 수·좌표 변화 0 (순수 최적화).
- **무회귀**: `cargo test --release --lib` **2143 passed / 0 failed**. clippy clean. fmt(변경 파일).

## 잔여 (Stage 2로)
- 과분할 +51(rhwp 213 vs 한글 162)은 성능과 무관한 별개 사안. 계측: rhwp 표높이 **128,407px**
  vs 한글 **96,498px (+33%)**. 단일행은 정합(17.08px), **다중행 셀 높이 과대측정**이 원인 →
  #1937/#1842 행/줄높이 드리프트 계열. Stage 2에서 국소 버그 여부(줄 과다 wrap 등) 판별.
