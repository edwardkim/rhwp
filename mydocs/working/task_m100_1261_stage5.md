# Task #1261 Stage5 보고 - PR CI 회귀 보정

## 배경

PR #1264 생성 후 GitHub Actions `Build & Test`에서 기존 회귀 테스트 `tests/issue_1116.rs`의 두 건이 실패했다.

- `sample16_hwp5_page3_heading_positions_follow_lineseg_vpos`
- `sample16_hwp5_2022_page3_bcp_tail_glyph_stays_on_hancom_line`

두 실패 모두 `samples/hwp3-sample16-hwp5*.hwp` 3쪽 본문이 한컴 기준보다 약 `10.4px` 아래로 밀리는 현상이었다.

## 원인

Stage1에서 문28 조건 박스 겹침을 막기 위해 공백뿐인 TAC Shape host 줄도 Shape 실제 높이를 유지하도록 바꿨다.
이 변경 자체는 문28의 `글앞으로` TAC Shape에 필요하지만, `hwp3-sample16-hwp5`의 `자리차지` TAC Shape 빈 host 줄에는 기존처럼 trailing line spacing을 렌더 진행량으로 다시 더하면 안 된다.

해당 `10.4px`는 `pi=71` TAC Shape host 줄의 `LINE_SEG.line_spacing` 값과 일치했다.

## 수정

- `src/renderer/layout/paragraph_layout.rs`
  - 현재 줄 TAC 목록에 `TextWrap::TopAndBottom`인 TAC Shape가 있는지 확인하는 helper를 추가했다.
  - 공백뿐인 `자리차지` TAC Shape host 줄은 `LINE_SEG.line_height`가 이미 개체 점유 높이를 포함한다고 보고 trailing line spacing을 추가하지 않도록 했다.
  - `글앞으로` TAC Shape인 문28 조건 박스는 이 분기에서 제외해 Stage1 보정을 유지했다.

## 검증

- `cargo test --test issue_1116 -- --nocapture` 통과: 13개 테스트.
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` 통과: 46개 테스트.
- `git diff --check` 통과.

## PR 상태

- PR: `https://github.com/edwardkim/rhwp/pull/1264`
- Stage5 커밋 후 PR 브랜치 `task_m100_1261`에 푸시해 CI를 다시 확인한다.
