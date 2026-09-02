# Task #3211 Stage 1 — Windows 한컴 LineSeg 재조판 보정

- Issue: [#3211](https://github.com/edwardkim/rhwp/issues/3211)
- Base: `upstream/devel` `d871bb8ce1`
- 구현 커밋: `494e1317e`
- 측정일: 2026-08-14 KST

## Windows 한컴 관측

`tools/hangul_version_oracle`를 Windows PowerShell에서 워커 1개로 실행했다. 한컴 2022
(`12.0.0.4605`)는 두 샘플을 모두 정상으로 열었고, 두 문서의 페이지 지문은 아래와 같이
동일했다.

| 샘플 | 쪽수 | 문단 수 | 페이지 지문 |
| --- | ---: | ---: | --- |
| `3-09월_교육_통합_2022.hwp` | 23 | 468 | `0@0,1@82,2@153,3@203,4@277,5@334,6@367,7@429,8@465` |
| `3-09월_교육_통합_2024-구분선아래20구분선위20.hwp` | 23 | 468 | `0@0,1@82,2@153,3@203,4@277,5@334,6@367,7@429,8@465` |

한컴 2024 실행 파일(`13.0.0.3901`)도 설치돼 있으나, 버전 오라클이 HKCU override 뒤에도
COM major 12를 받아 안전하게 중단했다. 2024 결과는 측정하지 않았으며, 즉시
`restore_com_default.ps1`로 2022 기계 기본 COM 등록을 복원했다. HKCU 키를 삭제하거나
강제로 재시도하지 않았다.

## 원인과 보정

두 샘플의 저장 `PARA_LINE_SEG`를 지운 뒤 재조판하면, 수식·그림처럼 글자처럼 취급되는
인라인 제어문이 visible text에 없다는 이유로 줄 폭에서 빠졌다. 기존 구현은 모든 제어문의
높이만 첫 줄에 얹어 여러 줄이 축소됐다.

`reflow_line_segs()`는 이제 제어문의 문단 문자 위치별 HWPUNIT 폭을 토큰 폭에 더하고, 실제
해당 제어문을 포함한 줄에만 최대 높이를 적용한다. 저장 LineSeg 배열은 이 진단의 계산 입력으로
사용하지 않는다.

## 수치 검증

두 샘플 모두 저장 LineSeg 보유 문단 165개에서 아래 결과를 보였다.

| 항목 | 보정 전 | 보정 후 |
| --- | ---: | ---: |
| 줄 수 일치 | 139/165 (84.2%) | 160/165 (97.0%) |
| 줄바꿈 위치 일치 | 123/165 (74.5%) | 130/165 (78.8%) |

남은 차이는 좁은 wrap-zone의 저장 `segment_width`와 제어문 전후의 한컴 줄 경계 규칙으로,
이번 폭·높이 누락 보정 범위를 벗어난다. 임의의 허용오차 확대나 저장 LineSeg 재사용으로 숨기지
않았다.

## 로컬 검증

모든 Cargo 명령은 `target\\pr-review`를 재사용해 순차 실행했고 `CARGO_INCREMENTAL`은 설정하지
않았다.

- `cargo test --lib issue3211_uncached_endnote_body_preserves_inline_control_flow -- --nocapture`
  — 통과: 두 샘플 각각 줄 수 160/165, 줄바꿈 130/165.
- `cargo test --test issue_1082_endnote_multicolumn_drift` — 5/5 통과.
- `cargo test --test issue_1139_inline_picture_duplicate` — 85/85 통과.
- `cargo test --lib line_breaking:: -- --nocapture` — 2/2 통과.
- 직접 `rustfmt.exe --check`(edition 2021), `git diff --check` — 통과.

Cargo의 `fmt` subcommand는 이 Windows 경로에서 OS error 206(파일명/확장명 길이)로 help를
출력해 판정을 제공하지 못했다. 위 직접 rustfmt 검사를 대신 사용했다.

## CI 후속 보정

`devel` 최신 병합을 반영한 PR CI에서 HML fixture의 middle-anchored table 회귀가 발견됐다.
이 fixture는 `abc + table + efg`의 control 경계를 renderer가 별도의 `TextRun`/`Table`로
표현한다. 표 전체 폭을 control 뒤 visible character에 합산하면 그 경계가 사라져 trailing
`efg` run을 찾지 못한다.

`b7fbd8b02`는 `flow_inline_controls()`에서 `Control::Table`을 제외해 기존 table 배치 경로를
보존했다. 표의 cell-split/empty-paragraph 크기 계산은 그대로 두며, Windows Hancom HWP 샘플에서
검증한 수식·그림 control 폭·높이 보정에는 영향을 주지 않는다.

- `pr_2219_hml_middle_anchor` 대상 회귀 — 1/1 통과.
- #3211 대조 — 두 샘플 각각 160/165 line count, 130/165 line break 유지.
- #1082 미주 드리프트 5/5, #1139 인라인 그림·미주 85/85, Clippy 통과.
