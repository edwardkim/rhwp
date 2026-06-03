# Task M100 #1189 Stage 7

## 목적

PR #1194 생성 이후 `3-10월_교육_통합_2022.hwp` 11쪽에서 미주 간격이 한컴오피스/PDF 기준과 다르게 보이는 문제를 다시 확인하고 보정한다.

## 시작 기준

- 이슈: [#1189](https://github.com/edwardkim/rhwp/issues/1189)
- PR: [#1194](https://github.com/edwardkim/rhwp/pull/1194)
- 작업 브랜치: `local/task_m100_1189`
- 선행 커밋: `334addcd` (`task 1189: 1쪽 인라인 수식 간격 보정`)
- 대상 문서: `samples/3-10월_교육_통합_2022.hwp`
- 대상 PDF: `pdf/3-10월_교육_통합_2022.pdf`
- 대상 페이지: 11쪽
- 사용자 시각 판정: 페이지 하단 미주 문항 간격이 이상하게 보인다.

## 초기 판단

1. `3-10월_교육_통합_2022.hwp` 11쪽의 미주 영역에서 `문15)` 이후와 오른쪽 단 `문16)`~`문19)` 사이의 문항 간격 또는 줄 간격이 PDF/한컴 기준과 어긋난 것으로 보인다.
2. Stage5/6의 compact 미주 하단 fit 및 HWP5 placeholder 폭 보정이 다른 미주 페이지에 영향을 주었는지 확인한다.
3. PDF 11쪽과 현재 SVG/PNG 11쪽을 같은 기준으로 뽑아 문항 제목 y 좌표와 미주 본문 줄 간격을 비교한다.

## 진행 계획

1. PDF 11쪽 PNG와 현재 SVG 11쪽 PNG를 생성한다. SVG 변환은 `rsvg-convert`를 사용한다.
2. `dump-pages`/debug 로그로 11쪽에 배치된 미주 문단 index, y 좌표, overflow 여부를 확인한다.
3. 한컴/PDF 기준과 다른 간격 원인을 좁혀 최소 보정을 적용한다.
4. `tests/issue_1139_inline_picture_duplicate.rs`에 회귀 테스트를 추가하거나 기존 테스트를 강화한다.
5. PR #1194 브랜치에 추가 커밋으로 보정한다.

## 현재 상태

- 2026-06-01: 작업지시자가 `3-10월_교육_통합_2022.hwp` 11쪽 미주 간격 이상을 보고했다. Stage7 문서를 생성하고 PDF 비교를 시작한다.
- 2026-06-01: PDF 11쪽과 현재 SVG/PNG를 비교했다. `문18)` 위치는 거의 맞지만, `문19)` 이후가 약 40px 아래로 밀려 `문18)`~`문19)` 사이 미주 간격이 과대해졌다.
- 2026-06-01: 원인은 compact 미주 새 문항 제목 보정에서, 직전 문단이 빈 문단이어도 중간 단 기본 완충분 40px을 추가하는 흐름이었다. 빈 문단은 이미 시각 spacer 역할을 하므로 추가 완충을 생략하도록 보정했다.
- 2026-06-01: `tests/issue_1139_inline_picture_duplicate.rs`에 `3-10월_교육_통합_2022.hwp` 11쪽 `문18)`→`문19)`→`문20)` 간격 회귀 테스트를 추가했다.

## 검증 기록

- PDF 기준 산출물:
  - `output/task1189_stage7_3-10_page11/pdf96/pdf_page-11.png`
- 보정 후 SVG/PNG:
  - `output/task1189_stage7_3-10_page11/fixed/3-10월_교육_통합_2022_011.svg`
  - `output/task1189_stage7_3-10_page11/fixed/rhwp_page11.png`
- `RHWP_VPOS_DEBUG=1 cargo run --quiet --bin rhwp -- export-svg samples/3-10월_교육_통합_2022.hwp -p 10 -o output/task1189_stage7_3-10_page11/fixed_debug 2>&1 | rg "VPOS_CORR: path=.*pi=5(69|70|71|72|73|74|82)|LAYOUT_OVERFLOW"`: `문19)` 보정 후 해당 페이지 overflow 없음.
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`: 통과, 34개 테스트 성공.
- `cargo fmt --all --check`: 통과.
- `cargo test --tests`: 통과.
- `wasm-pack build --target web --out-dir pkg`: 통과.
