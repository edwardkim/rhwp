# Stage 203: HWP3 layout control 저장 줄 권한

## 목적

Stage 202 이후에도 남은 HWP3 그림 앵커 회귀 세 건을 해결한다.

## 관찰

`issue_1139_inline_picture_duplicate`는 Stage 202 뒤 `82 passed, 3 failed`였다.

1. `issue_1209_2022_page8_question29_square_picture_starts_at_wrap_line`
2. `issue_1245_2022_page7_square_pictures_use_relative_line_vpos`
3. `issue_1293_2024_visible_separator_large_tac_picture_tail_starts_next_page`

첫 두 사례는 HWP3 본문 `Picture`가 처음 좁아지는 저장 줄에 붙어야 한다. 세 번째는
TAC 그림 문단의 저장 줄 흐름이 다음 쪽 시작을 결정한다. Stage 156의 fresh probe는
텍스트 폭만으로 줄 수를 재계산하므로, 그림·표·수식 등 layout control이 있는 문단에서
`stored > fresh`는 stale 증거가 아니다.

## 변경 계약

HWP3의 stored-vs-fresh 줄 수 보정은 control이 없거나 `Field`/`Hyperlink` 같은 인라인
텍스트 메타데이터만 가진 본문 문단에만 적용한다.

그림·도형·표·수식·각주처럼 줄 폭, 줄 높이, 앵커, 물리 흐름에 참여하는 control이 하나라도
있으면 저장 `LineSeg`가 정본이다. 기존 마스킹 문단의 stale 판정은 변경하지 않는다.

이 조건은 문단 텍스트, fixture, 페이지 번호, 좌표 여유값을 사용하지 않는다.

## 검증 명령

```sh
cargo test --profile release-test --test issue_1139_inline_picture_duplicate -- --nocapture
```

## 상태

구현 및 집중 회귀 검증 완료.

1. `issue_1139_inline_picture_duplicate`: `85 passed, 0 failed`
2. `issue_1035_alignment`, `issue_1105`, `issue_1086`, #3820 직접 회귀 네 개,
   `issue_3930_hwpx_hwp_save_layout`: 합계 `33 passed, 0 failed`

전체 라이브러리와 integration 회귀는 다음 검증 단계에서 실행한다.
