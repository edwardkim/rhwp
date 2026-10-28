# task_m100_4324 Stage 1 — 문단 서식 흐름 영향 술어와 머리말/꼬리말 stale 스타일

- **이슈**: [#4324](https://github.com/edwardkim/rhwp/issues/4324)
- **PR**: [#4380](https://github.com/edwardkim/rhwp/pull/4380)
- **브랜치**: `fix/issue-4324-para-flow-predicate`
- **분기 기준**: `upstream/devel` `e48fe8694`
- **상태**: 로컬 전체 검증 통과, PR 게시
- **기록일**: 2026-08-09 KST

## 1. 결함 — 게이트가 줄간격만 봤다

```rust
if mods.line_spacing.is_some() || mods.line_spacing_type.is_some() {
```

여백·들여쓰기는 사용 가능 폭을, 줄나눔 단위는 토큰 경계를 바꾸는데 리플로우를 건너뛰어 문단 서식을
바꿔도 줄바꿈이 변경 전 상태로 남았다.

같은 조건이 세 곳에 있었다 — `apply_para_format_native`(`formatting.rs:1423`),
`apply_para_format_in_cell_native`(`:1592`), 그리고 **이슈에 없던**
`apply_para_format_in_hf_native`(`header_footer_ops.rs:857`).

## 2. `ParaShapeMods` 27개 필드 전수 판정

`char_shape_mods_affect_text_flow`(`formatting.rs:16`)의 문단모양 대응물을 그 형태 그대로 만들었다.
흐름에 영향을 주는 것은 7개다.

| 필드 | 근거 |
|---|---|
| `margin_left`/`margin_right` | 호출부가 `available_width = 폭 - margin_left - margin_right` |
| `indent` | `fill_lines` 의 `eff_w()` 가 첫 줄 유효 폭을 줄인다(`line_breaking.rs:681-696`) |
| `english_break_unit`/`korean_break_unit` | `tokenize_paragraph`/`fill_lines` 가 토큰 경계를 바꾼다(`:360`, `:375`, `:798`) |
| `line_spacing`/`line_spacing_type` | 원래 게이트 |

나머지 20개는 `reflow_line_segs`/`fill_lines` 가 읽지 않는다. `alignment` 는 `line_breaking.rs` 에
참조 0건으로 이미 나뉜 줄 안에서의 배분이고, 문단 테두리는 레이아웃 후 장식이며, 문단 간격·쪽나눔
휴리스틱은 `rebuild_section` 이 매번 다시 계산하는 단계에서 처리된다.

`tab_def_id` 는 오늘 시점 흐름에 영향이 없다 — `resolve_single_para_style` 이 `default_tab_width` 를
4000 HWPUNIT 상수로 고정하고 `tab_stops` 를 `fill_lines` 에 넘기지 않는다. 별개 사안으로 #4403 에
등록했다.

## 3. 리뷰가 잡아낸 두 번째 결함

1번 수정만으로는 **머리말/꼬리말이 동작하지 않았다.**

`apply_para_format_in_hf_native` 는 `find_or_create_para_shape` 로 새 para_shape 를 만든 직후
(`:848`) `reflow_hf_paragraph` 를 부른다(`:859`). 그 사이 `self.styles` 갱신이 없어,
`reflow_hf_paragraph` 가 캐시된 `self.styles.para_styles.get(new_id)` 를 읽으면 **`None` → 여백
0.0 폴백**이다.

재현: 200자 머리말에 `marginLeft:8000` → `before=5 lines, after=5 lines`(변화 없음).

`resolve_styles(&self.document.doc_info, self.dpi)` 로 새로 뽑도록 고쳤다 —
`reflow_cell_paragraph`(`text_editing.rs`)와 `apply_para_format_native` 가 이미 쓰는 규약이다.

**줄간격 경로도 같은 뿌리라 함께 해소된다.** 종전에도 새 para_shape 가 만들어지면 `reflow_line_segs`
가 stale 스냅샷에서 `line_spacing` 을 읽어 기본값(`Percent`/160.0)으로 폴백했다. 같은 `styles`
변수를 공유하므로 한 번의 수정으로 둘 다 닫힌다.

## 4. 게이트가 잡은 세 번째

재게이트에서 `issue_3494_char_count_convention` 규약 가드가 실패했다 — 새로 쓴 테스트 헬퍼가
`para.char_count = text.chars().count()` 로 대입했는데, 공통 IR 의 `char_count` 는 **문단 종결자를
포함**한다(`model/paragraph.rs`). `+ 1` 로 고쳤다.

프로덕션이 아니라 **테스트 코드의 규약 위반**을 잡은 사례다.

## 5. 검증 (완료)

- 술어 단위 테스트 2건 + 본문·셀 재현 2건(이슈 실측 규모: 셀 폭 20000 HWPUNIT, `marginLeft:8000`).
- **행위 테스트** `para_format_margin_change_in_header_reflows_with_correct_width_not_stale_styles`
  — 구현 전에 먼저 작성해 미수정 코드에서 실패를 확인했고, 수정 후 stale 읽기를 일부러 되돌리니
  다시 실패하는 것까지 확인했다. 술어 단위 테스트로는 3절 결함이 잡히지 않는다.
- `cargo test --profile release-test --tests` 전체 통과.
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` 통과.

남은 미래 조건은 GitHub Actions 와 작업지시자 승인, merge 다.
