---
kind: pr-review
status: accepted-with-maintainer-correction
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-26
---

# PR #6019 review - 보호 문서 평문 복구본 차단 (#5992)

## 접수 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#6019](https://github.com/edwardkim/rhwp/pull/6019) |
| 작성자 | [@RaghavShubham](https://github.com/RaghavShubham) |
| 원 head | `21e8d1d5febae6ba10ea659c7f82da5233e15f61` |
| 통합 적용 commit | `30fb8cb90` |
| GitHub 상태 | non-draft, `MERGEABLE/CLEAN`, CI 13 success/11 skip/CodeQL neutral |
| review 상태 | `CHANGES_REQUESTED` |
| 통합 판정 | **메인터너 보정 포함 수용 권고** |

## 검토 요약

보호 문서에서 복구용 자동 저장이 `exportHwp()`를 호출해 평문 draft를 IndexedDB에 남길 수 있던
보안성 결함을 fail-closed로 막는 방향은 타당하다. 원 PR은 `isRecoveryBlocked` 가드를
`exportBytes()`보다 앞에 두고, 보호 전환 전에 남아 있던 draft를 폐기하며, 차단 상태를 별도
status로 노출한다.

다만 원 head는 reviewer 지적대로 차단 상태에서 `schedule()`이 debounce를 우회해
`document-mutated`/`document-changed`마다 즉시 `flushNow()`를 호출한다. 또 자동 저장을 꺼도
차단 알림이 발생하고, `blocked` status가 `saving` 상태를 거치지 않아 상태바 복원 타이머가 걸리지
않는다. 따라서 원 PR 그대로는 보류가 맞고, 통합 후보에서 메인터너 보정으로 처리했다.

## 메인터너 보정

- `AutosaveManager.schedule()`에서 `isRecoveryBlocked()` 즉시 flush 분기를 제거해 보호 문서도 기존
  idle/recovery 스케줄러로 coalescing되게 했다.
- `recoveryEnabled=false` 및 `idleEnabled=false`이면 보호 문서 차단 알림도 예약하지 않는다.
- `handleAutosaveStatus()`의 `blocked` 경로가 현재 상태 메시지를 복원 대상으로 잡아, 차단 메시지가
  상태바를 영구 점유하지 않게 했다.
- 보호 문서 경로에서 `exportBytes()`가 호출되지 않는다는 테스트, 차단 debounce 테스트, 자동 저장
  비활성화 시 차단 알림 미예약 테스트를 추가했다.

## 로컬 검증

- `node --test rhwp-studio/tests/autosave-manager.test.ts`: 14 pass
- `npm --prefix rhwp-studio run build`: `tsc && vite build` 통과
- `cargo fmt --check`: 통과
- `cargo check --profile release-test --target-dir target/pr-review --tests`: 통과

## 권고

메인터너 보정 후 핵심 불변식인 “보호 문서에서는 평문 복구본을 만들지 않는다”가 테스트로 잠겼고,
차단 알림·draft 삭제가 입력 핫패스를 과도하게 때리지 않는다. 통합 PR에 포함해 수용 가능하다.
