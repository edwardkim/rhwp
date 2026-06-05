# PR #1290 리뷰 — HWPX Bookmark/Field dispatcher 연결

- PR: https://github.com/edwardkim/rhwp/pull/1290
- 관련 이슈: #1289
- 작성일: 2026-06-04
- 작성자: `Martinel2`
- 제목: `Task #1289 fix: HWPX 시리얼라이저 Bookmark/Field dispatcher 연결`
- base: `devel`
- head: `devel` / `ca5a69b2c74a5ca30c2aedbf767411a2366c3eed`
- 상태: open, draft 아님
- GitHub mergeable: true

## 1. PR 요약

PR #1290은 HWPX serializer에서 이미 존재하던 `field.rs`의 Bookmark/Field writer를
`section.rs` 본문 직렬화 경로에 연결한다.

해결하려는 문제:

- `Control::Bookmark`가 HWPX 저장 시 누락됨
- `Control::Field`의 `<hp:fieldBegin>`이 dispatcher에 연결되지 않아 필드 컨트롤이 소실됨
- `<hp:fieldEnd>`는 `para.controls[]`가 아니라 `para.field_ranges[]`에만 위치 정보가 있어 별도 삽입이 필요함

## 2. 변경 범위

| file | 변경 |
|---|---|
| `src/serializer/hwpx/field.rs` | 모듈 전체 `dead_code` 허용 제거, 미연결 함수만 개별 허용 |
| `src/serializer/hwpx/section.rs` | Bookmark prefix 출력, Field begin/end 출력, `fieldEnd` slot count 보정, 단위 테스트 3개 추가 |

통계:

```text
2 files changed, 180 insertions(+), 8 deletions(-)
```

## 3. 현재 CI 상태

PR head `ca5a69b2c74a5ca30c2aedbf767411a2366c3eed` 기준:

- CodeQL: success
- CI: success

## 4. 코드 검토

### 4.1 방향성

HWPX 출력 경로에 `Control::Field` dispatcher가 빠져 있던 것은 실제 결함이다.
HWP 저장 경로(`src/serializer/body_text.rs`)는 이미 `field_ranges`를 사용해 FIELD_END를
텍스트 흐름 안에 삽입하고 있으므로, HWPX 저장 경로도 같은 IR 계약을 사용해야 한다.

따라서 다음 변경은 타당하다.

- `Control::Field`를 `<hp:ctrl><hp:fieldBegin .../></hp:ctrl>`로 출력
- `FieldRange.end_char_idx` 기준으로 `<hp:fieldEnd beginIDRef="..."/>` 출력
- `fieldEnd`가 `controls[]`에 없는 점을 고려해 slot count에서 `field_ranges.len()`을 감산
- Bookmark를 완전히 소실시키지 않고 `<hp:ctrl><hp:bookmark .../></hp:ctrl>`로 보존

### 4.2 Bookmark 위치 한계

PR은 Bookmark 위치 정보가 IR에 없으므로 문단 시작 prefix로 출력한다.

이는 완전 소실보다 낫지만, mid-paragraph bookmark의 정확 위치 보존은 아직 해결하지 않는다.
PR 본문도 이 한계를 별도 이슈 범위로 분리하고 있어 수용 가능하다.

### 4.3 주요 위험: 빈 필드 / 0-length field range 순서

현재 PR 패치의 `render_run_content()` 후반부 흐름은 다음 형태다.

```text
1. 텍스트 루프 중 fieldEnd 출력
2. 루프 후 아직 미출력 fieldEnd 출력
3. 남은 control slot 출력
```

이 순서는 텍스트가 있는 일반 필드에는 동작하지만, 다음 케이스에서 XML 순서가 깨질 수 있다.

- `para.text`가 비어 있는 필드
- `FieldRange { start_char_idx: 0, end_char_idx: 0, ... }`
- ClickHere 안내문 삭제 후 빈 필드로 정규화된 케이스

이 경우 텍스트 루프가 없거나 `end_char_idx == 0`을 루프에서 감지하지 못한다.
그 뒤 루프 후 처리에서 `fieldEnd`가 먼저 출력되고, 마지막 `while slot_idx < slots.len()`에서
`fieldBegin`이 뒤에 출력될 가능성이 있다.

결과 예:

```xml
<hp:ctrl><hp:fieldEnd beginIDRef="..."/></hp:ctrl>
<hp:ctrl><hp:fieldBegin id="..." .../></hp:ctrl>
```

이는 기대 순서인 `fieldBegin -> fieldEnd`와 반대다.

HWP 저장 경로는 이 문제를 피하기 위해 `trailing_end_after_ctrl` 구조를 사용한다.
즉 남은 FIELD_BEGIN 컨트롤을 출력한 직후 대응 FIELD_END를 인터리빙한다.
HWPX 저장 경로도 같은 계약을 따라야 한다.

### 4.4 테스트 보강 필요

PR의 테스트 3개는 다음을 확인한다.

- Bookmark wrapper 출력
- fieldBegin -> 텍스트 -> fieldEnd 순서
- `end_char_idx == text.len()` 케이스

