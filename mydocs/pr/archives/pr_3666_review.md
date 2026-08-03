---
kind: review
status: active
canonical: mydocs/pr/archives/pr_3666_review.md
last_verified: 2026-08-01
---

# PR #3666 리뷰 기록

## 라우팅

```text
base route: collaborator self-merge
modifiers: intake_and_review.md, local_validation.md,
  visual_fixture_evidence.md, multi_pr_update_branch.md
integration branch: integrate/lpaiu-planet-20260731
integration PR: #3671
```

원 PR은 #3665와 함께 누적 체리픽 검토했다. 적용 순서·원 author 보존·maintainer 보정·후속 처리는
[공유 계획](pr_3665_review_impl.md)을 따른다.

## PR metadata

| 항목 | 값 |
| --- | --- |
| PR | [#3666](https://github.com/edwardkim/rhwp/pull/3666) |
| 작성자 | `@planet6897` |
| base / source head | `devel` / `5f4d7d9f…` (작성 시점 참고) |
| 기능 commit | `98c63b1ab…` |
| 통합 반영 | `ab12d4f29…` (`-x` 추적), maintainer 보정 `0d1839ae9…` |
| reviewer | `@edwardkim` 요청 완료 |
| 관련 issue | [#3637](https://github.com/edwardkim/rhwp/issues/3637), 이미 closed |

source head의 CI는 lint, frontend gate, Native Skia, default-feature 8 shards, `Build & Test`,
CodeQL, Canvas visual diff가 성공한 상태로 확인했다. volatile source 상태를 merge 근거로 단정하지 않고,
통합 code candidate CI를 별도로 통과시켰다.

## 변경 범위와 판정

이 PR은 분할된 표 셀의 중첩 표 뒤 문단이 셀 전체 좌표인 `LINE_SEG.vertical_pos`를 그대로 사용해
쪽 밖으로 밀리던 #3637 기전을 다룬다. 기능 commit은 첫 visible paragraph의 vpos를 조각 원점으로 빼고,
셀 바닥에서 상한을 둔다.

통합 코드 리뷰에서 두 경계 보정이 필요했다.

- 기존 상한 `text_y_start + cell_h`는 Center/Bottom vertical alignment나 top padding이 있을 때 물리적
  셀 바닥이 아니었다. `cell_y + cell_h - pad_bottom`을 사용해 content bottom을 고정했다.
- 조각이 문단 중간 line에서 시작해도 `line_segs.first()`를 원점으로 쓰지 않고, 실제 first visible line
  (`start_line`)의 vpos를 사용한다. stored `LINE_SEG`가 recomposed line보다 짧은 경우의 안전 fallback도
  고정했다.

두 보정은 `split_cell_fragment_origin_uses_first_visible_line_not_paragraph_start`와
`split_cell_snap_cap_uses_physical_content_bottom_not_valign_start` unit 회귀로 고정했다. 변경은
다른 표/문단 경로를 넓게 재조판하지 않는다.

## 로컬 검증

| 검증 | 결과 |
| --- | --- |
| `cargo test --profile release-test --lib split_cell_` | 14 passed |
| `cargo test --profile release-test --test issue_3637_split_cell_nested_table_vpos` | 1 passed |
| `cargo test --profile release-test --tests` | 최종 exit code 0 (lib 3,036 passed, integration 완료) |
| `cargo fmt --check`, `git diff --check` | 성공 |
| `cargo clippy --profile release-test --all-targets -- -D warnings` | 성공 |
| Native Skia 공식 3종 | 58/58, 2/2, 4/4 passed |
| `cargo test --doc` | 4 passed / 2 ignored |
| `wasm-pack build --target web --out-dir pkg` | 성공 |
| 신규 HWPX IR field sweep | 실행 완료; HWP5의 기존 2개 `extra_child_records` baseline 차이는 원인 미확정이라 baseline에 추가하지 않음 |

모든 Cargo 검증은 `CARGO_INCREMENTAL=0`,
`CARGO_TARGET_DIR=target/review-lpaiu-planet-20260731`에서 한 번에 하나씩 실행했다. 이후 #3665의
Studio-only commit은 Rust tree를 바꾸지 않았고, 그 commit은 별도의 TypeScript·705개 Studio test·build와
통합 CI frontend gate로 확인했다.

`dump-extents --outside`의 최종 후보는 13쪽 중 p10–p13에서 page-boundary node가 없음을 보였다.
p2에는 다른 3×1 table subtree의 최대 17.1px 및 `TextLine` 5.5px 초과가 남는다. 이는 #3666이
제거한 p10 nested-cell vpos 기전과 다른 기존 후보이며, 30px focused 계약을 통과한 것을 전체 overflow 0으로
확대하지 않는다.

## 시각·fixture 증적

| 자산 | 역할 | SHA-256 |
| --- | --- | --- |
| `samples/issue3637/press_release_split_cell_nested_table.hwpx` | 원본 재현 fixture | `e2cb077d51293ae081a83e49c3bf2bca701087fddb4068d29295607b326be410` |
| `pdf/issue3637/press_release_split_cell_nested_table-2020.pdf` | HWP 2020 기준 PDF, 12쪽, 1,022,238 bytes | `fffc0aca6ecce34ad4ea9d071474b010d7e915a8c1ae883ccaa06c90999f690f` |
| `mydocs/pr/assets/pr_3671_issue_3637_visual_review_p010.png` | #3637 발동 구간 p10 review panel | `a938a133418682541384d624204c1080981e07a67f67e5c2dab82552faa7f440` |
| `mydocs/pr/assets/pr_3671_issue_3637_visual_review_p012.png` | 잔존 PDF/page-flow 후보 p12 review panel | `3816063431f7975aa60ee3e39bd63909887a43db7d84cdf2e9c0b16dc907c3c0` |

기준 PDF는 direct HWPX→PDF 요청이 source metric 전달 실패로 client 수신을 완료하지 못해, 동일 HWP
2020 MCP에서 HWPX→HWP(job `cef40753-d9e3-4468-a052-353da5facb92`, validation ok, run_status 0) 뒤
HWP→PDF(job `c4dc4e2e-009e-4e63-b15c-dfec2323fb59`, validation ok, run_status 0)로 생성했다. server URL·token은
기록하지 않는다.

최종 debug binary로 `visual_sweep.py`를 p1–8, p9–12 두 batch로 실행했다. 각 batch는 13개 SVG/
render tree를 export하고, 기준 PDF의 요청한 12쪽 raster·compare·overlay·review를 모두 완료했다.

| 페이지 | 구조 후보 | pixel match | ink proxy | 사람 판정 |
| --- | --- | ---: | ---: | --- |
| p10 | 없음 | 84.78560% | 19.79698% | 분할 셀 뒤 내용이 쪽 밖으로 사라지는 기존 기전은 발견하지 못함 |
| p12 | frame/tail/content-bottom/line·column drift | 86.09069% | 8.26627% | 기준 PDF와 대응 흐름이 달라 별도 후보로 보존 |

임시 산출 경로는
`output/visual-pr3666-final/p1-8/pr3666_split_cell_nested_table/` 및
`output/visual-pr3666-final/p9-12/pr3666_split_cell_nested_table/`이다. 영구 증적은
[p10 review](../assets/pr_3671_issue_3637_visual_review_p010.png),
[p12 review](../assets/pr_3671_issue_3637_visual_review_p012.png)다.

rhwp SVG는 13쪽, 한컴 기준 PDF는 12쪽이며 p2·p4·p12에서 frame/line 후보가 남는다. p13에는 rhwp의
공개자료 footer가 독립 페이지로 남아 있다. 따라서 이 evidence는 #3666의 p10 overflow 감소 계약을
뒷받침하지만, PDF 전체 시각 동등 또는 전수 무결함을 주장하지 않는다. pixel/ink 지표는 font raster와
페이지 대응 차이를 포함하는 보조값일 뿐 최종 판정이 아니다.

## CI와 권고

통합 code candidate `3a1db3234ff80466bc8dfd49364de49f66db8e0d`의 CI preflight, lint, frontend
package gates, Native Skia, test archive, default-feature 8 shards, `Build & Test`, CodeQL,
Canvas visual diff는 모두 성공했다. 이 review·PDF·PNG·오늘할일 tail은 code CI 뒤 추가하므로,
최종 merge 조건은 최신 #3671 head의 review-only fast-pass preflight와 `Build & Test` aggregate 성공,
`CLEAN`·`MERGEABLE` 재확인이다. 조건 충족 전 권고는 **보류**, 충족 후 권고는 **통합 merge**다.
