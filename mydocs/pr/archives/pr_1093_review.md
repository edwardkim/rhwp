# PR #1093 검토 문서

- PR: <https://github.com/edwardkim/rhwp/pull/1093>
- 제목: Task #1046: 본문 하단 overflow 정합 — 측정 통일(B) [#1048 rebase 재제출]
- 관련 이슈: <https://github.com/edwardkim/rhwp/issues/1046>
- 참고 이슈: <https://github.com/edwardkim/rhwp/issues/1065>
- 원 PR: <https://github.com/edwardkim/rhwp/pull/1048>
- 작성일: 2026-05-26
- 작성자: Codex

## 1. PR 상태

| 항목 | 값 |
|---|---|
| 상태 | open |
| base | `devel` |
| head | `pr/task1046-overflow` |
| head sha | `f269935f6b1f341d2ea0da3f91d8a481361a18cd` |
| mergeable | true |
| 변경 파일 | 19개 |
| 증감 | +1177 / -11 |
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

## 2. PR 성격

PR #1093은 닫힌 PR #1048의 rebase 재제출이다.

PR #1048 당시 메인테이너 코멘트 요지:

```text
- 측정 통일 B 노선, LAYOUT_OVERFLOW 18→5, 렌더링 출력 불변 설계와 정량 검증은 양호.
- CI, cargo test, 골든 SVG, Canvas visual diff 통과는 확인됨.
- 다만 devel 최신 변경과 rendering.rs 충돌이 있어 rebase 요청 후 close.
```

PR #1093은 해당 요청에 따라 최신 devel 계열에 재제출된 단일 커밋이다.

## 3. 이슈 요약

이슈 #1046은 페이지네이터의 cut/추정 높이와 렌더러의 실제 height가 어긋나, 본문 하단
`LAYOUT_OVERFLOW`가 남는 문제다. 당초 사후 reflow 방식은 폐기되고, 본 PR은 측정 통일(B)
노선으로 정리되어 있다.

PR 본문 기준 기대 효과:

```text
전수 sweep baseline 1156 lines / 46163px / 97파일
→ 583 lines / 41696px / 56파일
신규 overflow 0
```

## 4. 변경 요약

핵심 변경은 두 축이다.

### 4.1 배치 판정 overhead 차감

분할 표의 첫 fragment를 현재 페이지에 둘 수 있는지 판단할 때, 렌더러에서 실제로 발생하는
다음 overhead를 잔여 공간에서 차감한다.

```text
host_spacing.before
TopAndBottom + vert=Para 표의 positive vertical_offset
```

또한 다행 표의 비분할 첫 행/블록이 현재 페이지에는 안 들어가지만 fresh page에는 들어가면,
현재 페이지에 억지로 붙잡지 않고 다음 페이지로 이월한다.

### 4.2 trailing 간격 overflow 오검출 정정

표나 문단 콘텐츠 자체는 본문 안에 있는데, 표 뒤/문단 끝의 trailing 간격이 더해진 `y_offset`으로
overflow를 판정하던 false-positive를 줄인다.

이를 위해 `LayoutEngine`에 `last_item_content_bottom`을 추가하고, 표/문단 렌더 시 실제 콘텐츠
하단을 기록해 overflow 검출에 사용한다.

## 5. 주요 코드 변경

| 파일 | 내용 |
|---|---|
| `src/renderer/typeset.rs` | `force_break_before` 훅 유지, 표 분할 첫 fragment overhead 차감, 다행 표 조건부 이월 |
| `src/renderer/layout.rs` | `LayoutOverflow`에 `section_index`, `is_first_in_column` 추가, `last_item_content_bottom` 기반 overflow 판정 |
| `src/renderer/layout/paragraph_layout.rs` | 문단 렌더링 중 실제 텍스트 콘텐츠 하단 기록 |
| `src/document_core/queries/rendering.rs` | `paginate_pass(force_breaks)` 구조와 typeset 호출부 정합 |
| `mydocs/plans`, `mydocs/working`, `mydocs/report` | #1046 설계/작업/결과 문서 추가 |

## 6. 현재 코드 반영 여부

현재 `local/devel`에는 PR #1093의 핵심 구현이 아직 없다.

확인 결과:

