# Task #1270 최종 보고서 — 캡션 내 인라인 이미지 렌더링 스레딩

## 개요

- 이슈: #1270 — HWPX 온라인 데모에서 1페이지 상단 이미지가 표시되지 않는 문제
- 브랜치: `local/task1270`
- 기준: `upstream/devel` `48e96704`
- 범위: 캡션 문단 안의 `treat_as_char` 인라인 이미지 렌더링 복구, depth 1 한정

## 메인테이너 지시 반영

메인테이너는 (a) 인라인 캡션 이미지와 (b) 플로팅 캡션 이미지를 분리하고, 이번 PR은 (a)만 진행하도록 지시했다.

- (a) 인라인 캡션 이미지: 이번 작업 범위
- (b) 플로팅 캡션 이미지: 후속 이슈 범위
- 캡션 속 그림의 캡션 재귀 렌더링: 이번 범위 제외
- 이슈 #1270 close: 이번 PR merge 후에도 보류

## 구현 내용

`layout_caption()`이 캡션 문단을 `layout_composed_paragraph()`에 넘길 때 기존에는 인라인 이미지 렌더링에 필요한 원본 `para`와 `bin_data_content`를 `None`으로 전달했다.

이번 작업에서는 다음을 반영했다.

- `src/renderer/layout/picture_footnote.rs`
  - `layout_caption()` 시그니처에 `bin_data_content: &[BinDataContent]` 추가
  - 캡션 문단 루프의 실제 `para`와 `bin_data_content`를 `layout_composed_paragraph()`에 전달
- `src/renderer/layout.rs`
- `src/renderer/layout/shape_layout.rs`
- `src/renderer/layout/table_layout.rs`
- `src/renderer/layout/table_partial.rs`
  - `layout_caption()` 호출부에 `bin_data_content` 전달
- `src/renderer/layout/paragraph_layout.rs`
  - 텍스트 없이 TAC picture만 있는 표 캡션 문단을 위해 빈 줄 TAC picture 렌더링 조건에 `cell_index == 65534` 캡션 센티널 예외 추가
  - 실제 표 셀 내부 중복 렌더링 방지 조건은 유지

기존 좌표 계산, 캡션 높이 계산, caption y advance, 표/그림 본문 배치 정책은 변경하지 않았다.

## 테스트

신규 회귀 테스트를 추가했다.

- `tests/issue_1270_caption_inline_image.rs`
  - 기존 fixture에서 실제 파싱된 TAC picture 문단을 찾아 첫 top-level 표의 TOP caption으로 복제
  - `build_page_render_tree(0)` 후 caption sentinel context의 `ImageNode`가 정확히 1개 방출되는지 검증
  - 이미지 payload 존재 여부도 함께 확인
  - 텍스트가 함께 있는 캡션 문단과 picture-only 캡션 문단을 모두 검증

## 검증 결과

다음 검증을 통과했다.

| 검증 | 결과 |
|------|------|
| `cargo fmt --check` | 통과 |
| `cargo test --test issue_1270_caption_inline_image` | 통과 — 2 passed |
| `cargo test --test issue_1139_inline_picture_duplicate` | 통과 — 85 passed |
| `cargo test --test issue_1352_table_cell_tac_picture_text` | 통과 — 1 passed |
| `cargo test --test issue_1459_topbottom_picture_reflow` | 통과 — 3 passed |
| `cargo test --lib` | 통과 — 1959 passed, 6 ignored |
| `cargo clippy --lib -- -D warnings` | 통과 |
| `cargo test --test issue_530` | 통과 — 1 passed |
| `cargo test --test issue_1486_hwpx_partial_tac_table` | 통과 — 6 passed |

## 첨부 샘플 시각검증 해석

작업지시자가 `rhwp-studio`에서 첨부 HWPX를 직접 로드해 확인한 결과, 이슈 기대 이미지의 오른쪽 상단 `SEOUL MY SOUL` 로고는 여전히 표시되지 않았다.

이는 이번 구현 실패가 아니라 범위 차이에 따른 기대 결과다.

- 첨부 샘플의 누락 대상 `image2`는 캡션 내부 그림이지만 `textWrap="TOP_AND_BOTTOM"`인 플로팅 배치다.
- 이번 작업은 `treat_as_char` 인라인 캡션 이미지만 복구한다.
- 플로팅 캡션 이미지는 caption 영역에 앵커, 줄 매핑, wrap 배치 로직을 일반화해야 하므로 후속 작업으로 남긴다.

PR description에는 이 내용을 명시해야 한다.

## PR 설명 필수 문구

PR 본문에는 다음 범위 제한을 포함한다.

```md
Scope:
- Fixes caption inline images where the picture is `treat_as_char`.
- `layout_caption` now threads the caption paragraph and `bin_data_content` into paragraph layout.
- This is intentionally depth 1 only.

Out of scope:
- The attached issue sample's `image2` / `SEOUL MY SOUL` logo is a floating caption image with `textWrap="TOP_AND_BOTTOM"`.
- It is expected to still not render in this PR.
- Floating caption image layout needs a follow-up because it requires generalizing anchor/line mapping/wrap logic into caption layout.
- Therefore #1270 should remain open after this PR.
```

## 잔여 작업

- PR 생성
- PR description에 depth 1 인라인 한정과 플로팅 후속 범위 명시
- #1270은 PR merge 후에도 close하지 않음
- 플로팅 캡션 이미지 일반화는 후속 이슈 또는 후속 작업으로 분리

## 결론

메인테이너가 지시한 (a) 인라인 스레딩 범위는 완료했다. 신규 회귀 테스트는 텍스트 포함 캡션과 picture-only 캡션을 모두 검증하며, 관련 회귀 검증도 통과했다. 첨부 샘플의 완전 시각 해소는 (b) 플로팅 캡션 이미지 후속 범위로 남긴다.
