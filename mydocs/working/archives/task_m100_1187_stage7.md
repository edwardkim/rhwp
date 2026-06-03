# Task M100-1187 Stage 7 완료 보고 — PR #1190 CI snapshot 실패 보정

## 개요

- 이슈: #1187 `BookReview.hwp 글상자 내용이 영역 밖으로 출력되는 회귀`
- PR: #1190
- CI 실패: `Build & Test`
- 실패 로그:
  - `cargo test --test svg_snapshot`
  - `issue_267_ktx_toc_page`
  - `issue_617_exam_kor_page5`

## 원인

Stage 6 에서 글상자 내부 문단의 column-top `line_seg.vertical_pos`
중복 보정을 막기 위해 `cell_ctx.is_none()` 조건을 추가했다. 그러나
`cell_ctx` 는 글상자뿐 아니라 표 셀 문단에도 사용된다.

그 결과 표 셀 문단에서 기존 column-top vpos fallback 이 비활성화되어
일부 snapshot 의 표 셀 텍스트 y 좌표가 달라졌다. 이 변화는 #1187 의
글상자 clip 범위를 벗어난 부작용이다.

## 수정

- `layout_composed_paragraph` 에
  `suppress_column_top_vpos_fallback` 플래그를 추가했다.
- 일반 본문, 표 셀, 캡션, 각주 호출은 `false` 로 유지했다.
- 글상자 내부 가로쓰기 문단 호출만 `true` 로 지정했다.
- 기존 `cell_ctx.is_none()` 기반 조건은 제거하고, 명시 플래그 기준으로
  column-top vpos fallback 적용 여부를 결정한다.
- 의도한 텍스트박스 clip 출력 변화가 반영되도록 다음 SVG snapshot golden 을
  갱신했다.
  - `tests/golden_svg/issue-267/ktx-toc-page.svg`
  - `tests/golden_svg/issue-617/exam-kor-page5.svg`

## 검증

통과:

```bash
cargo fmt --all -- --check
git diff --check
cargo build --bin rhwp
cargo test --test svg_snapshot issue_267_ktx_toc_page -- --nocapture
cargo test --test svg_snapshot issue_617_exam_kor_page5 -- --nocapture
cargo test --test svg_snapshot -- --nocapture
cargo test --test issue_1187_textbox_clip -- --nocapture
cargo test --test issue_1052_footnote_in_textbox --test issue_919_textbox_hit_test --test issue_1028_hwpx_textbox_vertical -- --nocapture
cargo test --lib paint::builder::tests -- --nocapture
cargo test --lib renderer::svg_layer::tests -- --nocapture
cargo test --lib paint::json::tests::serializes_textbox_clip_kind -- --nocapture
cargo test --lib paint::schema::tests::layer_tree_schema_constants_match_schema -- --nocapture
wasm-pack build --target web --dev
npm run build
```

결과:

- `svg_snapshot`: 8 passed
- #1187 회귀 테스트: 2 passed
- 기존 글상자/각주/히트테스트/세로쓰기 관련 테스트: 11 passed
- paint builder/svg/json/schema 관련 테스트: 12 passed
- `wasm-pack build --target web --dev`: 통과
- `npm run build`: 통과

## 비고

- `issue_267_ktx_toc_page` 실행 중 기존 `LAYOUT_OVERFLOW` 진단 로그가 출력되지만
  snapshot 자체는 통과한다.
- 테스트 실패 재현 중 생성된 `.actual.svg` 임시 파일은 커밋 대상에서 제거했다.
