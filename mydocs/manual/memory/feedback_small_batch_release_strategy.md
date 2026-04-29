---
name: 작은 단위 PATCH 회전 운영 철학
description: 활발한 컨트리뷰션 사이클에서 큰 묶음으로 완성도를 높이려는 시도가 위험을 키움. 작은 단위 PATCH 빠른 회전 유지
type: feedback
originSessionId: 263bacf0-af73-4169-8b90-2cc13f939a31
---
활발한 외부 컨트리뷰션 사이클에서는 **PATCH 단위 빠른 회전을 유지** 한다. 미해결 이슈를 모두 해결하고 묶어서 완성도를 높이려는 시도가 오히려 위험요소를 키우는 정황을 회피.

**Why:** 작업지시자 발언 (2026-04-29, v0.7.8 PATCH 결정 후):
> "우리의 프로젝트는 운영의 기술이 절실하게 필요한 때 입니다. 자칫 너무 많은 걸 묶어 완성도를 높이려는 행위가 위험요소를 높인다는 판단입니다."

본 결정의 배경:
- 컨트리뷰터 9명이 활발하게 PR 요청하는 상황
- 머지 회전이 빠르면 컨트리뷰터 동기 유지 + PR 간 충돌 감소
- 큰 묶음은 회귀 추적 + 롤백을 어렵게 함 (위험 누적)
- 작은 묶음은 위험 분산 + 사용자에게 변경 도달 속도 빠름

소프트웨어 엔지니어링 고전 원칙 (small batch size / continuous delivery / risk decomposition) 과 일치.

**How to apply:**

1. **MINOR/PATCH 경계 유연 적용** (v0.x 단계):
   - 신규 API 추가 + 신규 모듈도 opt-in 이고 하위 호환성 100% 라면 PATCH 로 처리 가능
   - 예: v0.7.8 — `paint` 모듈 신규 (opt-in `RHWP_RENDER_PATH=layer-svg`) + 신규 API 3건 (`getPageLayerTree`, `editor.exportHwp()`, `Paragraph::control_text_positions`) 모두 PATCH 로 묶음
   - 단 v1.0.0 진입 시점에는 정석 SemVer 적용 필요

2. **머지 결정 시 점검 항목**:
   - 미해결 이슈가 있어도 **본 사이클 머지된 PR 만으로** 릴리즈 진행 가능
   - 미해결 이슈는 다음 사이클로 이연 — "완성도 100% 후 릴리즈" 함정 회피
   - 단 회귀 (regression) 발견 시는 예외 — 회귀 발견 후 의도된 미해결은 다음 사이클로 이연 가능하나 회귀가 사용자 영향 크면 hotfix 우선

3. **CHANGELOG 명시로 보강**:
   - PATCH 라도 신규 API / 신규 기능은 CHANGELOG 에 명확히 기재
   - opt-in / 하위 호환성 정황 명시
   - 한글/영문 동시 작성 (영문 컨트리뷰터 접근성)

4. **컨트리뷰터 동기 유지가 핵심 자산**:
   - 본 사이클 (v0.7.8) 외부 PR 16건 머지 — 빠른 회전이 다음 사이클 PR 유발
   - 머지 지연 → 컨트리뷰터 이탈 위험 → 프로젝트 정체

5. **반대 함정** (회피 대상):
   - "완성도 높여서 릴리즈" — 큰 묶음은 회귀 추적 어려움
   - "이슈 다 해결하고 릴리즈" — 미해결 이슈는 항상 존재. "다음 사이클" 정책 명시 필요
   - "MINOR 만들어서 한 번에" — v0.x 단계에서 SemVer 경직 적용은 운영 부담

**관련 메모리**:
- `feedback_release_sync_check.md` — 릴리즈 작업 전 main 동기화 점검
- `feedback_release_manual_required.md` — 릴리즈 매뉴얼 정독 필수
- `feedback_v076_regression_origin.md` — 큰 묶음 (v0.7.6) 의 회귀 누적 정황

**관련 정황**:
- `mydocs/manual/publish_guide.md` 의 SemVer 정의는 정석을 명시하나 본 운영 정책은 미명시 — 매뉴얼 갱신 후보
- v0.7.6 / v0.7.7 / v0.7.8 모두 PATCH 회전 (3 사이클 연속) — 본 정책의 일관성 정황
