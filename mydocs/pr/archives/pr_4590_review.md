---
kind: pr-review
status: local-ci-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-11
---

# PR #4590 리뷰 - Subsecond 실패 진단과 플랫폼 경계

## 라우팅과 접수

```text
base route: collaborator_external_pr.md
modifiers: intake_and_review.md, local_validation.md,
  multi_pr_update_branch.md, rework_and_exceptions.md
```

| 항목 | 문서 작성 시점 기록 |
| --- | --- |
| 원 PR | [#4590](https://github.com/edwardkim/rhwp/pull/4590) · @humdrum00001010 |
| 관련 이슈 | [#4578](https://github.com/edwardkim/rhwp/issues/4578) |
| 원 head | `764bcd64b705356fe1d18537405e7fb455521921` |
| 규모 | 7 files, +766/-23 |
| 원 PR 상태 | `MERGEABLE`, `CLEAN`; Build & Test 성공, CodeQL 언어별 Analyze 성공, 집계 `CodeQL`은 `neutral` |
| 누적 적용 | `72e7473d1`, `b37ced8e0`, `946255b60`, `756ceaa35`, `2d60cb008` |
| 메인터너 보정 | `ed8e0387a` - Node 검증에서 브라우저 전역 객체가 없을 때 기본 진단을 no-op으로 처리 |
| 로컬 검증 후보 | rebase 전 `a08be5d1051016adb0378c40fc0010b677628c15` |
| 현재 rebase 후보 | `ed8e0387ad249cacae8edab85dd2283ea559ba21` |

## 변경 판단

wasm32의 `apply_patch`는 fetch·compile·instantiate를 비동기로 시작한 뒤 즉시 반환하므로, 실제
적용 성공을 `bool`로 보고할 수 없다. 이 PR은 결과를 `DevtoolsMessageOutcome` 코드로 바꾸고,
성공 표기를 `patch-dispatched`로 제한해 "전달됨"과 "적용됨"을 구분한다.

Studio runtime은 패치 전달 뒤의 전역 `error`와 `unhandledrejection`을 관찰해 Rust panic, Network의
patch wasm, `dx serve` 로그라는 다음 진단 지점을 연결한다. Unix 구현 조건의 rlib 심링크 생성 실패도
`cargo:warning`으로 드러낸다. 사용자 문서는 Linux·macOS·WSL 지원과 Windows 네이티브 제한으로
정정돼 있다.

## 메인터너 보정과 완료한 검증

누적 검토에서 `subsecond-runtime.ts`의 기본 진단이 Node test runtime에서도 `window`을 직접
참조하는 결함을 발견했다. `ed8e0387a`는 브라우저에서는 기존 전역 오류 listener를 유지하고,
Node에서는 기본 report를 no-op으로 바꿔 테스트 환경 오류만 제거한다. 패치 전달·오류 분류·브라우저
진단 의미는 바꾸지 않는다.

- `npx tsc --noEmit`과 `npm --prefix rhwp-studio test`를 rebase 전 누적 후보에서 실행해 각각 통과했고,
  후자는 847건 통과했다.
- rebase 전 누적 후보에서 전체 nextest 5,764건 통과, `cargo fmt --check`, clippy, release build, release lib
  test, Native Skia 3종, WASM build가 모두 통과했다.
- 원 head의 CI preflight·CodeQL preflight·Render Diff preflight는 성공했다. 최신 source head에서
  CodeQL 집계가 `neutral`인 것은 language Analyze 세 job 성공과 구분해 기록하며, 실패나 skip을
  성공으로 취급하지 않는다.
- production Studio bundle에 Subsecond 설명 문자열이 남지 않도록 하는 source 회귀와 runtime 행동
  회귀는 Studio test 묶음에서 통과했다.

## 한계와 권고

이 변경은 Linux·macOS·WSL 개발 경로의 실패 관측성을 높이는 것이며 Windows 네이티브에서 hot-patch를
가능하게 만들지는 않는다. 실제 patch wasm의 비동기 instantiate 성공 여부는 반환 코드가 아니라
브라우저 오류와 Network/`dx serve` 신호로 판정해야 한다.

**최종 권고: 수용.** 정직한 결과 코드 계약, Node/브라우저 환경 분리, 플랫폼 제약 문서화가 함께
갖춰졌고 누적 검증이 통과했다. merge 전에는 통합 PR 최신 head의 CI와 작업지시자 승인을 다시 확인한다.
