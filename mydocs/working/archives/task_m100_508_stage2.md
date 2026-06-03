# Task #508 Stage 2 완료 보고서 — JSON serialization 구현

## 진행 범위

구현 계획서 승인에 따라 `src/paint/json.rs`의 `PaintOp::Image` JSON serialization만 수정했다.

## 변경 내용

`PaintOp::Image` 분기에서 기존 `effect` 출력 뒤에 `brightness`, `contrast`를 항상 출력하도록 변경했다.

변경 후 image paint op JSON은 다음 형태가 된다.

```json
{
  "type": "image",
  "bbox": { "x": 3.000, "y": 4.000, "width": 30.000, "height": 20.000 },
  "effect": "blackWhite",
  "brightness": 0,
  "contrast": 0,
  "transform": { "rotation": 0.000, "horzFlip": false, "vertFlip": false }
}
```

`ImageNode::new()` 기본값과 맞춰 `brightness=0`, `contrast=0`도 생략하지 않는다.

## schemaVersion

`PAGE_LAYER_TREE_SCHEMA_VERSION`은 변경하지 않았다.

사유:

- 기존 필드 제거 없음
- 기존 필드 의미 변경 없음
- image paint op에 `brightness`, `contrast`를 추가하는 additive 변경
- 기존 PageLayerTree 정책상 incompatible change가 아니면 schemaVersion 유지

## 변경 파일

| 파일 | 내용 |
|------|------|
| `src/paint/json.rs` | `PaintOp::Image` serialization에 `brightness`, `contrast` 출력 추가 |

## 검증

기존 paint JSON 테스트를 실행했다.

```bash
cargo test --lib paint::json
```

결과:

- `paint::json::tests::serializes_text_and_shape_ops_for_browser_replay` 통과
- `paint::json::tests::serializes_backend_replay_payload_fields` 통과
- `paint::json::tests::serializes_layer_node_metadata` 통과
- `paint::json::tests::serializes_layer_output_options` 통과

총 4개 통과, 실패 없음.

컴파일 중 기존 경고 4건이 출력되었지만 이번 변경과 무관한 기존 테스트 코드 경고다.

## 다음 단계

작업지시자 승인 후 Stage 3에서 `serializes_backend_replay_payload_fields` 테스트에 `brightness`, `contrast` assertion을 추가한다.
