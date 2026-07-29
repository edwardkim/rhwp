# task_m100_2696 처리결과 보고서 — HWP5 도형 직렬화 (최상위 Picture 무출력)

- **이슈**: [#2696](https://github.com/edwardkim/rhwp/issues/2696)
- **브랜치**: `task/m100-2696-hwp5-shape-serializer-symmetry` (base `devel` @ `2cd4d78b`)
- **범위**: `src/serializer/control.rs`, `src/serializer/control/tests.rs`
- **분류**: 결함 수정 1건 + 오판 정정 1건

## 0. 정정 고지 — OLE 건은 결함이 아니었다 (철회)

이 보고서는 처음에 결함 2건을 주장했다. **그중 OLE 건은 오판이었고 철회한다.**

최초 주장은 "Chart 형제 arm 은 `serialize_drawing_shape_component` 로 `DrawingObjAttr`
전체를 기록하는데 OLE arm 만 base-only 이므로 비대칭 결함"이었다. 그러나 CI 검증
(`cargo test --profile release-test --tests`)에서 기존 통합 테스트
`issue_1283_hwpx_to_hwp_save_keeps_ole_as_storage` 가 깨졌다:

```
assertion `left == right` failed
  left: 239
 right: 196
```

**한컴 저장본을 직접 실측해 판정했다.** `samples/143E433F503322BD33.hwp` 의 레코드 덤프:

```
[ 14]  SHAPE_COMPONENT  sz=252     ← 도형 (테두리/채우기/그림자 꼬리 있음)
[ 27]  SHAPE_COMPONENT  sz=252     ← 도형
[ 49]  SHAPE_COMPONENT  sz=196     ← 그림 (뒤에 SC_PICTURE)
[225]  SHAPE_COMPONENT  sz=196     ← OLE (데이터 선두 65 6c 6f 24 = "$ole")
```

**한컴 자신이 OLE 의 `SHAPE_COMPONENT` 를 196B(base-only)로 쓴다.** 즉 base-only 는
누락이 아니라 한컴의 실제 OLE 포맷이며, `#1283`("Fix HWPX OLE chart save contract")이
한컴 편집기 파일 읽기 오류를 잡으면서 확정한 계약이다.

파서가 `parse_shape_component_full` 로 꼬리를 읽을 수 있는 것은 **관대함이지 직렬화
의무가 아니었다.** 형제 arm 비대칭이라는 관찰은 사실이었지만, 그 비대칭에는 근거가
있었고 내가 그 근거를 확인하지 않고 결함으로 단정했다.

조치:
- OLE arm 2곳(최상위/그룹 자식)을 `devel` 동작으로 되돌리고, **왜** base-only 인지
  주석으로 근거를 남겼다.
- 잘못된 전제로 쓴 테스트 `issue2696_ole_shape_component_keeps_border_fill_shadow` 를
  제거하고, 반대로 **196B 계약을 고정하는** `issue2696_ole_shape_component_stays_base_only`
  를 추가했다. 같은 오판이 반복되지 않게 하기 위해서다.

아래 본문의 OLE 관련 서술은 이 정정에 따라 무효다. Picture 건(2절)만 유효하다.

## 1. 문제

### 1-1. 전제 — raw 패스스루가 가려주지 않는다

`src/serializer/body_text.rs:313-320` 은 컨트롤별로 `ctrl_data_records`(= `HWPTAG_CTRL_DATA`)
**하나만** 원본 바이트로 되살린다. 도형의 `CTRL_HEADER` / `HWPTAG_SHAPE_COMPONENT` / 도형별
기하 레코드는 저장할 때마다 IR 로부터 매번 새로 합성되고, `serialize_shape_control` 에는
원본 레코드 폴백이 없다. `OleShape::raw_tag_data` 는 `HWPTAG_SHAPE_COMPONENT_OLE` 태그
레코드만 보호하며 그 앞의 `HWPTAG_SHAPE_COMPONENT` 는 보호 대상이 아니다.

따라서 아래 두 결함은 HWPX→HWP5 변환 같은 특수 경로가 아니라 **HWP5 파일을 열어 저장하는
가장 흔한 경로에서 무조건 발동**한다.

### 1-2. 결함 (1) — OLE `SHAPE_COMPONENT` 이 base-only 로 기록됨

파서는 모든 gso 도형의 `HWPTAG_SHAPE_COMPONENT` 를 `parse_shape_component_full` 로 완전
파싱해 `border_line` / `fill` / `shadow_*` / `inst_id` / `shadow_alpha` 를 `drawing` 에
채우고(`parser/control/shape.rs:76-88`), OLE 분기에서 `ole.drawing = drawing;`
(`shape.rs:324`)으로 그대로 보존한다. **IR 에는 값이 정확히 살아 있다.**

그런데 직렬화기는 최상위(`control.rs:1463`)와 그룹 자식(`:1760`) 양쪽에서 base-only
`serialize_shape_component(tags::SHAPE_OLE_ID, &drawing.shape_attr, …)` 을 호출했다. 이
함수는 `write_shape_component_base` 한 줄이 전부이며 주석에도 *"ShapeComponentAttr만 —
Picture, Group용"* 이라고 적혀 있다. 값을 손에 쥔 채(`ole_drawing_with_shape_component_contract`
가 이미 `ole.drawing` 전체를 clone 해서 돌려준다) `.shape_attr` 만 꺼내 쓰고 나머지를 버렸다.

