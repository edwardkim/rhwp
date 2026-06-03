# 최종 결과보고서 — Task M100 #1205

**이슈**: [#1205](https://github.com/edwardkim/rhwp/issues/1205) HWPX 문단 borderFill NONE side 렌더링 정합성 수정
**마일스톤**: v1.0.0 (M100)
**브랜치**: `issue-1205-para-border-none-side` (별도 worktree: `/private/tmp/rhwp-issue-1205`)
**완료일**: 2026-06-03

---

## 1. 문제

HWPX 문단 `borderFill`에서 좌우 side가 `NONE`, 상하 side가 `SOLID`인 경우에도 렌더러가 4면 `RectangleNode` stroke 또는 좌우 `LineNode`를 만들어 수직선이 표시될 수 있었다.

재현 샘플에서는 10쪽 하단 문단 박스가 PDF 기준과 다르게 좌우 수직선을 포함해 보이는 문제가 있었다.

## 2. 원인

HWPX parser는 `leftBorder`, `rightBorder`, `topBorder`, `bottomBorder`를 side별로 파싱하고 `type="NONE"`도 `BorderLineType::None`으로 보존한다.

문제는 renderer 경로였다.

- 문단 border group 렌더링이 top border를 대표 stroke처럼 사용했다.
- 일반 경로는 `RectangleNode` stroke 하나로 4면을 그렸다.
- partial/cross-column 경로도 좌우 수직선을 side visible 여부와 무관하게 생성할 수 있었다.

## 3. 수정

`src/renderer/layout.rs`:

- `para_border_is_visible()` 추가: `BorderLineType::None` 또는 width 0 side는 비가시로 판단.
- `para_border_same_stroke()` 추가: side별 line type, width, color 동일성 비교.
- `para_border_can_use_rect_stroke()` 추가: 4면 모두 visible이고 동일 stroke이며 top/bottom partial skip이 없을 때만 기존 `RectangleNode` stroke 경로 사용.
- 그 외 문단 border는 fill-only `RectangleNode`와 visible side별 `LineNode`로 분해.
- side별 line 생성은 `border_rendering::create_border_line_nodes()`를 사용해 기존 border line type 표현을 보존.

`src/renderer/layout/integration_tests.rs`:

- synthetic 문단 border 렌더 helper 추가.
- `[NONE, NONE, SOLID, SOLID]`에서 수직선과 4면 stroke rect가 없는지 검증.
- 기존 rect stroke 최적화 경로가 4면 visible/same stroke 조건에서만 쓰이는지 검증.

SVG golden:

- `stroke` 없는 `fill="none"` 문단 박스 노드 제거에 맞춰 snapshot 4건 갱신.

## 4. 검증

| 항목 | 결과 |
|------|------|
| `cargo fmt --all --check` | 성공 |
| `cargo test --lib task_1205 -- --nocapture` | 성공 |
| `cargo test --lib test_469_partial_start_box_does_not_cross_col_top -- --nocapture` | 성공 |
| `cargo test --lib test_471_cross_column_box_no_bottom_line_in_col0 -- --nocapture` | 성공 |
| `cargo test --tests` | 성공 |
| `/Users/melee/Downloads/[2027] 온새미로 1 본교재.hwpx` 10쪽 SVG export | 성공 |
| `wasm-pack build --target web` | 성공 |
| `rhwp-studio` Vite server | `http://127.0.0.1:7701/` 실행 및 200 OK 확인 |

## 5. 산출물

- 소스: `src/renderer/layout.rs`
- 테스트: `src/renderer/layout/integration_tests.rs`
- 골든: `tests/golden_svg/issue-{147,157,617,677}/...`
- 계획: `mydocs/plans/task_m100_1205.md`, `mydocs/plans/task_m100_1205_impl.md`
- 단계 보고: `mydocs/working/task_m100_1205_stage{1,2,3,4}.md`
- 최종 보고: 본 문서

## 6. 비고

`rhwp-studio` 실행을 위해 별도 worktree에서 `pkg/`와 `rhwp-studio/node_modules/`를 생성했지만, 둘 다 git status에는 포함되지 않는다.

이슈 close는 작업지시자 승인 전까지 수행하지 않는다.
