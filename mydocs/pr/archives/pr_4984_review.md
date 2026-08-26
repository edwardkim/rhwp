---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-17
---

# PR #4984 검토 - HWP3 개방 거부 계약 누적 보완

## 접수

| 항목 | 기록 |
| --- | --- |
| PR | [#4984](https://github.com/edwardkim/rhwp/pull/4984) |
| 작성자 / source | `planet6897` / `fix/4367-hwp3-marker-consume` |
| 원 source head | `9281e6f7abe4b340c2b75535ce16adda4f4ad010` |
| 기준 | `devel` |
| 통합 검토 branch | `review/planet6897-20260817` |
| 원 PR 상태 | 작성 시점 `OPEN` / `DIRTY`; required checks와 CodeQL은 성공 또는 해당 없음 |
| 규모 | 8 files, +350 / -9 |
| 관련 이슈 | #4367 (작성 시점 `OPEN`) |

원 PR의 최초 계약뿐 아니라 이후 contributor가 같은 PR에 추가한 다섯 번째·여섯 번째 계약도 현재
head에 포함되어 있다. 따라서 초기 PR 본문만 기준으로 축소하지 않고 `9281e6f7` 전체를 검토 대상으로
삼았다.

## 변경 범위

- HWP3 사각형 글상자의 storage 필드·회전 중심·꼭짓점·최대 폭을 HWP5 저장 계약에 맞게 보정한다.
- HWP3 수식의 크기를 읽고 EQEDIT 글꼴·크기·baseline을 정규화한다.
- 다각형과 닫힌 다각형의 점 배열을 IR에 보존한다.
- 표 셀을 `(row, col)` 행 우선으로 정렬하고, 글상자 없는 사각형에는 글상자 storage bit를 세우지 않는다.
- 회귀 fixture, integration test, generated manifest와 unit-tier manifest를 추가·갱신한다.

PR 내부의 `Merge remote-tracking branch 'stream/devel'` commit은 통합 검토에서 재적용하지 않았다.
공통 선행 변경은 통합 branch에서 필요한 고유 commit으로만 누적했다.

## 로컬 적용과 검증

`upstream/devel@d4cf27eeb`에서 `review/planet6897-20260817`을 만들고, #4984의 기능·테스트 commit을
순서대로 적용했다. #5136과 #5172의 변경을 함께 누적한 최종 통합 head에서 다음을 실행했다.

- `cargo fmt --all -- --check` 통과
- `node scripts/rust-test-suite-manifest.mjs --check` 통과
- `node scripts/rust-unit-test-tiers.mjs --check` 통과
- `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`
  통과: **6538 passed, 38 skipped, 8 slow**
- `cargo clippy --all-targets --target-dir target/pr-review -- -D warnings` 통과
- `git diff --check` 통과
- 신규 `issue_4367_hwp3_convert_fourth_contract` focused test: 7 passed

HWP3 parser·serializer와 저장 fixture가 변경되므로 Rust 회귀 검증을 생략하지 않았다. renderer나
Studio UI 변경은 없어 WASM·브라우저 시각 검증은 적용하지 않았다. 원 PR의 한컴 COM 실측은 source PR에
기록된 근거로 확인했으며, 이 Linux 검토 서버에서는 COM을 재실행하지 않았다.

## 판단

현재 통합 head에서 차단 결함과 추가 메인터너 코드 보정 필요 사항은 발견하지 못했다. 단, 원 PR은
작성 시점에 GitHub에서 `DIRTY`이므로 최종 통합 PR을 만들 때 최신 source head·mergeable·required
checks를 다시 확인해야 한다. **로컬 통합 수용 권고.**
