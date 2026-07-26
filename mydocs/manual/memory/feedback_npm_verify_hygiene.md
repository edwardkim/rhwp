---
kind: memory
status: historical
canonical: mydocs/manual/memory/MEMORY.md
last_verified: 2026-07-26
name: feedback_npm_verify_hygiene
description: "PR 브랜치 npm 검증 위생 — npm ci 선행, untracked node_modules 정리, 신설 패키지 .gitignore, studio 재현 검증 시 dev 서버 재기동 필수 (2026-07-12 갱신)"
metadata: 
  node_type: memory
  type: feedback
---

**PR 브랜치에서 npm 검증을 돌린 뒤의 위생 루틴** (2026-07-11, PR #2174/#2188 처리 중
실사고 2건에서 확립 — 작업지시자 기록 지시).

**Why**:
- 브랜치를 오가면 node_modules 가 **직전 브랜치의 lockfile 상태로 잔류**한다 — #2188
  시각 판정 후 studio 가 TS 6.0.3(구 lockfile)으로 남아 lockfile(7.0.2)과 어긋났던 실례.
- checkout 은 tracked 파일만 제거하므로, PR 브랜치에만 존재하는 패키지 디렉터리에서
  npm ci 를 돌리면 **untracked node_modules 만 고아로 잔류**한다 — #2174 재검증 후
  `scripts/frontend-metrics/node_modules` 57MB 가 남아 "대량 미커밋 파일"로 보였던 실례.
- 이 repo 는 **루트 .gitignore 에 node_modules 패턴이 없다** — 패키지별 .gitignore
  (rhwp-studio/.gitignore 등)로 관리. 신설 패키지 디렉터리는 개별 .gitignore 동반 필수.

**How to apply**:
1. 프론트 검증은 항상 `npm ci` 선행 — lockfile 버전이 강제되어 stale 이 자동 교정된다
   (글로벌 도구 버전은 무관 — 각 패키지 로컬 tsc/vite 사용).
2. PR 브랜치에서 npm ci 를 돌렸다면, 브랜치 정리 시 `git status --short` 로 untracked
   잔류물(특히 그 PR 이 신설한 디렉터리의 node_modules)을 확인·삭제한다.
3. 패키지 디렉터리를 신설하는 PR 리뷰 시 `.gitignore`(node_modules) 동반 여부를 점검
   항목에 넣는다 (#2174 에 실제 요청한 전례).

관련: [[feedback_pr_ci_before_pr]] [[project_extension_publicdir_false]]

**추가 (2026-07-12, #2228 검증)**: studio 측 결함의 로컬 재현 검증 시 **dev 서버
재기동(또는 hard reload)이 선행**되어야 한다 — 프론트 코드가 머지된 직후에도
돌고 있는 vite dev 서버/SW 캐시는 구 번들을 서빙해 "재현 안 됨" 위음성을 만든다
(#2228: task 2217 회귀가 서버 재기동 후에야 재현). 레이어 트리를 네이티브로
덤프해 TS 게이트 로직을 시뮬레이션하면 브라우저 없이 존재 검증이 가능하다.
