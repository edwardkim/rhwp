# Task #990 Stage 1 완료보고서 — 재현 + 진단 (RED)

**이슈**: [#990](https://github.com/edwardkim/rhwp/issues/990)
**브랜치**: `local/task990`

> **후속 정정**: 아래 재현 픽스처(`samples/issue-990-tac-box.hwpx`)와 RED
> 테스트(`tests/issue_990.rs`)는 비공개 문서 기반이라 작업지시자 지시로
> git 에서 제거됨 — 개발 단계 로컬 검증 기록으로만 참조. 커밋된 회귀
> 가드는 `issue_table_vpos_01_page5_cell_hit_test`.

---

## 1. 재현 픽스처

`samples/issue-990-tac-box.hwpx` — 4쪽(global_idx=3)에 `※ …` 글상자 3개가
빈 문단(`pi=54/55/56`) 위에 연속 배치된 문서.

## 2. RED 테스트

`tests/issue_990.rs` — `build_page_render_tree(3)` 의 `Rectangle` 노드
(글상자, `para_index=54/55/56`)를 수집해 박스 사이 세로 advance 를 검사.

```
issue #990 box y=[344.08, 476.96, 609.84] h1=49.11 advance=[132.88, 132.88]
test result: FAILED
```

- 박스↔박스 advance = **132.88px** — 기대(빈 호스트 문단 LINE_SEG 1회분
  `lh + ls = 4983 HU ≈ 66.44px`)의 **정확히 2배**.

## 3. 진단 — 회귀 확정 (bisect)

| 지점 | advance | 상태 |
|------|---------|------|
| `c3e32151~1` (Task #974 직전) | 66.44px | ✅ 정상 |
| **`c3e32151`** ("Fix textbox picture rendering", Task #974) | 132.88px | ❌ 회귀 도입 |
| `devel` HEAD (`39d90d9d`) | 132.88px | ❌ |

**원인**: 커밋 `c3e32151` 가 `layout_shape_item()` 에 `Control::Shape` 분기를
**신규 추가**(`c3e32151~1` 에는 해당 분기 자체가 없음). 추가된 코드 말미:

```rust
let line_advance = ls.line_height + ls.line_spacing;   // 4983 HU
result_y = shape_y + line_advance.max(shape_h);         // ← 신규 재진행
```

빈 문단 위 treat-as-char 글상자는 `FullParagraph` PageItem 의
`layout_paragraph()` 가 이미 LINE_SEG 만큼 advance 한 상태인데, 추가된
`Shape` PageItem 분기가 `result_y` 를 **또 한 번 진행** → `4983 + 4983 = 9966 HU`.

부수: `shape_y = y_offset`(FullParagraph 진행 후 값) 이라 글상자 자체도
`para_start + 4983` 위치에 그려짐 — 위치도 어긋남.

## 4. 브랜치 정비

`local/devel` 이 PR #538 시점(merge-base `e585b589`)에서 분기해 stale
(local 139 / stream 635 비-merge 커밋). `local/devel` 의 커밋들은 git rebase
판정상 대부분 "already upstream".

- `local/devel` → `stream/devel`(= `devel` HEAD `39d90d9d`) 정렬.
  stale 커밋 139개는 `backup/local-devel-20260518` 에 보존.
- `local/task990` → 새 `local/devel` 베이스로 재정렬(`rebase --onto`).
  → 회귀(Task #974) 가 베이스에 포함되어 RED 재현 성립.

## 5. 다음 단계 (Stage 2)

`layout_shape_item()` 의 `!has_real_text` 분기 정정:
- `shape_y` 를 `para_start_y[para_index]`(호스트 문단 시작)로 산출.
- `result_y` 재진행 제거(입력 `y_offset` 유지) — `FullParagraph` 가 이미 진행.
- Task #974 의도(`set_inline_shape_position` 등록, hy-001 글상자 그림) 보존 확인.
