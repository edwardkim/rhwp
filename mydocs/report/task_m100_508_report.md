# Task #508 최종 보고서 — PageLayerTree image brightness/contrast JSON contract 보강

## 이슈

- **Issue**: [#508](https://github.com/edwardkim/rhwp/issues/508)
- **제목**: PageLayerTree image paint op에 brightness/contrast 필드 추가 필요
- **브랜치**: `local/task508`

## 문제

`PageRenderTree`의 `ImageNode`에는 이미지 보정 필드가 이미 포함되어 있었다.

- `effect`
- `brightness`
- `contrast`
- `crop`
- `original_size_hu`

하지만 `PageLayerTree` JSON의 `PaintOp::Image` serialization은 `effect`, `crop`, `fillMode`, `originalSize`, `transform` 등만 내보내고 `brightness`, `contrast`를 누락했다.

그 결과 downstream native renderer가 `PageLayerTree`만 backend replay contract로 사용할 때 image paint op 하나만 보고 core SVG와 같은 이미지 보정 결과를 재현할 수 없었다.

## 정정 내용

`src/paint/json.rs`의 `PaintOp::Image` JSON serialization에 `brightness`, `contrast`를 추가했다.

변경 후 image paint op는 다음 값을 항상 포함한다.

```json
{
  "type": "image",
  "effect": "blackWhite",
  "brightness": -50,
  "contrast": 70,
  "transform": { "rotation": 0.000, "horzFlip": false, "vertFlip": false }
}
```

기본값 `brightness=0`, `contrast=0`도 생략하지 않는다. downstream renderer가 필드 존재 여부 분기 없이 동일 contract를 사용할 수 있게 하기 위함이다.

## 변경 파일

| 파일 | 내용 |
|------|------|
| `src/paint/json.rs` | `PaintOp::Image` JSON에 `brightness`, `contrast` 출력 추가 |
| `src/paint/json.rs` | `serializes_backend_replay_payload_fields` 테스트에 non-zero 값 assertion 추가 |
| `mydocs/orders/20260501.md` | #508 상태 갱신 |
| `mydocs/plans/task_m100_508.md` | 수행 계획서 |
| `mydocs/plans/task_m100_508_impl.md` | 구현 계획서 |
| `mydocs/working/task_m100_508_stage1.md` | contract 진단 보고서 |
| `mydocs/working/task_m100_508_stage2.md` | JSON serialization 구현 보고서 |
| `mydocs/working/task_m100_508_stage3.md` | serialization test 보강 보고서 |

## schemaVersion

`PAGE_LAYER_TREE_SCHEMA_VERSION`은 `1`로 유지했다.

판단 근거:

- 기존 필드 제거 없음
- 기존 필드 의미 변경 없음
- image paint op에 필드를 추가하는 additive 변경
- 기본값 `0`은 기존 시각 의미와 동일
- 기존 PageLayerTree 정책: additive 유지, incompatible change 시 bump

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

### 전체 lib 테스트

```bash
cargo test --lib
```

결과:

- 1102 passed
- 0 failed
- 1 ignored

컴파일 중 기존 경고 4건이 출력되었지만 이번 변경과 무관한 기존 테스트 코드 경고다.

## 결론

`PageLayerTree` image paint op JSON이 `ImageNode`의 이미지 보정 값 중 `brightness`, `contrast`를 잃지 않도록 보강됐다. downstream renderer는 이제 `PageLayerTree` JSON만 보고 core SVG renderer와 같은 brightness/contrast 입력값을 재현할 수 있다.

## 후속

작업지시자 승인 후 이슈 #508 close 여부를 판단한다.
