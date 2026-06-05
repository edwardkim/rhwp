# 최종 결과 보고서 — Task #1302: 미주 다줄 문단 → 같은 미주 연속 문단 줄간격 과소

- **이슈**: edwardkim/rhwp#1302 (M100 / v1.0.0)
- **브랜치**: `local/task1302` (base: `stream/devel` 9d3aa212)
- **기간**: 2026-06-05
- **성격**: 레이아웃 버그 수정 — 미주 줄간격 (조건 게이트)

## 1. 문제

`3-11월_실전_통합_2022.hwpx` 18쪽 좌측 단 미주: 다줄 문단 pi=852(분수 포함 키 큰 줄)
마지막 줄 "극솟값…갖는다" 다음, 같은 문제(문30) 연속 텍스트 문단 pi=853 "(나)를 고려하기…"
줄간격이 과소(12px, PDF ~18px).

## 2. 원인 (확정)

`src/renderer/height_cursor.rs::vpos_adjust` 의 `compact_endnote_page_tail_backtrack`
(page-path compact 미주 하단 frame-fit 보정)이, **컬럼 하단에서 같은 미주 연속 문단**에까지
발동. page_base 절대매핑(end_y)이 다줄 문단 로컬앵커 대비 ~20px drift 하면서
`end_y < y_offset - 8` 이 성립 → `end_y.max(prev_content_bottom_y).min(y_offset)` 가
trailing 줄간격(~6px)을 깎음.

핵심: stored vpos delta(curr_first − prev_vpos = 1502HU = prev_lh+prev_ls)는 **정상 한 줄
전진**을 인코딩하는데 backtrack 이 overlap 으로 오인. 기존 #1236(중간 컬럼 trailing)·
#1246 rescue(다음이 "문" 제목일 때만)는 이 **연속(비제목)+컬럼 하단** 경계를 미커버.

(#1300 회귀 아님 — 직전 커밋에서도 재현. #1236/#1246 코드는 이미 stream/devel 에 존재.)

## 3. 수정 — 조건 게이트

backtrack 은 stored 가 overlap(작은/rewind gap)을 가리키는 tail 에만 적용해야 한다.
curr 첫 줄 stored vpos 가 **정상 한 줄 전진(lh+ls) 이상**을 인코딩하는 **breakable 텍스트**
연속 문단은 비발동(y_offset=trailing 포함 정답 유지). **수식-only tail(#1274)은 atomic
이므로 제외** — frame-fit backtrack 유지.

| 파일 | 변경 |
|------|------|
| `height_cursor.rs` | `curr_first_full_advance`(수식-only 제외 + stored full-advance) 게이트를 `compact_endnote_page_tail_backtrack` 에 `&& !curr_first_full_advance` 로 추가 |
| `tests/issue_1139_inline_picture_duplicate.rs` | 회귀 핀 `issue_1302_...page18...` |

## 4. 검증

| 항목 | 결과 |
|------|------|
| 18쪽 극솟값→(나)를 gap | 12 → **18px** (PDF ~18 정합) |
| 신규 회귀 테스트 | 수정 전 FAIL(14) → 수정 후 PASS(18) |
| 전체 `cargo test` | **2036 passed, 0 failed** |
| 회귀 diff (3-11월 21p) | 18쪽만 변경(trailing 복원), 나머지 동일, 페이지수 21 불변 |
| 회귀 diff (3-10월) | 9쪽만 변경 — PDF 정합 개선(같은 버그 인스턴스) |
| 회귀 diff (3-09월) | 무변경 |

## 5. 변경 파일

- `src/renderer/height_cursor.rs`
- `tests/issue_1139_inline_picture_duplicate.rs`

## 6. 교훈 / 후속

- 미주 tail backtrack 은 **atomic 수식 tail ↔ breakable 텍스트 연속**을 구분해야 한다
  (전자만 frame-fit, 후자는 trailing 보존). 메모리 `tech_trailing_model_no_ssot` 의
  "전면 통일 금지, 게이트가 정답" 원칙 정합.
- 단일 페이지(-p N) 렌더와 전체 렌더의 HeightCursor 상태가 달라 분기가 갈릴 수 있으므로,
  검증은 전체 렌더 기준으로도 확인해야 한다(조사 중 확인).
