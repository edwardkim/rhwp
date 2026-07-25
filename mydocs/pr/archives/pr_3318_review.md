# PR #3318 검토 기록

## 라우팅

```text
base route: collaborator_external_pr
modifiers: intake_and_review, local_validation, multi_pr_update_branch,
  visual_fixture_evidence
loaded documents: pr_review_workflow.md, pr_review/README.md,
  collaborator_external_pr.md, intake_and_review.md, local_validation.md,
  multi_pr_update_branch.md, visual_fixture_evidence.md
current head: 작성 시점 참고값 2e60b0645b99d748f2abe858da81cbf20ec4dd1b
```

## PR metadata

| 항목 | 작성 시점 참고값 |
| --- | --- |
| PR | [#3318](https://github.com/edwardkim/rhwp/pull/3318) |
| 제목 | `fix(#3314): 굵기 접미사 face 폴백에 base family 삽입` |
| 작성자 / base | `planet6897` / `devel` |
| 원 head / commit | `fix/3314-base-family-fallback` / `2e60b0645b99d748f2abe858da81cbf20ec4dd1b` |
| 관련 이슈 | [#3314](https://github.com/edwardkim/rhwp/issues/3314) |
| 규모 | 4 files, +112/-9, 1 commit |
| 원 PR 상태 | `MERGEABLE`, `BEHIND`; maintainer 수정 허용, reviewer `jangster77` 요청 완료 |
| 통합 branch | `review/planet6897-font-20260726` (`upstream/devel` `61b13fad4` 기준) |

## 변경과 누적 적용

- 원 commit `2e60b06`을 #3310 뒤에 `7da56d169`으로 cherry-pick했다. `text_replay.rs` 자동 병합은
  충돌 없이 두 변경의 순서를 보존했다.
- 원 변경은 SVG 네 경로, HTML, Native Skia가 요청 face → base family → generic fallback을 사용하도록
  하고 text measurement는 그대로 둔다.
- 같은 폴백 계약을 쓰는 Web Canvas 2D의 회전 text, 일반 text, char overlap, text control mark 네 경로가
  빠진 것을 발견했다. 보정 commit `678494aa0`은 공용 `canvas_font_family_chain()`으로 네 경로를 통일해
  Canvas에서도 base family가 generic 앞에 오도록 했다.

## 검증

- `test_base_family_without_weight_suffix`: 요청 face, base family, SVG·Canvas 인용 chain을 함께 검증해 통과
- `cargo test --profile release-test --features native-skia skia --lib`: 57 passed
- `issue_2225_missing_picture_placeholder`: 2 passed
- `render_p37_direct_pdf_export`: 4 passed
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 전수 실행 완료
  (IR field sweep 포함; fixture 변경 없음)
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `git diff --check`: 통과
- WASM build는 작업지시자가 수동 실행하는 범위라 이 review에서 재실행하지 않았다.

## 시각·fixture 판정

#3314가 언급한 `1.hwpx`와 한컴 기준 PDF가 PR·issue에 첨부되지 않았고 저장소에도 없다. 따라서
실제 page PNG·PDF 비교를 수행하거나 대표 visual asset을 만들 수 없다. 이는 visual sweep 통과가 아니라
재현 자료 부재다. 새 unit test는 모든 Canvas 조립점과 SVG/Skia helper의 순서를 고정하며, 원본이 제공되면
독립 시각 대조를 추가해야 한다.

## 최종 권고

#3310과 분리 merge하지 말고 통합 branch의 새 PR로 준비한다. 최신 통합 head의 full CI와 작업지시자
push·PR 생성 승인이 충족되면 merge를 권고한다. merge 뒤 #3318 원 PR과 #3314의 close 상태를 확인하고
동일 내용의 contributor comment를 중복 게시하지 않는다.