재파싱 시에는 `parse_shape_component_full` 의 길이 가드가 전부 실패한다 —
`remaining() >= 13`(테두리, `shape.rs:574`), `>= 4`(채우기, `:582`), `>= 16`(그림자, `:590`),
`>= 4`(inst_id, `:602`), `>= 1`(shadow_alpha, `:610`). 에러 하나 없이 조용히 기본값이 된다.

유실 바이트: 테두리 13 + 채우기 8(None)~16(Solid) + 그림자 16 + inst/예약/alpha 6
= **43~51바이트 / 저장 1회당**.

**비대칭 증거**: 구조적으로 동일한 Chart arm 은 최상위(`:1441`)·그룹 자식(`:1740`) 모두
`serialize_drawing_shape_component` 를 쓴다. Line/Rectangle/Ellipse/Arc/Polygon/Curve 도
마찬가지다. `DrawingObjAttr` 를 가진 도형 중 **OLE 두 곳만** base-only 를 호출했다.

### 1-3. 결함 (2) — 최상위 `ShapeObject::Picture` 무출력 + `char_count` 어긋남

`control.rs:1426-1428` 은 레코드를 하나도 push 하지 않았다:

```rust
ShapeObject::Picture(_pic) => {
    // 그룹 내 그림: 그룹 직렬화 시 자식으로 처리됨 (단독 Picture는 Control::Picture로 직렬화)
}
```

주석의 전제("그룹 자식으로만 등장한다")가 거짓이다. `ungroup_shape_native`
(`document_core/commands/object_ops/shape.rs`)는 그림 자식을 **최상위 컨트롤로** 삽입한다 —
`:2297` 에 `ShapeObject::Picture(p) => &mut p.shape_attr` 가 명시적으로 있고, `:2317-2318`
에서 `para.controls.insert(insert_idx, Control::Shape(Box::new(child)))`, `:2319` 에서
`ctrl_data_records.insert(insert_idx, None)` 를 수행한다. 직렬화 이전에 이를
`Control::Picture` 로 정규화하는 코드는 저장소 어디에도 없다.

**단순 유실이 아니라 문서 손상인 이유**: 바로 이어지는 `:2320-2321` 이
`para.char_count += 8; para.control_mask |= 0x0000_0800;` 로 확장 컨트롤 문자 8 wchar 를
확보한다. 저장 시 `CTRL_HEADER` 가 0개 방출되면 `HWPTAG_PARA_TEXT` 의 컨트롤 문자와
`HWPTAG_CTRL_HEADER` 레코드의 1:1 결합이 한 칸씩 밀린다. 그림이 사라지는 데 그치지 않고
**같은 문단의 이후 컨트롤이 전부 잘못된 문자 위치에 결합**되며, 파싱 단계에서 에러가 나지
않는다.

## 2. 변경

두 건 모두 "이미 올바른 형제 코드를 그대로 따라간다" 성격이다. 새 직렬화 로직을 쓰지 않았다.

**(1) OLE — Chart arm 과 동일하게 (`control.rs` 2곳)**

```rust
// 최상위
data: serialize_drawing_shape_component(tags::SHAPE_OLE_ID, &drawing, true),
// 그룹 자식
data: serialize_drawing_shape_component(tags::SHAPE_OLE_ID, &drawing, false),
```

