# 구현 계획서 — Task #1860

**이슈**: #1860 분할(RowBreak) 행 valign=Center 라벨 세로 위치 어긋남
**브랜치**: `local/task1860` (통합 베이스 `local/prstack-0703`, #1841 워킹트리 유지)
**수행계획서**: `task_m100_1860.md` (승인·자동승인)

---

## 근본원인 요약 (수행계획서 확정분)

- 증상: p44 라벨 −24.6pt / p45 균일 +40.8pt (반대방향).
- 진단: **valign 정상**. rhwp 가 row1(공공데이터법 947px 내용셀) RowBreak 분할을
  한글보다 **~55px(~3줄) 일찍 컷**(rhwp line17 / 한글 line20). 두 엔진 모두 body
  하단까지 120~180px 여유 → 페이지 용량 아님, **intra-row 분할 예산 과소**.
- 컷 결정 경로: `pagination/engine.rs::split_table_rows`. PartialTable 은
  `end_cut=[]` 로 push 되고 실제 유닛 컷은 **layout 시 `content_offset`(높이 예산)**
  에서 파생. 컷 위치를 정하는 유일 변수 = `split_end_limit`
  (= `avail_for_rows − max_padding_for_row(r) − cs`).
- `avail_for_rows = page_avail − header_overhead − sb_extra`,
  `page_avail = table_available_height − current_height − caption_extra − host_extra − v_extra`.
  본 표 ci=1: `v_offset=4327HU≈204px`, 상단 캡션표 ci=0, host 텍스트 존재.

## 단계 (5)

### Stage 1 — 계측·과소 항 특정
- `split_table_rows` 에 env-gated(`RHWP_SPLIT_DBG`) eprintln 임시 추가:
  para_idx/ctrl_idx, table_available_height, current_height,
  caption_extra/host_extra/v_extra, header_overhead/sb_extra, avail_for_rows,
  range_height, max_padding_for_row(1), cs, **split_end_limit**, 그리고 layout
  이 이 예산으로 실제 컷한 줄 수.
- 한글 20줄(≈629px 내용) vs rhwp 17줄(≈574px)의 55px 차이가 어느 항에서
  발생하는지 확정: (a) v_extra 중복차감, (b) caption/host_extra 과다,
  (c) max_padding 과다, (d) split_end_limit→줄 환산 반올림/여유(MIN_SPLIT/…),
  (e) layout 컷과 pagination 예산 불일치.
- **산출물**: `mydocs/tech/task_m100_1860_split_budget.md` (계측표 + 특정 결론).
  이 결과로 Stage 2 수정 범위 확정. (본 단계는 계측 only, 로직 불변.)

### Stage 2 — 교정 설계
- 특정된 과소 항만 최소 수정하는 패치 설계. 전면 상수 변경 지양.
- fixture 컷을 line17→line20 으로 이동시키되, 다른 RowBreak 표 컷을 흔들지 않는
  조건(가드) 도출. 필요 시 layout 컷과 pagination 예산의 산식 일치화.
- `table_partial.rs` valign 로직은 **불변** 원칙 유지.

### Stage 3 — 구현 + fixture 검증
- Stage 2 패치 적용. `RHWP_SPLIT_DBG` 계측 제거.
- fixture export-pdf → compare_line_baselines p44/p45:
  - 목표: p44 라벨 Δ −24.6→~0, p45 균일 +40.8→~0(±2pt 이내).
  - 성공 판정: 공공데이터법·전자정부법 라벨이 한글 y(445.0 / 344.2)에 수렴.

### Stage 4 — 회귀 게이트
- `cargo test` 전체 + 분할표 계열 테스트(issue_1809/1748/993/1025/474 등) 통과.
- big_hwpx/big_hwp render-diff 코퍼스: 통합 베이스 대비 회귀 0, 개선 확인.
  (기준선: rd_big_*_1841 또는 현 통합 재빌드.)
- #1809/#1841/#1836 불변 확인. 보상 좌표 핀 발생 시 **권위(한글 PDF/저장 vpos)**
  로만 정정.

### Stage 5 — 보고서
- `mydocs/report/task_m100_1860_report.md` 최종 보고서.
- 오늘할일(orders)은 불가침 규칙(`no-touch-mydocs-orders`)으로 미수정.
- 단계별 커밋: 각 stage 소스+`_stage{N}` 보고서 동반 커밋.

## 검증 기준 (권위)

- 1차: 한글 2024 PDF `pdf/issue1853_caption_precedes_body_split-2024.pdf`
  (fixture 동봉, merge 커밋 772dc2c7).
- 보조: 저장 vpos 산술. 직전 렌더값 기반 핀 재설정 금지.

## 리스크·롤백

- 분할 예산은 razor-thin → 코퍼스 게이트 미통과 시 즉시 롤백/조건 정밀화.
- 단일 표 fixture 개선이 코퍼스 회귀를 유발하면 fixture 특성(v_offset+캡션+host
  동시 존재) 한정 가드로 좁힘.
