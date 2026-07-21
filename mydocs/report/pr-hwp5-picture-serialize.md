# PR #2713: 최상위 ShapeObject::Picture HWP5 직렬화 무출력 버그 수정

## 이슈
- **Issue**: #2696 — 최상위 ShapeObject::Picture 무출력으로 그림 유실 + char_count 어긋남

## 분석

`src/serializer/control.rs`의 `serialize_shape_control` 함수에서 `ShapeObject::Picture` arm이 레코드를 하나도 방출하지 않는다. 주석에는 "그룹 내 그림: 그룹 직렬화 시 자식으로 처리됨 (단독 Picture는 Control::Picture로 직렬화)"라고 되어 있으나, `ungroup_shape_native`(그룹 해제)가 `Control::Shape(ShapeObject::Picture(..))`를 최상위 컨트롤로 생성하므로 이 전제는 거짓이다.

### 영향

그룹 해제 후 저장 시:
- Picture의 CTRL_HEADER가 방출되지 않아 그림 소실
- `char_count += 8`로 확보된 확장 컨트롤 문자와 CTRL_HEADER 개수가 불일치하여 이후 컨트롤 위치가 어긋남 (문서 손상)

## 변경

`serialize_picture_control` 함수(그림 전용 직렬화, 이미 검증됨)에 위임:

```rust
// before
ShapeObject::Picture(_pic) => {}
// after
ShapeObject::Picture(pic) => {
    serialize_picture_control(pic, level, ctrl_data_record, records);
}
```

## 검증
- `cargo fmt --all -- --check` — 통과
- `cargo clippy --all-targets -- -D warnings` — 통과
