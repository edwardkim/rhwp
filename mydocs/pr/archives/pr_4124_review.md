---
kind: pr_review
status: accepted-with-maintainer-correction
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-07
---

# PR #4124 검토 - 교차 출처 열기 피커 폴백

## 대상과 변경 경계

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 | [#4124](https://github.com/edwardkim/rhwp/pull/4124) / @humdrum00001010 |
| contributor 원 head | `e7b58d8e179b41b96cebe24378041ef0cfca8186` |
| base / 규모 | `devel`, 1개 파일, +28/-17 |
| 관련 이슈 | [#4123](https://github.com/edwardkim/rhwp/issues/4123) |
| 작성 시점 원격 상태 | `mergeable=UNKNOWN`, `mergeStateStatus=UNKNOWN`; GitHub 재계산 중이므로 merge 전 재확인이 필요하다. |
| maintainer 수정 권한 | `maintainerCanModify=true` |
| 시각 검증 | 비대상. File System Access 열기 경로만 바꾸며 renderer, layout, paint, fixture를 변경하지 않는다. |

원 변경은 교차 출처 iframe에서 `showOpenFilePicker()`가 `SecurityError`를 던질 때 오류가 사용자 alert로
누출되던 문제를 잡고, 기존 숨김 `#file-input` 경로로 전환한다. `AbortError`는 사용자가 명시적으로
취소한 경우이므로 fallback을 다시 열지 않는 기존 계약을 유지한다.

## 메인터너 보정

원 구현은 동작상 타당하지만 실제 `file:open` 제어 흐름을 검증하는 자동 회귀가 없었다. 보정은
`src/command/file-open-picker.ts`로 picker 흐름만 분리하고, `commands/file.ts`는 기존의 저장 전 확인과
event bus 연결을 주입하는 얇은 wrapper로 유지한다. 사용자 동작과 fallback 기준은 바꾸지 않았다.

새 `tests/file-open-picker-fallback.test.ts`는 다음을 고정한다.

1. `SecurityError`면 경고 뒤 숨김 input을 정확히 한 번 열고 `skipUnsavedGuard=true`를 설정한다.
2. `AbortError`면 input·alert·경고 없이 종료한다.

## 로컬 검증

| 검증 | 결과 |
| --- | --- |
| 새 picker 회귀 | `node --test tests/file-open-picker-fallback.test.ts` 2 passed |
| Studio 단위 | `npm test` 765 passed |
| Studio 타입·번들 | `npm run build` 통과 |
| Chromium UI | Vite 개발 화면에서 `file:open`을 클릭하고 `SecurityError`를 주입했다. 숨김 input click 1회, alert 0회, `skipUnsavedGuard=true`를 확인했다. |
| 공백 검사 | `git diff --check` 통과 |

원 head의 CI preflight, Frontend unit gates, Build & Test, Render Diff preflight 및 CodeQL JavaScript/TypeScript는
성공했다. Rust·WASM 관련 job은 프론트엔드 변경으로 의도적으로 skip되었다.

## 최종 권고

**메인터너 보정 포함 수용.** 원격 head가 재계산을 마친 뒤 최신 head의 required checks,
`mergeable`, `mergeStateStatus`를 다시 확인하고, 작업지시자 승인 후에만 원격 반영 또는 병합한다.
