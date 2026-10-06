# Task M100 #1904 최종 보고서 — 1차 리팩토링 라운드 1 (복잡도 해체)

- 이슈: #1904 (계획 #1883, umbrella #1582) / 브랜치: `local/task1904` / 작성일: 2026-07-05

## 1. 결과 요약

| 단계 | 성과 |
|---|---|
| Phase 0 freeze | PR inventory **0건**(최적 시점) + baseline manifest(`00014ecf`) + **OVR baseline 5샘플** 커밋 |
| object_ops 분할 | 9,845줄·7도메인 응집 → **8모듈** (함수 이동만, 외부 인터페이스 무변경) |
| typeset 추출 1 | `typeset_section_with_variant` **7,059→1,555줄 / CC 282→104** — 함수의 78%였던 미주 배치(분기 0회)를 `typeset_section_endnotes`(5,554줄·CC 179)로 분리 |
| 재평가 | 대시보드 2차 스냅샷(`2026-07-04-r1`, 추이 첫 가동) — typeset 이 CC 1위에서 내려옴 |
| **행동 회귀** | **0건** — 모든 단계 게이트 전수 통과(전체 테스트 2,858·OVR 5샘플·golden·roundtrip) |

## 2. 원칙 준수 기록

- 복잡도 높은 순 × 위험 낮은 순(작업지시자 확정) — 최저위험(모듈 분할)으로 게이트 리허설 후
  최대 복잡도(typeset) 공략. 소스-포맷 분기 비접촉 유지(추출 블록 분기 0회).
- **stage-gate 규칙 3 적용 1회**: 추출 2 후보(4,933줄 body)가 외부 의존 32개로
  `EndnoteFlowState` struct 설계 선행 필요 판명 → 무리하지 않고 축소, 다음 라운드 설계 입력화.
- jangster77 보완 1·2(PR inventory, manifest) 실행 선반영. 분할 과도기 지표(+1/+2)가
  보완 3(산식 고정) 필요성을 실증 — v2 반영 근거.

## 3. 다음 라운드 제안 (관문 통과 후 별도 이슈)

① `layout_composed_paragraph`(CC 288, 현 1위) ② `typeset_section_endnotes` 내부 분해
(EndnoteFlowState 설계) ③ `parse_paragraph_list`(CC 234) ④ metrics.sh 스냅샷 라벨 개선.

## 4. 산출물

manifest·OVR baseline·r1 스냅샷·stage1~4 보고서·본 보고서 + 소스 2커밋(분할·추출).
