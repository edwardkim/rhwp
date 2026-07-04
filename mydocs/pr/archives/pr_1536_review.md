# PR #1536 검토 기록 — HWPX form caption XML 특수문자 이중 escape 누적 해소

- PR: https://github.com/edwardkim/rhwp/pull/1536
- 제목: `Task #1534: HWPX 폼 caption XML 특수문자 이중 이스케이프 누적 해소`
- 작성자: `planet6897` (Jaeuk Ryu)
- 관련 이슈: #1534
- 후속 sub-issue: #1562
- 작성일: 2026-06-26
- 처리 경로: collaborator-mediated 외부 PR 처리 경로. `maintainerCanModify=true`이므로 review 문서를
  PR head의 `mydocs/pr/archives/`에 직접 포함한다.
- base/head: `edwardkim/rhwp:devel` `2fb92ccd` <- `planet6897/rhwp:pr-task1534` `21153c21`
- 규모: 9 files, +693 / -5

`draft`, `mergeable`, `head SHA`, `CI 상태`는 변하는 값이므로 최종 판단 전 최신 상태를 다시 확인한다.

## 1. 목적

PR #1536은 HWPX 저장/roundtrip 과정에서 form control caption 같은 XML attribute 문자열이 이미
escape된 상태로 IR에 들어가고, serializer가 다시 escape하면서 저장할 때마다 손상이 누적되는 문제를
다룬다.

대상 증상:

- `samples/hwpx/form-002.hwpx`
- `<hp:checkBtn caption="...">`에 포함된 `&`가 저장할 때마다 `&amp;` 한 겹씩 추가된다.
- 원본 `R&amp;&amp;D`가 1회 저장 후 `R&amp;amp;&amp;amp;D`, 2회 저장 후
  `R&amp;amp;amp;&amp;amp;amp;D`로 누적된다.
- 연결 이슈: #1534 `HWPX 저장 시 폼 컨트롤 속성값(caption) XML 특수문자 이중 이스케이프 누적`

이 문서는 코드 리뷰에서 확인할 축, 로컬 검증, 시각 검증, 후속 이슈 분리, 최종 처리 조건을 함께 기록한다.

## 2. 현재 PR 메타

| 항목 | 내용 |
|---|---|
| state | open |
| draft | false |
| mergeable | `MERGEABLE` |
| merge state | `CLEAN` |
| review decision | `CHANGES_REQUESTED` |
| base | `devel` |
| head branch | `planet6897:pr-task1534` |
| head SHA | `21153c2181270d39276dab669391429eb7660d15` |
| commit 수 | 2 (`efce0d66`, `21153c21`) |
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

주의: 이전 review에서 `R&&D` 표시 차이에 대해 Request changes를 남겼다. 이후 이 표시 문제는
#1534의 sub-issue #1562로 분리했고, PR #1536은 XML attribute escape 누적 방지 범위로 계속 검토한다.
문서 push 후 merge review를 진행하려면 review decision을 새 review로 갱신해야 한다.

## 3. 커밋별 검토 범위

| 커밋 | 내용 | 주요 검토 축 |
|---|---|---|
| `efce0d66` | `attr_str` XML entity unescape, form caption 회귀 테스트, golden SVG 갱신, 작업 문서 추가 | attribute decode 대칭성, serializer 재escape와의 역할 분리, form caption roundtrip 안정성 |
| `21153c21` | `devel` merge into `pr-task1534` | 최신 base 정합, diff 오염 여부, CI 재실행 상태 |

## 4. 코드 리뷰 체크리스트

### 4.1 XML attribute decode

- `src/parser/hwpx/utils.rs`의 `attr_str`가 `quick_xml::escape::unescape`를 사용해 XML entity를 해제하는지 확인한다.
- `quick-xml` attribute 값이 원문 escape 상태라는 전제를 코드 주석과 동작이 일치하는지 확인한다.
- malformed entity 입력에서 panic하지 않고 기존 lossy raw string으로 fallback하는지 확인한다.
- 숫자/열거형 attribute는 entity가 없으므로 unescape가 no-op으로 유지되는지 확인한다.
- parser가 semantic value를 IR에 저장하고, serializer가 유일한 escape 권위가 되는 대칭 구조인지 확인한다.

