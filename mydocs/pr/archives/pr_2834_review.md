# PR #2834 검토 — Task #2809 나눔정렬과 Canvas 음수 자간

- PR: [#2834](https://github.com/edwardkim/rhwp/pull/2834)
- 기준 브랜치: `upstream/devel` `58991a768`
- 작업 브랜치: `task/2809-distribute-align`
- 구현·증적 기준 source head: `f2d0e0968cc85e915802c1126c41da2383e5834e`
- 상태: Open PR, 신규 샘플의 기존 IR 직렬화 갭 baseline 반영 후 CI 재검증 예정

## 판정

승인 가능. `Alignment::Split`의 마지막 줄 분배 의미를 일반 `Justify`와 분리했고,
Canvas 2D가 음수 자간을 glyph 폭 축소로 잘못 적용하던 경로를 제한했다. `Split`
분배는 마지막 glyph의 실제 잉크 여유를 예약한다. 최종 화면은 첫 glyph 폭이
`28/28px`로 동일하고 마지막 `이`도 `22px` 온전히 표시된다. HWP 2020 PDF는 정상
기준으로 유지했다.

## 범위 검토

- 일반 `Justify`, 강제 줄바꿈과 자간 0% 행의 기존 동작은 유지된다.
- visual sweep의 `--dpi`가 SVG에도 적용되어 PDF/rhwp가 같은 배율로 비교된다.
- 이슈 원본 HWP, 정상 기준 PDF, 최종 review PNG와 WASM E2E HTML이 포함됐다.
  원본 ZIP 묶음은 후속 정리 PR에서 추적 대상에서 제외한다.
- golden 변경은 HWP 2022 기준에 가까워진 `Split` 좌표 2개로 제한된다.

## 검증

- 전체 lib: `2512 passed; 0 failed; 7 ignored`.
- SVG snapshot: `8 passed; 0 failed`.
- clippy `-D warnings`, fmt, diff, Python 구문 검사: 통과.
- WASM build와 rhwp Studio production build: 통과.
- rhwp Studio E2E assertion `7/7`, Canvas `1126×1587`, 실제 편집기 100% 캡처.
- visual sweep 144dpi: `flagged=0/1`.
- 저장용 review PNG의 rhwp 패널은 실제 Studio `canvas2d` E2E 캡처로 교체해
  `다 같 이` 마지막 glyph가 온전히 보이는 화면을 사용한다.
- 첫 CI에서 신규 샘플 `issues/2809/jubo_20260104.hwp`의 기존
  `list_header_width_ref` 직렬화 갭 174건이 baseline 대비 증가로 검출됐다. 이번 변경은
  직렬화기를 수정하지 않았고 동일 필드는 기존 HWP 표 샘플에도 기록된 광역 항목이므로,
  의도된 신규 코퍼스 편입으로 판정해 baseline 1행을 추가한다.
- `cargo test --profile release-test --test ir_field_sweep_baseline -- --nocapture`:
  샘플 799건(스킵 1), 발산 경로 703종, 총 110376건, `2 passed; 0 failed`.

상세 증적은 [`assets/task2809/README.md`](assets/task2809/README.md)와
[`task_m100_2809_report.md`](../report/task_m100_2809_report.md)를 따른다.

## PR 후속

1. baseline 보완 head의 CI/CodeQL/Render Diff 완료를 확인한다.
2. merge 권한과 사용자 승인이 확인된 경우에만 후속 merge/issue close 절차를 수행한다.
