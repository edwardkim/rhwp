---
kind: pr-review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-18
---

# PR #5483 검토 - HWP3 개체 자리표시자 슬롯 보정

## 접수 메타데이터

| 항목 | 검토 시점 참고값 |
| --- | --- |
| PR / 작성자 | [#5483](https://github.com/edwardkim/rhwp/pull/5483) / planet6897 |
| base / contributor head | devel / `e529e59706d43d9aba17595cbfac784031aa760f` |
| 가시성 branch | `review/planet6897-20260818` |
| local cherry-pick | `54510eb99` |
| 원격 상태 | OPEN, 비 draft, MERGEABLE, BLOCKED |
| 검토 기준 | `upstream/devel@e5ef2620bd469aa2d0118097c4d04f63cfdacdc3` 위에 #5483 후 #5515 누적 |

## 변경 범위

HWP3 파서가 표·그림·도형과 쪽 번호 계열 개체를 본문 글자 `U+FFFC` 하나로만 세어,
HWP5/HWPX 저장 시 확장 컨트롤의 8유닛 슬롯 판정이 실패하던 문제를 보정했다.
개체 뒤 슬롯 폭을 파서에서 보존하고, HWP5 직렬화기가 `U+FFFC`를 자리표시자로 인식하도록
갭 전후를 판정한다. 개요번호와 비가시 컨트롤은 기존 오프셋 계약을 보존하도록 별도 처리했다.

적용된 주요 파일은 `src/parser/hwp3/mod.rs`, `src/serializer/body_text.rs`와
`tests/cases/issue_3532_trailing_charshape_boundary.rs`,
`tests/issue_3492_hwp3_outline_marker_leak.rs`,
`tests/issue_3495_endnote_space_eaten.rs`이다.

## 체리픽 및 충돌

- `upstream/devel`에서 시작한 검토 branch에 #5483의 source head를 먼저 적용했다.
- #5483 적용 과정에 충돌은 없었다.
- 뒤이어 #5515를 같은 branch에 적용했으며, #5483 contributor commit은 rewrite하지 않았다.
- 통합 branch의 변경 범위는 `upstream/devel...review/planet6897-20260818`로 확인했다.

## 검증

- `cargo fmt --all -- --check` 통과
- `node scripts/rust-test-suite-manifest.mjs --prepare` 실행 후 `--check` 통과
- `node scripts/rust-unit-test-tiers.mjs --check` 통과
- `git diff --check upstream/devel...HEAD` 통과
- `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`를 실행해 **7300 passed, 38 skipped, 8 slow**을 확인했다. 전체 실행 시간은 452.801초였고 release-test 빌드 포함 종료 시간은 15분 11초였다.

PR 본문의 HWP3 코퍼스 수치도 변경 범위와 일치했다. HWP5 저장 축의 U+FFFC 유출은
전 135자/28문서에서 후 0/0으로 줄었다. HWPX 축의 잔여 2자/2문서는 숨은 설명과 머리말의
비가시 컨트롤 축 배정 문제로 PR 본문에서 범위 밖으로 구분되어 있다.

## 남은 범위와 판정

차단 결함은 발견하지 못했다. 다만 이슈 [#4957](https://github.com/edwardkim/rhwp/issues/4957)은
아직 OPEN이며, HWPX 축의 숨은 설명·머리말 비가시 컨트롤 2건은 별도 후속 범위로 남아 있다.
이 잔여 범위는 이번 PR의 HWP5 U+FFFC 유출 수정 판정을 뒤집는 근거로 보지 않았다.

GitHub에서는 이 시점에 두 source branch 모두 required check가 보고되지 않았고 상태가
BLOCKED였다. 따라서 이 문서는 로컬 통합 검토 기록이며, 원 PR 승인·병합은 수행하지 않았다.
