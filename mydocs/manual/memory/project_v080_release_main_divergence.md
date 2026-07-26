---
kind: memory
status: historical
canonical: mydocs/manual/memory/MEMORY.md
last_verified: 2026-07-26
name: v0-8-0-release-main-devel
description: "v0.8.0 릴리즈 PR #3328은 devel이 아닌 release/v0.8.0-main-sync가 head — main 문서 분기 해소 merge 포함, 반드시 merge commit 방식(squash 금지)"
metadata: 
  node_type: memory
  type: project
---

v0.8.0 릴리즈(2026-07-26, #3326)에서 devel→main 직접 PR이 불가능했다. v0.7.19 직후 main에
직접 반영된 문서 커밋(#2331~#2335, `Merge local/devel` 계열)으로 main↔devel 이력이 분기,
문서 20파일 충돌 발생.

**분석 결과 (안전성 증명)**: 충돌 20파일 + main측 비충돌 변경 전부, main측 blob이 devel
이력에 존재하는 과거본(`git log --find-object` 전수 대조) — devel이 내용상 엄격한 superset.
`origin/devel` + `origin/main` merge를 devel측으로 해소한 결과 트리가 **origin/devel 트리와
tree hash 동일(361a0b30)** — 내용 유실 위험 0.

**해소**: `release/v0.8.0-main-sync` = devel(8bb8f277d) + merge origin/main(충돌 devel측
해소, 44c3aa17a) → PR #3328 (base main).

**제약**:
- PR #3328은 **반드시 merge commit 방식으로 merge** — squash 시 ancestry 소실로 분기 재발.
- merge 후에는 main이 devel을 ancestor로 포함 → 이후 릴리즈는 clean merge 복귀.
- merge 보류 사유: 릴리즈 중 분기 발견 시 작업지시자 결정 후 진행 원칙([[릴리즈 작업 전 main 동기화 점검 필수]]).

**교훈**: 릴리즈 사이에 main에 직접 문서 merge(`local/devel` 경유)를 하면 다음 릴리즈
devel→main merge가 충돌한다. main 직접 반영분은 devel에도 동일 커밋(또는 상위 내용)이
들어가야 한다.