### 4.2 serializer와 roundtrip

- `src/serializer/hwpx/form.rs`가 `form.caption`을 그대로 attribute writer에 넘기고, writer가 XML escape를 맡는 구조인지 확인한다.
- 원본 HWPX `caption="R&amp;&amp;D"`가 parse 후 내부값 `R&&D`로 유지되고, serialize 후 다시
  `caption="R&amp;&amp;D"`로 기록되는지 확인한다.
- 2회 저장해도 `&amp;amp;`가 증가하지 않는지 확인한다.
- 본문 `<hp:t>` 경로는 기존 GeneralRef 처리와 별도이며 이번 변경이 본문 텍스트를 깨뜨리지 않는지 확인한다.

### 4.3 테스트와 golden

- `tests/issue_1534_hwpx_form_caption_escape.rs`가 다음을 직접 검증하는지 확인한다.
  - fixture에 `&` 포함 form caption이 존재한다.
  - parse -> serialize -> reparse 후 caption 목록이 불변이다.
  - 저장본 XML에 `&amp;amp;`가 없다.
  - 2회 roundtrip 후에도 caption이 누적 변형되지 않는다.
- `tests/golden_svg/form-002/page-0.svg`의 변경은 기존 rhwp 표시 기준에서 `R&amp;amp;&amp;amp;D`를
  `R&amp;&amp;D`로 되돌리는 것인지 확인한다.
- 단, 한컴 뷰어 표시 기준으로는 `R&&D`가 `R&D`로 보이는 것이 맞으므로 이 golden은 #1562 후속 작업에서
  다시 갱신될 수 있음을 기록한다.

### 4.4 후속 이슈 #1562 분리

검토 중 한컴 뷰어에서 원본 `form-002.hwpx`가 `R&&D`가 아니라 `R&D`로 보인다는 별도 문제가 발견됐다.
이 문제는 XML escaping 계층이 아니라 form control caption 표시 계층의 mnemonic/access-key prefix 해석으로
보이며, 다음 이유로 PR #1536의 직접 blocker가 아니라 후속 이슈로 분리한다.

- PR #1536은 저장할 때마다 XML attribute escape가 누적되는 데이터 손상 문제를 해결한다.
- #1562는 저장값 `R&&D`를 표시할 때 `R&D`로 렌더링해야 하는 시각 호환성 문제다.
- 저장값과 표시값을 분리해야 하며, serializer에서 `&&`를 `&`로 치환하면 #1534의 저장 안정성을 해칠 수 있다.
- #1562는 #1534의 sub-issue로 등록했고, 공식 문서 조사 및 출처를 이슈 본문에 남겼다.

참조:

- #1562 `HWPX 폼 컨트롤 caption &&가 한컴과 다르게 &&로 표시됨`
- PR comment: https://github.com/edwardkim/rhwp/pull/1536#issuecomment-4807059354

## 5. 로컬 검증 기록

검증 head:

```text
21153c2181270d39276dab669391429eb7660d15
```

실행한 로컬 검증:

| 명령 | 결과 |
|---|---|
| `cargo test --test issue_1534_hwpx_form_caption_escape` | 통과, 4 passed |
| `cargo test --test hwpx_roundtrip_baseline` | 통과, 4 passed |
| `cargo test --test svg_snapshot form_002` | 통과, 1 passed |
| `cargo test --test hwpx_roundtrip_integration` | 통과, 22 passed |
| `cargo fmt --check` | 통과 |
| `git diff --check upstream/devel...local/pr1536` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |

추가로 PR head와 최신 `FETCH_HEAD`가 모두 `21153c2181270d39276dab669391429eb7660d15`임을 확인했다.

## 6. 시각/roundtrip 검증 준비

작업지시자 확인용으로 before/after worktree와 rhwp-studio local server를 준비했다.

| 구분 | worktree | 기준 |
|---|---|---|
| before | `/private/tmp/rhwp-pr1536-before` | `upstream/devel` `2fb92ccd` |
| after | `/private/tmp/rhwp-pr1536-after` | PR head `21153c21` |

