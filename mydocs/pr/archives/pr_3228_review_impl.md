---
kind: reference
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-24
---

# PR #3228 통합 실행 계획 — lpaiu-cs 머리말/undo 묶음

## 목적과 범위

원 PR들이 모두 `BEHIND`여서 각각 update branch와 CI를 반복하지 않고, 최신 `upstream/devel`
`c8611dd84d002d2a776c040387bf21cf270f6448` 위 `review/lpaiu-cs-20260724`에서 수용 가능 변경만
누적 검토한다. 대상은 #3213, #3223, #3228, #3231, #3240이며 원 PR별 판단은 각각의 review 문서에
분리한다.

## 적용 순서와 롤백 단위

1. #3228 `c21a65063`, `cb5aaec88` — 머리말 편집 대상 질의
2. #3231 `5f70c9ab4` — Studio 선택 삭제 snapshot
3. #3240 `4253b1c53`, `f5bb1db2` — renderer 활성 머리말 선택
4. #3213 `8c34d63dd` + maintainer `0cab08c80` — HF field undo 모델 offset
5. #3223 `05b4dd807` + maintainer `0cf1f98f6` — square-OLE merge undo metadata

각 기능 원 커밋은 `-x` 체리픽으로 원 SHA를 보존했다. #3223은 `src/wasm_api.rs` import 충돌만
양쪽 import를 보존해 해소했고, 동작 충돌은 없었다. #2370은 Draft이므로 포함하지 않는다. 통합 PR에서
회귀가 발견되면 마지막 독립 기능 단위부터 revert할 수 있으며, 원 contributor branch를 직접 변경하지
않는다.

## 완료 조건

review 문서·#3240 실제 PNG 자산·오늘할일을 같은 통합 PR diff에 넣고, 최신 GitHub Actions 성공 및
작업지시자 승인 뒤에만 merge한다. 원 PR들에는 통합 완료 뒤 원 PR별 감사·supersede 판단을 남긴다.
