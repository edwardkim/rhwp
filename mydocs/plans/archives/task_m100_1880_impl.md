# Task M100 #1880 구현계획서 — convert-HWP 자리차지 표 host_before 비대칭 해소

- 이슈: #1880
- 마일스톤: M100 (v1.0.0)
- 브랜치: `local/task1880`
- 수행계획서: `mydocs/plans/task_m100_1880.md`
- 작성일: 2026-07-05

## Stage 1 — typeset.rs 자리차지 판정 교체 + 회귀 테스트

- `src/renderer/typeset.rs` `format_table`:
  - `10617` `let table_text_wrap = (table.attr >> 21) & 0x07;` 제거,
    `!self.is_hwpx_source.get() && matches!(table.common.text_wrap, TextWrap::TopAndBottom)`
    로 교체 (사용처는 `10711` `before` 분기 1곳뿐 — 전수 확인 완료).
  - 근거 주석: 원시 attr 는 HWPX 파스 미채움 → convert-HWP 재파스와 비대칭
    (#1880 실측: 2780073 host_before 6.7↔0.0px), native 는 비트⇔열거형 동치
    (`shape.rs:394`)로 불변, #1886 origin 전달의 연장.
- 회귀 테스트: 자리차지(비-TAC TopAndBottom) 표 + spacing_before>0 문서를
  HWPX 직렬화 → convert(HWPX-origin 마커 포함 HWP) → 재파스하여 양 경로
  pagination(쪽수/배치) 자기정합 확인. #1886 테스트(samples/issue1770 패턴)
  또는 in-memory 왕복으로 구현 — 기존 테스트 인프라 조사 후 결정.

## Stage 2 — 실측 검증 (로컬 코퍼스)

- 3075729: convert-HWP heading(sec1 pi=121) p12 → p13 확인 (HWPX·oracle 정합).
- 2780073: `RHWP_TABLE_DRIFT` host_before/host_sp 양 경로 일치 + dump-pages 대조.
- 2959953 / 3171755 (PI_MOVED 잔존 나머지 2건, 코퍼스 검색 후 존재 시):
  hwpx vs convert dump-pages 대조.
- 2776741 (phantom 케이스, oracle 1쪽): 불변 확인 — 본 수정이 #1836 억제와
  별개 층위임을 실증.

## Stage 3 — 전체 회귀 + 최종 보고서

- `cargo test` 전체, `cargo test --test hwpx_roundtrip_baseline`,
  hwp5 baseline·issue_rowbreak 계열 통과.
- `cargo clippy` 신규 경고 없음.
- 최종 보고서 `mydocs/report/task_m100_1880_report.md`: 결과 + 범위 외 관찰
  (TAC bit0 미러 비대칭 후보, ir-diff 원시 attr 비교 부재) 기록.

## 커밋 계획

- Stage 1: `Task #1880: 자리차지 판정 원시 attr → 의미 필드 + 소스 게이트` + stage1 보고서
- Stage 2: 실측 검증 결과 stage2 보고서
- Stage 3: 최종 보고서 + orders 갱신
