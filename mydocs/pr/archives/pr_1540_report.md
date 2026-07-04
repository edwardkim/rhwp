# PR #1540 사전 처리 판단 보고서 — HWPX 도형 외곽선 NONE 보존

- PR: https://github.com/edwardkim/rhwp/pull/1540
- 제목: `Task #1531: HWPX 도형 외곽선 '선 없음'(NONE) 보존 — 라운드트립 사각형 박스 정정`
- 작성자: `planet6897` (Jaeuk Ryu)
- 관련 이슈: #1531
- 검토일: 2026-06-26
- 검증 head: `2047136e3bec36b191633e99d3f967b916091f61`
- 검증 worktree: `/private/tmp/rhwp-pr1540-review`
- 처리 경로: collaborator-mediated 외부 PR 처리 경로
- 문서 경로: `mydocs/pr/archives/pr_1540_review.md`, `mydocs/pr/archives/pr_1540_report.md`

## 1. 사전 판단

**보정 commit 포함 기준 수용 가능. Blocking finding 없음.**

PR #1540은 HWPX 도형 lineShape의 `style="NONE"`이 roundtrip 후 `SOLID`로 바뀌어 원본에 없던
사각형 외곽선이 생기는 #1531 문제를 좁은 범위에서 해결한다.

핵심 변경은 다음과 같다.

- parser: `style="NONE"`을 `0x40`이 아니라 정본 code `0`으로 저장한다.
- serializer: style code `0`을 `NONE`, `1`을 `SOLID`로 명시 방출한다.
- test: `NONE`과 `endCap` 비트가 같이 있어도 `NONE`이 유지되는지 검증한다.

최초 contributor commit은 동작 방향이 맞았으나, 새 테스트의 `0 | (1 << 6)` 표현이
`cargo clippy --all-targets -- -D warnings`에서 `clippy::identity-op`로 실패했다. 이 문제는 contributor
commit을 rewrite하지 않고 collaborator 보정 commit `2047136e`로 별도 정정했다.

## 2. PR 상태

문서 작성 전 GitHub 확인값:

| 항목 | 값 |
|---|---|
| state | open |
| draft | false |
| mergeable | `MERGEABLE` |
| merge state | `BEHIND` |
| GitHub head SHA | `d87ea71db13537adde831a98a9fd66e1b5143751` |
| labels | `bug`, `hwpx`, `roundtrip`, `rendering` |
| milestone | `v1.0.0` |
| assignee | `planet6897` |
| review request | `postmelee` |
| `closingIssuesReferences` | 비어 있음. PR body는 `closes #1531` 언급 |

GitHub Actions, 문서 작성 전 원 PR head 기준:

| 체크 | 결과 |
|---|---|
| Build & Test | success |
| Analyze (rust) | success |
| Analyze (javascript-typescript) | success |
| Analyze (python) | success |
| CodeQL | success |
| WASM Build | skipped |

주의:

- 보정 commit과 문서 commit push 후 최신 head SHA/check 상태를 다시 확인해야 한다.
- `mergeStateStatus=BEHIND`이므로 merge 전 최신 base 기준 merge 가능 상태를 다시 확인한다.

## 3. 변경 검토

| 파일 | 변경 | 판단 |
|---|---|---|
| `src/parser/hwpx/section.rs` | `parse_line_shape_attr`에서 `style="NONE"`을 `0`으로 저장 | 타당 |
| `src/serializer/hwpx/shape.rs` | `write_line_shape`에서 `0 => "NONE"`, `1 => "SOLID"` 추가 | 타당 |
| `src/serializer/hwpx/shape.rs` tests | `task1531_line_shape_none_preserved` 추가 | 타당, collaborator 보정 후 clippy 통과 |

핵심 확인:

- `style`은 하위 6비트, `endCap`은 bit 6~9로 분리해서 serializer가 읽는다.
- 종전 `NONE=0x40`은 `endCap`과 같은 영역을 사용해 소실될 수 있었다.
- serializer가 `0`을 fallback `SOLID`로 취급하지 않게 되어 #1531의 원본 없는 외곽선 생성을 직접 차단한다.
- `SOLID=1`, `DASH=2` 등 기존 style code는 유지된다.

## 4. 로컬 검증

| 명령 | 결과 |
|---|---|
| `cargo test --lib task1531_line_shape_none_preserved` | 통과, 1 passed |
| `cargo test --test hwpx_roundtrip_baseline baseline_all_samples_roundtrip` | 통과, 1 passed |
| `cargo test --test visual_roundtrip_baseline visual_baseline_all_samples` | 통과, 1 passed |
| `cargo fmt --check` | 통과 |
| `git diff --check` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |

`BEHIND` 상태 merge simulation:

| 명령 | 결과 |
|---|---|
| `git merge upstream/devel --no-commit --no-ff` | 충돌 없음 |
| merge simulation 상태의 `cargo clippy --all-targets -- -D warnings` | 통과 |
| `git merge --abort` | PR branch 상태 복구 |

## 5. 대표 샘플 검증

샘플:

```text
samples/hwpx/2026_oss_rst.hwpx
```

산출물:

```text
/private/tmp/rhwp-pr1540-review/output/poc/pr1540/2026_oss_rst.rt.hwpx
```