```text
LayoutOverflow.section_index / is_first_in_column 없음
LayoutEngine.last_item_content_bottom 없음
typeset_section force_break_before 파라미터 없음
task_m100_1046 문서 없음
```

현재 `local/devel`은 PR #1084, PR #1091이 반영된 상태다.
PR #1093의 merge-base는 `27441c84`로, 현재 devel보다 이전이다.

따라서 단순 branch diff에서는 PR #1084/#1091 문서와 코드가 사라지는 것처럼 보인다.
수용 시에는 반드시 `-x` cherry-pick으로 현재 `local/devel` 위에 적용해야 한다.

## 7. 충돌/리스크

### 7.1 예상 충돌

`git merge-tree` 기준으로 `src/renderer/typeset.rs`는 양쪽에서 변경된 파일이다.

현재 `local/devel`에는 다음 변경이 이미 들어와 있다.

```text
PR #1084: 그림 pushdown/vpos 이중 계상 정정
PR #1091: 다단 미주 vpos delta 누적 정합
```

PR #1093도 `typeset.rs`의 표/문단 페이지네이션 경로를 수정하므로, 체리픽 시 충돌 또는 의미 충돌
가능성이 있다.

### 7.2 기능 리스크

이 PR은 단순 버그 수정이 아니라 overflow 측정/판정 정책을 바꾸는 대형 변경이다.

주의 지점:

```text
1. overflow 검출이 trailing 간격 false-positive를 줄이는 방향이라 로그 건수는 줄지만,
   실제 콘텐츠 초과를 놓치지 않는지 확인해야 한다.
2. 다행 표 조건부 이월은 표 페이지 분할 위치를 바꿀 수 있다.
3. PR #1091의 미주 overflow 개선과 같은 typeset 누적 경로에서 충돌하지 않는지 확인해야 한다.
4. PR #1084의 그림 pushdown 조건부 생략과 페이지 배치가 함께 유지되는지 확인해야 한다.
```

### 7.3 재현 자료 한계

PR 본문에는 비공개 185p 문서 기준 결과가 포함되어 있다.
따라서 모든 정량 수치를 로컬 공개 샘플만으로 재현하기는 어렵다.
수용 판단은 다음 조합으로 해야 한다.

```text
원 PR #1048에서 이미 메인테이너가 설계/정량 검증을 양호로 판정한 사실
현재 PR CI 통과
현재 devel 기준 체리픽 후 자동 검증
메인테이너 시각 판정/광범위 sweep
```

## 8. 권장 처리 방향

권장안:

```text
1. PR #1093 단일 커밋을 현재 local/devel에 -x cherry-pick 한다.
2. 충돌이 나면 PR #1084/#1091 변경을 보존하면서 typeset.rs를 수동 정리한다.
3. 현재 devel 기준으로 fmt/check/test를 다시 수행한다.
4. wasm build 후 메인테이너 시각 판정을 받는다.
5. 통과하면 devel에 반영하고 PR #1093 및 이슈 #1046을 체리픽 수용으로 close 한다.
```

바로 merge commit으로 받기보다는 `-x` cherry-pick이 적합하다.

이유:

```text
- PR base가 현재 devel보다 오래되었다.
- 현재 devel에는 #1084/#1091의 typeset 변경이 이미 들어와 있다.
- 외부 기여자 출처를 커밋 메시지에 남길 수 있다.
- 현재 devel 기준 충돌/회귀를 명시적으로 검증할 수 있다.
```

이슈 #1065는 PR 본문에서 개선 효과를 언급하지만, 별도 후속/잔여 이슈 성격이다.
이번 PR 수용으로 자동 close하지 않고, 필요하면 별도 메인테이너 판단 후 처리한다.

## 9. 승인 요청

다음 절차로 진행해도 되는지 승인 요청한다.

```text
git cherry-pick -x f269935f6b1f341d2ea0da3f91d8a481361a18cd
cargo fmt --check
cargo check
cargo test --lib
docker compose --env-file .env.docker run --rm wasm
```

체리픽 충돌이 발생하면 `typeset.rs`, `layout.rs`, `rendering.rs` 중심으로 PR #1084/#1091 변경을
보존하는 방향으로 해결한 뒤 검증한다.
