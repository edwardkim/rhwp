# PR #1536 사전 처리 판단 보고서 — HWPX form caption XML 특수문자 이중 escape 누적 해소

- PR: https://github.com/edwardkim/rhwp/pull/1536
- 제목: `Task #1534: HWPX 폼 caption XML 특수문자 이중 이스케이프 누적 해소`
- 작성자: `planet6897` (Jaeuk Ryu)
- 관련 이슈: #1534
- 후속 sub-issue: #1562
- 검토일: 2026-06-26
- 검증 head: `21153c2181270d39276dab669391429eb7660d15`
- 처리 경로: collaborator-mediated 외부 PR 처리 경로
- 문서 경로: `mydocs/pr/archives/pr_1536_review.md`, `mydocs/pr/archives/pr_1536_report.md`

## 1. 사전 판단

**PR #1536 본 범위 기준 수용 가능. #1562는 별도 후속 이슈.**

PR #1536은 HWPX form control caption이 XML attribute로 저장되는 경로에서 `&` 같은 XML 특수문자가
저장할 때마다 `&amp;` 한 겹씩 추가되는 누적 손상을 해결한다. 핵심 변경은 parser 공통 attribute helper
`attr_str`에서 XML entity를 unescape해 IR에는 semantic value를 저장하고, serializer가 유일하게 XML
escape를 담당하도록 대칭을 맞추는 것이다.

로컬 검증과 GitHub Actions는 통과했다. before/after visual sample도 저장 횟수별 XML 누적 여부를 직접
확인할 수 있게 준비했고, PR 적용 후 2회 저장까지 `R&amp;&amp;D`가 안정적으로 유지됨을 확인했다.

다만 한컴 뷰어는 같은 저장값 `R&&D`를 화면에 `R&D`로 표시한다. 이 문제는 XML 저장 안정성이 아니라 form
caption 표시 규칙 문제이므로 #1534의 sub-issue #1562로 분리했다. 따라서 이 보고서는 #1536을
serialization roundtrip 수정 PR로 판단한다.

## 2. PR 상태

| 항목 | 값 |
|---|---|
| state | open |
| draft | false |
| mergeable | `MERGEABLE` |
| merge state | `CLEAN` |
| review decision | `CHANGES_REQUESTED` |
| head SHA | `21153c2181270d39276dab669391429eb7660d15` |
| 변경량 | 9 files, +693 / -5 |
| labels | `hwpx`, `serialization`, `roundtrip` |
| milestone | `v1.0.0` |
| assignee | `planet6897` |

GitHub Actions, 문서 작성 시점:

| 체크 | 결과 |
|---|---|
| Build & Test | pass |
| Analyze (rust) | pass |
| Analyze (javascript-typescript) | pass |
| Analyze (python) | pass |
| CodeQL | pass |
| WASM Build | skipped |

주의:

- 이전 Request changes review 때문에 `reviewDecision=CHANGES_REQUESTED`가 남아 있다.
- 문서 커밋 push 후 merge 진행 전에는 새 review decision을 별도로 갱신해야 한다.
- review decision 갱신, merge, issue close는 작업지시자 별도 승인 없이 수행하지 않는다.

## 3. 변경 검토

| 파일 | 변경 | 판단 |
|---|---|---|
| `src/parser/hwpx/utils.rs` | `attr_str`가 XML entity를 unescape 후 반환. 실패 시 기존 raw lossy fallback | 타당 |
| `tests/issue_1534_hwpx_form_caption_escape.rs` | form caption attribute escape 누적 방지 회귀 테스트 4건 추가 | 타당 |
| `tests/golden_svg/form-002/page-0.svg` | 기존 `R&amp;amp;...` 손상 golden을 `R&amp;&amp;...`로 갱신 | #1536 범위에서는 타당, #1562 후속에서 재갱신 가능 |
| `mydocs/plans/*`, `mydocs/working/*`, `mydocs/report/*` | contributor 작업 계획/보고 문서 추가 | PR head 문서로 수용 가능 |

핵심 확인:

- XML 원문 `caption="R&amp;&amp;D"`는 parser에서 내부값 `R&&D`가 되어야 한다.
- serializer는 내부값 `R&&D`를 XML attribute로 쓰면서 `R&amp;&amp;D`로 escape해야 한다.
- PR 적용 전에는 저장본을 다시 저장할수록 `&amp;amp;`가 누적된다.
- PR 적용 후에는 2회 저장까지 같은 XML caption이 유지된다.

## 4. 로컬 검증

| 명령 | 결과 |
|---|---|
| `cargo test --test issue_1534_hwpx_form_caption_escape` | 통과, 4 passed |
| `cargo test --test hwpx_roundtrip_baseline` | 통과, 4 passed |
| `cargo test --test svg_snapshot form_002` | 통과, 1 passed |
| `cargo test --test hwpx_roundtrip_integration` | 통과, 22 passed |
| `cargo fmt --check` | 통과 |
| `git diff --check upstream/devel...local/pr1536` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |

최신 head 확인:

```text
git rev-parse HEAD       -> 21153c2181270d39276dab669391429eb7660d15
git rev-parse FETCH_HEAD -> 21153c2181270d39276dab669391429eb7660d15
```

## 5. before/after 시각 검증

검증 worktree:

| 구분 | 위치 | 기준 |
|---|---|---|
| before | `/private/tmp/rhwp-pr1536-before` | `upstream/devel` `2fb92ccd` |
| after | `/private/tmp/rhwp-pr1536-after` | PR head `21153c21` |

dev server:

| 구분 | URL |
|---|---|
| before | http://127.0.0.1:7736/ |
| after | http://127.0.0.1:7737/ |