`drawing` 은 두 지점 모두 `ole_drawing_with_shape_component_contract(...)` 반환값으로 이미
준비돼 있었다. `ctrl_id` 기본값·`is_two_ctrl_id` 처리는 두 함수가 공유하는
`write_shape_component_base` 안에 있으므로 **base 부분 출력은 바이트 단위로 동일**하고
뒤에 43~51바이트가 덧붙을 뿐이다.

**(2) Picture — 검증된 함수에 위임 (`control.rs` 1곳)**

```rust
ShapeObject::Picture(pic) => {
    serialize_picture_control(pic, level, ctrl_data_record, records);
}
```

`serialize_picture_control`(`control.rs:983-1026`)은 `Control::Picture` 가 이미 쓰고 있는
함수로, gso `CTRL_HEADER` → 캡션 → `SHAPE_COMPONENT`(`SHAPE_PICTURE_ID`) → `CTRL_DATA` →
`SHAPE_COMPONENT_PICTURE` 를 방출한다. 4개 인자(`pic`, `level`, `ctrl_data_record`,
`records`)가 `serialize_shape_control` 스코프에서 그대로 사용 가능해 어댑터가 필요 없었다
(호출 전 시그니처 대조 확인함).

세 지점 모두 `[#2696]` 주석을 달았다.

## 3. 검증

### 신규 테스트 (`src/serializer/control/tests.rs`, 3건)

기존 serialize→reparse 하네스(`test_roundtrip_group_picture_child`)를 그대로 따랐다.

| 테스트 | 검증 내용 |
|---|---|
| `issue2696_ole_shape_component_keeps_border_fill_shadow` | 테두리(color/width/attr/outline) + 단색 채우기(bg/pattern/type) + 그림자(type/color/offset×2) + `inst_id` + `shadow_alpha` **11개 필드**가 왕복 보존 |
| `issue2696_top_level_shape_picture_is_serialized` | 최상위 `Control::Shape(ShapeObject::Picture)` 가 컨트롤 1개로 왕복, `bin_data_id == 7` |
| `issue2696_top_level_shape_picture_emits_exactly_one_ctrl_header` | `HWPTAG_CTRL_HEADER` 가 **정확히 1개** + `SHAPE_COMPONENT_PICTURE` 동반 방출 (char_count 짝 고정) |

세 번째 테스트를 별도로 둔 이유: 그림 유실보다 **컨트롤 문자/레코드 짝 어긋남**이 더
위험한 손상 벡터이므로 개수 자체를 회귀 방지선으로 고정했다.

### red→green 실증 (실제 실행·캡처)

**결함 (1) — OLE 수정 2곳을 원래대로 되돌린 상태**

```
running 1 test
test serializer::control::tests::issue2696_ole_shape_component_keeps_border_fill_shadow ... FAILED

failures:

---- serializer::control::tests::issue2696_ole_shape_component_keeps_border_fill_shadow stdout ----

thread 'serializer::control::tests::issue2696_ole_shape_component_keeps_border_fill_shadow' (9960) panicked at src\serializer\control\tests.rs:686:5:
assertion `left == right` failed: 테두리 색이 보존돼야 함
  left: 0
 right: 1193046
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

failures:
    serializer::control::tests::issue2696_ole_shape_component_keeps_border_fill_shadow

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2454 filtered out; finished in 0.00s
```

복원 후:

```
running 1 test
test serializer::control::tests::issue2696_ole_shape_component_keeps_border_fill_shadow ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2454 filtered out; finished in 0.00s
```

**결함 (2) — Picture arm 을 무출력으로 되돌린 상태**

```
running 2 tests
test serializer::control::tests::issue2696_top_level_shape_picture_emits_exactly_one_ctrl_header ... FAILED
test serializer::control::tests::issue2696_top_level_shape_picture_is_serialized ... FAILED

failures:

---- serializer::control::tests::issue2696_top_level_shape_picture_emits_exactly_one_ctrl_header stdout ----

thread 'serializer::control::tests::issue2696_top_level_shape_picture_emits_exactly_one_ctrl_header' (11684) panicked at src\serializer\control\tests.rs:789:5:
assertion `left == right` failed: 최상위 그림 1개는 CTRL_HEADER 를 정확히 1개 방출해야 함 (char_count += 8 과 1:1)
  left: 0
 right: 1
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- serializer::control::tests::issue2696_top_level_shape_picture_is_serialized stdout ----

thread 'serializer::control::tests::issue2696_top_level_shape_picture_is_serialized' (22220) panicked at src\serializer\control\tests.rs:749:5:
assertion `left == right` failed: 최상위 ShapeObject::Picture 가 컨트롤 1개로 왕복돼야 함
  left: 0
 right: 1

failures:
    serializer::control::tests::issue2696_top_level_shape_picture_emits_exactly_one_ctrl_header
    serializer::control::tests::issue2696_top_level_shape_picture_is_serialized

test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 2453 filtered out; finished in 0.01s
```

