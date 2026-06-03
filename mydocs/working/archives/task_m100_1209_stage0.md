# Task M100 #1209 Stage 0

## 목적

`3-11월_실전_통합_2022.hwp` 14쪽 `문22)` 미주 간격이 한컴오피스/PDF 기준과 다르게 보이는 원인을 분석한다.

## 시작 기준

- 이슈: [#1209](https://github.com/edwardkim/rhwp/issues/1209)
- 작업 브랜치: `local/task_m100_1209`
- 기준 브랜치: `devel` (`upstream/devel` 동기화 완료)
- 대상 문서: `samples/3-11월_실전_통합_2022.hwp`
- 대상 PDF: `pdf/3-11월_실전_통합_2022.pdf`
- 대상 페이지: 14쪽
- 대상 문항: `문22)` 미주
- 관련 맥락: task 1139 후속 보정, #1189 / PR #1194 이후 추가 확인 항목

## 분석 계획

1. PDF 14쪽과 현재 SVG 14쪽을 같은 해상도 PNG로 생성한다. SVG 변환은 `rsvg-convert`를 사용한다.
2. 현재 렌더링의 `dump-pages` 결과에서 14쪽 미주 문단 index, column, vpos, overflow 여부를 확인한다.
3. `RHWP_VPOS_DEBUG=1` 로그로 `문22)` 직전/직후 문단의 `HeightCursor` 보정 경로를 추적한다.
4. PDF/한컴 기준과 현재 렌더링의 차이가 문항 간 미주 간격, 빈 문단 처리, 또는 vpos base 이동 중 어디에서 발생하는지 정리한다.
5. 소스 수정이 필요한 경우 별도 구현 스테이지로 넘기고, Stage0에서는 원인과 수정 후보만 기록한다.

## 현재 상태

- 2026-06-01: 작업지시자가 Stage0 원인 분석 시작을 지시했다.

## 분석 기록

- 2026-06-01: PDF 14쪽과 현재 SVG 14쪽을 96dpi PNG로 생성했다. SVG PNG 변환은 전역 지침대로 `rsvg-convert`를 사용했다.
  - PDF: `output/task1209_stage0_3-11_page14/pdf96/pdf_page-14.png`
  - 현재 RHWP: `output/task1209_stage0_3-11_page14/current/rhwp_page14.png`
  - 현재 SVG: `output/task1209_stage0_3-11_page14/current/3-11월_실전_통합_2022_014.svg`
- 2026-06-01: PDF bbox 기준 `문22)` 직전 마지막 줄은 `33이다.` 줄 yMax `343.911pt`, `문22)` yMin `363.198pt`라서 약 `19.287pt`(`96dpi` 환산 약 `25.7px`)의 간격이 있다.
- 2026-06-01: 현재 SVG 기준 직전 마지막 줄은 y `456.1867px`, `문22)`는 y `467.4667px`로 약 `11.28px` 차이에 불과하다. 화면상으로도 `33이다.` 바로 아래에 `문22)`가 붙어 보인다.
- 2026-06-01: `dump-pages -p 13` 기준 대상 구간은 왼쪽 단의 미주 문단 `pi=631 -> pi=632`이다.
  - `pi=631`: `vpos=398876..400228`, 내용 `따라서 ... 합은 이다.`
  - `pi=632`: `vpos=444925`, 내용 `문22）   22_11_실전 22) 13`
- 2026-06-01: `RHWP_VPOS_DEBUG=1` 로그에서 `pi=632`는 `prev_ls=1984`, `y_in=457.87`, `end_y=1041.83`, `stale_forward=true`, `compact_new_note=false`, `applied=false`로 처리된다.
  - `1984HU`는 96dpi 기준 약 `26.45px`이다.
  - 현재 보정기는 `end_y`가 `y_offset + 120px`를 넘는 큰 전방 점프라서 `compact_endnote_new_note_jump`로 보지 않는다.
  - 그 결과 원문에 있던 새 문제 제목 앞의 미주 간격 후보(`prev_ls`)까지 버리고 `y_offset` 그대로 렌더한다.
- 2026-06-01: Stage0 원인 후보는 "compact 미주 새 문제 제목 앞에서 원문 VPOS는 낡은 큰 전방 점프라 그대로 쓸 수 없지만, 직전 문단의 trailing line spacing은 보존해야 하는데 현재는 둘을 함께 버리는 것"으로 좁혔다.

## 수정 후보

1. `src/renderer/height_cursor.rs`에서 compact 미주 질문 제목(`문...`)이 큰 stale forward로 판정되는 경우, 원문 `end_y` 전체는 쓰지 않되 직전 line spacing(`prev_line_spacing_px`)만 보존하는 별도 경로를 둔다.
2. 이 경로에서도 기존 compact gap 보정처럼 `vpos_lazy_base`를 함께 이동해 다음 문단에서 버린 큰 VPOS 점프가 되살아나지 않도록 한다.
3. 회귀 테스트는 `tests/issue_1139_inline_picture_duplicate.rs`에 `3-11월_실전_통합_2022.hwp` 14쪽 `문22)` 위치 또는 `pi=631 -> pi=632` 간격 조건을 추가한다. 기존 #1189 10~12/14/17쪽 조건이 깨지지 않는지 함께 확인한다.

## Stage0 판단

- 소스 수정 필요성이 있다.
- 구현은 별도 Stage1로 넘긴다. Stage0에서는 원인 분석과 수정 후보 기록까지만 수행한다.
