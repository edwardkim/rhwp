---
kind: memory
status: historical
canonical: mydocs/manual/memory/MEMORY.md
last_verified: 2026-07-17
name: project_refactor_governance
description: "리팩토링 기본 거버넌스 2원칙 — SOLID + 복잡도 (작업지시자 확정, 2026-07-04)"
metadata: 
  node_type: memory
  type: project
  originSessionId: 2560b31a-9f1c-4764-bbf1-7ba5fc27c7ce
---

rhwp 의 리팩토링 판단·평가는 **2개 기본 거버넌스**를 따르고, 실행은 **"한 번에 하지 않는다 —
3개 순차 단계 + 단계-관문"** 대원칙을 따른다 (작업지시자 확정, 2026-07-04. 선행 3회 경험 근거:
각 단계는 게이트+스냅샷+중간 재평가+승인 후 다음 착수, 병행 금지, 단계별 독립 종료 가능,
후속 단계는 직전 결과로 범위 재결정 — `plans/refactoring_plan_2026.md` §0):

1. **SOLID** — 단일책임/개방폐쇄/리스코프치환/인터페이스분리/의존성역전.
   r-code-review 시리즈가 5원칙 점수표(각 /10)로 추적해 온 축.
2. **복잡도** — 함수/모듈 크기와 복잡도. 리뷰 시리즈가 거대 함수 줄 수로 추적
   (예: `paginate_with_measured` 1,456→120줄, 4차 리뷰의 `layout_column_item` 827줄 재발견).

**Why**: 선행 리팩토링 3회([[project_1582_refactor_umbrella]])와 r-code-review 1~5차가 모두
이 두 축으로 진단·평가됐고, #1582(0.8/v1.0 umbrella) 이후 작업도 같은 기준으로 판단한다.

**공유 기준 문서 (2026-07-04)**: `mydocs/manual/solid_scoring_guide.md` — **100점 = 5원칙×20점**
앵커 채점표(20=구조 강제/16=국소/12=핫스팟/8=God Object급), rhwp 실사례 해설, 대시보드 정량
결합(CC>25 다수면 S≤16), 과거 /10 점수 ×2 환산(3차 90·4차 89·v1.0 목표 ≥90), PR 미니 채점.
복잡도 공식 측정 = `scripts/metrics.sh` 대시보드(파일 1,200줄/CC 15·25 임계).

**How to apply**:
- 리팩토링 계획서/보고서의 목표·완료 기준을 SOLID 위반 해소 + 복잡도(줄 수/함수 크기)
  감소로 정량화한다. 채점은 solid_scoring_guide 앵커를 따르고 근거(파일:함수·수치) 병기.
- 아키텍처 관련 PR 검토 시 이 두 축으로 평가 의견을 구성한다 (예: God Object 재발,
  거대 함수 신설은 두 거버넌스 위반 신호).
- 대형 메서드 재축적(4차 리뷰 패턴)을 조기 감지 — 기능 PR 이 단일 함수/파일을 비대화시키면
  복잡도 거버넌스 관점에서 지적.

관련: [[project_1582_refactor_umbrella]] [[feedback_fix_scope_check_two_paths]]
