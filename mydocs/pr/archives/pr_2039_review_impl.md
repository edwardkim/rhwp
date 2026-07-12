# PR #2039 처리 계획

작성일: 2026-07-08
대상 PR: https://github.com/edwardkim/rhwp/pull/2039

## 1. 커밋 구성

| SHA | 작성자 | 내용 |
| --- | --- | --- |
| `383b6012a8173f7e8f45425380aadc96be8a060b` | `lpaiu-cs` | 찾아 바꾸기/모두 바꾸기를 편집 라우터 snapshot 으로 기록 |
| `4ff9aaf74a6fa5f506ae0264a9a37210b1228806` | `jangster77` | `devel` merge commit |
| `7649e2180c6f9b823327081e48d897ad50806301` | `jangster77` | PR #2039 merge commit |

## 2. 처리 단계

1. PR 메타와 관련 이슈 #2037 확인.
2. reviewer `jangster77` 등록.
3. PR head 를 `local/pr2039` 로 fetch.
4. `pr2039-merge-test` 브랜치에서 최신 `upstream/devel` 포함 여부와 충돌 여부 확인.
5. 코드 리뷰 및 로컬 검증 실행.
6. GitHub Actions 완료 확인.
7. 작업지시자 옵션 2 승인 후 원 코드 PR merge.
8. review 문서를 archive 경로로 이동해 docs-only 후속 PR 로 보존.
9. docs-only PR merge 후 #2037 close/comment 및 PR 감사 코멘트 처리.

## 3. 검증 계획 및 결과

완료:

- `npm test` (`rhwp-studio`) — 177 passed.
- `npx tsc --noEmit` (`rhwp-studio`) — 통과.
- `npm run build` (`rhwp-studio`) — 통과.
- `git diff --check upstream/devel...HEAD` — 통과.
- merge simulation — `upstream/devel` 이 PR head 에 이미 포함되어 있어 `Already up to date`.

생략:

- `cargo test --profile release-test --tests` — Rust 소스/테스트 변경 없음.
- `wasm-pack build --target web --out-dir pkg` — WASM/Rust 변경 없음.
- visual sweep / MCP 기준 PDF — 렌더 출력 변경 없음.

대기:

- 없음. 최신 head 기준 GitHub Actions 전체 완료.

## 4. 코멘트 초안

docs-only PR merge 후 게시한다.

```markdown
@lpaiu-cs 감사합니다. 첫 기여 PR로 undo 계약을 정확히 짚어주셨습니다.

로컬에서 다음을 확인했습니다.

- `rhwp-studio` `npm test`: 177 passed
- `rhwp-studio` `npx tsc --noEmit`: passed
- `rhwp-studio` `npm run build`: passed
- `git diff --check`: passed

`find-dialog.ts` 의 `doReplace()` / `doReplaceAll()` 이 `executeOperation({ kind: 'snapshot' })` 경로를 타도록 바뀌어 #1320 편집 라우터 계약에 맞습니다. snapshot 경로에서 `afterEdit()`/`document-changed` 갱신도 수행되므로 라우팅 경로의 직접 emit 제거도 맞다고 봤습니다.

GitHub Actions 최신 head 통과 확인 후 `7649e2180c6f9b823327081e48d897ad50806301` 로 merge 했습니다. 관련 이슈 https://github.com/edwardkim/rhwp/issues/2037 도 후속 코멘트와 함께 정리하겠습니다.
```

## 5. merge 후 후속

- 원 코드 PR merge 완료: `7649e2180c6f9b823327081e48d897ad50806301`.
- `devel` fast-forward sync 완료.
- PR review 문서는 `mydocs/pr/archives/` 로 이동.
- docs-only 후속 PR merge 후 #2037 에 merge commit, 로컬 검증, CI 결과를 후속 코멘트로 기록하고 close 한다.
- docs-only 후속 PR merge 후 PR #2039 에 first-time contributor 감사 코멘트를 남긴다.
- `local/pr2039`, `pr2039-merge-test` 등 로컬 검토 브랜치 정리.
