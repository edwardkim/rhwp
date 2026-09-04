# task_m100_2695 처리결과 보고서 — HWPX charPr outline/shadow 열거값 절단 수정

- **이슈**: [#2695](https://github.com/edwardkim/rhwp/issues/2695)
- **브랜치**: `task/m100-2695-hwpx-charpr-outline-shadow` (base `devel` @ `2cd4d78b`)
- **범위**: `src/parser/hwpx/header.rs`, `src/serializer/hwpx/header.rs`
- **분류**: 결함 수정 (열거값 매핑 누락 — 파싱·직렬화 양측)

## 1. 문제

`<hh:charPr>` 자식 두 개가 **파싱측과 직렬화측 양쪽에서 동시에** 열거값을 잘라먹었다.

| 요소 | 스펙 값 수 | 파서가 구분한 수 | 직렬화가 방출 가능한 수 |
|---|---|---|---|
| `<hh:outline type>` | 8 | 4 | 4 |
| `<hh:shadow type>` | 3 | 2 | 2 |

- **outline**: 파서 `_ => 0`(`parser/hwpx/header.rs:751-764`), 직렬화 `_ => "NONE"`
  (`outline_type_str`, `serializer/hwpx/header.rs:759-767`). 값 4~7
  (`DASH_DOT`/`DASH_DOT_DOT`/`LONG_DASH`/`CIRCLE`)이 `NONE` 으로 떨어져 **외곽선이 다른 선으로
  열화되는 게 아니라 아예 소멸**했다.
- **shadow**: 파서가 `"DROP" | "CONTINUOUS" => 1` 로 두 종류를 한 슬롯에 합치고
  (`parser/hwpx/header.rs:765-782`), 직렬화는 `if shadow_type == 0 {"NONE"} else {"CONTINUOUS"}`
  (`serializer/hwpx/header.rs:649-665`)라 **`"DROP"` 문자열이 코드 어디에도 없었다** — 방출 불가능.

두 필드 모두 IR 은 이미 충분한 값 범위를 담고 있었다. `CharShape.outline_type`(`model/style.rs:124`)은
HWP5 `attr` bits 8-10 **3비트**를 그대로 받고(`parser/doc_info.rs:592`의 `(attr >> 8) & 0x07`,
재패킹 `serializer/doc_info.rs:470-472`), `CharShape.shadow_type`(`model/style.rs:126`)은
bits 11-12 **2비트**를 받는다(`parser/doc_info.rs:593`의 `(attr >> 11) & 0x03`, 재패킹 `:473-475`).
즉 손실 지점은 오직 HWPX 매핑 테이블이었다.

## 2. 분석

### 2.1 누락임을 보이는 자기모순

**같은 파서 파일**이 40여 줄 위에서 밑줄(`header.rs:695-710`)·취소선(`:727-742`)에 대해 선 종류
13종(0~12) 전체를 이미 매핑한다. 직렬화측 `line_shape_str`(`serializer/hwpx/header.rs:742-757`)도
0~12 를 전부 되돌린다. 즉 `DASH_DOT`/`DASH_DOT_DOT`/`LONG_DASH`/`CIRCLE` 네 문자열은 저장소가
**이미 알고 이미 처리하던 값**이고, outline 매핑에서만 빠져 있었다. 한 파일 안에서 같은 선 종류
계열을 한쪽은 13개, 다른 쪽은 4개로 다루는 것은 설계가 아니라 불일치다.

같은 파일의 기존 테스트 `tab_leader_str_emits_double_and_triple_line_types` 가 이미 동일 유형
(`fill_type 9/10/11` 이 `"NONE"` 으로 유실)을 결함으로 인정하고 고친 전례다.

### 2.2 손실 경로 4종

| # | 입력 | 경로 | 산출 | 피해 |
|---|---|---|---|---|
| L1 | HWPX `outline type="DASH_DOT"` | 파싱 → IR `0` → 직렬화 | `type="NONE"` | 외곽선 소멸 (파싱측 유실) |
| L2 | HWP5 attr bits 8-10 = `0b101` | `doc_info` → IR `5` → HWPX 직렬화 | `type="NONE"` | 외곽선 소멸 (직렬화측 유실) |
| L3 | HWPX `shadow type="DROP" offsetX/Y` | 파싱 → IR `1` → 직렬화 | `type="CONTINUOUS"` + 오프셋 잔존 | 종류 변조, 오프셋이 해석 주체를 잃음 |
| L4 | HWP5 attr bits 11-12 = `0b10` | `doc_info` → IR `2` → HWPX `CONTINUOUS` → **재파싱** → IR `1` → `doc_info` | bits = `0b01` | **바이너리 손상** (HWPX 경유만으로 2→1) |

L1 은 직렬화만, L2 는 파서만 고쳐서는 해결되지 않는다. 양측을 함께 고쳐야 하는 근거다.

### 2.3 마스킹 없음 확인

원문 보존(raw splice) 경로가 이 결함을 가리는지 확인했고 **해당 없음**으로 판정했다.

- `hwpx_head_tail` splice(파서 `:218-224` → 직렬화 `:92-103`)는 `</hh:refList>` **이후** 꼬리
  구간만 보존한다. `charPr` 은 `refList` **안**이라 범위 밖이다.
- `write_char_pr`(`serializer/hwpx/header.rs:574`)에 원문 재사용 단축 경로가 없다.
- `raw_para_heads` 보존(`:885-891`)은 numbering 의 `paraHead` 전용이다.

### 2.4 하위 호환 확인 (범위 확대의 안전성)

`outline_type`/`shadow_type` 의 **모든 소비 지점이 `!= 0` / `> 0` 불리언 판정 또는 단순 복사**임을
전수 확인했다. `outline_type <= 3` 이나 `shadow_type <= 1` 을 전제하는 코드는 없다.

`canvaskit_policy.rs:1834,1837` · `html.rs:352` · `web_canvas.rs:2189,2893,2901` · `svg.rs:2823` ·
`skia/text_replay.rs:531,539` · `paint/text_v2.rs:758-759` — 전부 비영 판정.
`paint/paint_op.rs:1551-1552` 는 `== 0` 빈 스타일 판정. `style_resolver.rs:379-380`,
`layout/text_measurement.rs:1446-1447` 은 단순 복사. `parser/hwp3/mod.rs:279-280` 은 HWP3 불리언을
`0`/`1` 로만 쓰므로 영향 없다.

## 3. 변경

매핑 테이블 4곳만 수정했다(파일 2개).

1. **파서 outline**(`parser/hwpx/header.rs`) — `DASH_DOT=4`, `DASH_DOT_DOT=5`, `LONG_DASH=6`,
   `CIRCLE=7` 4개 arm 추가. 문자열은 같은 파일 `underline_shape` 테이블이 쓰는 이름 그대로다
   (outline 은 `NONE` 이 0번을 차지하므로 선 종류 인덱스보다 1 크다).
2. **직렬화 `outline_type_str`** — (1)의 정확한 역함수로 `4..=7` 확장.
3. **파서 shadow** — `"DROP" | "CONTINUOUS" => 1` 을 `"DROP" => 1, "CONTINUOUS" => 2` 로 분리.
4. **직렬화 shadow** — `write_char_pr` 인라인 `if`/`else` 를 신설 `shadow_type_str(t)` 3분기
   `match` 로 교체. 인접한 `outline_type_str`/`line_shape_str` 과 동일한 헬퍼 형태를 따랐다.

주석은 저장소 관례대로 한국어 `[#2695]` 접두어로 달고, 각 값이 HWP5 어느 비트에 대응하는지를 남겼다.

## 4. 검증

### 신규 테스트 (`serializer/hwpx/header.rs` 기존 `#[cfg(test)]` 모듈)

파싱과 직렬화를 한 테스트에서 함께 고정하는 왕복 형태다(헬퍼 `parse_single_char_pr` /
`write_single_char_pr` 신설).

- `char_pr_outline_type_roundtrips_all_eight_values` — 8값 전부에 대해 `type=X` → IR 기대값 →
  재직렬화 `type=X` 를 단언.
- `char_pr_shadow_drop_survives_roundtrip_with_offsets` — `DROP` + `offsetX=10`/`offsetY=-7` →
  IR `1` → 재직렬화 `type="DROP"` 및 오프셋 보존.
- `char_pr_shadow_continuous_is_ir_two_not_one` — `CONTINUOUS` → IR `2` → 재직렬화
  `type="CONTINUOUS"` (L4 바이너리 손상 경로 고정).

### red→green 실증 (수정 4곳을 각각 되돌려 4회 수행)

각 수정을 **하나씩만** 되돌려 테스트가 실제로 실패하는지 확인하고 복원했다. 아래는 실제 캡처
출력이다.

**RED 1 — 파서 outline arm 4~7 제거**

```
test serializer::hwpx::header::tests::char_pr_outline_type_roundtrips_all_eight_values ... FAILED

thread 'serializer::hwpx::header::tests::char_pr_outline_type_roundtrips_all_eight_values' (3412) panicked at src\serializer\hwpx\header.rs:1984:13:
assertion `left == right` failed: outline type=DASH_DOT 은 IR 4 로 파싱돼야 함
  left: 0
 right: 4

test result: FAILED. 9 passed; 1 failed; 0 ignored; 0 measured; 2445 filtered out
```

**RED 2 — 직렬화 `outline_type_str` arm 4~7 제거** (IR 은 4를 들고 있는데 `NONE` 이 방출되는
소멸 현상이 출력에 그대로 찍힌다)

```
test serializer::hwpx::header::tests::char_pr_outline_type_roundtrips_all_eight_values ... FAILED

thread 'serializer::hwpx::header::tests::char_pr_outline_type_roundtrips_all_eight_values' (33564) panicked at src\serializer\hwpx\header.rs:1986:13:
outline_type=4 은 type="DASH_DOT" 으로 방출돼야 함: <hh:charPr id="0" height="1000" ... /><hh:outline type="NONE"/><hh:shadow type="NONE" color="#000000" offsetX="0" offsetY="0"/></hh:charPr>

test result: FAILED. 9 passed; 1 failed; 0 ignored; 0 measured; 2445 filtered out
```

**RED 3 — 파서 shadow 를 `"DROP" | "CONTINUOUS" => 1` 로 되돌림** (L4 의 값 붕괴 2→1 이 그대로 재현)

```
test serializer::hwpx::header::tests::char_pr_shadow_continuous_is_ir_two_not_one ... FAILED

thread 'serializer::hwpx::header::tests::char_pr_shadow_continuous_is_ir_two_not_one' (9720) panicked at src\serializer\hwpx\header.rs:2026:9:
assertion `left == right` failed: CONTINUOUS 는 IR 2(연속)
  left: 1
 right: 2

test result: FAILED. 9 passed; 1 failed; 0 ignored; 0 measured; 2445 filtered out
```

**RED 4 — 직렬화 shadow 를 `if`/`else` 로 되돌림** (L3 의 "종류는 변조되고 오프셋만 남는" 증상이
출력에 그대로 찍힌다: `type="CONTINUOUS" ... offsetX="10" offsetY="-7"`)

```
test serializer::hwpx::header::tests::char_pr_shadow_drop_survives_roundtrip_with_offsets ... FAILED

thread 'serializer::hwpx::header::tests::char_pr_shadow_drop_survives_roundtrip_with_offsets' (33392) panicked at src\serializer\hwpx\header.rs:2011:9:
shadow_type=1 은 type="DROP" 으로 방출돼야 함: <hh:charPr id="0" height="1000" ... /><hh:outline type="NONE"/><hh:shadow type="CONTINUOUS" color="#C0C0C0" offsetX="10" offsetY="-7"/></hh:charPr>

test result: FAILED. 9 passed; 1 failed; 0 ignored; 0 measured; 2445 filtered out
```

**GREEN — 4곳 모두 복원**

```
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 2445 filtered out
```

### 회귀

```
cargo test --lib hwpx    →  474 passed / 0 failed / 0 ignored
cargo test --lib header  →  130 passed / 0 failed / 1 ignored
cargo test --lib (전체)  → 2448 passed / 0 failed / 7 ignored
```

값 범위를 넓히는 변경이라 렌더러까지 영향이 갈 수 있어 전체 lib 테스트도 돌렸고, 실패는 없다.

### 미실행 항목 (투명 고지)

- **PR CI 전체 검증**(`cargo test --verbose`, `cargo clippy -- -D warnings`): 저장소 규약
  (`mydocs/manual/codex/docs_and_git_workflow.md`)상 작업지시자 별도 승인 사항이라 실행하지 않았다.
- **실제 한/글 시각 검증**: 외곽선 4종·비연속 그림자를 지정한 실문서를 한/글에서 열어 확인하는
  절차는 수행하지 않았다. 검증은 파싱·직렬화 왕복의 값 보존 수준에서 이뤄졌다. 다만 §2.4 에서
  모든 렌더러 소비 지점이 비영 판정만 한다는 점을 확인했으므로, 이 변경이 렌더링 동작을 바꾸는
  경로는 없다(렌더러는 종전에도 지금도 "외곽선 있음/없음"만 구분한다). 즉 이번 수정의 효과는
  **저장 파일의 값 보존**에 한정되며, 종류별 렌더링 차등화는 별개 과제다.
- **HWP5 왕복 통합 테스트**(L4 경로를 `.hwp` 바이트 수준에서 끝까지 검증)는 추가하지 않았다.
  `serializer/doc_info.rs:473-475` 가 `shadow_type` 을 `& 0x03` 으로 그대로 재패킹함을 코드로
  확인했고, 붕괴 지점이 HWPX 매핑임을 `char_pr_shadow_continuous_is_ir_two_not_one` 으로 직접
  고정했다. 포맷 간 완전 왕복 하네스는 이 범위를 넘는다고 판단했다.

## 5. 잔여 (범위 밖)

조사 중 같은 두 파일에서 확인했으나 이번 범위에 넣지 않은 항목. 이슈 #2695 §7 에 상세를 남겼다.

- **`breakSetting@lineWrap`** 이 `"BREAK"` 상수 하드코딩. 파서는 이 속성을 읽지도 않는다. IR
  귀속처는 `ParaShape.attr2` bits 0-1 (`ParaShapeMods::single_line`, `model/style.rs:974-976`).
  파서·직렬화·IR 세 곳을 함께 이어야 하는 별개 작업.
- **`<hh:autoSpacing>`** 이 `"0"/"0"` 하드코딩. **이것은 단순 매핑 문제가 아니라 `attr2` bit 5 의
  규약 충돌 때문에 의도적으로 보류했다.** `model/style.rs:977-982` 계열은 bit 4 =
  `auto_space_kr_en`, bit 5 = `auto_space_kr_num` 으로 쓰고, HWPX 파서(`header.rs:1023-1028`)·
  직렬화(`serializer/hwpx/header.rs:1048`)·렌더러(`style_resolver.rs:883`) 계열은 bit 5 =
  `widowOrphan` 으로 쓴다. 한쪽을 설정하면 다른 쪽이 오염되는 실버그이며, **어느 비트 규약을
  정본으로 삼을지에 대한 설계 결정이 선행**돼야 해서 이번 수정에 포함하지 않았다.
- **`charPr@shadeColor` 의 `0 → "none"` 특례**(`serializer/hwpx/header.rs:581-585`) — 검정
  음영(`#000000`)과 음영 없음이 구분되지 않아 검정 음영이 소멸한다. 센티넬 값 설계 사안이라 분리.
- **`paraPr@suppressLineNumbers` / `@checked`** — 대응 IR 필드가 아예 없어 모델 확장이 선행돼야 한다.
