---
kind: working
status: active
issue: 6924
---

# 감싼 칸의 clip 하단 글줄이 통째로 사라진다 (#6924)

작업 브랜치: `fix/6924-cell-clip-last-line`
대상: `src/renderer/layout/table_partial.rs`

## 한 줄

`suppress_bottom_clipped_text_residue`(#2007)가 clip 바닥에 걸친 글줄을 "다음 조각이
그릴 것" 이라 보고 끄는데, **그 전제를 확인하지 않아** 소유자가 없는 줄까지 꺼져
문서에서 사라진다.

## 잘리는 게 아니라 꺼진다

본문 진단("칸 clip 하단이 baseline 보다 위")은 정확한 관측이지만 SVG 가 자르는 게 아니다.
paint LayerBuilder 프로브:

```text
PROBE TextLine visible=false editor_only=false y=920.6 h=16.0
```

`build_node` 가 `!node.visible` 에서 `None` 을 돌려주므로 PaintOp 자체가 생기지 않는다.
`export-render-tree` 는 이 필터 **전** 트리라 정상으로 보인다 — 두 명령이 갈리는 이유다.

`visible = false` 자리는 다섯뿐이고 프로브로 범인을 특정했다: `SITE775` =
`suppress_bottom_clipped_text_residue`. 조건도 수치로 맞는다
(`sliver = 922.45 − 920.6 = 1.88 < 6.0`).

## 전제가 확인되지 않았다

주석은 *"the successor fragment remains responsible for the full line"* 이라 적어 두었지만
그것을 검사하는 코드가 없다. 실측:

```text
148751598  쪽 1 render tree 에 그 줄 있음 · 6쪽 SVG 전수 0건 · export-text 1쪽에는 있음
           → 1쪽에만 조판되고 1쪽에서 꺼지고 어디서도 다시 안 그려진다
```

## 소속은 이미 계산할 수 있다

분할 표 경로에는 줄 단위 소속 판정이 있다.

```rust
// src/renderer/layout/table_partial.rs
partial_table_page_contains_cell_position(table, cell, start_row, end_row,
    start_cut, end_cut, is_block_split, Some((para_idx, line_idx, true)), styles)
```

컷 부기(`start_cut`/`end_cut`)와 `TextLineNode::{para_index, line_index}`,
`TableCellNode::model_cell_index` 가 모두 갖춰져 있다. **전제를 추측이 아니라 계산으로
바꾼다.**

## 수정 — 억제를 고치지 않고 전제를 없앤다

`expand_fragment_cell_clip_for_owned_bottom_lines` 를 조각 표 조립 직후에 한 번 돌린다.

1. 조각 셀의 clip 바닥을 **걸치는** 글줄을 찾는다(완전히 안/밖인 줄은 다른 축).
2. `owns_line` 으로 이 조각이 그 줄을 소유하는지 묻는다. 아니면 그대로 둔다 — 다른 조각이
   그리므로 종전 억제가 옳다.
3. 소유하면 셀을 그 줄 아랫변까지 넓힌다. **단 형제 셀과 부딪치면 넓히지 않는다.**

넓히고 나면 그 줄은 더는 바닥에 "걸치지" 않으므로 **억제 조건 자체가 성립하지 않는다.**
`suppress_bottom_clipped_text_residue` 는 한 줄도 고치지 않았다.

### 왜 형제 충돌 가드가 필요한가 — 래칫이 강제했다

소유 판정만 넣은 1차 구현은 겹침 래칫에 걸렸다.

```text
증가: task2287/1342000_edu_curriculum_map.hwp — 94 → 97건
```

새 겹침 3건의 위치를 `layout-anomaly --json` 으로 특정했다.

```text
82쪽   Cell3/TextLine6  (y=240.0) ↔ Cell10/TextLine0 (y=246.0)
152쪽  Cell13/TextLine4 (y=217.2) ↔ Cell24/TextLine0 (y=223.8)
```

소유한 줄이라도 이웃 셀 내용 위로 나가면 두 글자 모두 못 읽는다. 아래 셀이 있는 셀은
넓히지 않는 것으로 막았고 94 로 복귀했다. `local_validation.md` 4.3.1 의
"기존 문서의 수치 증가는 회귀다 — baseline 으로 숨기지 않는다" 를 그대로 따른 것이다.

## 검증 실측

### 영향 모집단 전수

억제 진입점에 계측을 걸어 전수 조사했다.

| 모집단 | 억제 발생 문서 | 조각 셀(`page_fragment=true`) |
|---|---|---|
| native HWP5 코퍼스 702건 | 6 | 3 |
| 저장소 `samples/` 824건 | 9 | 4 |

### 복구 (한컴 2024 정답지 대조)

| 문서 | 쪽 | 글줄 | 기준선 | 수정 | 정답지 |
|---|---|---|---|---|---|
| 148751598 | 1 | `* 장관 참석행사는 ★로…` (31자) | 없음 | **있음** | 있음 |
| 148738070 | 1 | `‘발효생성아미노산복합물’을 뜻함…` (38자) | 없음 | **있음** | 있음 |
| 156060125 | 6 | `리스크 집중위험 점검 등` | 없음 | **있음** | — |
| 1490000 vietnam | 114 | `⑦ 물류 ⑧ 기타 ( )` | 없음 | **있음** | 114쪽 |

`156060125` 는 이슈 밖에서 새로 찾은 같은 결함이다.

### 회귀 없음

| 축 | 결과 |
|---|---|
| `text_overlap_baseline` 16 partition | 통과 (edu 94 유지) |
| `cargo test --release` 전 타깃(28 suite + lib, 순차) | **9095 passed / 0 failed / 38 ignored** |
| `rust-unit-test-tiers --check` | 4205 (기준선 유지) |

정답지 대조에서 중복으로 의심했던 두 건은 오독이었다.

- `1490000 vietnam '으로보고계십니까'` — 정답지 86·121 **2쪽**, 기준선은 87 **1쪽**뿐이었다.
  rhwp 쪽 번호가 정답지보다 +1 이라 121↔122 다. 중복이 아니라 **소실**이었고 수정이 옳다.
- `1342000 edu '3.사회적가치…'` — 기준선에서 이미 82·83 두 쪽이다. 이 변경과 무관한
  기존 중복이다.

## 회귀 테스트

`tests/cases/issue_6924_fragment_owned_line_kept.rs` — 저장소 표본
`samples/issue5714/1490000-200800034_vietnam_labor_report.hwp` 114쪽에서
`⑦물류⑧기타` 가 방출되는지 고정한다.

`src/` 안 `#[cfg(test)]` 총량은 래칫으로 묶여 있어 통합 시험으로 두었다.

> ⚠ 이 렌더러는 **글자마다 `<text>` 를 따로** 낸다. SVG 를 문자열 grep 하면 항상 0 건이라
> 존재 판정이 뒤집힌다(이 조사에서 두 번 잘못 결론냈다). 시험도 x 순 정렬 후 이어 붙인다.

## 남긴 것

- **`1342000 edu` 377쪽 `• 선언문 작성`** — 정답지(378쪽)에는 있지만 아래 셀이 있어 넓히면
  이웃 셀을 덮는다. 보수적으로 포기했다. 되살리려면 **행 높이 축**을 고쳐 그 줄이 들어갈
  자리를 만들어야 한다 — 이 이슈에서 풀 문제가 아니다.
- **`table_giant_cell_overfill`** 의 두 줄은 소유 판정에서 걸리지 않아 종전대로 남는다.
- `#2007` 의 원 문서(`1220000-202100002`)는 이 코퍼스에 없어 직접 통제군을 세우지 못했다.
  대신 영향 모집단 전수(9건)와 겹침 래칫으로 회귀 없음을 확인했다.