샘플:

| 파일 | 의미 |
|---|---|
| `00-original-form-002.hwpx` | 원본 |
| `01-saved-once-form-002.hwpx` | 1회 저장 |
| `02-saved-twice-form-002.hwpx` | 2회 저장 |

XML caption 원문:

| 구분 | 원본 | 1회 저장 | 2회 저장 |
|---|---|---|---|
| before | `R&amp;&amp;D` | `R&amp;amp;&amp;amp;D` | `R&amp;amp;amp;&amp;amp;amp;D` |
| after | `R&amp;&amp;D` | `R&amp;&amp;D` | `R&amp;&amp;D` |

결론:

- PR 적용 전에는 저장할 때마다 XML escape 누적 손상이 재현된다.
- PR 적용 후에는 저장을 반복해도 caption XML이 안정적이다.
- rhwp 표시에서 `R&&D`로 보이는 문제는 #1562에서 별도 처리한다.

## 6. #1562 분리 판단

검토 중 한컴 뷰어에서는 원본 `form-002.hwpx`의 `R&&D` 저장값이 화면에 `R&D`로 보이는 것을 확인했다.
공식 문서 조사 결과 한컴 공개 HWPML/OWPML 자료에서는 `Caption`/`caption` 필드 정의만 확인되고,
`&&` 표시 규칙은 명시적으로 찾지 못했다. 다만 한컴 뷰어 관측 결과와 Microsoft `DrawText` mnemonic prefix
관례가 일치한다.

이 문제는 다음 이유로 #1562로 분리했다.

- PR #1536은 저장/roundtrip 중 XML attribute escape 누적 손상 해결이다.
- #1562는 form caption 표시 문자열 생성 문제다.
- serializer에서 `&&`를 `&`로 치환하면 저장값 보존을 깨뜨릴 수 있다.
- 하이퍼-워터폴 규칙상 별도 원인 계층은 이슈 -> 브랜치 -> 문서 -> 구현으로 추적하는 것이 맞다.

관련 기록:

- #1562: https://github.com/edwardkim/rhwp/issues/1562
- PR 범위 업데이트 comment: https://github.com/edwardkim/rhwp/pull/1536#issuecomment-4807059354

## 7. Contributor credit와 문서 push

원본 contributor 코드 커밋:

| source commit | author | 내용 |
|---|---|---|
| `efce0d6639b878b69679d96ed1ac822341790a77` | Jaeook Ryu `<jaeook.ryu@gmail.com>` | #1534 구현 |

base update commit:

| commit | author | 내용 |
|---|---|---|
| `21153c2181270d39276dab669391429eb7660d15` | Taegyu Lee `<meleeisdeveloping@gmail.com>` | `devel` merge into `pr-task1534` |

이번 collaborator 문서 커밋은 review 문서 2건만 추가한다. contributor 구현 커밋은 rewrite하지 않고 보존한다.

push 대상:

```text
planet6897/rhwp:pr-task1534
```

## 8. merge 전 조건

1. 문서 커밋 push 후 PR diff에 `mydocs/pr/archives/pr_1536_review.md`와
   `mydocs/pr/archives/pr_1536_report.md`가 포함되는지 확인한다.
2. 문서 커밋 외 source/test/golden 변경이 추가되지 않았는지 확인한다.
3. 최신 PR head 기준 GitHub Actions 또는 doc-only fast-pass 판단을 확인한다.
4. 기존 `CHANGES_REQUESTED` review decision을 작업지시자 승인에 따라 새 review로 갱신한다.
5. merge 전 `mergeable`, `mergeStateStatus`, latest head SHA를 다시 확인한다.
6. merge 후 #1534 상태를 확인한다. #1562가 open 상태이므로 #1534 close 정책은 작업지시자 승인에 따른다.

## 9. 권장 처리

권고: **#1534 serialization 범위 기준 Approve 가능.**

Blocking finding은 없다. #1562는 한컴 표시 호환성 후속 이슈로 추적한다.

GitHub review 코멘트 초안:

```text
PR #1536 head 21153c2181270d39276dab669391429eb7660d15 기준으로 재검토했습니다.

이 PR은 HWPX form control caption attribute가 저장할 때마다 이중 escape되는 #1534 문제를 좁은 범위에서 해결합니다. parser의 attr_str에서 XML entity를 해제해 IR에는 semantic value를 저장하고, serializer가 유일하게 XML escape를 담당하도록 대칭을 맞춘 방향은 타당합니다.

로컬 검증은 모두 통과했습니다.

- cargo test --test issue_1534_hwpx_form_caption_escape
- cargo test --test hwpx_roundtrip_baseline
- cargo test --test svg_snapshot form_002
- cargo test --test hwpx_roundtrip_integration
- cargo fmt --check
- git diff --check
- cargo clippy --all-targets -- -D warnings

before/after 샘플도 확인했습니다. PR 적용 전에는 form-002 저장 횟수마다 caption XML이 R&amp;&amp;D -> R&amp;amp;&amp;amp;D -> R&amp;amp;amp;&amp;amp;amp;D로 누적되고, PR 적용 후에는 2회 저장까지 R&amp;&amp;D가 유지됩니다.

검토 중 발견한 한컴 뷰어 표시 기준 R&&D -> R&D 문제는 저장/roundtrip 계층이 아니라 form caption 표시 계층 문제로 판단해 #1534의 sub-issue #1562로 분리했습니다. 따라서 #1562는 이 PR의 blocker로 보지 않고 후속 이슈로 추적합니다.

Blocking finding 없습니다. Approve합니다.
```
