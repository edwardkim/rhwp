# Task #508 Stage 3 완료 보고서 — serialization test 보강

## 진행 범위

구현 계획서에 따라 `PaintOp::Image` JSON serialization 테스트에 `brightness`, `contrast` 검증을 추가했다.

## 변경 내용

`src/paint/json.rs`의 기존 테스트 `serializes_backend_replay_payload_fields`에서 image payload를 non-zero 보정값으로 설정했다.

```rust
let mut image = ImageNode::new(7, Some(vec![1, 2, 3]));
image.effect = ImageEffect::BlackWhite;
image.brightness = -50;
image.contrast = 70;
```

그리고 JSON assertion을 추가했다.

```rust
assert!(json.contains("\"brightness\":-50"));
assert!(json.contains("\"contrast\":70"));
```

## 변경 파일

| 파일 | 내용 |
|------|------|
| `src/paint/json.rs` | image paint op serialization test에 brightness/contrast assertion 추가 |

## 검증

### 단일 테스트

```bash
cargo test --lib paint::json::tests::serializes_backend_replay_payload_fields
```

결과:

- 1 passed
- 0 failed

### 관련 paint JSON 테스트

```bash
cargo test --lib paint::json
```

결과:

- 4 passed
- 0 failed

컴파일 중 기존 경고 4건이 출력되었지만 이번 변경과 무관한 기존 테스트 코드 경고다.

## schemaVersion

`PAGE_LAYER_TREE_SCHEMA_VERSION`은 변경하지 않았다. Stage 1 판단과 구현 계획서에 따른 additive JSON field 추가다.

## 다음 단계

작업지시자 승인 후 Stage 4에서 최종 검증과 보고서 작성을 진행한다.
