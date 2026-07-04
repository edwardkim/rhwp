# PR #1495 검토 기록 - 기본 미주 rewind 문단 단 overflow 보정

## PR 메타

| 항목 | 내용 |
|---|---|
| 번호 | #1495 |
| 제목 | `fix: keep reset rewind endnote off full column` |
| 작성자 | `snvtac` |
| 작성자 상태 | 저장소 PR 목록 기준 first-time contributor |
| base | `devel` |
| head | `snvtac/1375-endnote-rewind-column-overflow` |
| 관련 이슈 | `closes #1375` |
| 규모 | 문서 작성 시점 참고값: 2 files, +136 / -6 |
| head SHA | 문서 작성 시점 참고값: `fb60fba979ac698cb559635865a6c2b829b2219e` |
| maintainer can modify | 문서 작성 시점 참고값: true |
| merge 상태 | 문서 작성 시점 참고값: BEHIND, 최신 `devel` merge commit 추가 후 재확인 필요 |
| CI 상태 | 문서 작성 시점 참고값: 원 PR head 기준 GitHub Actions 통과, 문서/merge commit push 후 최신 head 기준 재확인 필요 |

`draft`, `mergeable`, `head SHA`, `CI 상태`는 변하는 값이므로 확정 사실로 기록하지 않는다.
최종 merge 판단은 PR head 최신 커밋 기준 GitHub Actions 통과와 작업지시자 승인 후에만 수행한다.

## 관련 이슈 요약

#1375는 `samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwp`에서 기본 미주 간격 문서의
내부 vpos reset/rewind 문단이 거의 찬 왼쪽 단에 head/tail로 split되어 프레임 밖으로 렌더되는 문제다.

확인된 핵심 증상은 다음과 같다.

- page 17: pi=894 rewind 문단의 앞 3줄이 왼쪽 단 하단에서 프레임을 넘는다.
- page 22: pi=1175 tail이 본문 프레임 밖으로 밀릴 수 있다.
- 기대 동작은 saved-vpos 압축 높이만 보고 현재 단에 쪼개 넣지 않고, 실제 render tree 기준으로 안전한 단/쪽에 배치하는 것이다.

## 변경 범위 분석

- `src/renderer/typeset.rs`
  - 기본 미주 간격, 보이는 구분선, compact endnote profile, 내부 vpos rewind/reset, 다단 문서, 현재 단이 거의 찬 상황으로 조건을 제한했다.
  - `en_fit` 기준으로는 현재 단에 들어가지만 실제 `total_advance_fit` 기준으로는 frame을 넘는 경우를 `internal_rewind_full_advance_needed`로 분리했다.
  - 해당 경우 `advance_for_fit`가 split 후보보다 우선하도록 하여 문단을 head/tail로 쪼개지 않고 다음 단에서 전체 문단으로 다시 시작하게 했다.
  - debug 로그에 `rewind_full_advance` 필드를 추가했다.
- `tests/issue_1375_endnote_rewind_column_overflow.rs`
  - page 17에서 pi=894가 왼쪽 단 partial paragraph로 남지 않고 오른쪽 단 top band에 전체 문단으로 렌더되는지 검증한다.
  - page 22/page 23에서 pi=1175 split이 render-safe 범위로 유지되고 page 22 tail bottom이 body frame 안에 남는지 검증한다.

## 로컬 검증 결과

검증은 2026-06-24 KST 기준 최신 `upstream/devel`(`04b58a4b`) 위에 PR head를 병합한 임시 worktree에서 수행했다.

- 최신 `devel` merge 시뮬레이션: 충돌 없음
- `cargo fmt --all -- --check`: passed
- `git diff --check`: passed
- `cargo clippy --lib -- -D warnings`: passed
- `cargo test --test issue_1375_endnote_rewind_column_overflow -- --nocapture`: passed, 2 passed
- `cargo test --test issue_1082_endnote_multicolumn_drift -- --nocapture`: passed, 5 passed
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`: passed, 85 passed
- `cargo test --test svg_snapshot`: passed, 8 passed
- `cargo test --lib`: passed, 1923 passed / 0 failed / 6 ignored

## 주요 문제점 / 리스크

- PR은 현재 원 head 기준으로 `BEHIND`였다. 이 review 문서 커밋 전에 최신 `upstream/devel`을 PR head에 merge commit으로 반영하여 base skew를 해소한다.
- 변경 지점은 미주 pagination의 조건 분기가 많은 영역이다. 다만 새 조건은 default between-notes gap, visible separator, compact profile, internal rewind reset, multi-column, near-bottom, saved-vpos fit과 full-advance overflow가 동시에 성립하는 경우로 좁혀져 있다.
- `issue_1082`, `issue_1139` 전체, `svg_snapshot`, `cargo test --lib`가 병합본 기준 통과하여 기존 미주/그림/렌더 snapshot 회귀는 발견되지 않았다.
- GitHub Actions는 review 문서와 merge commit push 후 최신 head 기준으로 다시 실행되어야 한다.

## 최종 권고

조건부 merge 후보로 본다.

최종 merge 조건:

- PR head 최신 커밋 기준 GitHub Actions 통과
- 이 review 문서와 `pr_1495_review_impl.md`가 PR diff에 포함됨
- GitHub review 또는 PR comment로 검토 결과를 contributor에게 남김
- merge 전 최신 `mergeable` / `mergeStateStatus` 재확인
- 작업지시자 승인
