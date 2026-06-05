# PR #1081 검토 문서

- PR: <https://github.com/edwardkim/rhwp/pull/1081>
- 제목: `refactor: CommonObjAttr raw_ctrl_data 바이트 오프셋 상수 모듈 (#698 후속)`
- 관련 이슈: <https://github.com/edwardkim/rhwp/issues/698>
- 작성일: 2026-05-26
- 작성자: Codex

## 1. PR 상태

| 항목 | 값 |
|---|---|
| 상태 | open |
| base | `devel` |
| head | `contrib/common-obj-attr-offset-constants` |
| head sha | `fd02e1ebc852f50716e236ff3a09adf4c91ee02a` |
| mergeable | true |
| 작성자 | `oksure` |
| 커밋 수 | 2 |
| 변경 파일 | 1개 |
| 변경 범위 | `src/model/shape.rs` |

CI 확인:

| workflow | conclusion |
|---|---|
| Build & Test | pass |
| CodeQL | pass |
| Analyze (rust) | pass |
| Analyze (javascript-typescript) | pass |
| Analyze (python) | pass |
| WASM Build | skipping |

## 2. PR 주장

PR #1081은 `raw_ctrl_data`의 CommonObjAttr 바이트 레이아웃을 직접 인덱스로 쓰는 패턴을 줄이기 위해
`common_obj_offsets` 모듈을 추가한다.

추가되는 상수:

```rust
pub(crate) mod common_obj_offsets {
    pub const FLAGS: std::ops::Range<usize> = 0..4;
    pub const V_OFFSET: std::ops::Range<usize> = 4..8;
    pub const H_OFFSET: std::ops::Range<usize> = 8..12;
    pub const WIDTH: std::ops::Range<usize> = 12..16;
    pub const HEIGHT: std::ops::Range<usize> = 16..20;
    pub const Z_ORDER: std::ops::Range<usize> = 20..24;
    pub const MARGIN_LEFT: std::ops::Range<usize> = 24..26;
    pub const MARGIN_RIGHT: std::ops::Range<usize> = 26..28;
    pub const MARGIN_TOP: std::ops::Range<usize> = 28..30;
    pub const MARGIN_BOTTOM: std::ops::Range<usize> = 30..32;
    pub const INSTANCE_ID: std::ops::Range<usize> = 32..36;
    pub const MIN_LEN: usize = INSTANCE_ID.end;
}
```

Copilot 피드백은 이미 반영되어 있다.

```text
1. pub mod -> pub(crate) mod
2. MIN_LEN = 36 -> MIN_LEN = INSTANCE_ID.end
```

## 3. 현재 코드의 중복 지점

현재 `local/devel`에는 CommonObjAttr raw byte offset을 직접 쓰는 코드가 여러 곳에 남아 있다.

대표 지점:

```text
src/document_core/html_table_import.rs
src/document_core/commands/table_ops.rs
src/document_core/commands/object_ops.rs
src/document_core/converters/hwpx_to_hwp.rs
src/model/table.rs
```

예:

```rust
raw_ctrl_data[0..4].copy_from_slice(&table_attr.to_le_bytes());
raw_ctrl_data[12..16].copy_from_slice(&total_width.to_le_bytes());
raw_ctrl_data[16..20].copy_from_slice(&total_height.to_le_bytes());
raw_ctrl_data[24..26].copy_from_slice(&outer_margin.to_le_bytes());
raw_ctrl_data[32..36].copy_from_slice(&instance_id.to_le_bytes());
```

#1077, #1078에서 실제로 발생한 문제는 이 레이아웃을 사람이 직접 맞추다가
`[0..4]`를 offset처럼 해석하는 식의 4바이트 밀림이 생긴 것이다.

따라서 "명명된 offset 상수로 재발 가능성을 낮춘다"는 PR의 문제의식은 타당하다.

## 4. 필요성 평가

상수 모듈 도입은 필수 버그 픽스는 아니다.

이유:

```text
1. #1077 / #1078의 직접 버그는 이미 devel에 수정되었다.
2. PR #1081 자체는 기존 직접 인덱싱 사용처를 바꾸지 않는다.
3. 따라서 PR #1081만 cherry-pick하면 런타임 동작 변화는 없다.
```

하지만 유지보수 관점에서는 도입 가치가 있다.