하지만 실제 위험 케이스인 빈 필드는 빠져 있다.

추가 권장 테스트:

```text
task1289_empty_field_begin_end_order
```

검증 조건:

- `para.text == ""`
- `char_count == 17` 또는 fieldBegin 8 + fieldEnd 8 + para end 1에 해당하는 값
- `controls = [Control::Field(...)]`
- `field_ranges = [{ start_char_idx: 0, end_char_idx: 0, control_idx: 0 }]`
- 출력 XML에서 `fieldBegin`이 `fieldEnd`보다 앞서야 함

## 5. 권장 처리

권장: **수용**.

근거:

- PR의 문제 진단과 dispatcher 연결 방향은 맞다.
- 코드 변경 범위가 작고 CI도 통과했다.
- 다만 빈 필드/0-length field range는 rhwp에서 실제로 존재하는 계약이며, 잘못 저장되면 한컴 호환성 문제가 될 수 있다.
- 현재 HWPX 시리얼라이제이션은 사용자에게 직접 사용을 막아둔 상태이므로, 이 위험은 후속 PR에서 보완하는 것으로 분리한다.
- HWP 저장 경로에는 이미 같은 문제를 피하는 인터리빙 패턴이 있으므로, 후속 보완 난이도는 높지 않아 보인다.

후속 PR 권장 보완:

1. 루프 후 남은 `fieldEnd`를 무조건 먼저 출력하지 말고, 남은 `Field` control slot 출력 직후 대응 `fieldEnd`를 출력한다.
2. `end_char_idx == 0` / 빈 필드 회귀 테스트를 추가한다.
3. 기존 테스트 3개와 함께 `cargo test --lib task1289`를 통과시킨다.

이번 PR 처리 절차:

1. 최신 `local/devel` 기준 통합 브랜치 생성
2. PR #1290 커밋 cherry-pick
3. `cargo fmt --all -- --check`
4. `cargo test --lib task1289`
5. 필요 시 `cargo test --lib field`
6. 통과 시 `local/devel`에 병합 후 push
7. PR #1290 및 issue #1289 종료 처리

## 6. 검증 계획

로컬 검증 권장:

```text
cargo fmt --all -- --check
cargo test --lib task1289
cargo test --lib field
cargo test --test issue_838_field_set_value -- --nocapture
```

필요 시 HWPX 저장 샘플을 추가로 확인한다. 현재 HWPX 시리얼라이제이션은 사용자에게 직접 노출하지 않는 기능이므로, 빈 필드 보완은 후속 PR 검증 항목으로 이관한다.

## 7. PR 코멘트 초안

```markdown
검토했습니다. HWPX serializer에서 `Control::Field` dispatcher가 누락되어 fieldBegin/fieldEnd 및 Bookmark가 소실되는 문제를 해결하는 방향은 맞다고 봅니다. HWP 저장 경로처럼 `field_ranges`를 기준으로 fieldEnd를 materialize하는 접근도 타당합니다.

리뷰 중 텍스트가 없는 빈 필드나 `FieldRange { start_char_idx: 0, end_char_idx: 0, ... }` 케이스에서 `fieldEnd`가 남은 `fieldBegin` control slot보다 먼저 출력될 가능성은 확인했습니다. 다만 현재 HWPX 시리얼라이제이션은 사용자에게 직접 노출하지 않는 경로이고, 이번 PR은 누락된 dispatcher를 연결해 기존 소실 문제를 줄이는 효과가 더 크다고 판단했습니다.

따라서 이번 PR은 메인테이너 통합 브랜치에서 수용하겠습니다. 빈 필드/0-length field range의 정확한 begin/end 인터리빙은 후속 PR에서 처리해 주세요. HWP 저장 경로의 `trailing_end_after_ctrl`처럼, 남은 fieldBegin control을 출력한 직후 대응 fieldEnd를 인터리빙하는 방식과 `para.text == ""` 테스트를 추가하면 좋겠습니다.

기여 감사합니다.
```

## 8. 통합 진행 기록

통합 브랜치:

```text
local/pr1290-integration
```

적용:

```text
git fetch origin pull/1290/head:local/pr1290-upstream
git cherry-pick 319b55d3
```

주의:

- PR head에는 merge commit `ca5a69b2`가 포함되어 있었지만, base가 최신 `devel`보다 오래되어 전체 head를 병합하면 최근 `devel` 변경이 되돌아가는 diff가 발생한다.
- 따라서 실제 기능 커밋 `319b55d3`만 cherry-pick 했다.

검증:

```text
cargo fmt --all -- --check
cargo test --lib task1289
cargo test --lib field
cargo test --test issue_838_field_set_value -- --nocapture
```

결과:

- `cargo fmt --all -- --check`: 통과
- `cargo test --lib task1289`: 3 passed
- `cargo test --lib field`: 35 passed
- `cargo test --test issue_838_field_set_value -- --nocapture`: 2 passed

판정:

- PR #1290 수용 가능
- 빈 필드/0-length field range begin/end 인터리빙 보완은 후속 PR 범위로 이관
