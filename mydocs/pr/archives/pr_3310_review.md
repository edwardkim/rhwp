# PR #3310 검토 기록

## 라우팅

```text
base route: collaborator_external_pr
modifiers: intake_and_review, local_validation, multi_pr_update_branch,
  visual_fixture_evidence
loaded documents: pr_review_workflow.md, pr_review/README.md,
  collaborator_external_pr.md, intake_and_review.md, local_validation.md,
  multi_pr_update_branch.md, visual_fixture_evidence.md
current head: 작성 시점 참고값 dc2f505ae5cbc468b26e692153b63b1ea596e4d4
```

## PR metadata

| 항목 | 작성 시점 참고값 |
| --- | --- |
| PR | [#3310](https://github.com/edwardkim/rhwp/pull/3310) |
| 제목 | `fix(#3300): skia 폰트 조달을 custom/번들로 분리` |
| 작성자 / base | `planet6897` / `devel` |
| 원 head / commit | `fix/3300-skia-custom-font-dirs` / `dc2f505ae5cbc468b26e692153b63b1ea596e4d4` |
| 관련 이슈 | [#3300](https://github.com/edwardkim/rhwp/issues/3300) |
| 규모 | 3 files, +99/-8, 1 commit |
| 원 PR 상태 | `MERGEABLE`, `BEHIND`; maintainer 수정 허용, reviewer `jangster77` 요청 완료 |
| 통합 branch | `review/planet6897-font-20260726` (`upstream/devel` `61b13fad4` 기준) |

위 상태와 SHA는 작성 시점 참고값이다. 원 PR은 최신 `devel`보다 뒤에 있으므로 직접 merge하지 않고
#3318과 함께 최신 devel 위 누적 검토한다.

## 변경과 누적 적용

- 원 commit `dc2f505`를 통합 branch에 `2c640c46`으로 충돌 없이 cherry-pick했다.
- `custom_font_dirs()`를 호출자 지정·환경변수로 한정하고, `bundled_font_dirs()`의
  `ttfs/opensource`를 별도 typeface map으로 적재한다. 따라서 시스템 family의 스타일 매칭은
  `FontMgr::match_family_style`에 남고, 번들은 custom·system 뒤 최후 폴백이 된다.
- 통합 검토에서 form caption 경로가 분리한 bundle map을 보지 않는 누락을 발견했다. 보정 commit
  `678494aa0`은 form 후보 끝에 bundled `Noto Sans KR ExtraLight`와 `Noto Sans KR`를 넣고,
  custom → system → bundled → legacy 순서를 유지한다.

원 contributor head는 rewrite·push하지 않는다. 보정은 통합 branch에만 있다.

## 검증

- `custom_font_dirs_excludes_system_and_bundled`, `bundled_font_dirs_is_repo_asset_only`: 통과
- `issue_3300_form_family_chain_includes_bundled_noto_fallbacks`: Native Skia에서 통과
- `cargo test --profile release-test --features native-skia skia --lib`: 57 passed
- `issue_2225_missing_picture_placeholder`: 2 passed
- `render_p37_direct_pdf_export`: 4 passed
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 전수 실행 완료
  (IR field sweep 포함; 신규·이동 fixture가 없으므로 baseline TSV 변경 없음)
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `git diff --check`: 통과
- WASM build는 작업지시자가 수동 실행하는 범위라 이 review에서 재실행하지 않았다.

## 시각·fixture 판정

#3300 이슈와 PR에는 재현 문서 `156467716`의 HWP/HWPX 원본·기준 PDF가 첨부되지 않았고, 저장소에도
동일 이름의 fixture가 없다. 따라서 원본 대 기준 PDF의 독립 visual sweep·대표 PNG는 만들 수 없다.
이는 실제 sweep 통과로 대체하지 않는다. Skia typeface 우선순위와 form fallback은 위 Native Skia 단위
검증으로 확인했고, 재현 파일이 제공되면 PDF/PNG 대조를 후속 검증으로 수행한다.

## 최종 권고

[#3318](https://github.com/edwardkim/rhwp/pull/3318)과 함께 통합 branch의 새 `devel` 대상 PR로
준비하는 것을 권고한다. 최신 통합 head의 full CI와 작업지시자 push·PR 생성 승인이 선행 조건이다.
통합 PR merge 뒤에는 #3310 원 PR close와 #3300 close 상태를 확인하고, contributor에게 통합 결과를
한 번만 알린다.
