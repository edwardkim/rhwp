# 완료 보고서 — Task M100-1418

- 이슈: https://github.com/edwardkim/rhwp/issues/1418
- 제목: 첫 페이지 글상자와 표 상단 중앙 overlap 정합
- 브랜치: `local/task_m100_1418`
- 작성일: 2026-06-16

## 1. 결과 요약

`samples/2026_oss_rst.hwp` 1페이지에서 제목 글상자와 큰 1x1 안내 표의 상단선 overlap이
한컴 PDF 기준과 다르게 보이는 문제를 수정했다.

교체된 정답 PDF `pdf-large/hwpx/2026_oss_rst.pdf`는 총 6페이지 문서이며, 이번 정합 기준은
그 PDF의 1페이지다. 정답 기준에서 제목 흰 배경은 `y≈133.1..171.1px` 영역에 있고,
큰 표 상단선은 `y≈153.4px`로 제목 배경 중앙을 지난다.

수정 전 rhwp는 큰 표 상단선을 `y=132.3px`에 그렸다. 수정 후 render tree 기준 큰 표 상단은
`y=153.60px`, 제목 TextBox는 `y=137.17px`로 확인되어 정답 overlap 구조와 맞아졌다.
작업지시자 SVG 시각 판정도 통과했다.

## 2. 변경 사항

| 파일 | 내용 |
|---|---|
| `src/renderer/layout.rs` | Paper 기준 `InFrontOfText` 글상자 중 `text_box`가 있는 Shape host 문단도 line advance를 보존하도록 조건 보강 |
| `tests/issue_1418_textbox_table_overlap.rs` | 큰 표 top과 제목 TextBox 위치를 고정하는 회귀 테스트 추가 |
| `samples/2026_oss_rst.hwp` | #1418 primary 재현 fixture |
| `samples/hwpx/2026_oss_rst.hwpx` | 동일 이름의 보조 fixture. 이번 정답 기준과는 별도 문서로 판정 |
| `pdf-large/hwpx/2026_oss_rst.pdf` | 작업지시자가 교체 제공한 6페이지 정답 PDF |
| `mydocs/plans/task_m100_1418.md` | 수행 계획서 |
| `mydocs/plans/task_m100_1418_impl.md` | 구현 계획서 |
| `mydocs/working/task_m100_1418_stage1.md` | 기준점 확정 보고서 |
| `mydocs/working/task_m100_1418_stage2.md` | 원인 분리 보고서 |
| `mydocs/orders/20260616.md` | 진행 기록 |

## 3. 원인과 수정

페이지네이션 단계는 `FullParagraph pi=0 h=21.3`을 가지고 있었다. 하지만 render tree layout 단계에서
빈 non-TAC floating shape host 문단 fast path가 적용되며, 제목 글상자가 `VertRelTo::Paper` 기준이라는
이유로 line advance 예약 대상에서 제외되었다.

기존 조건은 non-TAC `InFrontOfText` 중 `VertRelTo::Para`만 line advance를 보존했다. 문제 샘플의 제목
글상자는 non-TAC `InFrontOfText`이지만 `VertRelTo::Paper`이고, 내부 `text_box`가 있는 글상자다.

이번 수정은 전역 Paper 기준 도형 전체가 아니라 다음 조건에만 line advance 보존을 확장했다.

- `Control::Shape`
- non-TAC
- `TextWrap::InFrontOfText`
- `VertRelTo::Paper`
- `shape.drawing().text_box.is_some()`

`BehindText`, 일반 장식 도형, `text_box` 없는 Shape, Picture는 기존 비예약 경로를 유지한다.

## 4. 검증

실행한 검증:

```bash
cargo fmt --check
cargo test --test issue_1418_textbox_table_overlap -- --nocapture
cargo test --test issue_775 -- --nocapture
cargo test --test issue_919_textbox_hit_test -- --nocapture
cargo test --test issue_1052_footnote_in_textbox -- --nocapture
cargo test --test issue_716 -- --nocapture
cargo test --test issue_986 -- --nocapture
cargo check --lib
docker compose --env-file .env.docker run --rm wasm
git diff --check
```

결과: 모두 통과.

신규 테스트 출력:

```text
[issue_1418] table_top=153.60 table_h=853.83 title_textbox=[x=275.65 y=137.17 w=241.88 h=30.23]
```

CLI 산출물:

```bash
target/debug/rhwp export-svg samples/2026_oss_rst.hwp -p 0 -o output/poc/task1418-final-hwp --debug-overlay
target/debug/rhwp export-render-tree samples/2026_oss_rst.hwp -p 0 -o output/poc/task1418-final-render-tree
target/debug/rhwp dump-pages samples/2026_oss_rst.hwp -p 0
```

확인 결과:

- 최종 render tree: `Table pi=1 ci=0 y=153.6`
- 최종 render tree: 제목 `TextBox y=137.2`
- 최종 SVG: 큰 표 상단선 `y=153.60000000000002`
- WASM 산출물 생성 완료: `pkg/rhwp_bg.wasm` `5.5M`

## 5. 시각 판정

시각 판정 대상:

- `output/poc/task1418-final-hwp/2026_oss_rst_001.svg`

작업지시자 시각 확인 완료.

## 6. 후속 기록

구현, 로컬 검증, SVG 시각 판정, WASM 빌드, `devel` 반영, 이슈 close를 완료했다.

후속 기록:

- Task commit: `3964c7b5`
- `local/devel` merge: `bcf90462`
- `local/devel` → `devel` merge + push: `fbcf0c85`
- Issue #1418 close: https://github.com/edwardkim/rhwp/issues/1418
- Closed at: `2026-06-16T06:47:19Z`
