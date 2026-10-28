# task_m100_5838 처리결과 — HWPX 책갈피 이름을 HWP5 `CTRL_DATA` 로 되살린다

- 이슈: [#5838](https://github.com/edwardkim/rhwp/issues/5838)
- 기준: `0697bc559` (devel)
- 관련: [#5249](https://github.com/edwardkim/rhwp/issues/5249)(같은 조사 축) · [#4396](https://github.com/edwardkim/rhwp/issues/4396)(경계)

## 1. 결함

HWPX 는 책갈피를 `<hp:fieldBegin type="BOOKMARK" name="_top">` 으로 싣는다. rhwp 는 `%bmk`
컨트롤까지 정확히 방출하면서 **이름을 담는 `HWPTAG_CTRL_DATA` 를 만들지 않았다.**
이름 없는 책갈피는 상호참조·하이퍼링크가 가리킬 대상이 되지 못한다.

원인은 방출 조건 한 줄이다 — 이름 `CTRL_DATA` 를 `FieldType::ClickHere`(누름틀)에만 붙였다.
HWPX 파서는 이미 `Field::ctrl_data_name` 을 채워 두고 있었다.

## 2. 근거 — 정답지 바이트

`samples/hwpx/aift.hwpx` ↔ 같은 문서의 한컴 저작본 `samples/aift.hwp`:

| | `%bmk` | 그 아래 `CTRL_DATA` |
|---|---:|---|
| 정답지 | 1 | `1b02 01000000 0040 0100 0400 5f00 7400 6f00 7000` (ParameterSet 0x021b · item 0x4000 String `"_top"`) |
| 종전 변환 | 1 | **없음** |
| 수정 후 변환 | 1 | **정답지와 바이트 동일** |

컨트롤 구성은 양쪽 다 134개·13종으로 같다. 사라지던 것은 이름뿐이었다.

## 3. item ID 를 발명하지 않았다 (#4396 과의 경계)

- 누름틀 경로가 **이미 쓰는 바이트 모양**과 같다(`samples/form-01.hwp` 역공학, 코드 주석에 기록).
- 스펙 §4.2.10.11 은 책갈피에 대해 "책갈피 이름 밖에 없다"고 못박는다.
- #4396 이 되돌린 것은 MEMO·수식의 `ResultFormat` 처럼 **스펙에 item ID 가 없는** named param
  들이다. 이 건은 스펙과 정답지가 둘 다 있는 유일한 item ID 를 쓴다 — 반대 경우다.

## 4. 변경

`src/serializer/control.rs` — 이름 `CTRL_DATA` 방출 조건에 `FieldType::Bookmark` 추가.
`ctrl_data_record.is_none()` 가드는 그대로라 **HWP5 원본에서 파싱한 raw 는 손대지 않는다.**

`tests/cases/issue_5838_bookmark_ctrl_data.rs` (신규) — 변환 산출에서 `%bmk` 아래 `CTRL_DATA`
를 뜯어 정답지와 바이트 비교하고, ParameterSet 머리(0x021b)·item id(0x4000)·이름 문자열까지 판정.

## 5. 검증

### red → green

방출 조건에서 `Bookmark` 만 빼고 돌렸다.

```
test issue_5838_bookmark_ctrl_data::bookmark_name_survives_hwpx_to_hwp_and_matches_the_oracle ... FAILED
    변환본에서 책갈피 이름 레코드가 사라졌다 (#5838): 0건
```

되돌리면 통과.

### 실행한 것

| 명령 | 결과 |
|---|---|
| `cargo test --lib -p rhwp` | **3,893 passed** / 13 ignored |
| `cargo test --test regression_suite_020` (신규 계약 포함) | 115 passed / 2 ignored |
| `cargo test --test regression_suite_001` (`hwpx_to_hwp_adapter`) | 74 passed / 15 ignored |
| `cargo test --test regression_suite_028` (`convert_verify_corpus_ratchet`) | 126 passed / 2 ignored |
| `rustfmt --edition 2021` (변경 파일; `control.rs` 는 스텁 디렉터리 사본으로 검사) | 차이 없음 |
| `cargo clippy --all-targets -- -D warnings` | exit 0 |
| `node scripts/rust-unit-test-tiers.mjs --check --base-ref origin/devel` | 4,225 tests (기준선 유지) |

## 6. 조사에서 나온 "결함 아님" 기록

같은 대조에서 걸렸지만 고칠 수 없거나 고칠 필요가 없는 것들이다.

- `aift` 의 나머지 `CTRL_DATA` 10건 — 표 이름(`_표2.2`) 7건과 표 layout ParameterSet(0x0242)
  3건. **원본 HWPX 에 그 값이 없다.**
- `tbl`·`gso`·`eqed` 의 46 vs 48 바이트 — **버전 차이**. 48 은 5.1.1.0 저작본에서만 나오고
  rhwp 는 5.1.0.0 으로 저장하므로 46 이 그 버전의 정답이다(#5249 의 규칙과 같은 축).
- `PARA_RANGE_TAG` 소실(`온새미로` 104 → 0) — 원본 HWPX 에 범위 태그 요소 자체가 없다.
