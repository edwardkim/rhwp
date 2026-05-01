# Task #508 구현 계획서 — PageLayerTree image brightness/contrast JSON contract 보강

## 1. 구현 영역

### 핵심 결함

`PageRenderTree`의 `ImageNode`에는 `brightness`, `contrast`가 이미 존재하고 생성 지점에서도 `pic.image_attr` 값이 전달된다. 그러나 `PageLayerTree` JSON의 `PaintOp::Image` serialization에서 두 필드를 출력하지 않아 downstream native renderer가 image paint op만으로 core SVG와 같은 이미지 보정 입력값을 재현할 수 없다.

### 변경 대상

| 파일 | 변경 |
|------|------|
| `src/paint/json.rs` | `PaintOp::Image` JSON에 `brightness`, `contrast` 항상 출력 |
| `src/paint/json.rs` tests | 기존 `serializes_backend_replay_payload_fields`에 assertion 추가 |
| `mydocs/working/task_m100_508_stage2.md` | 구현 단계 완료 보고서 |
| `mydocs/working/task_m100_508_stage3.md` | 테스트 단계 완료 보고서 |
| `mydocs/report/task_m100_508_report.md` | 최종 보고서 |
| `mydocs/orders/20260501.md` | 최종 상태 갱신 |

## 2. 구현 정책

### JSON 필드 출력

`PaintOp::Image` 출력에 다음 필드를 추가한다.

```json
"brightness": -50,
"contrast": 70
```

필드 출력 위치는 `effect` 직후, `transform` 직전으로 둔다.

```json
{
  "type": "image",
  "bbox": { ... },
  "effect": "grayScale",
  "brightness": -50,
  "contrast": 70,
  "transform": { ... }
}
```

### 기본값 처리

`brightness=0`, `contrast=0`도 생략하지 않고 항상 출력한다.

이유:

- downstream renderer가 필드 존재 여부 분기 없이 image paint op를 replay할 수 있다.
- 기존 `ImageNode::new()` 기본값과 JSON contract가 직접 대응한다.
- 기본값 `0`은 기존 시각 의미와 동일하다.

### schemaVersion

`PAGE_LAYER_TREE_SCHEMA_VERSION`은 유지한다.

판단 근거:

- 기존 필드 제거 없음
- 기존 필드 의미 변경 없음
- image paint op에 필드만 추가하는 additive 변경
- 기존 PageLayerTree 정책: additive 유지, incompatible change 시 bump

## 3. 구현 단계 (3 stages)

### Stage 1 — JSON serialization 구현

**대상 파일**: `src/paint/json.rs`

**변경 내용**:

`PaintOp::Image` 분기에서 `effect` 출력 뒤에 `brightness`, `contrast`를 추가한다.

```rust
let _ = write!(
    buf,
    ",\"effect\":{},\"brightness\":{},\"contrast\":{}",
    json_escape(image_effect_str(image.effect)),
    image.brightness,
    image.contrast
);
```

또는 기존 `effect` 출력은 유지하고 별도 `write!`로 두 필드를 추가한다. rustfmt 후 가독성이 좋은 형태를 선택한다.

**산출물**:

- `mydocs/working/task_m100_508_stage2.md`

**완료 기준**:

- JSON 문자열에 `brightness`, `contrast`가 항상 포함됨
- `schemaVersion` 값은 `1` 유지

### Stage 2 — serialization test 보강

**대상 파일**: `src/paint/json.rs`

**변경 내용**:

기존 테스트 `serializes_backend_replay_payload_fields`에서 image 값을 non-zero로 설정한다.

```rust
let mut image = ImageNode::new(7, Some(vec![1, 2, 3]));
image.effect = ImageEffect::BlackWhite;
image.brightness = -50;
image.contrast = 70;
```

그리고 assertion을 추가한다.

```rust
assert!(json.contains("\"brightness\":-50"));
assert!(json.contains("\"contrast\":70"));
```

**산출물**:

- `mydocs/working/task_m100_508_stage3.md`

**완료 기준**:

- `cargo test --lib paint::json::tests::serializes_backend_replay_payload_fields` 통과
- 기존 image field assertion(`effect`) 유지

### Stage 3 — 검증 + 최종 보고

**검증 명령**:

```bash
cargo test --lib paint::json
```

필요 시 범위를 넓혀 다음을 실행한다.

```bash
cargo test --lib
```

대표 샘플 확인이 필요하다고 판단되면 `samples/복학원서.hwp`의 PageLayerTree JSON에서 `brightness=-50`, `contrast=70` 포함 여부를 확인한다.

**산출물**:

- `mydocs/report/task_m100_508_report.md`
- `mydocs/orders/20260501.md` 상태 갱신

**완료 기준**:

- 관련 paint JSON 테스트 통과
- 최종 보고서 작성
- 오늘할일 #508 상태 갱신

## 4. 테스트 계획

| 테스트 | 목적 | 기준 |
|--------|------|------|
| `cargo test --lib paint::json::tests::serializes_backend_replay_payload_fields` | image paint op JSON 필드 직접 검증 | `brightness`, `contrast` assertion 통과 |
| `cargo test --lib paint::json` | PageLayerTree JSON 직렬화 회귀 점검 | 관련 테스트 전체 통과 |
| `cargo test --lib` | 필요 시 전체 lib 회귀 점검 | 전체 통과 |

## 5. 위험 요소 / 대응

| 위험 | 대응 |
|------|------|
| downstream strict schema validator가 unknown field를 거부 | additive 정책 근거를 최종 보고서에 명시. 작업지시자 지시가 있으면 schemaVersion bump로 전환 |
| 기본값 생략으로 downstream 구현이 복잡해짐 | 기본값 `0`도 항상 출력 |
| JSON 필드 순서 의존 테스트 취약성 | 기존 테스트 패턴대로 substring assertion 사용 |
| source path 외 영향 확대 | `src/paint/json.rs` 단일 파일 변경으로 제한 |

## 6. 작업지시자 승인 요청

- [ ] `PAGE_LAYER_TREE_SCHEMA_VERSION` 유지 승인
- [ ] `brightness`, `contrast`를 image paint op에 항상 출력하는 정책 승인
- [ ] `src/paint/json.rs` 단일 파일 소스 변경 승인
- [ ] 위 3단계 구현 및 검증 계획 승인

승인 후 Stage 1 구현을 시작한다.
