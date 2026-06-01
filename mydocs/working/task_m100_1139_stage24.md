# Task M100 #1139 Stage 24

## 목적

Stage23 커밋 이후에도 `3-09월_교육_통합_2022.hwp` 17쪽이 한컴오피스 정답지와 다르게 보이는 문제를 이어서 분석한다.

## 시작 기준

- Stage23은 17쪽 우측 단에 `pi=928`(`문30) 260`) 시작이 들어가도록 compact 미주 advance 기준을 보정했다.
- Stage23 자동 검증:
  - `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - `cargo build`
  - 17/18쪽 `dump-pages`
  - 17/18쪽 `export-svg`
  - `cargo test invalid_lazy_base -- --nocapture`
  - `cargo test compact_endnote -- --nocapture`
  - `wasm-pack build --target web --out-dir pkg`
  - `rhwp-studio` `npm run build`
- Stage23 산출물:
  - `output/task1139_stage23_fixed_page17_svg/3-09월_교육_통합_2022_017.svg`
  - `output/task1139_stage23_fixed_page18_svg/3-09월_교육_통합_2022_018.svg`

## 새 문제

- 작업지시자 시각 검증 결과 17쪽은 아직 한컴오피스 정답지와 다르다.
- Stage23 출력은 17쪽 우측 단 하단에 `문30) 260` 시작만 남긴다.
- 작업지시자 제공 한컴 화면은 17쪽 우측 단 하단에서 `문30) 260` 뒤 풀이 본문 일부까지 이어진다.
- 즉 Stage23은 `문30)` 자체의 18쪽 이월은 줄였지만, `문30)` 내부 문단의 17→18쪽 분할 위치가 한컴보다 이르다.

## 진행 계획

1. Stage23 커밋 기준 17/18쪽 `dump-pages`와 SVG를 기준선으로 둔다.
2. 한컴 화면에서 17쪽 우측 단 하단에 남는 `문30)` 본문 줄 수를 문단/line index 단위로 대조한다.
3. `pi=928`, `pi=929`, `pi=930`, `pi=931`의 formatter line heights, LINE_SEG vpos, split 판정을 비교한다.
4. Stage23의 `advance_after_new_endnote_anchor`가 지나치게 강하게 끊는지, 또는 문30 내부 partial split 계산이 한컴보다 보수적인지 분리한다.
5. 원인이 확정되면 작업지시자 승인 후 최소 범위로 소스 수정한다.
6. 검증은 #1139 회귀 테스트, 17/18쪽 `dump-pages`, 17/18쪽 SVG export overflow 확인, `cargo build`, WASM/studio 빌드 순서로 진행한다.

## 승인 상태

- 2026-05-29 작업지시자 승인 완료.
- Stage24 소스 수정과 회귀 테스트 보강을 진행했다.

## 진행 기록

- 2026-05-29 작업지시자 지시:
  - 17쪽은 아직 한컴과 다르다.
  - 현재 상황을 커밋하고 Stage24로 전환한다.
- 2026-05-29 Codex 분석:
  - 현재 브랜치: `local/task_m100_1139`
  - 현재 HEAD: `04d873f9` (`task 1139: Stage23 17쪽 미주 분배 보정`)
  - 현재 실행 중인 장기 작업은 없고, Vite dev 서버만 `7700`에서 실행 중이다.
  - 워킹트리는 Stage24 문서만 새 파일로 남아 있다.
- Stage24 기준선 산출:
  - `target/debug/rhwp dump-pages samples/3-09월_교육_통합_2022.hwp -p 16`
    - 17쪽 우측 단은 `pi=900..928`까지 배치된다.
    - 마지막 항목은 `pi=928` `문30)   260`이다.
    - 우측 단 `used=919.4px`, body height `1001.6px`, 잔여 높이는 약 `82.2px`이다.
  - `target/debug/rhwp dump-pages samples/3-09월_교육_통합_2022.hwp -p 17`
    - 18쪽 좌측 단은 `pi=929`부터 시작한다.
    - 즉 현재 Stage23 상태는 문30 첫 문단만 17쪽에 남기고, 문30 풀이 본문은 전부 18쪽으로 넘긴다.
  - SVG:
    - `output/task1139_stage24_baseline_page17_svg/3-09월_교육_통합_2022_017.svg`
    - `output/task1139_stage24_baseline_page18_svg/3-09월_교육_통합_2022_018.svg`
- Stage22 비교:
  - 비교용 worktree `/private/tmp/rhwp_stage22_head_check`를 `4b7f75ac` 기준으로 생성해 확인했다.
  - Stage22의 17쪽 우측 단은 `pi=927`에서 끝나며 `pi=928`이 17쪽에 들어오지 않는다.
  - Stage23은 마지막 단의 새 미주 advance 기준을 95%로 늦춰 `pi=928`을 17쪽에 넣었지만, 같은 변경에서 추가한 `advance_after_new_endnote_anchor`가 `pi=928` 배치 직후 즉시 다음 쪽으로 넘긴다.
- 원인 판단:
  - 현재 차이는 `pi=929` 자체가 fit 실패해서 넘어간 것이 아니라, Stage23의 anchor-only 가드가 문30 첫 문단 뒤에서 흐름을 강제로 끊는 쪽이 가장 유력하다.
  - 17쪽 우측 단은 `pi=928` 배치 후에도 약 `82.2px`가 남는다.
  - `pi=929`, `pi=930`은 각각 한 줄 문단이고, `pi=931`은 여러 줄 문단이라 기존 `split_endnote_to_fit`이 개입할 수 있는 후보이다.
  - 따라서 한컴 기준처럼 `문30) 260` 뒤 풀이 본문 일부를 17쪽에 남기려면, 새 미주 첫 문단 뒤의 무조건 advance를 제거하거나 더 좁은 조건으로 제한해야 한다.

## Stage24 수정 방향 후보

1. `src/renderer/typeset.rs`의 `advance_after_new_endnote_anchor`를 제거하거나, 내부 VPOS 되감김/그림/수식 등 실제 overflow 위험이 확인되는 경우로만 좁힌다.
2. 새 미주 첫 문단을 넣은 뒤에는 이미 존재하는 `en_fit` 및 `split_endnote_to_fit` 판정에 후속 문단 분배를 맡긴다.
3. Stage22에서 막은 18쪽 하단 overflow 회귀를 피하기 위해 17/18쪽 `dump-pages`, 17/18쪽 SVG overflow, 9쪽 미주/정답표 회귀를 같이 재검증한다.
4. 수정 후 실제 분할 경계를 먼저 확인한 뒤, #1139 회귀 테스트에 17쪽 문30 풀이 본문이 함께 남는 조건을 구체적인 `pi`/`PartialParagraph` 기준으로 고정한다.

## 승인 이력

- 2026-05-29 작업지시자가 위 수정 방향 진행을 승인했다.

## 구현 기록

- `src/renderer/typeset.rs`
  - compact 미주 분배에서 기본 미주 사이 간격(`7mm` 계열)인 `문30`은 Stage23의 `advance_after_new_endnote_anchor` 강제 넘김을 적용하지 않도록 좁혔다.
  - `advance_after_new_endnote_anchor`는 20mm 미주 사이 기준 문서처럼 anchor-only 강제 넘김이 필요한 경우를 유지하고, `3-09월_교육_통합_2022.hwp`의 기본 간격 문30은 다음 문단 분배를 기존 `en_fit`/`split_endnote_to_fit` 판정에 맡기도록 했다.
  - 단 advance 이후 이전 단 하단 기준으로 계산된 `internal_rewind_split`이 다음 단 첫 문단에 잘못 재사용되는 경우를 막기 위해, 문30 기본 간격 경로에서는 advance 직후 stale split 후보를 비운다.
- `tests/issue_1139_inline_picture_duplicate.rs`
  - 17쪽 우측 단에 `pi=928`, `pi=929`, `pi=930`이 함께 남는 조건을 고정했다.
  - 18쪽은 `pi=931`의 `FullParagraph`로 시작하고 `PartialParagraph pi=931`이 생기지 않는 조건을 추가해 stale internal split 회귀를 방지한다.

## 검증 결과

- `cargo fmt --check`
  - 통과.
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - 통과: 11 passed.
  - 기존 로그인 `LAYOUT_OVERFLOW_DRAW: section=0 pi=673 ... overflow=0.9px`는 동일하게 남는다.
- `cargo build`
  - 통과.
- `target/debug/rhwp dump-pages samples/3-09월_교육_통합_2022.hwp -p 16`
  - 17쪽 우측 단이 `pi=928` `문30)   260` 뒤에 `pi=929`, `pi=930`까지 포함한다.
  - 우측 단 `used=955.4px`.
- `target/debug/rhwp dump-pages samples/3-09월_교육_통합_2022.hwp -p 17`
  - 18쪽 좌측 단은 `FullParagraph[미주] pi=931`로 시작한다.
  - `pi=931`이 `PartialParagraph`로 잘못 시작하지 않는다.
- `target/debug/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -p 16 -o output/task1139_stage24_final_page17_svg`
  - 통과.
  - 산출물: `output/task1139_stage24_final_page17_svg/3-09월_교육_통합_2022_017.svg`
- `target/debug/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -p 17 -o output/task1139_stage24_final_page18_svg`
  - 통과.
  - 산출물: `output/task1139_stage24_final_page18_svg/3-09월_교육_통합_2022_018.svg`
- `cargo test invalid_lazy_base -- --nocapture`
  - 통과: `invalid_lazy_base_skips_backtrack_after_tall_object` 1 passed.
- `cargo test compact_endnote -- --nocapture`
  - 통과: compact 미주 관련 2 passed.
- `wasm-pack build --target web --out-dir pkg`
  - 통과.
  - `wasm-bindgen` prebuilt 미지원 경고 후 `cargo install` fallback으로 계속 진행되어 완료.
- `rhwp-studio` `npm run build`
  - 통과.
  - Vite의 `canvaskit-wasm` `fs`/`path` browser externalized 경고와 chunk size 경고는 기존 성격의 production build warning으로 남는다.

## 현재 상태

- 커밋은 아직 하지 않았다.
- Stage24는 작업지시자의 시각 검증을 기다린다.
