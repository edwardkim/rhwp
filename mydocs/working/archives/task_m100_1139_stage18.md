# Task M100 #1139 Stage 18

## 목적

Stage 17 커밋 이후 남은 미주 흐름 차이를 다시 좁힌다.

## 기준 상태

- `upstream/devel` rebase 완료
- 충돌 파일: `rhwp-studio/src/ui/picture-props-dialog.ts`
- 충돌 해결 방향: upstream의 표 셀 경로 기반 속성 처리와 task 1139의 `group` 속성 처리를 병합
- `cargo build` 통과
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture` 결과: 7개 통과, 1개 실패

## 남은 실패

`issue_1139_endnote_spacing_reference_files_match_hancom_page_counts`가 실패한다.

- 대상: `samples/3-09월_교육_통합_2024-미주사이20.hwp`
- 한컴 기준: 24쪽
- 현재 rhwp: 23쪽

## 작업지시자 피드백

- `3-09월_교육_통합_2022.hwp`는 전체 페이지 수가 한컴과 같이 23쪽까지 맞춰졌다.
- 12쪽 주변은 많이 개선되었지만 아직 완전 정답지는 아니다.
- 그림과 이어지는 수식 내용이 하나의 묶음처럼 이동하면 안 된다.
- 그림 이후 수식 내용은 줄 단위로 흐르다가 페이지/단 범위를 넘으면 다음 단 또는 다음 쪽으로 이동해야 한다.
- 출력 산출물을 만들면 SVG보다 PNG 경로를 우선 보고한다.

## 초기 분석 계획

1. rebase 후 충돌 잔여물이 없는지 다시 확인한다.
2. `target/debug/rhwp info ... -p`와 `dump-pages`를 함께 사용해 page 11~12의 문단/줄/그림 흐름을 비교한다.
3. `미주사이20`과 `구분선아래20` 샘플의 페이지 수 차이를 만드는 미주 모양 값을 다시 확인한다.
4. 그림 문단 이후 수식 줄이 묶음처럼 이동되는 원인이 pagination 단계인지 layout 단계인지 분리한다.

## 검증 대기

- `cargo fmt` 완료
- `cargo build` 완료
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`는 기존과 같이 `미주사이20` 페이지 수 차이 1건이 남아 있다.
- 필요 시 `wasm-pack build --target web --out-dir pkg`

## Stage 18 결과

- 글자처럼 취급 그림이 있는 문단에서 동일한 큰 높이의 예약 줄이 중복으로 진행 높이에 반영되는 경우를 좁게 보정했다.
- `3-09월_교육_통합_2022.hwp` 12쪽에서 그래프 아래 수식 흐름이 한컴 기준에 가깝게 개선되었다.
- 페이지 수는 한컴 기준과 동일하게 23쪽을 유지한다.
- 시각 검증 PNG: `/Users/tsjang/Cloud/Devel/rhwp/output/task1139_stage18_tac_duplicate_skip_png/3-09월_교육_통합_2022_012.png`

## 다음 스테이지로 이월

- 12쪽 하단과 13쪽 시작부가 한컴 정답지와 아직 약간 다르다.
- 작업지시자 판단에 따라 현재 상태를 커밋하고 Stage 19에서 잔여 차이를 이어서 좁힌다.
