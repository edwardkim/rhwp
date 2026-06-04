# PR #1291 리뷰 — leading-gap 클릭 hit-test caret snap 보정

- PR: https://github.com/edwardkim/rhwp/pull/1291
- 작성일: 2026-06-04
- 작성자: `humdrum00001010`
- 제목: `Fix hit-test caret snapping to line start/end on leading-gap clicks`
- base: `devel`
- head: `fix/hit-test-leading-gap` / `156237ffc9378908743f984d987ad01ebaf813b9`
- 상태: open, draft 아님
- GitHub mergeable: true

## 1. PR 요약

PR #1291은 `DocumentCore::hit_test_native`에서 클릭 y가 글리프 bbox 내부가 아니라
줄의 leading gap에 떨어질 때 caret이 줄 시작/끝으로 스냅되는 문제를 고친다.

기존 문제:

- y가 TextRun bbox 내부이면 x 기준 문자 위치를 정확히 찾는다.
- y가 bbox 바로 위/아래 행간 영역이면 fallback 경로가 x를 무시하고 줄 시작 또는 줄 끝으로 보낸다.
- 다중 run 줄에서도 run 사이 빈틈 클릭이 line end로 붕괴될 수 있다.

PR 접근:

- 줄 단위 helper `resolve_x_on_line()` 추가
- cell/body/closest-line fallback 경로를 같은 x-resolution 로직으로 통합
- x가 run 내부이면 기존 `find_char_at_x` 의미로 문자 위치 계산
- x가 run 사이 빈틈이면 가까운 run 경계 선택
- 빈 입력칸처럼 `char_count=0`이지만 bbox가 넓은 run은 `char_start`로 clamp

## 2. 변경 범위

| file | 변경 |
|---|---|
| `src/document_core/queries/cursor_rect.rs` | `LineRunView`, `resolve_x_on_line` 추가 및 hit-test fallback 경로 통합 |
| `tests/hit_test_leading_gap.rs` | `samples/exam_social.hwp` 기반 leading-gap hit-test 회귀 테스트 추가 |

통계:

```text
2 files changed, 376 insertions(+), 72 deletions(-)
```

## 3. 현재 GitHub Actions 상태

PR head `156237ffc9378908743f984d987ad01ebaf813b9` 기준:

- CodeQL: `action_required`
- CI: `action_required`

처음 기여자의 PR이라 GitHub Actions 실행 승인이 대기 중인 상태로 보인다.
즉 현재 상태는 테스트 실패가 아니라 workflow 실행 전 승인 대기다.
수용 전 maintainer integration 브랜치에서 로컬 테스트를 반드시 실행해야 한다.

## 4. 코드 검토

### 4.1 문제 진단

현재 `cursor_rect.rs`에는 클릭 위치를 먼저 hit run으로 찾고, 실패하면 다음 fallback을 사용한다.

- 셀 내부 run fallback
- 본문 same-line fallback
- closest-line fallback

기존 fallback들은 y가 글리프 bbox 내부일 때만 정확한 x 해석을 수행하고,
그 외에는 줄 시작 또는 끝으로 보내는 경향이 있었다.
PR의 문제 정의는 실제 코드 흐름과 맞다.

### 4.2 `resolve_x_on_line` helper

새 helper는 line run들을 `bbox_x` 기준으로 정렬한 뒤 다음 규칙을 적용한다.

- 첫 run 왼쪽: 첫 run 시작
- run 내부: local x를 문자 offset으로 변환
- run 사이 빈틈: 가까운 run 경계
- 마지막 run 오른쪽: 마지막 run 끝

이 정책은 일반적인 caret hit-test 기대 동작과 맞다.
특히 leading-gap 클릭은 y로 줄만 선택하고, x는 선택된 줄 전체에서 다시 해석해야 하므로
이번 변경 방향이 적절하다.

### 4.3 기존 `find_char_at_x`와 의미 정합

기존 `src/document_core/helpers.rs::find_char_at_x`는 `positions[0] = 0.0`,
`positions.len() = char_count + 1`을 전제로 중간점 기준 offset을 반환한다.

PR의 `line_local_char_at_x`도 같은 미드포인트 규칙을 사실상 재구현하며,
그 뒤 `min(r.char_count)`로 clamp한다.

검토 의견:

- 의미 방향은 맞다.
- 빈 입력칸의 `char_positions = [0.0]`, `char_count = 0` 사례에서 clamp가 필요한 것도 타당하다.
- 다만 장기적으로는 helper 중복을 줄이기 위해 `find_char_at_x(...).min(char_count)`를 사용하는 방향도 고려할 수 있다. 이번 PR 수용의 blocker는 아니다.

### 4.4 셀 내부 경로

셀 내부 클릭에서 PR은 다음 순서로 줄을 고른다.

1. y가 bbox 안에 드는 run의 `bbox_y`
2. 없으면 y 중심이 가장 가까운 run의 `bbox_y`
3. 같은 `bbox_y`인 run들을 모아 x 해석

이는 leading-gap 클릭을 처리하는 데 필요한 동작이다.
같은 줄 run의 `bbox_y`가 1px 이내로 일치한다는 전제는 기존 closest-line 코드에서도 사용하던 전제이므로 범위 내 변경으로 볼 수 있다.

### 4.5 본문/closest-line 경로

본문 same-line fallback과 closest-line fallback도 `resolve_x_on_line`으로 통합한다.
기존 코드가 줄 왼쪽/오른쪽으로만 분기하던 부분을 줄 내부 x 해석으로 바꿔,
다중 run 줄에서도 run 사이 빈틈을 line end로 보내지 않게 한다.

