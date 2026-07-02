# PR #1656 처리 계획 — CanvasKit replay diagnostics 정합

## 대상

- PR: #1656
- 작성자: @seo-rii
- 관련 이슈: #536
- 문서 작성 시점 PR head: `f7632d306b6df86ad2407b9f920874875b963af4`
- merge commit: `220923692bdb60b8c9c3537ac3a07321eb58c88e`
- 처리 결과: GitHub Actions 통과 후 admin merge 완료

## 커밋

1. `780b00d050047b4c82c62b0a8a848aae2712a9cd`
   - `fix(render): align CanvasKit replay diagnostics`
   - 실제 기능 변경 커밋
2. `0e18f1f54af0c247608763746ae57767e8c1104f`
   - `Merge branch 'devel' into render-p31`
   - Draft 해제 전 base 동기화 merge
3. `f7632d306b6df86ad2407b9f920874875b963af4`
   - `Merge branch 'devel' into render-p31`
   - 최신 `devel` 동기화 merge

## 검토 단계

### Stage 1. PR 메타 확인

- base branch: `devel`
- draft: false (작성 시점 참고값)
- maintainerCanModify: true
- mergeable: `MERGEABLE` (작성 시점 참고값)
- mergeStateStatus: `CLEAN`
- 규모: 4 files, +180/-25

이 PR은 이전에 Draft 상태라 정식 검토 대상이 아니었고, 해당 안내 코멘트를 남긴 바 있다. 현재는 Draft가 해제되어
정식 리뷰 대상으로 전환됐다.

### Stage 2. 변경 내용 검토

완료.

- 소스 기준 대조 완료: Rust policy의 direct/detail 분류와 Studio CanvasKit runtime branch를 변경부 기준으로 비교했다.
- `TextRun` rotation은 Studio `renderTextRun`의 `canvas.rotate` direct path에 남고, Rust policy에서는 `rotatedText` transition detail에서 제외된 것을 확인했다.
- vertical/effect 계열은 Rust `text_run_transition_detail`과 Studio `recordTextRunCoverageGaps` 양쪽에서 diagnostics로 남는 것을 확인했다.
- `formObject`/`placeholder`는 Rust replay plan에서 `basicStaticReplay` direct로 분류되고, Studio runtime에는 `renderFormObject`/`renderPlaceholder` branch가 있는 것을 확인했다.
- `renderer-contract`는 switch coverage에 더해 TextRun diagnostics 문자열과 rotation replay 구조를 정적 contract로 guard한다.
- 한계: 브라우저에서 실제 CanvasKit 화면을 수동 시각 검증한 것은 아니며, 이번 확인은 소스 대조 + contract/e2e/build 검증 범위다.

### Stage 3. 로컬 검증

완료.

새 PR review 지침에 따라 cargo 검증 전 `target` 하위 항목을 삭제했다.

- `cargo fmt --all -- --check`: 통과
- `env CARGO_INCREMENTAL=0 cargo test renderer::canvaskit_policy::tests --lib`: 통과
- `node --check e2e/renderer-contract.test.mjs`: 통과
- `npm run e2e:renderer-contract`: 통과
- `npm run build`: 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과
- `git diff --check`: 통과

### Stage 4. GitHub Actions 확인

완료.

최신 PR head `f7632d306b6df86ad2407b9f920874875b963af4` 기준으로 GitHub Actions required checks가 통과했다.

- GitHub Actions required checks 전체 통과
- `MERGEABLE` + `CLEAN`
- 작업지시자 merge 승인

## merge 후 후속 처리 결과

`mydocs/manual/pr_review_workflow.md` 기준으로 처리한다.

1. merge 직전 최신 head SHA와 GitHub Actions 확인 완료
2. PR #1656 admin merge 완료: `220923692bdb60b8c9c3537ac3a07321eb58c88e`
3. `devel` fast-forward sync 완료
4. PR 감사 코멘트 작성 완료: https://github.com/edwardkim/rhwp/pull/1656#issuecomment-4864959740
5. #536은 전체 CanvasKit parity parent 성격이므로 open 유지 확인
6. 리뷰 문서 archive 이동 및 오늘할일 갱신은 별도 문서-only PR로 반영

## 후속 코멘트 요지

- Draft 해제 후 다시 검토했고, 로컬 검증과 CI 통과를 확인했다.
- 이번 PR은 CanvasKit runtime이 이미 직접 처리하는 branch와 replay diagnostics를 맞추는 변경으로 수용했다.
- 남은 CanvasKit parity 항목은 #536 후속 단계에서 계속 추적한다.
