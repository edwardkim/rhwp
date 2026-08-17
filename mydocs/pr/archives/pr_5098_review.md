---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-17
---

# PR #5098 검토 - vello 0.9.0

## 접수와 범위

| 항목 | 기록 |
| --- | --- |
| PR | [#5098](https://github.com/edwardkim/rhwp/pull/5098) |
| 작성자 / source | `app/dependabot` / `dependabot/cargo/devel/vello-0.9.0` |
| 원 source head | `71c4434c99b355cbbb4da1c7bb585a46efb5b443` |
| 기준 / 규모 | `devel`, 2 files, +423 / -232 |
| 원 PR 상태 | 작성 시점 `MERGEABLE` / `CLEAN` |
| 통합 PR | [#5186](https://github.com/edwardkim/rhwp/pull/5186) |

GPU renderer 의존성을 0.5.1에서 0.9.0으로 갱신한다.

## 통합 적용과 검증

원 SHA를 `cf090d85cb6d442eff1f9865192b03848a242a17`로 적용하고 `wgpu` 29의 `InstanceDescriptor`, 비동기 adapter
선택, `DeviceDescriptor`, `PollType` API로 renderer 초기화를 이행했다.

- GPU feature check·clippy 및 `export-png-gpu` smoke(199×281 PNG)를 통과했다.
- full release-test nextest는 6,522 passed, 38 skipped였다.
- #5186 code candidate의 Canvas visual diff, Native Skia, CI·CodeQL이 성공했다.

## 판단

renderer 경로의 API 호환성과 실제 GPU output을 확인했다. **통합 수용 권고.**
