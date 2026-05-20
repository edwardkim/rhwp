# 구현계획서 — #1027: 세로 측정 정합 (페이지네이터 과측정 + 박스 여백)

- 이슈: #1027 / 브랜치 `local/task1027` (base edf62865)
- 수행계획서: `task_m100_1027.md` (승인 완료)

## 코드 기준점 (사전 확인)

- TAC 표 host 측정: `src/renderer/height_measurer.rs:977~1038` (`raw_table_height` → `table_height` → `total_height = table_height + …`).
- 페이지 누적: `src/renderer/pagination/engine.rs` 의 `st.current_height += …` (단락 845/856/978, 표 part 954/1122/1184).
- 렌더러 표/Shape advance: `src/renderer/layout.rs` (TAC Shape advance ~4768, 표 layout).

페이지 8: 항목 h 합 743.6px, used=925.8px → **차 182.2px = 표 2개 host 간격 등**. 주석 21.3px 가 여유 15.3px 에 6px 초과로 거부됨.

## 단계 (5단계)

### Stage 1 — 증상 A 진단 (6px 출처 특정)
- 페이지 8 각 항목의 **페이지네이터 측정치**(HeightMeasurer total_height, engine current_height 증분)와 **렌더 실측 y**(SVG 요소 좌표)를 1:1 대조.
- pi=124(조직도 1x1 TAC, 내부표 9x8)·pi=126(추진일정 9x18 TAC)의 host_spacing/total_height 가 렌더러 advance 보다 큰지 측정 → 6px 누적 출처 확정.
- 한컴 PDF p8 의 표 바닥·주석 위치와 대조해 정답 높이 산정.
- → `working/task_m100_1027_stage1.md`

### Stage 2 — 증상 B 진단 (박스 여백 실재 판정)
- 페이지 10/11/12 박스를 `rsvg-convert` 렌더 + 한컴 PDF(필요 시 작업지시자 편집기) 정밀 대조.
- 박스(TAC Shape) 하단→다음 항목 간격을 px 정량화. 결함이면 렌더러 advance(layout.rs)·HeightMeasurer 중 어디 문제인지 특정. 미세·정상이면 B는 scope-out 기록.
- → `working/task_m100_1027_stage2.md`

### Stage 3 — 증상 A 수정 (HeightMeasurer ↔ 렌더러 정합)
- Stage 1 결과대로 TAC 표 host_spacing/total_height 과측정 보정 → 페이지네이터가 렌더러와 동일 높이 산출.
- 주석·SFR-008 이 한컴과 동일 쪽에 배치되는지 확인. 단일 측정 공간 유지.
- → `working/..._stage3.md` + 소스 커밋

### Stage 4 — 증상 B 수정 (결함 확정 시) + 골든 정리
- B가 실재 결함이면 박스 advance 보정.
- 병합본 골든(theirs) 재판정: 본 수정으로 이동한 골든을 한컴 2022 PDF 대조로 갱신, `svg_snapshot` 복구.
- → `working/..._stage4.md` + 소스 커밋

### Stage 5 — 검증 + 회귀 + 최종보고서
- 한컴 PDF 쪽 배치 대조(주석 8쪽·SFR-008 14쪽 등). 비공개 184p `LAYOUT_OVERFLOW`·페이지 수 무회귀.
- `cargo test` 전체 + `svg_snapshot` 통과. 광범위 회귀 sweep(다른 표·박스 문서).
- → `working/..._stage5.md` + `report/task_m100_1027_report.md`

## 리스크·완화
- HeightMeasurer 보정은 전 문서 페이지네이션 영향 → 각 단계 LAYOUT_OVERFLOW·페이지 수·테스트로 회귀 차단.
- A·B 별개 원인 가능 → Stage 1/2 분리 진단 후 독립 수정.
- 6px 이 표 host 가 아닌 단락 간격 누적이면 Stage 1에서 방향 재조정.

## 비범위
- #1025 page-larger 단일 셀 분할. WASM 재빌드.
