# PR #1540 검토 기록 — HWPX 도형 외곽선 NONE 보존

- PR: https://github.com/edwardkim/rhwp/pull/1540
- 제목: `Task #1531: HWPX 도형 외곽선 '선 없음'(NONE) 보존 — 라운드트립 사각형 박스 정정`
- 작성자: `planet6897` (Jaeuk Ryu)
- 관련 이슈: #1531
- 작성일: 2026-06-26
- 처리 경로: collaborator-mediated 외부 PR 처리 경로. `maintainerCanModify=true`이므로 review 문서를
  PR head의 `mydocs/pr/archives/`에 직접 포함한다.
- base/head: `edwardkim/rhwp:devel` <- `planet6897/rhwp:pr-task1531`
- 검증 기준: contributor source commit `d87ea71db13537adde831a98a9fd66e1b5143751` +
  collaborator 보정 commit `2047136e3bec36b191633e99d3f967b916091f61`
- 원 PR 규모: 2 files, +32 / -3

`draft`, `mergeable`, `head SHA`, `CI 상태`는 변하는 값이므로 최종 판단 전 최신 상태를 다시 확인한다.

## 1. 목적

PR #1540은 HWPX roundtrip에서 도형 `<hp:lineShape style="NONE">`이 저장 후 `SOLID`로
되살아나 원본에 없던 사각형 외곽선이 생기는 문제를 다룬다.

대상 증상:

- `samples/hwpx/2026_oss_rst.hwpx`
- roundtrip 후 1페이지 상단 `< 결과보고서 작성 안내 >` 제목 영역에 원본에는 없는 사각형 테두리 생성
- 연결 이슈: #1531 `HWPX roundtrip: 2026_oss_rst 1페이지 제목 영역에 원본에 없는 사각형 테두리 생성`

원인은 두 계층이다.

1. parser가 `style="NONE"`을 `0x40`으로 저장하여 `endCap` 비트 영역과 충돌했다.
2. serializer가 line style code `0`을 명시적으로 처리하지 않아 `_ => "SOLID"`로 떨어뜨렸다.

이 PR은 line style 정본 코드를 표 borderFill/HWP5 doc_info와 같은 `0=NONE`, `1=SOLID`,
`2=DASH` 계열로 맞추고, `endCap` 비트가 함께 있어도 `NONE`이 유지되는 회귀 테스트를 추가한다.

## 2. 현재 PR 메타

문서 작성 시점 GitHub API 확인값:

| 항목 | 내용 |
|---|---|
| state | open |
| draft | false |
| mergeable | `MERGEABLE` |
| merge state | `BEHIND` |
| review decision | 없음 |
| base | `devel` |
| head branch | `planet6897:pr-task1531` |
| GitHub head SHA | `d87ea71db13537adde831a98a9fd66e1b5143751` |
| labels | `bug`, `hwpx`, `roundtrip`, `rendering` |
| milestone | `v1.0.0` |
| assignee | `planet6897` |
| review request | `postmelee` |
| `closingIssuesReferences` | 비어 있음. PR body는 `closes #1531`을 언급하므로 merge 후 #1531 상태 수동 확인 필요 |

GitHub Actions, 문서 작성 전 원 PR head 기준:

| 체크 | 결과 |
|---|---|
| Build & Test | success |
| Analyze (rust) | success |
| Analyze (javascript-typescript) | success |
| Analyze (python) | success |
| CodeQL | success |
| WASM Build | skipped |

주의: 이 문서는 collaborator 보정 commit과 문서 commit을 PR head에 push하기 전 작성한다. push 후 최신
head SHA와 check rollup을 다시 확인해야 한다.

## 3. 커밋별 검토 범위

| 커밋 | author | 내용 | 주요 검토 축 |
|---|---|---|---|
| `d87ea71d` | Jaeook Ryu `<jaeook.ryu@gmail.com>` | HWPX lineShape `NONE` parser/serializer 정정, 회귀 테스트 추가 | style code 매핑, endCap 비트 충돌 제거, roundtrip 보존 |
| `2047136e` | postmelee `<meleeisdeveloping@gmail.com>` | 회귀 테스트의 `clippy::identity-op` 경고 보정 | contributor commit rewrite 없이 별도 maintainer 보정 |

원 contributor commit은 rewrite하지 않았다. `d87ea71d`의 `Co-Authored-By: Claude Opus 4.8 (1M context)
<noreply@anthropic.com>` trailer도 그대로 보존한다.

## 4. 코드 리뷰 체크리스트

### 4.1 parser lineShape style 매핑

- `src/parser/hwpx/section.rs`의 `parse_line_shape_attr`가 `style="NONE"`을 `0`으로 저장하는지 확인한다.
- `SOLID=1`, `DASH=2` 이후 기존 style code가 유지되는지 확인한다.
- 종전 `0x40`이 `endCap` 비트 영역(`bit 6~9`)과 충돌했다는 원인 설명이 코드와 맞는지 확인한다.
- `endCap` 파싱은 별도 비트 영역에 기록되며, serializer가 style 하위 6비트만 읽는 구조와 충돌하지 않는지 확인한다.

### 4.2 serializer lineShape style 역매핑

- `src/serializer/hwpx/shape.rs`의 `write_line_shape`가 `bl.attr & 0x3F` 기준으로 `0 => "NONE"`,
  `1 => "SOLID"`를 명시 처리하는지 확인한다.
- 알 수 없는 style code는 기존처럼 `SOLID` fallback을 유지해 호환성을 깨지 않는지 확인한다.
- `endCap`이 `1 << 6`으로 같이 설정되어도 style은 `NONE`으로 직렬화되는지 확인한다.

### 4.3 회귀 테스트와 collaborator 보정

