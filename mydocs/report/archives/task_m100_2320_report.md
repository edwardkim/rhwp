# #2320 최종 결과보고 — 문단 중간 vpos 되감김(쪽 경계) 미인식 정정

- 이슈: [#2320](https://github.com/edwardkim/rhwp/issues/2320)
- 마일스톤: M100 (v1.0.0) / 브랜치: `local/task2320`
- 계획서: `mydocs/plans/task_m100_2320.md` (버그 정정형, 구현계획서 생략)
- 단계 보고: `mydocs/working/task_m100_2320_stage{1,2}.md` (3·4단계는 2단계에
  선행 완료되어 시각 판정 요청으로 병합 — 승인됨)

## 결론

**시각 판정 통과** (작업지시자, 2026-07-17 — skia PNG + studio/wasm).
treatise p2 오른쪽 단이 한컴과 동일 지점("둘째, Solicited…so")에서 끊기고
잔여가 p3 로 넘어간다. 하단 잘림·내용 소실 해소, **p4 동종 결함(pi=51/52)
까지 함께 해소**.

## 근인과 정정

- **근인**: 문단 내 vpos 되감김 감지가 `current_column == 0` 게이트로 단 0
  시작 문단에만 적용 — 마지막 단에서 시작하는 문단의 되감김(= "다음 쪽 단 0
  으로 계속" 인코딩)은 미감지되어 문단 통째 배치 → 하단 잘림. 추가로
  `typeset_multicolumn_paragraph` 의 마지막 단 분할이 flush 후 단/높이를
  갱신하지 않는 잠복 결함 동반.
- **정정**: ①비-0 단 시작 문단에 전용 감지 `detect_near_top_rewind_breaks`
  적용 — 되감김 목표가 **단 상단 근방(본문 높이 15% 이내)**일 때만 경계 인정
  (treatise 7% ✓ / 143E 신문 스크랩 40% ✗ — 어울림 흐름 잔재 위양성 차단)
  ②미주 흐름 제외 가드 ③마지막 단 분할을 `advance_column_or_new_page` 로
  새 페이지 진행 ④legacy Paginator 동일 게이트 해제(일관성).
- 저장 지오메트리 신뢰 계보(#2112, #2299/PR #2314)와 방향 정합 — 로드된
  저장 신호를 재계산으로 상쇄하지 않고 경계로 승격.

## 게이트

- `tests/issue_2320_vpos_rewind_page_break.rs` 3/3 (red→green + 단 0 분할
  불변 가드 + 143E 위양성 핀)
- `cargo test --tests` release-test 실패 0 / fmt / clippy 0
- **전 샘플 632건 로드 페이지 수 devel 완전 일치** (의도외 변동 0)
- LAYOUT_OVERFLOW(treatise): devel 5건(내용 소실 최대 106.8px) → 2건
  (미세 draw 드리프트 10.3px 신규 + 5.7px 기존 — 시각 영향 없음, p5 PDF 일치)
- 한컴 2022 PDF 대조: p2/p4/p5 단 끝·시작 내용 일치
- WASM 빌드 + studio 실기 확인 (작업지시자)

## 과정 기록

- 계획 대비 변경: 정정 지점이 legacy Paginator 가 아닌 주 엔진 TypesetEngine
  이었음을 계측으로 확인 (`RHWP_USE_PAGINATOR` fallback 구조)
- Docker Desktop stale mount 2회 복구 (taskkill → wsl --terminate → 재시작)

## 남은 항목

- 관찰: 계측(used)-배치 y 미세 드리프트 일반 축 (pi=57 10.3px, 1줄 미만,
  시각 무영향) — 필요 시 별도 이슈

## 특성화 정정 (2026-07-17, lpaiu-cs 실측 대조)

이슈/보고의 "비-0 되감김(67050→4926)" 표현은 **오독** — 원시 LINE_SEG 재현
결과 실제 인코딩은 `[67050, 0, 1642, 3284, 4926]`, 즉 **문단 중간 0-리셋 +
새 쪽 로컬 재누적**이다 (`dump-pages` 의 `vpos=first..last` 표기가 유발한
오독. `[vpos-reset@line1]` 마커가 이미 0-리셋을 표시하고 있었다).

구현은 무영향: `detect_near_top_rewind_breaks` 조건이 "문단 내 감소 + 목표
단 상단 근방(0 포함)"이라 이 인코딩을 정확히 분할하고, 143E 의 진짜 비-0
중간 되감김(26644, 쪽 중간)만 배제한다 — 컨트리뷰터의 "감소(0 포함)" 제안과
정합. 기존 단 경계 경로가 이 0-리셋을 못 잡은 원인은 특성화가 아니라
`current_column == 0` 게이트였고, 그 해제가 본 정정의 본질.
