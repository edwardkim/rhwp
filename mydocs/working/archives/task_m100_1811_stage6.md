# Task #1811 Stage 6 — issue_1139 미주 회귀 재분리

## 정정 메모

이 문서는 `pr1875-review-latest` 브랜치에서 draft PR #1875 계열 미주 선행 커밋이 함께 섞인 상태로
`issue_1139_inline_picture_duplicate` 회귀를 분석하고 수정한 기록이다. 해당 회귀 수정 코드 자체는
#1811 HWPX RowBreak clean PR 에 포함하지 않는다.

`upstream/devel` 기준 clean 브랜치에 #1811 RowBreak/saved bounds 커밋만 적용한 뒤
`env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1139_inline_picture_duplicate -- --nocapture`
를 재실행한 결과 85개 테스트가 모두 통과했다. 따라서 이 Stage 6 분석은 버리지 않고 보존하되, #1811 PR 의
필수 코드 변경 범위에서는 제외한다.

## 배경

`upstream/devel` 동기화와 rebase 후
`env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1139_inline_picture_duplicate -- --nocapture`
를 재실행했을 때 85개 중 31개가 실패했다.

Stage 5 문서에는 Stage 3~5 RowBreak/lineSeg 변경만 분리해 기존 실패로 보았지만, 현재 브랜치 전체
변경 기준으로는 endnote 선행 커밋까지 포함한 회귀이므로 다시 원인을 좁힌다.

## 목표

1. `issue_1139_inline_picture_duplicate` 회귀 원인을 브랜치 변경 범위 안에서 확인한다.
2. 특정 샘플명, 페이지 번호, 테스트명 하드코딩 없이 문서 속성과 미주 구조에 근거해 수정한다.
3. 전체 `issue_1139_inline_picture_duplicate` 테스트를 통과시킨다.

## 진행 메모

- 대표 실패: `issue_1284_2024_between20_page19_question24_continues_from_pdf_top`
- 실패 양상: page 18 오른쪽 단에 `FullParagraph[미주] pi=937` 문24 제목이 들어오면 안 되는데
  들어와서 실패한다.
- `dump_page_items` 단계에서 이미 실패하므로 SVG layout 후처리보다는 typeset/pagination 단계의
  미주 본문 줄 나눔 또는 높이 산정 회귀로 본다.

## 원인

endnote 선행 커밋 중 `prepend_endnote_marker_text` 가 `AutoNumber` 제어의 raw text position 에
미주 번호를 삽입하도록 바뀌면서, 실제 placeholder 공백이 없는 실물 HWP 미주 문단까지 위치 삽입
분기를 탔다.

대표 문서 속성:

- source: `samples/3-09월_교육_통합_2024-미주사이20.hwp` `s0:p288:ci0:note0`
- 원본 미주 문단: `text="  ②"`, `char_offsets=[0,8,9]`, `char_count=11`
- `AutoNumber` control position: `pos=3`

이 구조는 `pos` 또는 `pos-1` 에 치환할 placeholder 공백이 없고, 이미 보이는 문제 표지 `②` 뒤에
`AutoNumber` 제어가 붙어 있다. 이때 번호를 `pos=3` 에 삽입하면 저장 LINE_SEG 폭과 다른 문자열이
되어 미주 제목/본문이 과도하게 앞쪽 페이지에 들어갔다.

추가로 SO-SUEOP HWP3 미주에는 trailing padding 공백 뒤에 `AutoNumber` 제어가 붙은 구조가 있다.
단순히 `pos-1` 이 공백인지만 보면 이 trailing padding 을 placeholder 로 오인해 marker 가 문두가
아니라 문장 끝에 붙는다.

반대로 편집기로 새로 만드는 주석 문단은 `text="  "`, `char_offsets=[0,8]` 구조로 실제
placeholder 공백이 있으므로 위치 치환을 유지할 수 있다.

## 수정

- `AutoNumber` 위치 치환은 `pos` 또는 `pos-1` 에 실제 placeholder 공백과 `char_offsets` 8-unit
  control slot gap 이 함께 확인되는 경우에만 적용한다.
- placeholder 가 없으면 기존 문두 prepend fallback 으로 되돌린다.
- 기준은 샘플명/페이지 번호가 아니라 문단의 `text`, `char_offsets`, `AutoNumber` control position
  조합이다.

## 검증

아래 검증은 `pr1875-review-latest` 브랜치에서 draft PR #1875 계열 미주 선행 커밋이 섞인 상태의
회귀 수정 코드 기준으로 완료한 기록이다. #1811 clean PR 에서는 해당 코드 변경을 제외하고,
`issue_1139_inline_picture_duplicate`, `issue_1749_saved_bounds_page_break`, `issue_1035_alignment`,
`cargo clippy --all-targets`, `git diff --check` 를 별도로 재확인했다.

- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1139_inline_picture_duplicate -- --nocapture`:
  통과, 85 passed
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1692 issue_1692_so_sueop_answer_endnote_pages_match_pdf_ranges -- --nocapture`:
  통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1749_saved_bounds_page_break -- --nocapture`:
  통과, 3 passed
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1035_alignment -- --nocapture`:
  통과, 4 passed
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`:
  통과
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`:
  통과
- `git diff --check`:
  통과
