# Task m100-2912 처리 결과

## 이슈

https://github.com/edwardkim/rhwp/issues/2912

## 대상

`src/document_core/commands/object_ops/shape.rs` — `ungroup_shape_native`

같은 파일의 `group_shapes_native`가 오늘 #2905/#2910에서 "그룹 삽입 시 char_offsets 전체를
무조건 +8" 하던 버그를 수정했다. 반대 방향 연산인 `ungroup_shape_native`(1개 컨트롤 →
N개 자식)에서 동일 계열 버그가 남아있는지 점검한 결과, 실제로 존재함을 확인했다.

## 근거

수정 전 코드(문단 전체 char_offsets에 조건 없이 net_delta 적용):

```rust
let children_count = insert_idx - control_idx;
if children_count > 1 && !para.char_offsets.is_empty() {
    let net_delta = ((children_count - 1) * 8) as u32;
    for co in para.char_offsets.iter_mut() {
        *co += net_delta;
    }
}
```

`delete_shape_control_native`(같은 파일, 991~1091행)는 `gap_start`/`threshold`를 계산해
삭제 지점 이후 항목만 시프트하도록 이미 올바르게 구현되어 있어, 대조군으로 삼아 버그를
확정했다.

## 수정

`group_shapes_native`의 #2905 수정과 동일한 패턴: `find_control_text_positions(para)`로
언그룹 지점의 텍스트축 위치를 구하고, 그 지점 이후의 `para.char_offsets` 항목에만
net_delta를 적용하도록 변경했다.

## 테스트 (red → green, 최소 1건)

`ungroup_shape_only_shifts_char_offsets_after_insertion_point`
(`resize_clamp_tests` 모듈, shape.rs)

- 문단에 텍스트 "A"(char_offsets=[0])를 두고, 그 뒤(char_offset=1)에 사각형 3개를 삽입.
- 첫 번째 사각형은 문단에 그대로 남기고, 나머지 두 개를 GroupShape로 직접 감싸 삽입한다
  (group_shapes_native 자체는 이 작업 범위 밖의 별도 결함(#2905 계열, 아직 devel에
  미병합)이 있어, 이 테스트가 검증하려는 ungroup_shape_native 로직만 독립적으로
  검증하기 위해 그룹 생성 단계는 직접 구성으로 우회했다).
- `ungroup_shape_native` 호출 후 char_offsets가 여전히 `[0]`인지 확인 — "A"의 오프셋이
  밀리면 안 된다.
- 수정 전 코드로 되돌려 실행 → `left: [8], right: [0]`로 실패(red) 확인.
- 수정 코드 복원 후 재실행 → 통과(green) 확인.

## 검증 범위 (경량)

디스크 용량 문제로 지시된 대로 `cargo check --lib`과 대상 테스트 1건만 실행했다.
전체 빌드/clippy/release-test는 생략했다. 변경 파일은 `rustfmt --edition 2021`로
포맷했다.

```
cargo check --lib   # 통과
cargo test --lib ungroup_shape_only_shifts_char_offsets_after_insertion_point  # 통과
```
