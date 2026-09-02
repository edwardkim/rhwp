# PR #4748 검토 기록 - 개발용 핫패치 경계 정리

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#4748](https://github.com/edwardkim/rhwp/pull/4748) |
| 제목 | `refactor: 개발용 핫패치 경계를 정리` |
| 작성자 | `jangster77` (collaborator) |
| base | `devel` |
| code candidate | `6c51110474d505d2e9eff9353b4a4e2c2d01c4b0` |
| 규모 | 구현 commit 기준 13 files, +307/-183 |
| 작성 시점 참고 상태 | `MERGEABLE`, GitHub Actions·CodeQL·Render Diff 진행 중 |
| 검토 방식 | collaborator self-merge 후보. 동일 collaborator가 작성자이므로 외부 reviewer를 추정해 요청하지 않았고, 작업지시자 승인과 최신 required check를 최종 조건으로 둔다. |

## 라우팅

```text
base route: collaborator_self_merge.md
modifiers: intake_and_review.md, local_validation.md, visual_fixture_evidence.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
collaborator_self_merge.md, intake_and_review.md, local_validation.md,
visual_fixture_evidence.md
current head: 6c51110474d505d2e9eff9353b4a4e2c2d01c4b0 (문서 작성 시점의 code candidate)
```

최신 `upstream/devel` `be7dabdd170117d6022656063b0482f04361bfcb` 위에서 branch를 만들었고,
`git merge-base --is-ancestor upstream/devel HEAD`가 통과했다. 이 review 및 오늘할일 commit은
동일 source branch에 trailing 문서 commit으로 추가한다. merge 직전에는 이 문서를 포함한 최신 PR head의
required check와 mergeability를 다시 확인해야 한다.

## 관련 이슈와 변경 범위

- [#4636](https://github.com/edwardkim/rhwp/issues/4636): 개발용 소켓·감시자·패치 수명 소유를
  `subsecond-runtime.ts`로 옮기고, `main.ts`의 DEV 동적 import에서만 시작하도록 정리했다.
- [#4641](https://github.com/edwardkim/rhwp/issues/4641): `WasmBridge`와 `CanvasView`에서 개발 전용
  상태·메서드·이벤트를 제거했다. 코드 리비전 변경은 현재 CanvasView의 `refreshPages()`를 직접 호출한다.
  production `dist` 아래 중첩 JavaScript chunk도 재귀 검사하도록 번들 계약을 보강했다.
- [#4642](https://github.com/edwardkim/rhwp/issues/4642): Rust 경계 모듈을
  `render_patch_boundary`로 도메인화하고, 공용 `HotFunction` 제네릭 helper를 제거했다. 적용된
  함수 주소는 경계 macro 안에서 `HotFn::current(...).ptr_address()`로 읽으며, 입력 `BoundingBox`도
  즉시 분해·재조립하지 않고 직접 검증·클리핑한다.

WASM의 JavaScript 공개 인자와 문서 포맷, 레이아웃 계산, 페이지네이션 계약은 바꾸지 않았다.

## 렌더 영향과 브라우저 검증

`src/wasm_api.rs`와 Studio Canvas 갱신 경로를 수정했으므로 시각·fixture 보조 경로를 적용했다.
이번 변경은 레이아웃 또는 paint 계산이 아니라 개발 시 코드 교체 뒤의 재도색 소유권을 옮기는 범위다.
따라서 HWP 2020 PDF 기준 대조나 페이지별 visual sweep을 merge 판단에 사용하지 않았다.

대신 headless Chrome에서 기존 차트 E2E를 실행했다. 차트 A/B는 각각 유채색 픽셀 비율
1.824%/2.697%로 비공백이었고, 두 캔버스의 픽셀 차이는 4.71%였다. 동일 문서에서
`refreshPages()`를 다시 호출한 뒤에도 B의 유채색 비율은 2.697%로 유지됐다. 이 결과는 개발용
리비전 갱신이 일반 문서 revision 이벤트 없이 현재 화면을 다시 그리는 경로를 확인한 것이다.
임시 산출물 `output/e2e/issue-1456/`은 이 PR의 레이아웃 fidelity 근거가 아니므로 증적 asset으로
commit하지 않았다.

## 완료한 로컬 검증

| 검증 | 결과 |
| --- | --- |
| `npm --prefix rhwp-studio test -- --runInBand` | 923/923 통과, 약 8.7초 |
| `npm --prefix rhwp-studio run build` | TypeScript 검사와 Vite production build 통과. CanvasKit externalization 및 chunk-size 안내만 발생 |
| `node --test scripts/frontend-studio-dist.test.mjs` | 5/5 통과. 중첩 JavaScript 산출물과 개발용 표지 누출을 확인 |
| `node rhwp-studio/e2e/issue-1456-chart-rerender.test.mjs --mode=headless` | 차트 A/B 비공백·비재사용, 동일 문서 `refreshPages()` 재도색을 통과 |
| `cargo test --profile release-test --target-dir target/pr-review --lib render_patch_boundary -- --nocapture` | 3/3 통과 |
| `cargo test --profile release-test --target-dir target/pr-review --features subsecond-dev --lib render_patch_boundary -- --nocapture` | 4/4 통과 |
| `cargo build --profile release-test --target-dir target/pr-review --target wasm32-unknown-unknown --features subsecond-dev --lib` | 통과 |
| `git diff --check` | 통과 |

`wasm-pack 0.15.0`으로 `--out-dir`을 지정한 development build는 stable Cargo가 아직 지원하지 않는
`--artifact-dir` 인자로 변환되어 Rust 컴파일 전에 중단됐다. 이는 코드 실패가 아니라 현재
wasm-pack/Cargo 조합의 도구 호환성 문제이며, 동등한 wasm32 Cargo build는 통과했다.

## 권고

로컬 검토에서 차단 이슈는 발견하지 못했다. PR 본문의 `Closes #4636`, `Closes #4641`, `Closes #4642`는
merge 후 실제 종료 상태를 다시 확인한다. 이 trailing 문서 commit을 포함한 최신 PR head의 GitHub Actions,
CodeQL, Render Diff, mergeability가 모두 통과하고 작업지시자가 승인할 때만 merge한다.
