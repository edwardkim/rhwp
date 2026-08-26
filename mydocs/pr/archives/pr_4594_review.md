---
kind: pr-review
status: local-ci-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-11
---

# PR #4594 리뷰 - Subsecond 감시 수명과 재연결 회복

## 라우팅과 접수

```text
base route: collaborator_external_pr.md
modifiers: intake_and_review.md, local_validation.md,
  multi_pr_update_branch.md
```

| 항목 | 문서 작성 시점 기록 |
| --- | --- |
| 원 PR | [#4594](https://github.com/edwardkim/rhwp/pull/4594) · @humdrum00001010 |
| 관련 이슈 | [#4579](https://github.com/edwardkim/rhwp/issues/4579) |
| 원 head | `1b7acf0c9ced58034b61dae1364a50d24e177cbc` |
| 규모 | 5 files, +565/-24 |
| 원 PR 상태 | `MERGEABLE`, `CLEAN`; Build & Test 성공, CodeQL 언어별 Analyze 성공, 집계 `CodeQL`은 `neutral` |
| 누적 적용 | `081954d42`부터 `a44dcaa83`까지 11 commits |
| 로컬 검증 후보 | rebase 전 `a08be5d1051016adb0378c40fc0010b677628c15` |
| 현재 rebase 후보 | `ed8e0387ad249cacae8edab85dd2283ea559ba21` |

## 변경 판단

Subsecond revision watcher의 재도색 callback이 예외를 던지면 감시 루프가 영구 중단될 수 있었고,
재연결 backoff도 실제 연결 성공이 아닌 준비 단계에서 초기화될 수 있었다. 이 PR은 callback 예외를
진단으로 전환해 watcher를 유지하고, 연결 성공 시점에만 backoff를 되돌린다.

또한 process lifetime 동안 회수되지 않는 patch 수를 누적해 임계값을 넘으면 개발자에게 경고한다.
시간 계산은 단조 시계와 한 경로로 정리했고, 오래된 listener·중복 예약·복구 시 revision 주석 범위도
행동 기반 테스트로 고정했다.

## 누적 통합과 완료한 검증

- #4584 다음, #4597 이전에 독립 단계로 누적 적용했다. #4597과 직접 충돌하지 않았고 누적 branch의
  source diff와 whitespace 검사가 통과했다.
- `npm --prefix rhwp-studio test`를 rebase 전 누적 후보에서 실행해 847건 통과했다. watcher 수명, 재연결 backoff,
  오류 보고와 patch 누적 경고의 Studio 회귀가 이 묶음에 포함된다.
- rebase 전 누적 후보에서 전체 nextest 5,764건 통과, fmt·clippy·release build·release lib test·Native Skia 3종과
  WASM build가 모두 통과했다.
- 원 head의 CI/Render Diff preflight와 Canvas visual diff는 성공했다. 최신 source head의 heavy Rust
  worker가 skipped인 것은 trailing 문서 변경에 따른 fast-pass 결과이며, Build & Test 집계 성공과
  CodeQL language Analyze 성공을 별도로 확인했다.

## 한계와 권고

이 PR은 개발 서버 수명 관리와 진단을 다룬다. 브라우저에서 장시간 patch를 누적해 경고 임계값과
재연결 회복을 시간 경과까지 측정하는 soak test는 실행하지 않았다. 경고는 회수를 수행하지 않고
운영자가 dev server를 재시작해야 할 시점을 알려 주는 관측 기능이다.

**최종 권고: 수용.** 감시 루프가 한 번의 재도색 오류로 멎지 않도록 하고, 실제 연결 성공에만 backoff
상태를 변경하며, 회수 불가 누적을 숨기지 않는다. merge 전에는 통합 PR 최신 head의 CI와
작업지시자 승인을 다시 확인한다.
