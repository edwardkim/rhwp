---
kind: pr_review_plan
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-07-31
---

# kevin9327 PR #3612–#3634 통합 검토·병합 계획

## 입력과 commit 경계

`@kevin9327`의 열린 PR 12건은 서로 의존하는 MCP/CLI 기능과 문서·증적을 포함한다. 원 PR을
개별 merge하지 않고, 최신 `upstream/devel` `187086d7f0acc8d628e8433c6dc071f8a5637bae` 위의 단일
candidate에서 원 contributor commit만 순서대로 재현 적용한다. 원 source branch의 base-update merge
commit은 기능 중복을 만들지 않도록 적용하지 않는다.

| 원 PR | 원 head | 통합에 반영한 contributor commit |
| --- | --- | --- |
| #3612 | `b533740cd` | `4c374a726`, `b533740cd` |
| #3613 | `51ea48618` | `c9e7b723d`, `ac1097f2c`, `38cfe710e`, `1c3546b80`, `51ea48618` |
| #3614 | `5a2f9e5e1` | `5a2f9e5e1` |
| #3615 | `a5e06d171` | `a5e06d171` |
| #3618 | `056f4c98a` | `056f4c98a` |
| #3620 | `80cfdbb1e` | `f65b4cc02`, `80cfdbb1e` |
| #3621 | `ed2427fa7` | `ed2427fa7` |
| #3623 | `1bf5ade0e` | `f0cd20fc7`, `c5cee6173` |
| #3624 | `e2a38512e` | `e2a38512e` |
| #3631 | `538d0082b` | `538d0082b` |
| #3632 | `05d1c1ac0` | `010a96db3` |
| #3634 | `d388e807d` | `d388e807d` |

누적 적용에는 conflict가 없었다. reviewer가 발견한 integration 경계 결함은 contributor 이력을 고치지
않고 별도 `94cdd74ce`와 `208c3f618`로 보정한다.

## 검토에서 발견·보정한 경계

- `mcp-serve --profile`이 `tools/list`에서 세션 도구를 숨겨도 `tools/call`은 `hwp_open` 등을 실행할 수
  있었다. profile filter를 dispatch 경계까지 적용하고 회귀를 추가했다.
- JSON `u64`를 `u32`/`u16`으로 직접 축소하던 세션 page·table cell row/column은 wraparound 가능성이
  있었다. 명시적 `try_from`과 범위 오류로 변경했다.
- 문서가 0쪽을 보고하는 방어 경로에서는 범위 진단의 `page_count - 1`이 underflow할 수 있었다. 빈 문서
  오류를 먼저 돌려 주도록 분리했다.
- `hwp_digest` MCP schema의 선택 `maxChars`가 CLI argument로 전달되지 않았다. 메타데이터의
  `optionalArgs`를 실행 경로에서 해석하고, 실제 MCP call의 최대 길이 회귀를 추가했다.

## 검증과 시각 근거의 경계

| 검증 | 결과 |
| --- | --- |
| `cargo fmt --all -- --check` / `git diff --check` | 통과 / 이상 없음 |
| `CARGO_TARGET_DIR=target/review-kevin9327-20260731 CARGO_INCREMENTAL=0 cargo build --release` | 통과 |
| 동일 전용 target의 `cargo clippy --all-targets -- -D warnings` | 통과 |
| 동일 전용 target의 `cargo test --profile release-test --tests --quiet` | 완료까지 대기, exit code 0 |
| `python3 scripts/check_markdown_links.py --changed-from upstream/devel` | 441개 문서, 상대 링크 이상 없음 |
| `python3 scripts/check_document_metadata.py` | 428개 문서, 이상 없음 |

PR에 포함된 PNG는 CLI/MCP 작업 결과의 설명 증적이다. renderer·typeset·layout·paint·pagination 또는
golden fixture 구현은 바뀌지 않았으므로 이 통합의 독립 visual sweep 성공 근거로 확대하지 않는다. 새 세션
render/HML/split/checkbox 경로의 계약은 release-test에 포함된 Rust contract와 PR 최신 head full CI로
검증한다.

## 통합 PR과 후속 처리

1. `integrate/kevin9327-20260731`만 `devel` 대상 PR로 올린다. code/test 변경이 있으므로 fast-pass가
   아니라 이 PR의 정확한 head full CI, CodeQL, aggregate와 `MERGEABLE`을 merge 전 조건으로 둔다.
2. push 전 모든 변경 파일을 LFS attribute와 `git lfs status`로 판독한다. LFS 대상이 없을 때만
   `GIT_LFS_SKIP_PUSH=1` dry-run과 실제 push를 쓴다.
3. merge 뒤 관련 issue가 완료됐는지 원 issue 본문과 상태를 다시 판단한다. 닫을 수 있는 issue에만 통합
   merge SHA를 근거로 comment·close하고, roadmap/후속 단계 issue는 유지한다.
4. 원 PR에는 실제 LF로 된 통합 결과·감사 comment를 남긴 뒤 supersede close한다. contributor fork
   branch는 삭제하지 않는다. 마지막에 `devel` sync, 통합 remote/local branch와 정확한 review target만
   정리한다.

## 완료

통합 PR [#3647](https://github.com/edwardkim/rhwp/pull/3647)은
[`a54ff52aa8bb2ede5dfe06bb493090d74f9065ab`](https://github.com/edwardkim/rhwp/commit/a54ff52aa8bb2ede5dfe06bb493090d74f9065ab)로
2026-07-31에 merge됐다. current head `c9790a52b`와 remote head가 일치한 상태에서 CI preflight,
Lint, Native Skia, default-feature 8개 shard, Build & Test, CodeQL, Canvas visual diff가 모두
success였다(WASM·frontend는 변경 범위상 skipped). 이 review-only 기록 PR merge 뒤 관련 issue의 완료
여부와 원 PR supersede close를 실제 상태로 처리한다.

원 PR #3620의 `hwp_set_checkbox`는 문자 `□`(U+25A1) occurrence 경로만 완료한다. #3395가 추적하는
문단 글머리표 `☐`(U+2610) 의미론은 구현되지 않았으므로 해당 issue는 open으로 유지한다.
