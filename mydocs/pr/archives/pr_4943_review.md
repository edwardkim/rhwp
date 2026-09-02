---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-16
---

# PR #4943 검토 - 본문 raw 캐시 출처 봉인

## 접수

| 항목 | 기록 |
| --- | --- |
| PR | [#4943](https://github.com/edwardkim/rhwp/pull/4943) |
| 작성자 / source | @planet6897 / `fix/4488-section-provenance` |
| 원 source head | `c2b5042c30fd8a99ec76d3c80325abe55e739b08` |
| 기준 devel | `76e407b127c261427854172990bde6b2e1793edf` |
| 가시성 검토 branch | `review/planet6897-20260816-r6` |
| local 적용 commit | `ce5a99ba28c2223e1303c6eae7f21e61aae53ba8` |
| 원 PR 상태 참고값 | `CONFLICTING` / `DIRTY`; 원 source의 오래된 base 상태이며 최신 누적 후보의 적용에는 충돌이 없었음 |

원 source의 선행 DocInfo provenance commit은 이미 `upstream/devel`에 반영되어 cherry-pick 시 빈 적용이 됐다.
남은 본문 Section·하위 컨트롤 raw 출처 봉인 commit만 누적 후보에 적용했다. 공개 IR 변경 뒤에도
오래된 raw bytes가 serializer를 우회하지 않도록 Section, Table, Equation, OLE 경계를 각각 봉인한다.

## 검증

| 범위 | 명령 또는 근거 | 결과 |
| --- | --- | --- |
| 본문 provenance | `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --test issue_4488_4495_body_provenance --test-threads 12 --no-fail-fast` | 6 passed |
| 누적 전체 Rust | `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast` | 6,514 passed, 38 skipped, 7 slow, 378.554초 |
| 품질 | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `git diff --check upstream/devel...HEAD` | 통과 |
| 원 source CI | 최신 head의 Build & Test, Native Skia, Canvas visual diff, CodeQL | 모두 성공 |

변경은 저장 전 raw 재사용의 허용 여부를 결정하는 모델·serializer 경계이며 renderer/paint 동작을 변경하지
않는다. 따라서 별도 PDF pixel sweep은 적용하지 않았고, provenance 전용 회귀와 누적 전체 baseline으로
출력 경계를 확인했다.

## 판단

원 PR의 GitHub 충돌 표시는 오래된 기준선에 대한 상태다. 최신 `devel` 위 누적 후보에서는 clean 적용과
전용·전체 검증이 모두 통과했으며, 추가 메인터너 보정은 필요하지 않았다.
**통합 수용 권고.**