dev server:

| 구분 | URL |
|---|---|
| before | http://127.0.0.1:7736/ |
| after | http://127.0.0.1:7737/ |

각 서버에 같은 sample set을 준비했다.

```text
/samples/pr1536-visual/00-original-form-002.hwpx
/samples/pr1536-visual/01-saved-once-form-002.hwpx
/samples/pr1536-visual/02-saved-twice-form-002.hwpx
```

XML 원문 확인:

| 구분 | 원본 | 1회 저장 | 2회 저장 |
|---|---|---|---|
| before | `R&amp;&amp;D` | `R&amp;amp;&amp;amp;D` | `R&amp;amp;amp;&amp;amp;amp;D` |
| after | `R&amp;&amp;D` | `R&amp;&amp;D` | `R&amp;&amp;D` |

판단:

- PR #1536 적용 전에는 저장 횟수마다 XML escape가 누적된다.
- PR #1536 적용 후에는 2회 저장까지 같은 caption XML이 유지된다.
- rhwp 표시에서 `R&&D`가 보이는 문제는 #1562 후속 이슈로 분리한다.

## 7. 위험 분류

### Blocking 후보

- `attr_str` 변경이 일반 attribute parse 경로 전체에 영향을 주므로, 문자열 attribute의 semantic value가 예상과 다르게 바뀌는 경우.
- `quick_xml::escape::unescape` 실패 fallback이 malformed 입력을 조용히 보존해 나중에 serializer에서 재손상되는 경우.
- golden SVG가 한컴 표시 기준과 다른 `R&&D`를 고정해버리는 경우.

### 검토 결과

- 정상 XML entity에 대해서는 parser/serializer 역할 분리가 맞고, targeted test가 회귀를 직접 잡는다.
- malformed 입력 fallback은 기존 raw 보존 동작과 가까우며 panic 회피 측면에서 타당하다.
- golden SVG의 `R&&D`는 PR #1536의 저장 안정성 범위에서는 기존 `R&amp;amp;` 손상보다 개선이지만, 한컴 표시
  기준으로는 #1562에서 별도 정정해야 한다.

### Non-blocking 후속

- #1562: form control caption 표시에서 mnemonic/access-key prefix 규칙을 적용해 `R&&D`를 `R&D`로 표시한다.
- #1534 merge 후 parent issue close 정책은 #1562 open 상태와 함께 확인한다. 수동 close는 작업지시자 승인 없이 수행하지 않는다.

## 8. 리뷰 문서 push 계획

Route A, original PR merge 후보로 유지한다.

문서 커밋에는 다음 파일만 포함한다.

```text
mydocs/pr/archives/pr_1536_review.md
mydocs/pr/archives/pr_1536_report.md
```

push 대상:

```text
planet6897/rhwp:pr-task1534
```

커밋/푸시 후 확인:

1. PR head SHA가 문서 커밋으로 갱신된다.
2. PR diff에 review/report 문서 2건만 추가된다.
3. code/test 파일 추가 변경이 없다.
4. 기존 Request changes review state는 그대로 남으므로, merge 전 새 review decision 업데이트가 필요하다.

## 9. 현재 결론

PR #1536은 #1534의 원래 범위인 HWPX form control caption XML attribute 이중 escape 누적 문제를
좁은 변경으로 해결한다. 로컬 targeted test, roundtrip baseline, SVG snapshot, integration test, fmt,
clippy, GitHub Actions가 통과했다.

검토 중 발견된 한컴 표시 기준 `R&&D` -> `R&D` 문제는 저장 안정성 계층과 분리해야 하며 #1562 sub-issue로
등록했다. 따라서 #1562를 PR #1536의 code blocker로 보지 않고, known follow-up으로 추적한다.

현재 권고: **PR #1536 본 범위 기준 수용 가능.** 다만 기존 review decision이 `CHANGES_REQUESTED`이므로,
문서 커밋 push 후 작업지시자 승인에 따라 새 review로 상태를 갱신해야 한다.
