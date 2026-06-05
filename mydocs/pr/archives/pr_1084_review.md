# PR #1084 검토 문서

- PR: <https://github.com/edwardkim/rhwp/pull/1084>
- 제목: Task #1079: 그림 pushdown ↔ 파일 vpos 이중 계상 정정 — 1페이지 정합
- 관련 이슈: <https://github.com/edwardkim/rhwp/issues/1079>
- 작성일: 2026-05-26
- 작성자: Codex

## 1. PR 상태

| 항목 | 값 |
|---|---|
| 상태 | open |
| base | `devel` |
| base sha | `ce879fe0acf077f7b46e36459573f028a7fbe089` |
| head | `pr/task1079-picture-pushdown` |
| head sha | `b2fca6c6c5b6c23fb2979a8db1ba2dd00f41dba9` |
| mergeable | true |
| 변경 파일 | 11개 |
| 증감 | +445 / -6 |
| 리뷰/댓글 | 없음 |
| 요청 리뷰어 | `edwardkim` |

CI 확인:

| check | status |
|---|---|
| Analyze (javascript-typescript) | pass |
| Analyze (python) | pass |
| Analyze (rust) | pass |
| Build & Test | pass |
| Canvas visual diff | pass |
| CodeQL | pass |
| WASM Build | skipped |

## 2. 이슈 요약

이슈 #1079는 `samples/pr-149.hwp`에서 그림 3개와 본문 마지막 텍스트가 한컴에서는 1페이지에
들어가지만, rhwp에서는 마지막 그림이 본문 하단을 약 109px 초과하고 2페이지로 분리되는 문제다.

권위 비교 자료와 샘플은 로컬에 존재한다.

```text
samples/pr-149.hwp
pdf/pr-149-2022.pdf
```

## 3. 변경 요약

PR #1084의 핵심 판단은 다음과 같다.

```text
비-TAC TopAndBottom, vert=Para 그림에서 파일 vpos가 이미 그림 공간을 반영하는 경우가 있다.
이때 typeset pushdown과 renderer 진행이 그림 높이를 다시 더하면 이중 계상이 된다.
```

수정 방향:

```text
gap_before = current_para_first_vpos - previous_para_end_vpos

gap_before >= picture_height - 8px 이면:
  파일 vpos가 이미 그림 공간을 반영한 것으로 보고 pushdown을 생략한다.

gap_before < picture_height - 8px 이면:
  #409 계열처럼 파일 vpos가 그림 공간을 반영하지 않은 케이스로 보고 기존 pushdown을 유지한다.
```

주요 코드 변경:

| 파일 | 내용 |
|---|---|
| `src/renderer/typeset.rs` | `pushdown_h`를 `(obj_h, extra)`로 분리하고 gap 판정 시 extra 가산 생략 |
| `src/renderer/layout.rs` | 그림 문단의 gap 기반 `vpos_accounts_for_height` 계산 후 renderer에 전달 |
| `src/renderer/layout/picture_footnote.rs` | already-accounted 그림은 gap 안에 그리고 flow 진행을 추가하지 않음 |
| `tests/issue_1079_picture_pushdown_vpos.rs` | `pr-149.hwp` 1페이지 수용과 본문/그림 하단 회귀 가드 추가 |

## 4. 현재 코드 반영 여부

현재 `local/devel`에는 PR #1084의 핵심 구현이 아직 없다.

확인 결과:

```text
vpos_accounts_for_height 없음
PUSHDOWN_GAP_TOL_PX 없음
tests/issue_1079_picture_pushdown_vpos.rs 없음
```

현재 기준 HEAD:

```text
27441c84
```

## 5. 검토 포인트

### 5.1 수용 가능성이 높은 점

- 이슈 #1079의 증상과 PR 본문의 원인 설명이 일치한다.
- `samples/pr-149.hwp`와 `pdf/pr-149-2022.pdf`가 로컬에 있어 재검증 가능하다.
- PR CI에서 Build & Test, Rust analyze, Canvas visual diff가 통과했다.
- 공개 샘플 기반 회귀 테스트가 추가되어 있다.
- 변경이 단일 커밋으로 정리되어 있어 현재 devel에 `-x` cherry-pick으로 수용하기 적합하다.

### 5.2 주의해야 할 점

이 변경은 그림 배치의 공유 경로를 건드린다.

특히 다음 부분은 현재 devel 기준으로 반드시 다시 확인해야 한다.

```text
1. pr-149.hwp가 1페이지로 수용되는지
2. 그림과 본문 마지막 "입니다"가 페이지 안에 남는지
3. #409 계열처럼 파일 vpos가 그림 공간을 반영하지 않는 케이스가 회귀하지 않는지
4. renderer와 typeset의 pushdown 판정이 서로 어긋나지 않는지
```

또한 PR의 renderer 변경은 `layout_body_picture` 중심이고, typeset 쪽은 Picture와 Shape 모두의
TopAndBottom 경로를 판정한다. 이번 이슈는 그림 타깃이므로 수용 가능하지만, 도형 쪽 영향은
현재 검증에서 간접적으로라도 확인해야 한다.

### 5.3 휴리스틱 리스크

`picture_height - 8px` 판정은 실무적으로 타당한 gap tolerance지만, 일반 규칙으로 확장되는 만큼
다음 리스크가 있다.

```text
- lineSeg vpos 또는 line_height가 특이한 문서에서 already-accounted 판정이 과하게 켜질 수 있음
- 반대로 그림 높이와 gap 차이가 8px보다 큰 근접 케이스는 기존 경로로 남을 수 있음
```

따라서 PR 수용 후에도 추후 유사 케이스가 나오면 tolerance를 문서화하거나 더 구조적인 판정으로
정리할 필요가 있다.

## 6. 권장 처리 방향

권장안:

```text
1. PR #1084 단일 커밋을 현재 local/devel에 -x cherry-pick 한다.
2. 현재 devel 기준으로 fmt/check/test를 다시 수행한다.
3. wasm build 후 메인테이너 시각 판정을 받는다.
4. 통과하면 devel에 반영하고 PR #1084 및 이슈 #1079를 체리픽 수용으로 close 한다.
```

바로 merge commit으로 받기보다는 `-x` cherry-pick이 적합하다.

이유:

```text
- PR base가 현재 devel보다 오래되었다.
- 변경 범위가 단일 커밋으로 정리되어 있다.
- 외부 기여자 출처를 커밋 메시지에 남길 수 있다.
- 현재 devel 기준 검증을 로컬에서 다시 수행하기 쉽다.
```

## 7. 승인 요청

다음 절차로 진행해도 되는지 승인 요청한다.

```text
git fetch origin pull/1084/head:pr/1084
git cherry-pick -x b2fca6c6c5b6c23fb2979a8db1ba2dd00f41dba9
cargo fmt --check
cargo check
cargo test --test issue_1079_picture_pushdown_vpos
cargo test --lib
docker compose --env-file .env.docker run --rm wasm
```