`left: 0` — 컨트롤 0개, CTRL_HEADER 0개. 이슈에서 예측한 값과 정확히 일치한다.

복원 후 (3건 전체):

```
running 3 tests
test serializer::control::tests::issue2696_top_level_shape_picture_emits_exactly_one_ctrl_header ... ok
test serializer::control::tests::issue2696_top_level_shape_picture_is_serialized ... ok
test serializer::control::tests::issue2696_ole_shape_component_keeps_border_fill_shadow ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 2452 filtered out; finished in 0.01s
```

### 회귀

```
cargo test --lib serializer::control  →  16 passed / 0 failed
cargo test --lib shape                → 161 passed / 0 failed
cargo test --lib serializer::         → 408 passed / 0 failed   (실제 HWP 파일 왕복
                                                                 test_serialize_real_hwp_files 포함)
cargo test --lib ole                  →  28 passed / 0 failed
cargo test --lib picture              → 111 passed / 0 failed
```

OLE 변경이 실제 파일의 바이트 출력을 바꾸므로 `serializer::` 전체와 실 파일 왕복 테스트까지
확인했다.

## 4. 미실행 항목 (투명 고지)

- **PR CI 전체 검증**(`cargo test --verbose`, `cargo clippy -- -D warnings`): 저장소 규약
  (`mydocs/manual/codex/docs_and_git_workflow.md`)상 작업지시자 별도 승인 사항이라 실행하지
  않았다.
- **실제 OLE 포함 HWP 파일을 이용한 시각/E2E 검증**은 하지 않았다. 저장소 샘플에 border/
  fill/shadow 가 비영인 OLE 개체가 있는 파일을 특정하지 못해, 대신 필드 단위 왕복 단언으로
  검증했다. `serializer::` 전체(408건, 실 파일 왕복 포함) 무회귀는 확인했다.
- **그룹 해제→저장→재열기 시나리오의 앱 레벨 재현**은 하지 않았다. 도달 경로는 코드로
  확인했고(`ungroup_shape_native` 의 삽입·char_count 증가 지점), 손상 벡터는
  CTRL_HEADER 개수 단언으로 고정했다.

## 5. 잔여 (별도 후속 과제)

**(a) 도형 캡션이 HWP5 직렬화에서 방출되지 않음**

파서는 모든 gso 도형의 캡션을 읽지만(`parser/control/shape.rs:136-147`, `:205`/`:213`/
`:222`/`:230`/`:240`), `serialize_shape_control` 의 **어느 arm 도 `serialize_caption` 을
호출하지 않는다**. HWP5 의 호출처는 표(`control.rs:492`)와 그림(`:997-999`) 둘뿐이며 HWPX
라이터는 이미 방출하고 있어(`serializer/hwpx/shape.rs:107`, `:254`, `:393`) 포맷 간
비대칭이기도 하다.

→ 본 수정 (2) 로 **그림의 캡션은 부수적으로 해결**된다(`serialize_picture_control` 이 캡션을
방출하므로). 나머지 도형(Line/Rect/Ellipse/Arc/Polygon/Curve/Group/Chart/Ole)의 일반
케이스는 범위를 넘어 손대지 않았다.

**(b) HWP5 출신 그림의 IR 편집이 raw 로 덮임**

`serialize_picture_data`(`control.rs:1068-1070`)는 `pic.raw_picture_extra` 가 비어 있지
않으면 이를 그대로 재생하고 투명도 1바이트만 패치한다
(`picture_raw_extra_with_transparency`, `:1093-1116`). 따라서 파서가 `raw_picture_extra` 를
채운 HWP5 출신 그림에서는 `pic.border_opacity`(`parser/control/shape.rs:929-931`),
`pic.instance_id`(`:932-939`), `pic.img_dim`(`:950-956`) 에 대한 IR 편집이 저장 시 무시된다.
HWPX 출신(빈 `raw_picture_extra`)은 정상 반영되므로 이 역시 출처 의존 비대칭이다.
