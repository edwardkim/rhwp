# PR #1590 처리 보고서 — HWPX 캡션 내 플로팅 이미지 렌더링 (#1270 b 범위)

- PR: https://github.com/edwardkim/rhwp/pull/1590
- 제목: `Task #1585: HWPX 캡션 내 플로팅 이미지 렌더링 지원`
- 작성자: postmelee (collaborator)
- 연결: #1585 (bug/hwpx/rendering) — #1270 의 (b) 범위, #1551(a 인라인) 후속
- base ← head: `devel` ← `postmelee:local/task1585`
- 처리일: 2026-06-27

## 1. 처리 결정

**admin merge.** 캡션 paragraph 내부 `TopAndBottom` picture 를 caption 영역 기준으로 렌더링한다.
#1270 진단의 (b) 플로팅 이미지 범위(이전에 `layout_caption` 이 para/bin_data_content 를 None
고정해 캡션 이미지가 미렌더되던 부분)를 해소. CI 전부 pass + 로컬 전체 회귀 통과 + 충돌 0건 +
PR 본문 시각검증 완료 명시.

## 2. 변경 범위

| 파일 | 내용 |
|---|---|
| `src/renderer/layout/picture_footnote.rs` | `layout_caption_topbottom_pictures` 헬퍼 — caption TopAndBottom picture 를 caption_area 기준 배치, `bin_data_content` 스레딩 |
| `src/renderer/layout/table_layout.rs` | `should_render_table_caption`(depth 가드), caption flow extra 계산 |
| `tests/issue_1585_caption_floating_image.rs` | top-level / nested table caption 회귀 (2 단언) |
| 문서 7건 | #1585 계획/보고서/오늘할일 |

## 3. 코드 검토

- caption 의 `TextWrap::TopAndBottom` picture 만 caption 영역 기준으로 배치하고 `layout_picture`
  경로 재사용(non-TAC 정규화).
- **중복 방출 방지**: `get_inline_shape_position(...).is_some()` 이면 continue — #1551 인라인 경로에서
  이미 등록된 control 은 skip.
- **depth 가드**: `should_render_table_caption` = `depth == 0 || (depth == 1 && caption 에
  TopAndBottom picture 있음)`. nested table caption 은 명시 조건에서만 렌더. #1270 진단의
  "depth 1 한정" 방향과 정합. caption 센티널(`cell_index=65534`) 유지.

## 4. 검증

### 자동 (로컬)

| 항목 | 결과 |
|---|---|
| GitHub CI (Build&Test/CodeQL/Analyze/Canvas visual diff) | 전부 pass |
| 충돌 시뮬레이션 | 0건 |
| 신규 `issue_1585` (2) | **red→green 확정** (수정 revert 시 2 FAILED, 적용 시 pass) |
| 회귀 `issue_1270`/`issue_1459`/`issue_1352` | 통과 |
| **golden SVG 불변** (svg_snapshot) | 8/8 |
| 전체 `cargo test --tests` | **FAILED 0건** (lib 1959 passed) |
| fmt / clippy | clean |

> 신규 테스트는 `hy-001.hwpx` 그림 문단을 가져와 합성으로 caption 에 TopAndBottom picture 를
> 붙인 뒤(top-level + nested table) image 노드 방출을 단언한다. 수정 없이는 미방출(FAILED).

### 시각 (게이트)

- PR 본문에 작업지시자 rhwp-studio 직접 로드 시각 검증 완료 명시 (#1270 첨부 샘플,
  image2/SEOUL MY SOUL 로고 복구).
- golden SVG 불변으로 기존 렌더 무영향.

## 5. 맥락 — #1270 시리즈 완결

#1270 진단에서 안내한 (a 인라인)=#1551, (b 플로팅)=#1590(본 PR). 두 범위가 이어져 캡션 이미지
렌더링 문제가 마무리된다.

## 6. 후속

- #1585: Closes 키워드 없어 수동 close 검토.

## 7. 산출물

- 본 처리 보고서: `mydocs/pr/archives/pr_1590_review.md`
