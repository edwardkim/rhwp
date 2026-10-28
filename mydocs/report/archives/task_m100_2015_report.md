# 최종 결과보고서 — #2015 HWPX saved-bounds RowBreak 표 잔존 드리프트

- 이슈: #2015 (#1811/#1887 후속, 마일스톤 v1.0.0)
- 브랜치: `fix/2015-saved-bounds-rowbreak-overflow` (base `origin/devel`)
- 대상 샘플: `samples/task1749/saved_bounds_cumulative_page_break.hwpx`
- 정답지: `samples/task1749/saved_bounds_cumulative_page_break-2024.pdf` (한글 2024)

## 1. 배경

PR #1887(task 1811)가 RowBreak/saved-bounds 조판을 보정했으나 visual sweep p4/p5 잉크 정합이
낮게 남았다(`ink_match` p4≈9.6% / p5≈5.1%). 재분석 결과 #1811 seam 의 **미해소 잔존분 2종**으로
특정됐다.

## 2. 원인 (2 발원지)

### 발원지 ① 부동(tac=false) RowBreak 표 91.2px 오버플로우 — 포맷 공통, **수정 완료**

- `pre_emit_visible_rowbreak_host_text`(#1811)가 host 텍스트를 pre-emit 하며 `current_height`
  를 `para_start → para_start+host_h(≈146px)` 로 전진.
- 이후 typeset 예산과 layout 배치가 각각 `vert_offset`(=144.3px, para_start 기준)을 **재차감/재적용**
  → host_h 만큼 이중계상 → 표 앵커가 body 바닥(1026.5px) 아래로 밀려 91.2px 초과.
- `RHWP_TABLE_DRIFT`: `page_avail = 930.5 − 806.0 − 144.3 = −19.8 → 0.0` (강제 1유닛 오버플로우).

### 발원지 ② 인라인(tac=true) 표 valign 무력화 — HWPX 전용, **findings 인계**

- HWPX 합성 셀 `total_content_height` 과대계산(362.8 vs HWP 293.3, 69.5px) → valign=Center 의
  `mechanical_offset` 이 0 으로 클램프 → 별표가 셀 상단 37px 위.
- 별표 줄 피치는 rhwp/PDF 동일(버그 아님). 오버플로우·페이지수·본문흐름 무관한 미관 수준.
- 상세: `mydocs/working/task_m100_2015_stage4.md`.

## 3. 수정 (발원지 ①)

`pre_emitted_host_heights: HashMap<usize,f64>` 를 TypesetState → PaginationResult → LayoutEngine
로 전파. host pre-emit 된 문단에 한해 `vert_offset` 을 `(vert_off − host_h).max(0)` 로 감액하여
typeset 예산(`typeset.rs`)과 layout 배치(`table_partial.rs`)를 정합. host pre-emit 아니면 `host_h=0`
→ 종전과 동일(비대상 문서 회귀 0).

변경 파일: `typeset.rs`, `pagination.rs`, `pagination/engine.rs`, `layout.rs`,
`layout/table_partial.rs`, `document_core/queries/rendering.rs`.

## 4. 결과

| 지표 | 수정 전 | 수정 후 |
|---|---|---|
| `LAYOUT_OVERFLOW` para=52 | 91.2px | **2.1px** (엔진 tolerance 수준) |
| 표 top (render tree, p4) | y=1056.4 | **y=912.1** (host 텍스트 직후) |
| HWPX end_cut | `[1]` | **`[3]`** (= HWP 저장 LINE_SEG 참조·한컴 PDF) |
| 페이지 수 | 5 | 5 |
| 시각 오라클 p5 ink_match | 11.62% | 13.12% |

- 잔여 2.1px 는 마지막 유닛이 경계에 걸치는 행높이 측정 드리프트(엔진
  `ROWBREAK_SPLIT_ROW_OVERFLOW_TOLERANCE=2.0px` 수준)로 컷 오류 아님. HWPX end_cut=[3] 이 HWP·PDF 와 일치.
- p4 총 ink 는 발원지 ②(상단 인라인 표)가 지배해 집계 변화가 작다. 하단 pi=52 는 오버레이로
  rhwp/PDF 정렬 확인.

## 5. 검증

- 전체 테스트: **2923 passed / 0 failed** (210 바이너리, release-test, --no-fail-fast).
- `cargo clippy --all-targets -- -D warnings`: 통과(무경고).
- `issue_1811_hwpx_pi52_rowbreak_cut_matches_hwp_reference`: HWPX 기대값 `[1]→[3]` 교정
  (종전 `[1]` 은 이중계상 버그값. 주석에 #2015 근거 명시).
- 신규 `tests/issue_2015_saved_bounds_rowbreak.rs`: Body 서브트리 content ≤ body 바닥 불변식(≤5px 게이트).
- 형제 샘플 `saved_bounds_cumulative_vpos`(HWPX/HWP): 2쪽·overflow 0 유지.

## 6. 부수 산출

- `scripts/visual_oracle_native.py`: Windows/래스터라이저 부재 환경용 자립형 시각 오라클
  (rhwp export-png(native-skia) + PyMuPDF + PIL/numpy). before/after 픽셀 검증 게이트.

## 7. 잔여·후속

- 발원지 ②(HWPX 인라인 표 합성 content-height 과대계산)는 수정 locus 가 전 HWPX 표 행높이·
  오버플로우에 영향(광범위 회귀 표면)하고 이득이 미관 수준이라 **별도 스코프 타스크 권장**.
  findings(`task_m100_2015_stage4.md`)로 인계.
- 최종 한컴 편집기 시각 판정은 오라클 보유(Windows+한컴) 환경에서 확정 권장.

## 8. 단계별 문서

- Stage1: `working/task_m100_2015_stage1.md` (기준선 하네스 + 국소화)
- Stage2: `working/task_m100_2015_stage2.md` (근본원인 확정), `..._stage2_oracle.md` (오라클 구축)
- Stage3: `working/task_m100_2015_stage3.md` (발원지 ① 수정)
- Stage4: `working/task_m100_2015_stage4.md` (발원지 ② findings)
