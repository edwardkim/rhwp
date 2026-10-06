# task_m100_4424 Stage 1 — 하이퍼링크가 컨트롤 짝짓기를 밀어버린다

- **이슈**: [#4424](https://github.com/edwardkim/rhwp/issues/4424)
- **브랜치**: `fix/issue-4424-hwp5-hyperlink-pairing`
- **분기 기준**: `upstream/devel` `9f5911e86` (0 behind)
- **상태**: 게이트 통과, PR 게시
- **기록일**: 2026-08-10 KST

## 1. 이슈가 지목한 축이 반쯤 틀렸다 — 먼저 정정한다

이슈는 "뒤따르는 **모든 컨트롤**이 한 칸씩 밀려 해석된다"고 적었다. **`para.controls` 벡터는
멀쩡하다.** 그것은 CTRL_HEADER 레코드 순서로 따로 만들어지므로 짝 없는 문자와 무관하다.

처음 쓴 회귀 테스트는 이 잘못된 축을 겨눴고, 그래서 **수정 전 코드에서 통과했다.**
그 테스트는 버렸다.

실제로 어긋나는 것은 **위치 ↔ 컨트롤 인덱스 매핑**이다. `parse_para_text` 는 확장 컨트롤
문자를 만날 때마다 `ctrl_idx` 를 올려 `controls[]` 와 1:1을 맞추는데
(`parser/body_text.rs:339`, `is_extended_only_ctrl_char` 가 11 을 포함), 짝 없는 0x000B
하나가 그 카운터를 한 칸 올린다.

## 2. 재현 (수정 전 RED)

`'A'` + Hyperlink + 필드(ClickHere, `%clk`) 한 문단.

```
DEBUG controls=["Unknown(UnknownControl {"]
      field_ranges=[FieldRange { start_char_idx: 1, end_char_idx: 2, control_idx: 1, ... }]

assertion `left == right` failed:
  짝 없는 0x000B 가 ctrl_idx 를 밀어 필드 인덱스가 어긋났다 (controls[1] 는 존재하지 않는다)
  left: 1   right: 0
```

`controls` 는 길이 1인데 `field_ranges[0].control_idx == 1` — **범위 밖을 가리킨다.**
이 값을 `wasm_api` 의 필드 조회들이 그대로 쓴다.

## 3. 선택 — 슬롯을 발명하지 않는다

이슈가 둘을 놓고 결정 규칙을 줬다. `Control::Hyperlink` 를 만드는 곳을 전수 확인했다.

```
src/parser/hwp3/mod.rs:1896   controls.push(Control::Hyperlink(...))
```

**HWP3 하나뿐이다. HWP5 파서는 이 컨트롤을 만들지 않는다.** 따라서 HWP5 필드(`%hlk`)로
강등하는 1안은 규정에 없는 매핑을 발명하는 일이 된다 — #4396 에서 `item_id=0x4010` 을
발명했다가 되돌린 것과 같은 부류다. **2안(문자를 방출하지 않는다)이 맞다.**

컨트롤 자체는 잃지만 짝짓기는 깨지지 않는다. 뒤 컨트롤 전체가 밀리는 것보다 낫다.

## 4. 고친 방식 — 세 곳이 함께 움직이게 한다

`serialize_control` 의 catch-all arm 은 `ctrl_id == 0` 이면 CTRL_HEADER 를 만들지 않는데,
PARA_TEXT 와 `control_mask` 는 그것과 무관하게 방출해 왔다. 그 조건을 하나로 맞췄다.

`emits_ctrl_header(ctrl)` 를 두고 세 자리에 적용했다 — PARA_TEXT 방출 3곳(갭 채우기,
필드 강제 방출, 잔여 컨트롤 루프)과 `compute_control_mask`. 방출을 건너뛸 때 `ctrl_idx` 는
전진시키되 `prev_end` 는 올리지 않는다.

**`Control::Ruby` 는 같은 arm 이지만 건드리지 않았다.** 규정된 ctrl_id(`tdut`,
`pdf/hwpspec-2024.pdf` §4.2.10.13 표 140)가 있어 방출을 막는 것이 답이 아니다 — #4397 의 일이다.
`Control::Unknown` 은 `ctrl_id != 0` 이면 종전대로 방출한다.

## 5. 게이트

- `cargo fmt --all -- --check` exit 0
- `cargo clippy --all-targets -- -D warnings` exit 0
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` exit 0 —
  `test result: ok` 블록 **502개, FAILED 0건**

#4388 이 이 영역을 건드려 깨뜨렸던 `bookmarks_survive_saving_to_hwp5` 도 통과한다.

## 6. 고치지 않은 것

하이퍼링크 **소실 자체**는 그대로다. HWP5 에 규정된 슬롯이 없다는 것이 이 수정의 전제이므로,
막으려면 사설 확장을 발명해야 한다 — 하지 않는다. 이슈 본문에 그 판단 근거를 남겼다.

## 7. 미처리

GitHub Actions, 작업지시자 승인, merge.
