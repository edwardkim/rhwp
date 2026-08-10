---
kind: pr-review
status: merged
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-11
---

# PR #4517 검토 - 76076 표 분할과 중첩 셀 편집 보정

| 항목 | 값 |
| --- | --- |
| PR / 작성자 | [#4517](https://github.com/edwardkim/rhwp/pull/4517) / `jangster77` |
| 관련 이슈 | [#3820](https://github.com/edwardkim/rhwp/issues/3820), 상태 `OPEN` 유지 |
| base / source head | `devel` / `cd25076700eef8f9b9cccda05d8b0675c84eff4e` |
| 병합 | `04520749b51b8ce8a10ccfb7b33516a87c3ce041`, 2026-08-10 |
| 접수 참고 상태 | MERGEABLE / CLEAN, 475 files, +72,172 / -717 |

## 경로와 범위

- base route: `collaborator_self_merge.md`
- modifiers: `intake_and_review.md`, `local_validation.md`,
  `visual_fixture_evidence.md`, `review_only_fast_pass.md`, `post_merge.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, 위 기본·보조 경로 문서

76076의 중첩·분할 표에서 다음 쪽 문단 소유와 테두리 경계를 보정했다. Studio는 중첩 셀
블록의 글자 서식을 path 기반 API와 snapshot undo 경로로 연결했고, 최근 파일이 이동·삭제된
경우에는 `NotFoundError`를 정상적인 목록 정리 경로로 처리한다. mutation-routing 원장은 이관된
직접 호출 수에 맞췄다.

## 검증과 판정

**병합 완료.** source head의 GitHub CI에서 Build & Test, CodeQL, Render Diff, Native Skia,
Lint와 frontend package gate가 성공했다. 로컬에서는 아래 검증이 통과한 결과를 기록했다.

~~~bash
wasm-pack build --target web --out-dir pkg
cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast
npm test  # 835 pass, 0 fail, 1 skipped
~~~

renderer 변경의 시각 증적은 원 PR에 포함된 `mydocs/pr/assets/task_m100_3820_stage106`부터
`stage111` 자료로 보존한다. 이번 병합은 확인된 76076 표 분할과 중첩 셀 편집 경로만 다룬다.

## 잔여 범위

[#3820](https://github.com/edwardkim/rhwp/issues/3820)은 닫지 않는다. 17쪽 HWPX의 5쪽에서
개체 주변 본문 흐름이 기준 PDF와 아직 다르며, 병합 후
[후속 기록](https://github.com/edwardkim/rhwp/issues/3820#issuecomment-5242666248)으로 열린 상태와
추적 범위를 남겼다.

이 문서와 오늘할일은 원 코드 PR 누락을 보완하는 문서 전용 fast-pass PR에서 별도로 반영한다.