- `task1531_line_shape_none_preserved`가 `0=NONE`, `1=SOLID`, `2=DASH`를 직접 검증하는지 확인한다.
- `endCap=FLAT` 비트가 같이 설정된 경우도 `style="NONE"`으로 남는지 검증하는지 확인한다.
- 최초 contributor 테스트의 `0 | (1 << 6)` 표현은 의미상 문제는 없지만 `cargo clippy --all-targets -- -D warnings`
  게이트에서 `clippy::identity-op`로 실패했다.
- collaborator 보정 commit `2047136e`는 테스트 입력을 `none_with_flat_end_cap` 변수로 분리하는 데 한정된다.

### 4.4 이슈와 시각 검증

- 이슈 #1531의 대표 샘플 `2026_oss_rst.hwpx`에서 roundtrip 후 `style="NONE"`이 유지되는지 확인한다.
- 기존 render geometry gate는 stroke style 변화만으로는 실패하지 않을 수 있으므로, XML style token 직접 확인을 병행한다.
- `render-diff`는 페이지 수, 구조, 좌표 회귀가 없는지 확인하는 보조 게이트로 사용한다.

## 5. 로컬 검증 기록

검증 worktree:

```text
/private/tmp/rhwp-pr1540-review
```

검증 기준 head:

```text
2047136e3bec36b191633e99d3f967b916091f61
```

실행한 로컬 검증:

| 명령 | 결과 |
|---|---|
| `cargo test --lib task1531_line_shape_none_preserved` | 통과, 1 passed |
| `cargo test --test hwpx_roundtrip_baseline baseline_all_samples_roundtrip` | 통과, 1 passed |
| `cargo test --test visual_roundtrip_baseline visual_baseline_all_samples` | 통과, 1 passed |
| `cargo fmt --check` | 통과 |
| `git diff --check` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |

`BEHIND` 상태 확인:

- `git merge upstream/devel --no-commit --no-ff`: 충돌 없음
- merge simulation 상태에서 `cargo clippy --all-targets -- -D warnings`: 통과
- 검증 후 `git merge --abort`로 PR branch 상태 복구

## 6. 대표 샘플 roundtrip/시각 검증

생성물:

```text
/private/tmp/rhwp-pr1540-review/output/poc/pr1540/2026_oss_rst.rt.hwpx
```

검증 명령:

| 명령 | 결과 |
|---|---|
| `cargo run --bin rhwp -- hwpx-roundtrip samples/hwpx/2026_oss_rst.hwpx -o output/poc/pr1540` | `PASS`, `diff=0`, `r2=0` |
| `cargo run --bin rhwp -- render-diff samples/hwpx/2026_oss_rst.hwpx --via hwpx -p 0` | `PASS`, page count 6->6, max_disp 0.00px, structure mismatch 0 |
| `unzip -p output/poc/pr1540/2026_oss_rst.rt.hwpx Contents/section0.xml` style token 확인 | `style="NONE"` 1건 유지 |

원본 `Contents/section0.xml`도 `style="NONE"` 1건이며, roundtrip 산출물에서 `SOLID`로 바뀌지 않는다.

## 7. 위험 분류

### Blocking 후보

- line style code `0`을 `NONE`으로 바꾸면서 기존에 `0`을 사실상 `SOLID`처럼 기대하던 경로가 있는 경우.
- `style`과 `endCap` 비트 분리 과정에서 다른 line style code가 손상되는 경우.
- GitHub 최신 head 기준 CI가 보정/문서 push 후 실패하는 경우.

### 검토 결과

- 표 borderFill/HWP5 doc_info 계열과 동일한 `0=NONE`, `1=SOLID` 매핑으로 정리하는 방향은 타당하다.
- serializer가 `0`을 `NONE`으로 쓰도록 바뀌어 #1531의 원인인 `NONE -> SOLID` 회귀를 직접 차단한다.
- 단위 회귀 테스트와 대표 샘플 XML token 확인이 같은 문제를 직접 잡는다.
- collaborator 보정 후 clippy blocker는 해소됐다.

### Non-blocking 후속

- 현재 render geometry gate는 stroke style token 변화만으로는 실패하지 않을 수 있다. 이슈 #1531 본문이 언급한
  style/pixel 기반 diff 강화는 별도 후속 후보로 남긴다.
- GitHub API `closingIssuesReferences`가 비어 있으므로 merge 후 #1531 자동 close 여부는 반드시 수동 확인한다.

## 8. 문서 push 계획

Route A, original PR merge 후보로 유지한다.

PR head에는 두 커밋을 push한다.

1. `2047136e test(hwpx): avoid clippy identity-op in #1531 guard`
2. `docs: PR #1540 검토 기록`

문서 커밋에는 다음 파일만 포함한다.

```text
mydocs/pr/archives/pr_1540_review.md
mydocs/pr/archives/pr_1540_report.md
```

push 대상:

```text
planet6897/rhwp:pr-task1531
```

커밋/푸시 후 확인:

1. PR head SHA가 보정+문서 커밋으로 갱신된다.
2. PR diff에 review/report 문서 2건이 포함된다.
3. contributor source commit은 rewrite되지 않는다.
4. 최신 PR head 기준 GitHub Actions 또는 문서-only trailing commit fast-pass 판단을 확인한다.
5. merge 전 `mergeable`, `mergeStateStatus`, latest head SHA를 다시 확인한다.

## 9. 현재 결론

현재까지의 코드 검토와 로컬 검증 기준으로 PR #1540은 #1531 원인과 직접 맞고, contributor 원 변경은
수용 가능하다. 최초 clippy blocker는 collaborator 보정 commit으로 해소했다.

권고: **보정+문서 commit push 후 최신 CI/check 상태를 확인하고 Approve 가능.**
