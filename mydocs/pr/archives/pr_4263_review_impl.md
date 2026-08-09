---
kind: implementation-plan
status: completed
pr: 4263
issue: 2550
last_verified: 2026-08-08
---

# PR #4263 메인터너 보정 계획

## 기준 고정

- contributor 원 head: `6e5ee415679a440ca0ba59ff4b594a6ed2feab16`
- 검토 branch: `review/kevin9327-20260808`
- 작성 시점 최신 `upstream/devel`: `59b31e5ce1d29aa6300e777bfa682ee123fa4f98`
- contributor 원 commit은 rewrite하지 않는다. 보정 code와 회귀 test를 그 위에 별도 commit으로
  추가하고, 검토 기록·오늘할일은 최종 검증 뒤 별도 trailing commit으로 작성한다.

## 보정 목표

1. 상한 초과 lazy BinData는 어떤 저장 경로에서도 무제한 `load()`로 되돌아가지 않게 한다.
2. HWP5 source에서 저장 형태와 압축 상태가 일치할 때만 raw stream passthrough를 허용한다.
3. HWPX source의 상한 초과 BinData는 HWP/HWPX 저장 모두 정의된 빈 placeholder로 접는다.
4. HWPX resolver의 길이·빈 상태 질의도 bounded ZIP reader를 사용하게 해 DocLang과 외부 이미지
   상태 확인이 전체 entry를 해제하지 않게 한다.
5. 합성 HWPX BinData deflate bomb으로 render/query, HWPX -> HWP, HWPX -> HWPX의 상한 계약을
   회귀 시험으로 고정한다.

## 수행 결과

1. 원 contributor head `6e5ee415`와 원격 PR head가 같은 SHA이며 `maintainerCanModify=true`인 것을
   확인했다. 원 contributor commit은 rewrite하지 않았다.
2. `84e378e82`에서 HWPX ZIP entry를 시험 중 합성하고, 256MB를 초과한 비압축 payload가 central
   directory에 기록되도록 회귀 helper를 구현했다.
3. HWPX lazy BinData가 `load_limited()` 실패 뒤 `load()`로 되돌아가지 않게 CFB writer를 보정하고,
   HWPX resolver의 길이·빈 상태 질의도 bounded reader로 고정했다.
4. DocLang adapter의 선행 `is_empty()` 호출을 bounded load로 치환했다. HWP5 raw passthrough의 기존
   무손실 조건은 유지했고, HWPX에 ZIP raw bytes를 CFB stream으로 복사하지 않았다.
5. 최신 `upstream/devel`을 `c4bfc73e2` merge commit으로 반영했다. `layout.rs` 충돌은 양쪽 import를
   합쳐 해소했으며 contributor의 보안 호출과 devel의 numbering 호출을 모두 보존했다.
6. focused 6건, HWPX reader 1건, DocLang 8건, fmt, diff check, Clippy, 전체 `release-test --tests`를
   최종 merge head에서 통과했다.
7. 원격 source branch push와 GitHub CI는 작업지시자 승인 전이라 수행하지 않았다.

## rollback 경계

- 보정 commit `84e378e82` 하나만 되돌리면 contributor 원 commit `6e5ee415`로 돌아간다.
- HWP5 raw passthrough 계약은 그대로 유지하며, HWPX ZIP raw bytes를 HWP5 CFB stream으로 직접
  복사하지 않는다.
- contributor 원 commit은 rewrite하지 않았다. 충돌 해소를 위해 최신 `devel`을 별도 merge commit
  `c4bfc73e2`으로 반영했으며, 이는 사용자 승인 후에만 원격 source branch로 게시한다.
