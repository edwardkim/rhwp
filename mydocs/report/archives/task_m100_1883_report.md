# Task M100 #1883 최종 보고서 — 리팩토링 계획 수립

- 이슈: #1883 (umbrella #1582) / 브랜치: `local/task1883` / 작성일: 2026-07-04

## 1. 결과

리팩토링 실행 전 관문인 **계획 수립을 완료**하고 컬래버레이터 리뷰에 부쳤다.

| 산출물 | 내용 |
|---|---|
| `mydocs/tech/investigations/issue-1883/task_m100_1883_diagnosis.md` | 현황 재진단 — 거버넌스 2축(SOLID·복잡도) 정량. **복잡도 공식 측정 = 코드 품질 대시보드**(`scripts/metrics.sh`) |
| `mydocs/plans/refactoring_plan_2026.md` | 마스터 플랜 초안 v1 — Phase 0~4, freeze 전략, 게이트 배치, 정량 성공 기준, 리뷰 안건 5 |

## 2. 핵심 진단 (영점)

- src ×2.7 성장(3.5개월) / 1,200줄 초과 .rs 70개 / **CC>25 함수 80개(최대 288)** /
  clippy 0·테스트 2,820 pass — "동작 건강, 구조 비대".
- 최우선 해체 후보(교차 검증): `typeset_section_with_variant`(7,059줄·CC 282),
  `layout_composed_paragraph`(3,771줄·CC 288).
- #1582 감사 지적 전부 실측 일치. ③차 정리 지점은 개선 유지(단계 정리 유효성 전례).

## 3. 계획 요지

Phase 0(baseline freeze, 0.8 릴리즈 자산 겸용) → 1(#1582 Provenance/Profile 분리, behavior
불변) → 2(복잡도 해체, 1 PR=1 함수 롤링) → 3(Document 축 분리, 고위험 후순위) → 4(6차 리뷰).
성공 기준: Phase 2 후 최대 CC<100, v1.0 에서 CC>25 = 0·SOLID ≥9.0·행동 회귀 0.
Feature freeze 는 Phase 단위 영역 freeze 제안(전면 freeze 대안과 함께 리뷰 안건).

## 4. 후속 (본 타스크 범위 밖)

- 컬래버레이터(@jangster77, @postmelee) 리뷰 회신 → 계획 개정(v2)·확정 → 실행 이슈 분리 등록.
- Phase 0 착수는 계획 확정 후 작업지시자 지시로.
