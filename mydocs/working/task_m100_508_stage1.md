# Task #508 Stage 1 완료 보고서 — contract 진단 + schemaVersion 판단

## 진행 범위

수행 계획서 승인에 따라 소스 수정 없이 `PageLayerTree` image paint op의 값 전달 경로와 JSON contract를 진단했다.

## 진단 결과

### 1. `ImageNode`에는 brightness/contrast가 이미 존재

`src/renderer/render_tree.rs::ImageNode`는 다음 필드를 이미 가진다.

| 필드 | 타입 | 기본값 |
|------|------|--------|
| `effect` | `ImageEffect` | `RealPic` |
| `brightness` | `i8` | `0` |
| `contrast` | `i8` | `0` |
| `crop` | `Option<(i32, i32, i32, i32)>` | `None` |
| `original_size_hu` | `Option<(u32, u32)>` | `None` |

따라서 모델 또는 render tree 구조 변경은 필요 없다.

### 2. `ImageNode` 생성 지점은 값 전달 완료

확인한 생성 경로:

- `src/renderer/layout.rs`
- `src/renderer/layout/paragraph_layout.rs`

대표 생성 코드들은 모두 다음 값을 채운다.

```rust
effect: pic.image_attr.effect,
brightness: pic.image_attr.brightness,
contrast: pic.image_attr.contrast,
```

즉 HWP/HWPX parser에서 읽은 `pic.image_attr` 값은 `PageRenderTree`의 `ImageNode`까지 정상 전달된다.

### 3. `LayerBuilder`는 `ImageNode`를 그대로 복제

`src/paint/builder.rs`의 `RenderNodeType::Image(image)` 분기는 다음처럼 동작한다.

```rust
RenderNodeType::Image(image) => Some(vec![PaintOp::Image {
    bbox: node.bbox,
    image: image.clone(),
}]),
```

따라서 `PageRenderTree -> PageLayerTree` 변환 단계에서 brightness/contrast가 손실되는 구조가 아니다.

### 4. 누락 지점은 JSON serialization

`src/paint/json.rs`의 `PaintOp::Image` 직렬화는 현재 다음 필드를 내보낸다.

- `type`
- `bbox`
- `base64`
- `fillMode`
- `originalSize`
- `crop`
- `effect`
- `transform`

반면 `image.brightness`, `image.contrast`를 JSON에 쓰는 코드가 없다. 이슈 #508의 본질은 이 지점의 serialization 누락으로 확정했다.

## schemaVersion 판단

기존 PageLayerTree 도입 문서와 리뷰 기록에는 다음 정책이 있다.

- additive 변경은 `schemaVersion` 유지
- 호환되지 않는 변경 시 `schemaVersion` 증가

이번 변경은 image paint op에 `brightness`, `contrast` 필드를 추가하는 additive 변경이다.

- 기존 필드 제거 없음
- 기존 필드 의미 변경 없음
- 기본값 `0`은 기존 시각 의미와 동일
- downstream이 알 수 없는 필드를 무시하는 일반 JSON consumer라면 호환 유지

따라서 Stage 1 판단은 **`PAGE_LAYER_TREE_SCHEMA_VERSION` 유지**다.

다만 downstream strict schema validator가 이미 존재하고 unknown field를 오류로 처리한다면 작업지시자 판단에 따라 구현 계획서 단계에서 schemaVersion bump로 변경할 수 있다.

## 구현 방향

구현 계획서에서는 다음 방향으로 정리한다.

1. `src/paint/json.rs`의 `PaintOp::Image` serialization에 `brightness`, `contrast`를 항상 출력한다.
2. 기본값 `0`도 생략하지 않는다.
3. 기존 `serializes_backend_replay_payload_fields` 테스트에서 image 값을 non-zero로 설정하고 JSON assertion을 추가한다.
4. schemaVersion은 유지한다.

## 검증

Stage 1은 코드 변경 전 진단 단계라 테스트 실행 대상은 없다. 실제 테스트는 Stage 3 구현 후 수행한다.

## 다음 단계

작업지시자 승인 후 `mydocs/plans/task_m100_508_impl.md` 구현 계획서를 작성한다.
