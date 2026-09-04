# #1937 최종 결과보고서 — 각주 많은 RowBreak 표 연속 페이지 과분할 수정

- 이슈: edwardkim/rhwp#1937 (M100)
- 브랜치: `local/task1937` (upstream/devel 기준)
- 유형: 렌더 페이지네이션(표 행 분할) — 시각 충실도
- 결과: **수정 완료, 무회귀 검증 통과**

## 1. 문제

표 밀집 장문 보고서에서 rhwp 페이지 수가 한글 대비 폭주.
대표: `소상공인 중간보고서(2).hwp` — **rhwp 231쪽 vs 한글 50쪽**.

## 2. 근본 원인 (측정 기반 재진단)

이슈 제목 가설("대형 데이터 표 행높이 ~3배 과대측정")은 **오진**이었다. 실측으로
확정한 진짜 원인:

- 231쪽 폭주의 **180쪽이 단 하나의 표(pi=306, 122행×6열)** 에서 발생(누적 delta 는
  pi=306 직전까지 0, pi=306 에서 +180 급증). 행높이는 정상(`cut_row_h`≈`mt.row_heights`
  ≈33~113px, 합 5733px ≈ 6.5쪽분).
- 표 pi=306 은 **셀 각주 22개**(각주 콘텐츠 721px, projected 820px)를 가진다.
  `typeset_block_table` 이 `available = base_available − total_footnote = 895.8 − 820
  = 75.8px` 로 한 번 계산해 `table_available` 로 굳힌다.
- 행 분할 while 루프가 이 `table_available(75.8)` 을 **모든 연속(continuation) 페이지에
  재사용**한다(`typeset.rs:13192`, `page_avail = if is_continuation { table_available }`).
- 연속 페이지는 각주가 쌓이지 않은 신선 full-page(가용 895.8)인데 시작 페이지의 좁은
  잔여 75.8 을 물려받아 페이지당 ~1행만 배치 → 122행 표가 188쪽으로 폭주.
- 정상 표 pi=99(각주 없음)는 `table_available=895.8` 이라 정상(8쪽).

진단 도구: `RHWP_TABLE_DRIFT`(내장) + 임시 계측(측정 후 원복). `dump-pages`/`dump`/
`export-render-tree`/한글 COM 오라클(para→page)로 폭주 위치·행높이·각주 성분을 실측.

## 3. 수정

`src/renderer/typeset.rs` `typeset_block_table` 행 분할 루프: 연속 페이지 `page_avail`
을 시작 페이지 `table_available` 대신 **신선 페이지 `base_available`**(zone offset·border
tolerance 유지)로 변경. 레퍼런스 Paginator `engine.rs:2502-2503`(continuation 에
`base_available_height` 사용)과 두 엔진 동작을 통일. 표 각주는 첫 fragment fit 판정에서만
보수적으로 예약한다.

실질 1줄 변경(+주석).

## 4. 검증

### 효과
| 문서 | 전 | 후 | 한글 |
|---|---:|---:|---:|
| 소상공인 중간보고서(canonical) | 231 | **52** | 50 |
| └ pi=306 표 단독 | 188쪽 | **9쪽** | ~9 |
| 문체부 GCC 중간보고서 | 404 | **322** | 320 |

canonical pi-page 오라클: PAGE_DELTA **+181 → +2**(잔여 +2 는 pi73~ ±1 시프트·일부
#1920 캐럿 계열 — 본 이슈와 무관한 별개 축).

### 무회귀
- `cargo test --lib`: **2126 passed, 0 failed**(pagination 유닛 포함).
- 통합 테스트: `issue_1073_nested_table_split`(3)·`issue_1070_tac_table_post_text_overflow`(3)
  ·`issue_1417_pagination_cursor_render`(1)·`diag_1042_table_row_height`(2)
  ·`issue_1050_footnote_serialize`(7) 전부 통과.
- #1658 게이트: byeolpyo1=4쪽, byeolpyo4=26쪽(기대값 유지).
- 정상 분할 표 pi=99: 8쪽 무변화.
- **과도 수정 아님**: 별개 기전 문서 +88 공급망(212→212)·+27 거제시(387→387) 무변화.
- 풀 스위트·SVG 스냅샷: CI 확인(작업지시자 지침).

## 5. 잔여/후속

- +88 공급망·+27 거제시 등 **비-각주 과대 페이지**는 각주 예약 버그가 아닌 별개 기전
  (행높이/기타). 본 이슈 범위 밖 — 필요 시 별도 이슈로 분리 추적.
- 연속 페이지에 실제 각주가 걸치는 표의 per-page 각주 정밀 예약은 레퍼런스 Paginator 도
  미적용이라, 본 수정으로 두 엔진 동작이 통일된 상태.

## 6. 산출물

- 계획: `mydocs/plans/task_m100_1937.md`
- 단계 보고: `mydocs/working/task_m100_1937_stage1~3.md`
- 소스: `src/renderer/typeset.rs`
- 회귀 가드: `tests/issue_1937_rowbreak_footnote_overpagination.rs`
  (페이지 수 45..=80 밴드 — 수정 전 231쪽 폭주 재발·과소 페이지 양방향 감지)
- 재현 샘플(공개): `samples/issue1937_rowbreak_footnote_overpagination.hwp`
  (정책연구정보서비스 "소상공인 중간보고서(2)", 한글 2022 = 50쪽)
