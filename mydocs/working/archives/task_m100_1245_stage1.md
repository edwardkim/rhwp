# Task M100 #1245 Stage1 — Square 그림 상대 LINE_SEG 보정

## 작업 범위

Stage0에서 확인한 원인 후보에 따라 `Square/어울림` 그림의 세로 위치 보정 방식을 수정했다.

대상 파일:

- `src/renderer/layout.rs`
- `tests/issue_1139_inline_picture_duplicate.rs`

## 원인

`src/renderer/layout.rs`의 `square_wrap_first_narrow_line_vpos_px`는 `Square/어울림` 그림이 문단 중간부터 본문을 감싸는 경우, 처음 좁아지는 `LINE_SEG.vertical_pos`를 그림 상단 위치로 사용한다.

기존 구현은 좁아지는 줄의 raw `vertical_pos`를 px로 변환해 `para_base_y`에 더했다.

`3-09월_교육_통합_2022.hwp` 7쪽 `pi=386` 같은 문단은 첫 줄 `vertical_pos`가 이미 `31648`처럼 누적 흐름값이다. 따라서 좁아지는 줄의 raw `vertical_pos=37804`를 그대로 더하면 `para_base_y + absolute_vpos`가 되어 그림이 페이지 하단 밖으로 밀렸다.

## 수정

`Square/어울림` 그림 상단 위치 보정에 raw `vertical_pos`가 아니라 문단 첫 줄 대비 상대 delta를 사용하도록 바꾸었다.

```text
delta = first_narrow_line.vertical_pos - first_line.vertical_pos
```

첫 줄 `vertical_pos=0`인 기존 page8 `문29)` 유형은 delta가 기존 raw 값과 같으므로 기존 동작을 유지한다.

## 회귀 가드

`tests/issue_1139_inline_picture_duplicate.rs`에 다음 테스트를 추가했다.

- `issue_1245_2022_page7_square_pictures_use_relative_line_vpos`

검증 내용:

- 7쪽 `문25)` 타원 그림 `pi=386 ci=11`이 `pi=386`의 첫 좁은 줄과 같은 y에 붙는지 확인
- 7쪽 `문28)` 포물선 그림 `pi=420 ci=9`도 같은 방식으로 확인
- `문25)` 그림이 페이지 하단 밖으로 밀리지 않는지 확인

## 검증

명령:

```bash
cargo test --test issue_1139_inline_picture_duplicate -- --nocapture
cargo fmt --all --check
```

결과:

- `issue_1139_inline_picture_duplicate`: 42개 통과
- `cargo fmt --all --check`: 통과

## 시각 확인 자료

수정 전:

- `output/task1245_stage0/3-09월_교육_통합_2022_007.png`

수정 후:

- `output/task1245_stage0_after/3-09월_교육_통합_2022_007.png`

수정 후 `pi=386 ci=11`, `pi=420 ci=9` 그림이 페이지 하단 밖으로 밀리지 않고 각 문항의 어울림 줄 옆에 배치되는 것을 확인했다.

## 남은 확인

작업지시자의 한컴오피스/PDF 기준 시각 확인을 기다린다.
