---
kind: working
status: active
issue: 6887
---

# 문단 기준 어울림 표가 outMargin.left 를 안 실어 그림이 왼쪽으로 치우친다 (#6887)

작업 브랜치: `fix/6887-para-float-table-outmargin-left`
대상: `src/renderer/layout.rs` · `src/renderer/float_placement.rs`

## 한 줄

`horzRelTo="PARA"` + `horzAlign="LEFT"` 인 **어울림(Square) 표**가 저장 `horzOffset` 을
표 자신의 왼끝으로 읽어, 바깥 왼쪽 여백(`outMargin.left`)만큼 왼쪽으로 치우친다.
그 결과 표 안 그림이 호스트 본문 글자 위로 올라와 겹친다.

## 이슈가 요구한 것

- 그림 x 를 정본과 맞춘다(모든 쪽 상수 −18.6px 해소).
- 폭·본문 글줄은 이미 정본과 맞으므로 건드리지 않는다.
- `#6378`·`#1133` 의 이중 가산 금지 계약을 깨지 않는다.

## 원인 — 이슈가 지목한 자리가 아니었다

이슈 본문은 `table_layout.rs` `compute_table_x_position` 의 `HorzRelTo::Para` 분기를
지목했다. 그 자리에 `outMargin.left` 를 실어 빌드했더니 **좌표가 전혀 변하지 않았다**
(Image x=431.7 그대로). 이 형상은 그 분기에 도달하지 않는다.

실제 산출 지점은 `layout.rs` 의 `tbl_inline_x` 오버라이드다.

```rust
} else if !is_tac
    && tbl_is_square
    && matches!(t.common.horz_rel_to, HorzRelTo::Para)
{
    // [Issue #480 / #590]
    let area_x = col_area.x + effective_margin;
    …
    _ => area_x,          // ← outMargin.left 가 없다
};
Some(x)
```

여기서 확정된 `x` 가 `inline_x_override` 로 `compute_table_x_position` 에 들어가고,
비-TAC 경로에서 `h_offset` 만 더해져 끝난다 — `HorzRelTo::Para` 분기는 지나가지 않는다.

```text
  col.x 75.6 + effective_margin 0 + horzOffset 26319HU(350.9px) = 426.5   ← rhwp 실측 426.5
  여기에 outMargin.left 1417HU(18.89px)                          = 445.4   ← 정본
```

**같은 파일 바로 위 TAC 분기**(`base_x = col_area.x + effective_margin + leading + om_l`)와
**개체 경로**(`shape_layout::form_object_origin` 의 `ref_x + m_left + h_offset`)는 이미
이 여백을 싣고 있다. 어울림 표 경로만 빠져 있었다.

## 수정

`float_placement.rs` 에 `#6378` 헬퍼와 같은 결로 게이트 헬퍼를 두고,
`layout.rs` 의 왼쪽 정렬 갈래에서만 그 값을 싣는다.

```rust
pub(crate) fn para_relative_left_aligned_outer_margin_left_hu(table: &Table) -> Option<i32>
// 비-TAC · horzRelTo=PARA · horzAlign=LEFT|INSIDE · outMargin.left > 0
```

오른쪽·가운데 정렬은 기준 폭(`area_w`) 산식이 달라 실측 근거가 나올 때까지 두었다.
`#6378` 이 연 `HorzRelTo::Column` 형상과는 기준이 달라 겹치지 않는다.

## 검증 실측

문서: `156492236_220119_(보도참고자료)_규제샌드박스_시행_3주년(규제자유툭구단).hwpx`
명령: `rhwp export-render-tree <문서> -p {4,6,8}`

| 쪽 | before | 정본 | after | 잔차 |
|---|---|---|---|---|
| 5 | 431.7 | 450.3 | 450.6 | +0.3 |
| 5 | 433.3 | 451.9 | 452.2 | +0.3 |
| 7 | 462.4 | 481.0 | 481.3 | +0.3 |
| 7 | 464.5 | 483.1 | 483.4 | +0.3 |
| 9 | 470.3 | 488.9 | 489.2 | +0.3 |
| 9 | 462.3 | 480.9 | 481.2 | +0.3 |

잔차 +0.3px 는 이슈가 예고한 반올림(1417HU = 18.89px ↔ 관측 Δ 18.6px)이다.
폭은 전후 동일하고, `LAYOUT_OVERFLOW` 진단도 전후 동일하다(p4 pi=59 24.8px,
p8 pi=91 82.5px — 둘 다 이 축과 무관한 기존 값).

시각 근거(5쪽 SVG 전후): 종전에는 그림이 호스트 본문 글자
(「보완」「서비스는」「및」「가능」)를 덮었고, 수정 뒤 글자와 그림이 분리된다.

## 게이트

| 게이트 | 결과 |
|---|---|
| `cargo fmt --all -- --check` | OK (수정 없음) |
| `cargo clippy --locked -- -D warnings` | OK |
| `cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown -- -D warnings` | OK |
| `cargo build --locked --workspace` | OK |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | OK |
| `node scripts/rust-test-suite-manifest.mjs --check` | 48/48 targets |
| `cargo test --release --workspace` | **9421 passed / 0 failed / 49 ignored** (94 suites, `convert_verify_corpus_ratchet` 전 파티션 포함) |

## 남긴 것

- `horzAlign=Right/Center` 인 같은 형상은 손대지 않았다 — 기준 폭 산식이 달라
  별도 실측이 필요하다.
- `compute_table_x_position` 의 `HorzRelTo::Para` 분기(어울림이 아닌 문단 기준 float)도
  같은 여백이 빠져 있을 수 있으나, 이 문서의 증거로는 닿지 않아 열지 않았다.