### 4.6 테스트 구성

테스트는 두 층이다.

- 단위 테스트: `resolve_x_on_line` 직접 검증
- 통합 테스트: `samples/exam_social.hwp`의 실제 line click sweep 검증

좋은 점:

- 기존 devel에서 실패하는 leading-gap 케이스를 고정한다.
- inside glyph box와 leading gap을 같은 x sweep으로 비교한다.
- empty run clamp 케이스를 단위 테스트로 포함한다.

주의점:

- 통합 테스트가 `samples/exam_social.hwp`의 렌더 좌표에 의존한다.
- font/측정 변경이 있으면 tolerance 재조정이 필요할 수 있다.
- 그러나 repo 내 기존 렌더링 회귀 테스트도 샘플 좌표 기반이 많으므로 현재 프로젝트 관행과 어긋나지는 않는다.

## 5. 권장 처리

권장: **수용 방향으로 진행**.

근거:

- 실제 사용자 UX에서 클릭 y가 glyph box 내부에 정확히 들어가지 않는 것은 흔하다.
- PR이 문제를 body/cell/closest-line fallback 모두에서 공통 helper로 줄이는 방향이라 중복/분기 위험을 낮춘다.
- empty input cell guard까지 포함해 기존 답안지 빈칸 hit-test 회귀를 방지한다.
- 변경 파일이 `cursor_rect.rs`와 신규 테스트 하나로 제한되어 있다.

단, 처음 기여자 PR이라 GitHub Actions가 `action_required` 상태이므로 maintainer integration에서 다음 검증을 선행해야 한다.

권장 절차:

1. 최신 `local/devel` 기준 통합 브랜치 생성
2. PR #1291 실제 기능 커밋만 cherry-pick
3. `cargo fmt --all -- --check`
4. `cargo test --test hit_test_leading_gap -- --nocapture`
5. `cargo test --test issue_850_answer_sheet_name_hit_test -- --nocapture`
6. `cargo test --lib cursor_rect`
7. 필요 시 `cargo test --tests --quiet`
8. 통과 후 wasm 빌드 및 maintainer 동작 테스트
9. 통과 시 `devel` 병합/push 및 PR 종료 처리

## 6. 확인해야 할 UX 포인트

수동 동작 테스트 권장:

- `samples/exam_social.hwp`
- 본문 줄에서 글자 위가 아닌 줄 위쪽 행간을 클릭했을 때 caret x가 클릭 x 근처로 이동
- 같은 줄의 앞/중간/뒤 클릭이 각각 다른 offset으로 이동
- 빈 입력칸/답안지 셀 클릭 시 offset이 1 이상으로 새지 않음

## 7. PR 코멘트 초안

```markdown
검토했습니다. `hit_test_native`의 leading-gap 클릭에서 x 해석이 줄 시작/끝으로 붕괴되는 문제를 공통 `resolve_x_on_line` helper로 정리한 접근이 적절해 보입니다.

특히 cell fallback, body same-line fallback, closest-line fallback이 각각 다른 방식으로 줄 시작/끝 스냅을 하던 부분을 같은 line-level x-resolution 정책으로 통합한 점이 좋습니다. 빈 입력칸처럼 `char_count=0`이지만 bbox가 넓은 run을 `char_start`로 clamp한 것도 기존 답안지 입력칸 동작을 지키는 데 필요해 보입니다.

현재 GitHub Actions는 처음 기여자 PR이라 실행 승인 대기(`action_required`) 상태입니다. maintainer integration 브랜치에서 로컬 검증을 먼저 진행하겠습니다. 우선 `cargo fmt --all -- --check`, `cargo test --test hit_test_leading_gap -- --nocapture`, `cargo test --test issue_850_answer_sheet_name_hit_test -- --nocapture`, `cargo test --lib cursor_rect`를 확인한 뒤 수용 절차를 진행하겠습니다.

기여 감사합니다.
```

## 8. 통합 진행 기록

통합 브랜치:

```text
local/pr1291-integration
```

적용:

```text
git fetch origin pull/1291/head:local/pr1291-upstream
git cherry-pick f1d21602
git cherry-pick 156237ff
```

주의:

- PR head는 최신 `devel`보다 오래된 base 기준이라 전체 head diff를 보면 최근 `mydocs` archive 정리와 다른 PR 통합 내용이 대량으로 되돌아가는 모양이 나온다.
- 따라서 실제 기능 커밋 `f1d21602`와 임시 `UPSTREAM_PR.md` 삭제 커밋 `156237ff`만 적용했다.
- `cargo fmt --all -- --check`에서 테스트 코드 formatting 차이가 발견되어 `cargo fmt --all` 적용 후 maintainer 포맷 커밋을 추가했다.

검증:

```text
cargo fmt --all -- --check
cargo test --test hit_test_leading_gap -- --nocapture
cargo test --test issue_850_answer_sheet_name_hit_test -- --nocapture
cargo test --lib cursor_rect
docker compose --env-file .env.docker run --rm wasm
```

결과:

- `cargo fmt --all -- --check`: 통과
- `cargo test --test hit_test_leading_gap -- --nocapture`: 2 passed
- `cargo test --test issue_850_answer_sheet_name_hit_test -- --nocapture`: 3 passed
- `cargo test --lib cursor_rect`: 7 passed
- WASM build: 통과, `pkg/rhwp_bg.wasm` 5.3M 생성

판정:

- PR #1291 수용 가능
- GitHub Actions `action_required`는 처음 기여자 PR의 실행 승인 대기이며 실패가 아님
- maintainer local integration 검증 통과