이유:

```text
1. CommonObjAttr byte layout은 여러 생성/수정 경로가 공유한다.
2. raw slice 숫자만 보면 [0..4]가 flags인지 vertical_offset인지 즉시 드러나지 않는다.
3. 최근 PR #1077, #1078에서 같은 4바이트 밀림 유형이 반복되었다.
4. 상수 이름은 코드 리뷰와 향후 수정에서 의미 검증 비용을 줄인다.
```

## 5. 효과 평가

### 5.1 PR 그대로 수용할 때의 효과

```text
효과:
  - CommonObjAttr layout의 단일 명명 기준점이 생긴다.
  - 후속 마이그레이션의 준비물이 생긴다.

한계:
  - 기존 raw_ctrl_data 직접 인덱싱은 그대로 남는다.
  - 즉시 재발 방지 효과는 제한적이다.
  - 상수가 parser/writer와 실제로 연결되어 검증되는 테스트는 없다.
```

즉 PR #1081은 "완성된 재발 방지"라기보다 "재발 방지 마이그레이션의 첫 단계"다.

### 5.2 상수 도입 후 바로 사용처를 일부 마이그레이션할 때의 효과

다음 고위험 쓰기 경로까지 바꾸면 효과가 분명해진다.

```text
src/document_core/html_table_import.rs
src/document_core/commands/object_ops.rs
src/document_core/commands/table_ops.rs
src/model/table.rs
```

예:

```rust
raw_ctrl_data[common_obj_offsets::WIDTH].copy_from_slice(&total_width.to_le_bytes());
raw_ctrl_data[common_obj_offsets::INSTANCE_ID].copy_from_slice(&instance_id.to_le_bytes());
```

이 방식이면 리뷰 시 "WIDTH를 쓰는가"를 보면 되고, 숫자 범위를 매번 다시 계산하지 않아도 된다.

## 6. 위치와 API 판단

PR은 상수를 `src/model/shape.rs`에 둔다.

장점:

```text
1. CommonObjAttr 구조체와 가까워 발견 가능성이 높다.
2. parser, serializer, document_core 어디서든 crate 내부 접근이 가능하다.
3. pub(crate)라 외부 API 안정성 문제는 없다.
```

주의점:

```text
1. raw_ctrl_data layout은 IR 모델보다는 HWP5 binary contract 성격이 강하다.
2. 장기적으로는 `common_obj_attr_writer` 또는 HWP5 contract 전용 모듈이 더 자연스러울 수 있다.
```

현재 코드 구조에서는 `model::shape::CommonObjAttr` 자체가 HWP5 common attr와 강하게 연결되어 있으므로,
PR 위치는 수용 가능한 수준이다.

## 7. 판단

컨트리뷰터의 주장은 절반은 맞고, 절반은 보완이 필요하다.

```text
맞는 부분:
  CommonObjAttr raw byte offset 상수화는 필요하다.
  같은 4바이트 밀림 버그가 반복된 이상, 숫자 slice를 계속 방치하면 재발 가능성이 있다.

부족한 부분:
  PR #1081은 상수를 추가할 뿐 기존 사용처를 바꾸지 않는다.
  따라서 이 PR 하나만으로는 실질적 재발 방지 효과가 작다.
```

## 8. 권장 처리

권장안:

```text
수용하되, 단순 cherry-pick으로 끝내지 않는다.
```

구체 절차:

```text
1. PR #1081의 상수 모듈을 cherry-pick한다.
2. 같은 작업에서 최소한 고위험 쓰기 경로를 상수 사용으로 바꾼다.
   - html_table_import.rs
   - object_ops.rs
   - table_ops.rs 중 raw_ctrl_data CommonObjAttr write 지점
   - model/table.rs update_raw_dimensions
3. 기존 테스트를 실행한다.
   - cargo fmt --check
   - cargo check
   - CommonObjAttr / HTML table import / table ops 관련 테스트
4. 이후 PR #1081은 "상수 도입 + maintainer side migration"으로 close한다.
```

대안:

```text
PR #1081 그대로 cherry-pick 후 별도 follow-up 이슈로 마이그레이션
```

하지만 이 경우 PR의 실질 효과가 너무 약하다.
이번 처리에서 최소 고위험 쓰기 경로까지 같이 바꾸는 편이 낫다.