검증 결과:

| 항목 | 결과 |
|---|---|
| `hwpx-roundtrip` | `PASS`, `diff=0`, `r2=0` |
| `render-diff --via hwpx -p 0` | `PASS`, page count 6->6, max_disp 0.00px, structure mismatch 0 |
| 원본 `Contents/section0.xml` style token | `style="NONE"` 1건 |
| roundtrip `Contents/section0.xml` style token | `style="NONE"` 1건 |

결론:

- PR 적용 후 대표 샘플 roundtrip에서 `NONE`이 `SOLID`로 바뀌지 않는다.
- render geometry 관점에서도 1페이지 구조/좌표 회귀가 없다.
- stroke style token 회귀는 geometry gate만으로 충분히 보장되지 않으므로 XML token 직접 검증을 병행했다.

## 6. Contributor credit와 commit provenance

원본 contributor 코드 commit:

| source commit | author | 내용 |
|---|---|---|
| `d87ea71db13537adde831a98a9fd66e1b5143751` | Jaeook Ryu `<jaeook.ryu@gmail.com>` | #1531 구현 |

보정 commit:

| commit | author | 내용 |
|---|---|---|
| `2047136e3bec36b191633e99d3f967b916091f61` | postmelee `<meleeisdeveloping@gmail.com>` | clippy identity-op 보정 |

원본 commit은 rewrite하지 않았다. contributor author와 `Co-Authored-By: Claude Opus 4.8 (1M context)
<noreply@anthropic.com>` trailer를 보존한다.

## 7. 문서 push

이번 문서 commit은 다음 파일만 추가한다.

```text
mydocs/pr/archives/pr_1540_review.md
mydocs/pr/archives/pr_1540_report.md
```

push 대상:

```text
planet6897/rhwp:pr-task1531
```

push 후 확인할 항목:

1. PR head가 contributor commit + collaborator 보정 commit + 문서 commit 순서로 갱신됐는지 확인한다.
2. PR diff에 `mydocs/pr/archives/pr_1540_review.md`와 `mydocs/pr/archives/pr_1540_report.md`가 포함되는지 확인한다.
3. PR diff에 의도하지 않은 파일이 추가되지 않았는지 확인한다.
4. 최신 head 기준 GitHub Actions/check 상태를 확인한다.

## 8. merge 전 조건

1. 최신 PR head SHA 재확인.
2. 최신 `mergeable` / `mergeStateStatus` 재확인.
3. 최신 head 기준 GitHub Actions 통과 확인. 문서 commit이 trailing인 경우 section 9.3.1 fast-pass 적용 가능 여부 확인.
4. review 문서 2건이 PR diff에 포함됐는지 확인.
5. GitHub review approval은 작업지시자 승인 후 별도 수행.
6. merge는 작업지시자 승인 후 별도 수행.

## 9. issue close check plan

PR body는 `closes #1531`을 언급하지만 GitHub API `closingIssuesReferences`는 비어 있다.

merge 후 처리 계획:

1. #1531 state를 다시 확인한다.
2. 자동 close되지 않았으면 수동 close comment 초안을 작성한다.
3. 이슈 close는 작업지시자 명시 승인 후에만 수행한다.

## 10. 권장 처리

권고: **Approve 가능.**

단, 이 보고서 기준으로 곧 보정+문서 commit이 push되므로, 최종 review/merge 전에는 push 후 최신
head SHA와 check 상태를 다시 확인해야 한다.

GitHub review 코멘트 초안:

```text
PR #1540을 contributor commit d87ea71d와 collaborator 보정 commit 2047136e 기준으로 검토했습니다.

이번 변경은 HWPX lineShape style="NONE"이 roundtrip 후 SOLID로 되살아나 원본에 없던 사각형 외곽선이 생기던 #1531 문제와 직접 맞습니다. parser에서 NONE을 정본 code 0으로 저장하고, serializer에서 0 => "NONE", 1 => "SOLID"를 명시 처리하는 방향은 표 borderFill/HWP5 doc_info 계열과도 일치합니다.

초기 테스트 표현의 clippy identity-op 문제는 contributor commit을 rewrite하지 않고 별도 collaborator 보정 commit으로 해소했습니다.

로컬 검증은 모두 통과했습니다.

- cargo test --lib task1531_line_shape_none_preserved
- cargo test --test hwpx_roundtrip_baseline baseline_all_samples_roundtrip
- cargo test --test visual_roundtrip_baseline visual_baseline_all_samples
- cargo fmt --check
- git diff --check
- cargo clippy --all-targets -- -D warnings

또한 PR이 BEHIND 상태라 최신 upstream/devel과 no-commit merge simulation을 수행했고, 충돌 없이 병합되며 해당 merge simulation 상태에서도 clippy가 통과했습니다.

대표 샘플 samples/hwpx/2026_oss_rst.hwpx는 hwpx-roundtrip PASS(diff=0, r2=0), render-diff 1페이지 PASS(max_disp 0.00px)입니다. roundtrip 산출물의 Contents/section0.xml에서도 원본과 동일하게 style="NONE" 1건이 유지되어 SOLID로 바뀌지 않음을 확인했습니다.

Blocking finding 없습니다. Approve합니다.
```
