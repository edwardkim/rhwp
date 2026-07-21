# task_m100_2698 처리결과 보고서 — 커넥터 뮤테이터의 구역 패스스루 무효화 누락

- **이슈**: [#2698](https://github.com/edwardkim/rhwp/issues/2698)
- **브랜치**: `task/m100-2698-connector-raw-stream-invalidation` (base `devel`)
- **범위**: `src/document_core/commands/object_ops/connector.rs`
- **분류**: 계약 위반 수정 (패스스루 무효화 누락)

## 1. 문제

`connector.rs`(347줄, 공개 뮤테이터 3개)에 패스스루 무효화가 **0건**이었다.
`raw_stream` / `raw_data` 문자열이 파일 전체에 한 번도 등장하지 않았다.

`serialize_section`(`src/serializer/body_text.rs:26-30`)은 `Section::raw_stream` 이
`Some` 이면 구역 전체를 원본 바이트 그대로 반환한다. 따라서 IR 을 고친 뮤테이터가
그 층을 무효화하지 않으면 편집이 저장 결과에서 사라진다.

### object_ops 무효화 밀도 (`grep -c 'raw_stream'`)

| 모듈 | 등장 | 판정 |
|---|---:|---|
| `picture.rs` | 11 | 정상 |
| `shape.rs` | 8 | 정상 |
| `table.rs` | 7 | 정상 |
| `equation.rs` | 3 | 정상 |
| `note.rs` | 3 | 정상 |
| `common.rs` | 2 | 정상 |
| **`connector.rs`** | **0** | **누락** |
| `mod.rs` | 0 | 해당 없음 (`pub fn` 0개, 모듈 선언 전용) |

뮤테이션을 수행하는 모듈 중 0건인 것은 `connector.rs` 가 유일하다. `#1904`
도메인 분할 때 이 모듈만 누락된 것으로 보인다.

## 2. 분석 — 현재 관측 가능한 유실은 없다 (정직 고지)

주장을 과장하지 않기 위해 세 함수의 **모든** 호출 지점을 전수 조사했다.

| # | 호출 지점 | 선행 무효화 | 현재 |
|---|---|---|---|
| 1 | `wasm_api.rs:3567` | `create_shape_control_native` → `shape.rs:1492` | 안전 |
| 2 | `wasm_api.rs:3576` | 동일 블록 | 안전 |
| 3 | `common.rs:328` | 바로 위 `common.rs:325` | 안전 |
| 4 | `wasm_api.rs:3748` (wasm 공개 진입점) | **없음** | 호출자 위임 |
| 5 | `input-handler-picture.ts:1061`, `input-handler-table.ts:1324` | 직전 `setObjectProperties` | 안전 |

**결론: 현재 studio UI 경로에서는 선행 뮤테이션이 우연히 무효화를 대신해 주고 있어
사용자에게 관측되는 유실이 없다.** 이 사실을 숨기지 않는다.

### 그럼에도 고친 이유

1. **저장소가 선언한 계약과 구현이 어긋나 있다.** `mutation-method-registry.ts` 는
   자신을 "문서 변경 WasmBridge 메서드의 단일 권위 목록"으로, `MUTATING_METHODS` 를
   "문서 IR(**직렬화 결과**)을 바꾸는 메서드 전수"로 정의하고 `:44` 에
   `'updateConnectorsInSection'` 을 등재해 두었다. 그러나 Rust 구현은 `raw_stream` 이
   `Some` 인 한 직렬화 결과를 바꾸지 않는다.
2. **안전성이 지역적이지 않고 호출자 순서에 의존한다.** 새 호출자 추가나 순서 변경으로
   깨지며, 깨져도 컴파일 에러·테스트 실패·런타임 경고가 전혀 없다.
3. **공개 wasm API 다.** 제3자 임베더는 "호출 전 다른 뮤테이션 필요"라는 암묵 조건을 알 수 없다.
4. **비용이 사실상 0이다** (3줄, 이미 `None` 이면 무연산).

## 3. 변경

무효화를 뮤테이션 지점에 두되, **실제로 IR 을 바꾼 경우에만** 수행하도록 조건부로 걸었다.
인덱스가 빗나가 아무것도 바꾸지 않은 경로에서 불필요하게 완전 라운드트립을 깨뜨리지 않기
위해서다.

- `update_connector_subject_ids` — `mutated` 플래그, SubjectID 4개 대입 시에만 무효화
- `recalculate_connector_routing` — **갈래가 3개**이므로 각각 처리
  - 꺾인 연결선: `conn.control_points = pts` 후 `routed = true`
  - 곡선 연결선: `conn.control_points = vec![...]` 후 `routed = true`
  - 직선 연결선: `control_points.clear()` 후 **조기 반환**하므로 그 자리에서 직접 무효화
- `update_connectors_in_section` — bbox/로컬좌표 갱신 루프에 `geometry_updated` 플래그.
  3단계 제어점 재계산은 `recalculate_connector_routing` 이 자체 무효화하므로, 여기서는
  좌표만 바뀌고 라우팅 대상이 없는 경우를 덮는다.

### 구현 중 발견한 자체 결함

최초 구현은 `recalculate_connector_routing` 의 **꺾인 갈래에만** 무효화를 걸었다.
곡선 갈래와 직선(clear 후 조기 반환) 갈래는 `control_points` 를 실제로 바꾸면서도
무효화를 타지 않는 구멍이 남아 있었다. 갈래별 테스트를 추가하는 과정에서 발견해
두 갈래를 모두 막았다.

## 4. 검증

### 신규 테스트 (`connector_passthrough_invalidation_tests`, 4건)

1. `update_connector_subject_ids_invalidates_section_passthrough`
2. `recalculate_connector_routing_invalidates_section_passthrough`
3. `every_routing_branch_invalidates_section_passthrough` — StrokeBoth / ArcBoth /
   StraightNoArrow 세 갈래 전부
4. `no_op_call_keeps_passthrough_intact` — 조건부 설계 고정 (반대 방향)

픽스처는 빈 문서에 커넥터를 심고 `raw_stream = Some(...)` 로 "방금 로드한 원본"을
모사한다. 빈 문서 첫 문단은 이미 SectionDef/ColumnDef 컨트롤을 갖고 있어
`control_idx` 를 하드코딩하지 않고 삽입 시점의 실제 인덱스를 돌려준다.

### red→green 실증

**(1) 무효화 2개소를 무력화** (`if mutated && false`, `if routed && false`):
```
recalculate_connector_routing_invalidates_section_passthrough ... FAILED
update_connector_subject_ids_invalidates_section_passthrough ... FAILED
panicked at connector.rs:421:9:
[#2698] SubjectID 를 바꿨으면 구역 패스스루가 무효화되어야 한다 — 그렇지 않으면
저장 시 원본 바이트가 그대로 나가 편집이 사라진다
panicked at connector.rs:435:9:
[#2698] 제어점을 재구성했으면 구역 패스스루가 무효화되어야 한다
test result: FAILED. 1 passed; 2 failed
```
`no_op_call_keeps_passthrough_intact` 는 통과 — 테스트가 무차별이 아니라 계약 위반
지점만 잡는다는 증거다.

**(2) 곡선 갈래의 `routed = true` 만 제거**:
```
every_routing_branch_invalidates_section_passthrough ... FAILED
panicked at connector.rs:461:13:
[#2698] ArcBoth 갈래에서 구역 패스스루가 무효화되지 않았다
test result: FAILED. 3 passed; 1 failed
```
세 갈래 중 **ArcBoth 만 정확히 지목**했다.

**(3) 전부 복원**:
```
test result: ok. 4 passed; 0 failed
```

### 회귀

```
cargo test --lib document_core::  →  259 passed / 0 failed / 2 ignored
cargo test --lib serializer::     →  405 passed / 0 failed
```

### 미실행 항목 (투명 고지)

- **PR CI 전체 검증**(`cargo test --verbose`, `cargo clippy -- -D warnings`)은 저장소
  규약(`mydocs/manual/codex/docs_and_git_workflow.md`)상 작업지시자 별도 승인
  사항이라 실행하지 않았다.
- 실제 커넥터가 연결된 문서를 저장→재열기하는 **바이트 수준 왕복 검증**은 하지 않았다.
  이 수정의 계약은 "뮤테이션 후 `raw_stream` 이 `None` 이어야 한다"는 것이고, 그 이후의
  재직렬화 정확성은 기존 직렬화 테스트(405건)가 담당한다고 판단했다.
- `update_connectors_in_section` 의 무효화는 단위 테스트로 고정하지 못했다.
  `conn_points` 맵이 채워지려면 inst_id 를 가진 비-Line 도형과 그것을 참조하는 커넥터가
  함께 필요해 픽스처 비용이 커서다. 대신 그 함수가 위임하는
  `recalculate_connector_routing` 쪽을 갈래별로 고정했다. **확인하지 않은 것을 확인한
  것처럼 적지 않기 위해 밝혀 둔다.**

## 5. 잔여

- `mutation-method-registry.ts` 의 저작 시점 가드를 "Rust 뮤테이터가 무효화하는가"까지
  검사하도록 확장하는 정적 분석 과제.
- 다른 모듈의 무효화 누락 여부는 밀도 조사로 0건이 아님만 확인했을 뿐, 뮤테이터 단위
  전수 검증은 하지 않았다.
