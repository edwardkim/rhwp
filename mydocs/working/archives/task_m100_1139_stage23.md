# Task M100 #1139 Stage 23

## 목적

Stage22 커밋 이후 `3-09월_교육_통합_2022.hwp` 17쪽이 한컴 정답지와 다르게 배치되는 문제를 분석하고 수정한다.

## 시작 기준

- Stage22 커밋: `4b7f75ac` (`task 1139: Stage22 수식 속성 경로와 미주 VPOS 보정`)
- Stage22에서 수식 개체 속성 경로를 `수식 속성` 대화상자로 분리했다.
- Stage22에서 9쪽 정답표/미주 풀이 오버랩과 18쪽 하단 overflow를 보정했다.
- Stage22 검증:
  - `cargo test invalid_lazy_base -- --nocapture`
  - `cargo test compact_endnote -- --nocapture`
  - `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - `cargo build`
  - `target/debug/rhwp export-svg` 9쪽/18쪽
  - `wasm-pack build --target web --out-dir pkg`
  - `npm run build`

## 새 문제

- 작업지시자 확인 결과 17쪽이 한컴 정답지와 다르다.
- 전달된 비교 화면 기준:
  - rhwp 출력은 17쪽 좌우 단의 미주 문항 배치/진행 위치가 한컴 기준과 다르다.
  - 한컴 기준 화면에는 조판부호가 보이며, 같은 17쪽에서 문27/문28/문29/문30의 이어짐과 단 하단 배치가 rhwp 산출물과 다르다.
- Stage23에서는 17쪽을 우선 대상으로 삼고, 16→17→18쪽 사이 compact 미주 단 나눔과 VPOS 기준 전환을 다시 비교한다.

## 진행 계획

1. Stage22 커밋 기준으로 17쪽 SVG/PNG와 debug overlay를 새로 산출한다.
2. 한컴 기준 화면과 rhwp 17쪽의 문항 시작 위치를 문단 인덱스 단위로 대조한다.
3. `dump-pages -p 16`, `dump-pages -p 17`과 `RHWP_VPOS_DEBUG=1 export-svg -p 16/17` 로그를 비교한다.
4. page 17 차이가 pagination 단계인지, renderer VPOS 보정 단계인지, 또는 debug/control-code 표시 차이인지 분리한다.
5. 원인과 최소 수정 범위를 정리한 뒤 작업지시자 승인 후 소스 수정한다.
6. 검증은 #1139 회귀 테스트, 17쪽/18쪽 SVG/PNG 산출물, `cargo build`, WASM/studio 빌드 순서로 진행한다.

## 승인 상태

- 2026-05-29 작업지시자 승인 후 Stage23 소스 수정 및 자동 검증 완료.
- 커밋은 작업지시자 시각 검증/승인 후 진행한다.

## 진행 기록

- 2026-05-29 작업지시자 지시:
  - Stage22 현재 상태를 커밋했다.
  - 17쪽이 한컴 정답지와 다르므로 Stage23으로 전환한다.
- Stage23 초기 산출물:
  - SVG: `output/task1139_stage23_page17_svg/3-09월_교육_통합_2022_017.svg`
  - debug SVG: `output/task1139_stage23_page17_debug_svg/3-09월_교육_통합_2022_017.svg`
  - PNG: `output/task1139_stage23_page17_png/3-09월_교육_통합_2022_017.png`
  - VPOS debug SVG: `output/task1139_stage23_page17_vpos_debug_svg/3-09월_교육_통합_2022_017.svg`
- 초기 관찰:
  - rhwp 17쪽은 좌측 단 `문27)`, `문28)`, 우측 단 `문29)`까지 배치하고 끝난다.
  - 작업지시자가 제공한 한컴 기준 화면은 우측 단 하단에 `문30)` 시작이 이어진다.
  - 따라서 17쪽 차이는 단순 draw overflow가 아니라 17→18쪽 미주 문단 분배가 한컴보다 이르게 끊기는 문제로 본다.
  - `dump-pages -p 16` 기준 우측 단은 `pi=900..927`까지이며 `used=901.3px`, body height는 `1001.6px`이다.
  - 다음 문단 `pi=928`은 18쪽으로 넘어가며, 17쪽 우측 단에 남은 시각 공간과 pagination fit 판정이 왜 다르게 계산되는지 확인해야 한다.
- 2026-05-29 Codex 재분석:
  - 이슈 #1139는 OPEN 상태이며 assignee는 권한 제한으로 미지정 상태를 유지한다.
  - 열린 PR은 #1170(`public/repo-hygiene-devel`)과 #1159(`render-p20`)를 확인했다.
  - `dump-pages -p 16`에서 17쪽 우측 단은 `pi=900..927`, `used=901.3px`로 종료한다.
  - `RHWP_VPOS_DEBUG=1 export-svg -p 16`에서 17쪽 우측 단 렌더는 `pi=927`까지 정상 보정되며 overflow는 없다.
  - `pi=928`은 문30의 첫 문단이고, 한컴 기준에서는 17쪽 우측 단 하단에 시작해야 한다.
  - `src/renderer/typeset.rs`의 compact 미주 가드가 `ep_idx == 0`, `emitted_endnote_count > 0`, `current_height > available * 0.88` 조건에서 새 미주 첫 문단을 fit 판정과 별개로 다음 단/쪽으로 넘긴다.
  - 현재 17쪽 우측 단은 `901.3 / 1001.6 = 90.0%` 사용 상태라 이 88% 가드에 걸린다.
  - 따라서 Stage23 원인은 renderer draw/VPOS 보정보다 pagination 단계의 새 미주 첫 문단 강제 advance 가드로 보는 것이 가장 유력하다.

## 수정 방향 후보

1. compact 미주 새 미주 첫 문단의 `available * 0.88` 강제 advance 가드를 제거하거나 더 좁게 제한한다.
2. 앞선 일반 fit 판정(`current_height + en_fit > available`)과 partial split 판정이 이미 있으므로, 새 미주가 실제로 들어갈 수 있으면 현재 단에 남긴다.
3. Stage22에서 막았던 18쪽 하단 overflow와 9쪽 정답표 오버랩은 회귀 위험이 있으므로, #1139 회귀 테스트에 "17쪽에 문30 시작이 포함됨"을 추가하고 9쪽/18쪽 산출물도 함께 재확인한다.

## 승인 기록

- 위 원인 판단과 수정 방향으로 Stage23 소스 수정(`src/renderer/typeset.rs`의 compact 미주 새 문항 강제 advance 가드 조정) 및 회귀 테스트 추가 승인을 받았다.

## Stage23 수정 결과

- `src/renderer/typeset.rs`:
  - compact 미주에서 이전 미주 묶음에 VPOS 되감김이 있을 때의 선행 단/쪽 넘김 기준을 단 위치별로 분리했다.
    - 다음 단이 남아 있는 경우: 기존 85% 기준 유지.
    - 마지막 단인 경우: 95% 기준으로 늦춰 17쪽 우측 단 하단에 새 문항 시작을 허용.
  - 새 미주 첫 문단 강제 advance 기준도 단 위치별로 분리했다.
    - 다음 단이 남아 있는 경우: 기존 88% 기준 유지.
    - 마지막 단인 경우: 95% 기준.
  - 마지막 단에서 88~95% 구간에 새 미주 첫 문단이 들어오면 첫 문단만 현재 단에 남기고 즉시 다음 쪽/단으로 넘기도록 했다.
    - 17쪽 우측 단에는 `pi=928`(`문30)`) 시작만 남긴다.
    - 18쪽은 `pi=929`부터 시작해 Stage22에서 막았던 하단 overflow 재발을 피한다.
- `tests/issue_1139_inline_picture_duplicate.rs`:
  - `issue_1139_page17_endnote_question30_starts_on_right_column` 테스트를 추가했다.
  - 17쪽에 `pi=928`이 있고, 18쪽에는 `pi=928`이 없어야 함을 고정했다.

## Stage23 검증 결과

- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - 11개 테스트 통과.
  - 기존 12쪽 `LAYOUT_OVERFLOW_DRAW` 0.9px 로그는 유지.
- `cargo build`
  - 통과.
- `target/debug/rhwp dump-pages samples/3-09월_교육_통합_2022.hwp -p 16`
  - 17쪽 우측 단에 `pi=928` `문30)   260` 포함.
  - 17쪽 우측 단 `used=919.4px`, body height `1001.6px`.
- `target/debug/rhwp dump-pages samples/3-09월_교육_통합_2022.hwp -p 17`
  - 18쪽은 `pi=929`부터 시작.
- `target/debug/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -p 16 -o output/task1139_stage23_fixed_page17_svg`
  - `LAYOUT_OVERFLOW` 없음.
- `target/debug/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -p 17 -o output/task1139_stage23_fixed_page18_svg`
  - `LAYOUT_OVERFLOW` 없음.
- `cargo test invalid_lazy_base -- --nocapture`
  - 통과.
- `cargo test compact_endnote -- --nocapture`
  - 통과.
- `wasm-pack build --target web --out-dir pkg`
  - 통과.
- 루트 `npm run build`
  - 루트 package script가 없어 `Missing script: "build"`로 실패. 실제 studio 빌드는 아래에서 수행.
- `(cd rhwp-studio && npm run build)`
  - 통과.

## Stage23 산출물

- `output/task1139_stage23_fixed_page17_svg/3-09월_교육_통합_2022_017.svg`
- `output/task1139_stage23_fixed_page18_svg/3-09월_교육_통합_2022_018.svg`

## 남은 확인

- 자동 검증 기준으로는 17쪽 `문30)` 시작 위치와 18쪽 overflow 회귀가 해소됐다.
- 2026-05-29 작업지시자 시각 검증 결과:
  - 17쪽은 아직 한컴오피스 정답지와 다르다.
  - Stage23 출력은 17쪽 우측 단 하단에 `문30) 260` 시작만 남긴 상태다.
  - 작업지시자 제공 한컴 화면은 같은 17쪽 우측 단 하단에서 `문30) 260` 뒤의 풀이 본문 일부까지 이어진다.
  - 따라서 Stage23 보정은 “문30 시작 이월”은 줄였지만, 17→18쪽 문30 내부 분할 위치가 아직 한컴보다 이르다.
- 현재 Stage23 상태는 커밋하고, Stage24에서 문30 내부 분할 위치를 다시 분석한다.
