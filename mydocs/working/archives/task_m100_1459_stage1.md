# Task M100 #1459 Stage 1

- 작성일: 2026-06-22
- 모드: 기여자 모드. PR 전 오늘할일 문서는 생성하거나 갱신하지 않는다.
- 이슈: https://github.com/edwardkim/rhwp/issues/1459

## 목표

한 문단 안에 `글자처럼 취급` 그림과 `자리차지(TopAndBottom)` 그림이 함께 있을 때, 한컴처럼 자리차지 그림이 먼저 문단 흐름을 예약하고 TAC 그림은 그 아래 흐름 위치에 렌더되도록 한다.

## 확인 내용

- 샘플 `samples/투명도0-50-2nd그림글차처럼off.hwp/.hwpx`는 같은 문단에 두 그림을 포함한다.
- `ci=2`는 `treat_as_char=true`, 투명도 0 그림이다.
- `ci=3`은 `treat_as_char=false`, `TopAndBottom`, 투명도 50 그림이다.
- 기존 렌더 트리에서는 자리차지 그림만 렌더되고 TAC 그림은 누락됐다.
- 원인은 빈 문단에 비-TAC float 그림이 있으면 `FullParagraph`를 건너뛰는 최적화가 같은 문단의 TAC 그림까지 숨기는 것이었다.

## 수정

- 빈 float host 문단 최적화는 보이는 인라인 컨트롤이 없는 경우에만 적용하도록 좁혔다.
- 같은 문단의 `TopAndBottom` 예약 높이 계산 대상을 표에서 그림/도형까지 확장했다.
- TAC 그림 y 보정은 확장된 예약 높이 helper를 사용하도록 갱신했다.
- 샘플 HWP/HWPX 기반 통합 테스트를 추가했다.

## 검증

- `cargo test --profile release-test --test issue_1459_topbottom_picture_reflow -- --nocapture`
- `cargo test --profile release-test --lib topandbottom -- --nocapture`
- `cargo test --profile release-test --lib issue1452 -- --nocapture`
- `cargo test --profile release-test --test issue_1452_saved_caret -- --nocapture`
- `cargo test --profile release-test --test issue_1139_inline_picture_duplicate -- --nocapture`
