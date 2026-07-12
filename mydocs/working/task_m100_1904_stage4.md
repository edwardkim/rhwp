# Task M100 #1904 4단계 완료보고서 — 중간 재평가 (라운드 1)

- 이슈: #1904 / 브랜치: `local/task1904` / 작성일: 2026-07-05 / 단계: 4/4

## 재평가 (대시보드 2차 스냅샷 `mydocs/metrics/2026-07-04-r1/` — 추이 첫 가동)

| 지표 | 영점 | 재평가 | 해석 |
|---|---|---|---|
| `typeset_section_with_variant` CC | **282** | **104** | −63%, 전체 1위 자리에서 내려옴 |
| `typeset_section_endnotes` (분리 신규) | — | 179 | 책임 분리 — 다음 라운드 분해 대상 |
| 최대 CC (전체) | 288 | 288 | 1위는 layout_composed_paragraph — 다음 라운드 대상 |
| CC>25 / 1,200줄 초과 .rs | 80 / 70 | 81 / 72 | **분할 과도기 개수 증가** (총량 아님) — v2 산식 고정 필요성 실증 |
| 테스트 | 2,820 / 0f | **2,858 / 0f** | +38 (분할 검증 포함) |

## 라운드 1 종합

- Phase 0 freeze(manifest+OVR baseline) → object_ops 8모듈 분할(−62%) → typeset 미주 분리
  (−78%) — **모든 단계 게이트 전수 통과, 행동 회귀 0**.
- 축소 결정 1회(stage-gate 규칙 3): 추출 2 후보가 의존 32개 → `EndnoteFlowState` struct
  설계 선행 필요 판명 — 다음 라운드 설계 입력으로 기록.

## 다음 라운드 입력 (재평가 제안)

1. `layout_composed_paragraph`(3,771줄·CC 288 — 현 1위) 정찰·해체.
2. `typeset_section_endnotes`(CC 179) 내부 분해 — `EndnoteFlowState` struct 설계 포함.
3. `parse_paragraph_list`(HWP3, CC 234).
4. 운영 개선: metrics.sh `--snapshot` 같은 날짜 덮어쓰기 방지(라벨 접미어) — 이번에 영점
   복원+`-r1` 분리로 우회, 스크립트 개선은 별도 소타스크.
