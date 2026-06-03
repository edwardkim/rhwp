# Task #1261 Stage4 계획 - 문12 미주 overflow 분석

## 배경

Stage3에서 `3-09월_교육_통합_2024-미주사이20.hwp` 10쪽 `문8）` 미주 제목 위치를 `HeightCursor` 공통 compact 미주 제목 로직으로 정정했다.

이후 같은 10쪽 오른쪽 단 하단에서 `문12）` 이후 본문(`pi=568`, `pi=569`)이 단 하단을 넘는 `LAYOUT_OVERFLOW` 진단이 남아 있음을 확인했다.

## 목표

1. `문12）` overflow가 Stage3의 `미주 사이 20mm` 공통 간격 보존으로 생긴 후속 밀림인지, 기존 페이지네이터 분기/렌더 분기 불일치인지 확인한다.
2. 한컴 PDF 기준으로 `문12）`가 10쪽에 어디까지 들어가야 하는지 확인한다.
3. 문12 이후 overflow도 문단 번호별 특수 보정이 아니라 미주 페이지네이션/렌더 공통 로직으로 처리한다.

## 조사 절차

- `dump-pages`와 렌더 로그로 p10 오른쪽 단 `pi=567`~`pi=569`의 배치와 overflow 좌표를 확인한다.
- 한컴 PDF bbox에서 `문12）` 시작과 후속 줄의 실제 위치를 뽑아 rhwp 좌표와 비교한다.
- `typeset.rs` 페이지 분배 결과와 `layout.rs` 렌더 결과가 같은 단 하단 정책을 쓰는지 확인한다.
- 필요하면 `HeightCursor`의 page-path/safe-backtrack, 혹은 미주 하단 overflow 판정에서 공통 조건을 보정한다.

## 검증 대기

- `cargo test --lib height_cursor -- --nocapture`
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
- `cargo fmt -- --check`
- `git diff --check`
- `미주사이20` p10 SVG/PNG 및 한컴 PDF bbox 비교

## 조사 결과

- p10 오른쪽 단의 `문12）` overflow는 `문12）` 자체 문제가 아니라 `문10）`부터 누적된 위치 밀림이었다.
- 한컴 PDF bbox 기준:
  - `문10）`: `302.118pt * 96/72 = 402.8px`
  - `문11）`: `463.878pt * 96/72 = 618.5px`
  - `문12）`: `746.718pt * 96/72 = 995.6px`
- 수정 전 rhwp 로그:
  - `pi=550` 문10 제목 렌더 y가 `478.5px`
  - `pi=557` 문11 제목 렌더 y가 `694.1px`
  - `pi=567` 문12 제목 렌더 y가 `1071.3px`
- 원인은 단일줄 빈 미주 separator가 이미 `미주 사이 20mm` trailing을 포함해 `y_offset`을 만든 뒤, page-path vpos가 제목을 소폭 아래로 밀고 `layout.rs`의 `prev_endnote_title_gap_px` 보존 클램프가 다시 `미주 사이`를 더하면서 오른쪽 단 전체가 20mm씩 누적 밀린 것이다.

## 수정

- `HeightCursor::vpos_adjust()`의 단일줄 injected between-notes 경계를 확장했다.
- 기존 #1256 음수 backtrack 복원은 유지한다.
- Stage4에서는 확장된 `미주 사이`(`endnote_between_notes_hu > 3000`)에서 page-path vpos가 제목을 소폭 아래로 미는 경우도 `y_offset`을 유지한다.
- 선택한 `y_offset`과 저장 vpos 기준의 차이는 활성 vpos base에 반영해 후속 미주 문단도 같은 기준을 따르게 했다.
- 기본 7mm 미주 케이스는 기존 의도 gap을 보존해야 하므로 positive forward 억제 대상에서 제외했다.

## 검증 결과

- `cargo test --lib height_cursor -- --nocapture` 통과: 34개 테스트.
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` 통과: 46개 테스트.
- `cargo fmt -- --check` 통과.
- `git diff --check` 통과.
- 수정 후 로그:
  - `pi=550 y_in=402.9`
  - `pi=557 y_in=618.6`
  - `pi=567 y_in=995.7`
  - `pi=569 y_out=1089.8`
- `미주사이20` p10 SVG/PNG 생성:
  - `output/task1261_stage4_page10/3-09월_교육_통합_2024-미주사이20_010.svg`
  - `output/task1261_stage4_page10_compare/rhwp_page10.png`
- Stage4 p10 export에서는 `LAYOUT_OVERFLOW` 진단이 발생하지 않았다.
