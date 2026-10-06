# 구현 계획서 — #1658 통합 content-height(vpos) 회계 (라운드 3)

- 브랜치: local/task1658-vpos (upstream/devel `30931679`) / 수행계획서: `task_m100_1658_v2.md`
- 통제 게이트(전 단계 무회귀): valign offset / 클리핑(controlset 92) / 페이지수 소형 75·대형 442 /
  lib(#1488 등) / hwpx_roundtrip 4.

## Stage 1 — 측정 정합 진단 (소스 무변경, 진단 계측만)
over-count 3종 + 가드 1종 셀에서 `total_content_height` 3개 항을 계측:
`stored last_seg_end(px)` / `calc_composed_paras_content_height` / `calc_nested_controls_bottom_height`
+ `inner_height`, `text_y_start`. env-gated `eprintln` 디버그(RHWP_VALIGN_DBG).
- 산출: 어느 항이 over-count 의 원인인지, stored 대비 얼마나 과대인지, 한컴 기대(FIX offset 역산)와 대조.
- 가드(under-count): 어느 항이 stored 과소를 보정하는지(제거 시 #44 회귀 지점) 확인.
- **완료기준**: over 원인 항 + #44 보정 항을 정량 확정 → Stage 2 회계식 도출.

## Stage 2 — total_content_height 회계 재설계 + 구현
Stage 1 결과로 `max(stored, computed...)` 를 **stored last_seg_end 권위 + 중첩 컨트롤이 stored 바닥
초과 시에만 초과분 보정** 회계로 교체(naive max 제거). `table_cell_content.rs:650-676`.
- **완료기준**: 빌드 통과 + over 3종 offset 정상(중앙/하단) + 가드 불변(로컬 valign 게이트).

## Stage 3 — (A) 전 통제 게이트 무회귀 검증
valign 게이트(over 3종 FIX + 가드 불변) + 클리핑 92 + 소형 75 + 대형 442 + lib(#1488) + roundtrip 4.
- **완료기준**: 6게이트 전부 무회귀. 회귀 시 회계식 재조정 또는 분리.

## Stage 4 — (B) block-continuation 연계 분석/적용
Stage 2 회계 원리를 별표4 연속분 블록의 선행 행 content-height 측정(render full ↔ pagination
remainder ~70px)에 적용 가능한지 분석. 가능·저위험이면 적용(클리핑 23.5px↓), 불가/고위험이면
분리 보고(안전 우선, 별표4 ceiling 유지).
- **완료기준**: 적용 시 클리핑↓ + 양 게이트 무회귀 / 미적용 시 사유·후속 문서화.

## Stage 5 — 최종 검증·보고
전 게이트 재확인, 합성 fixture 회귀 게이트 상시 편입, 최종 보고서(`report/`).
- **완료기준**: 최종 보고서 + 오늘할일 갱신, git status 클린.

> 각 단계 완료 후 단계별 완료보고서(`working/task_m100_1658_v2_stage{N}.md`) + 승인 요청.
> 소스/보고서 커밋은 해당 단계 타스크 브랜치 커밋.
