# 구현계획서 v2 (재설계) — #2019 부동 폼 앵커 통합 게이트

- 이슈 #2019 / 브랜치 `fix/2019-through-wrap-overlay-vpos`
- Stage2 findings(`task_m100_2019_stage2.md`)에서 3층 메커니즘 규명. 본 v2는 **단일 술어로 3층을 통합 무력화**하는 재설계.

## 핵심 통찰

부동 폼(별지 서식)의 stored LINE_SEG vpos 는 **섹션 절대 좌표(≈17p 캔버스)**를 인코딩한다.
이 문단들은 흐름 콘텐츠가 아니라 Paper/Para 절대위치 개체의 앵커이므로, **흐름 footprint 가 0**
이어야 한다. rhwp 는 이를 흐름으로 취급해 3경로에서 과분할:

| 층 | 지점 | 현상 | 게이트 수정 |
|----|------|------|------------|
| ① 높이 | `format_paragraph` total_height (typeset.rs ~10145) | stored line_height=개체높이를 흐름 예약 | 빈문단 fallback 로 대체 |
| ② 단나누기 | 단나누기 핸들러 (typeset.rs ~2271) | 부동앵커 단나누기→단일단 페이지분할 | advance 억제 |
| ③ zone 오프셋 | `vpos_zone_height` (typeset.rs ~14019) | 절대 vpos(2204px)를 zone 오프셋→candidate_offset>page→push_new_page/zone전환 | max_vpos_end 대신 st.current_height 사용 |

**공통 술어** `para_is_floating_overlay_anchor(para)` (layout.rs 신설, Stage2 검증):
빈 텍스트 + 전 컨트롤이 부동 비-TAC (Shape/Picture: 통과/글앞/글뒤; Table: +어울림).

## 단계

### Stage 2b — 통합 게이트 구현 + 74312 수렴
- layout.rs: `para_is_floating_overlay_anchor` 신설(Shape/Picture 통과·글앞·글뒤, Table 어울림 포함, tac=false, 빈텍스트).
- ① format_paragraph: 게이트 시 line_heights 를 `empty_paragraph_fallback_line_metrics` 로 대체.
- ② 단나누기 핸들러: 게이트 시 advance 억제.
- ③ vpos_zone_height: 이전 zone 마지막 문단이 게이트 대상이면 `max_vpos_end` 대신 `st.current_height`.
- 목표: **74312 rhwp 81p → 18p(±2)** + export-png 서식 격자 정상 렌더 시각확인(한글 PDF 대조).

### Stage 3 — 광범위 무회귀 계측
- Stage1 baseline.tsv 80문서 페이지수 **불변** 검증(변동 시 개별 한글 오라클 판정).
- **다단(multi-column) 문서 표적 회귀**: process_multicolumn_break 는 shortcut.hwp 등 다단 문서
  핵심 경로 → shortcut.hwp/hwpspec.hwp/exam 등 다단 샘플 페이지수·시각 불변 확인 필수.
- #2004/#2015 재현 파일 불변.

### Stage 4 — 회귀테스트 + 최종검증
- `tests/issue_2019_floating_form_overpagination.rs`: 74312 페이지수 assert(≤20) + 다단 무회귀 1~2.
- `cargo test`(renderer/document_core + baseline 4/4) 그린. roundtrip 무결.
- 최종보고서 `task_m100_2019_report.md`.

## 위험/완화

| 위험 | 완화 |
|------|------|
| ③ vpos_zone_height 변경이 정상 다단 문서 회귀 | 게이트를 floating-form-anchor 로 한정 + 다단 샘플 표적 회귀 |
| Table 에 Square 포함이 광범위 영향 | Table 한정(Shape/Picture 는 Square 제외) |
| format_paragraph 게이트가 다른 부동앵커 문서 변경 | baseline 80문서 + 다단 샘플 계측 |
| stale 바이너리 | rebuild 전 rm |

## 승인 요청
위 통합 게이트 재설계(Stage2b~4)로 진행 승인 요청합니다. 승인 시 Stage2b 구현→74312 수렴 확인→보고.
