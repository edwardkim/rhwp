# PR #1738 리뷰 — #1735 방점(U+302E/U+302F) dotted-circle 아티팩트 제거

- PR: #1738 `방점(U+302E/U+302F) 렌더 정합: dotted-circle 아티팩트 제거 (Refs #1735)`
- 작성자: @johndoekim
- 기준 브랜치: `devel`
- PR head: `00dc17ef962b39e1248297e8c80e8dc7ddeb2da1` (문서 작성 시점 참고값)
- 규모: 10 files, +258/-2
- 관련 이슈: #1735
- 검토 중 상태: `MERGEABLE`, `BLOCKED` (GitHub required check 진행 중)
- 최종 처리: GitHub Actions 통과 후 admin merge 완료
- merge commit: `18720d08a63da6fb396a9ad1effbd01833cca300`

## 변경 요약

한컴 기준 PDF에서는 선두 방점 `〮`(U+302E)이 독립된 가운데 점처럼 보이지만, rhwp 렌더링에서는 유니코드
combining mark 특성 때문에 dotted-circle(U+25CC) placeholder가 함께 보이는 문제를 고친다.

핵심 변경:

- `src/renderer/composer.rs`
  - 렌더 전용 `expand_pua_display_text` 경로에 `tone_mark_display` 추가
  - U+302E → U+00B7, U+302F → U+205A로 표시용 치환
- `src/renderer/layout/text_measurement.rs`
  - U+302E/U+302F를 `is_narrow_punctuation`에 추가해 narrow advance로 측정
- `src/renderer/composer/tests.rs`
  - 렌더 치환 단위 테스트 추가
- `samples/unicode/`
  - 재현 HWP와 한컴 기준 PDF 추가
- `mydocs/plans`, `mydocs/working`, `mydocs/report`
  - #1735 작업 계획/단계/결과 기록 추가

## 로컬 merge 검증

`upstream/devel` 기준 merge 시뮬레이션 결과 충돌 없음.

```bash
git checkout -B local/pr1738-merge-test upstream/devel
git merge local/pr1738 --no-commit --no-ff
```

결과: automatic merge 성공.

## 로컬 검증

새 PR review 지침에 따라 cargo 검증 전 `target` 하위 항목을 삭제한 뒤 수행했다.

- `CARGO_INCREMENTAL=0 cargo test --lib`: 2046 passed, 0 failed, 7 ignored, real 162.48s
- `cargo fmt --check`: 통과
- `git diff --check`: 통과
- `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과, real 28.65s
- `CARGO_INCREMENTAL=0 cargo test --test svg_snapshot`: 8 passed, 0 failed, real 17.51s
- `CARGO_INCREMENTAL=0 cargo build --bin rhwp`: 통과, real 19.01s
- `target/debug/rhwp info samples/unicode/각 항목에 명시되어 있는_유니코드.hwp`: 1페이지, 문단 2개
- `target/debug/rhwp dump-pages samples/unicode/각 항목에 명시되어 있는_유니코드.hwp`: 1페이지

## 시각 검증

검증 산출물은 repo 밖 `/tmp/rhwp-pr1738/`에 생성했다.

```bash
target/debug/rhwp export-svg "samples/unicode/각 항목에 명시되어 있는_유니코드.hwp" -o /tmp/rhwp-pr1738/svg
target/debug/rhwp export-pdf "samples/unicode/각 항목에 명시되어 있는_유니코드.hwp" -o /tmp/rhwp-pr1738/rhwp_unicode.pdf
pdftoppm -png -f 1 -singlefile "samples/unicode/각 항목에 명시되어 있는_유니코드.pdf" /tmp/rhwp-pr1738/png/oracle
pdftoppm -png -f 1 -singlefile /tmp/rhwp-pr1738/rhwp_unicode.pdf /tmp/rhwp-pr1738/png/rhwp
```

확인 결과:

- SVG에 U+302E, U+302F, U+25CC 없음
- 선두 방점은 `<circle>` 1개로 렌더됨(Task #257 middle-dot vector 경로)
- 기준 PDF와 rhwp 생성 PDF 모두 선두에 독립 점이 보이고 dotted-circle 아티팩트는 보이지 않음
- `pdftotext` 기준 PDF는 원문 U+302E를 보존하지만, rhwp PDF는 vector circle이라 텍스트로 추출되지 않는다. 이번
  결함은 렌더 아티팩트 문제이므로 PNG 시각 확인을 기준으로 판단한다.

## GitHub Actions

merge 직전 PR head `00dc17ef962b39e1248297e8c80e8dc7ddeb2da1` 기준:

- CI preflight: success
- CodeQL preflight: success
- Render Diff preflight: success
- Canvas visual diff: success
- WASM Build: skipped
- Analyze (javascript-typescript): success
- Analyze (python): success
- Analyze (rust): success, 8m21s
- Build & Test: success, 19m9s
- CodeQL: success

최종 merge 직전 `MERGEABLE` + `CLEAN` 상태를 확인했다.

## 이전 SHA run 정리

PR branch update 이후 이전 SHA `00bc0428173c9f413171f3b070c923089320d050`의 run이 남아 있었다.

- Render Diff: 이미 cancelled
- CI: `force-cancel` API 적용 후 `cancelled`
- CodeQL: `force-cancel` API 적용 후 `cancelled`
- 최종 결과: 이전 SHA의 CI/CodeQL/Render Diff 모두 `completed` + `cancelled`

이 경험을 `mydocs/manual/pr_review_workflow.md`에 반영했다. 앞으로 update branch 이후 이전 SHA run 정리는
처음부터 force-cancel API를 사용한다.

후속 문서-only PR에는 이 리뷰 문서, 오늘할일 갱신, `mydocs/manual/pr_review_workflow.md`의 force-cancel 절차
명문화 변경을 함께 포함한다.

## 리뷰 결과

Blocking finding 없음.

렌더 치환은 기존 PUA 표시 확장 경로에만 들어가고, 측정은 U+302E/U+302F 2자만 narrow punctuation으로
추가해 범위가 좁다. 로컬 테스트와 실제 SVG/PDF 시각 확인에서도 dotted-circle 제거가 재현됐다.

## 비차단 확인 사항

- `src/renderer/composer.rs`의 새 주석에는 "측정 폭 정합은 text_measurement 의 전각 분류로 맞춘다"는 표현이
  남아 있지만, 최종 구현은 `is_narrow_punctuation` 기반이다. 동작에는 영향이 없지만 merge 전 또는 후속
  정리에서 "narrow 분류"로 문구를 맞추는 편이 좋다.
- U+302F는 샘플 부재로 기준 PDF 시각 검증은 되지 않았다. PR 본문도 best-effort 한계로 명시하고 있다.

## merge 후 후속 처리

- PR #1738 admin merge 완료: `18720d08a63da6fb396a9ad1effbd01833cca300`
- PR 감사 코멘트: https://github.com/edwardkim/rhwp/pull/1738#issuecomment-4855097960
- #1735 자동 close 실패 확인 후 수동 close 완료
- #1735 close comment: https://github.com/edwardkim/rhwp/issues/1735#issuecomment-4855097451
- `devel` fast-forward sync 완료

## 최종 판단

수용 및 merge 완료.

U+302E dotted-circle 아티팩트는 한컴 기준 PDF와 rhwp 생성 PDF 비교에서 제거된 것으로 확인했다. U+302F는 기준
샘플 부재로 best-effort 한계가 남아 있으나, 구현 범위가 좁고 로컬/CI 검증을 통과했으므로 후속 샘플이 생기면
별도 검증한다.
