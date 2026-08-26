---
kind: pr_review
status: active
pr: 4766
issue: 4765
last_verified: 2026-08-14
---

# PR #4766 검토: Vite native config loader 호환성 개선

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#4766](https://github.com/edwardkim/rhwp/pull/4766) |
| 관련 이슈 | [#4765](https://github.com/edwardkim/rhwp/issues/4765) |
| 작성자 | `jangster77` |
| base / head | `devel` / `task_m100_4765` |
| code candidate | `724664ed3e7201dbf9736ae52b1a00093fbb9a0d` |
| 작성 시점 merge 상태 | `MERGEABLE`, `CLEAN` |
| 규모 | 2 files, +36 / -11 |

base route: collaborator_self_merge
modifiers: intake_and_review.md, local_validation.md
loaded documents: pr_review_workflow.md, pr_review/README.md, collaborator_self_merge.md, intake_and_review.md, local_validation.md
current head: `724664ed3e7201dbf9736ae52b1a00093fbb9a0d`

## 변경 범위와 판정

- `rhwp-studio/vite.config.ts`에서 설정 파일 기준 디렉터리를 `import.meta.dirname`으로 한 번 계산한다.
- `package.json`, WASM, sample, npm 경로는 모두 같은 설정 파일 상대 위치를 계속 사용한다.
- Canvas 렌더러, 문서 레이아웃, 런타임 WASM 계약은 바꾸지 않으므로 별도 시각 fixture 증적은 필요하지 않다.
- Render Diff CI는 변경 분류 후 canvas/PDF 비교와 CanvasKit readiness gate를 실제로 실행해 성공했다.

## 검증 결과

로컬 Node.js 24 환경에서 다음을 완료했다.

- `(cd rhwp-studio && npx tsc --noEmit)` 통과
- `npm --prefix rhwp-studio test` 통과: 922 passed, 0 failed, 1 skipped
- `cd rhwp-studio && npx vite --configLoader native --host 127.0.0.1 --port 7701 --clearScreen false` 정상 기동
  - Vite native config-loader의 `__dirname` 경고는 재현되지 않았다.
  - Node/Vite 의존 도구 체인의 WASI experimental warning은 남았으며, 이번 설정 경고와 별개다.

로컬에는 Node.js 22가 설치되어 있지 않아, CI의 Node.js 22 환경은 GitHub Actions 결과로 확인했다.

- [CI run 31789409561](https://github.com/edwardkim/rhwp/actions/runs/31789409561): Frontend package gates와 Build & Test aggregate 성공
- [CodeQL run 31789409187](https://github.com/edwardkim/rhwp/actions/runs/31789409187): JavaScript/TypeScript, Python, Rust 분석 성공
- [Render Diff run 31789409185](https://github.com/edwardkim/rhwp/actions/runs/31789409185): canvas/PDF visual diff와 CanvasKit readiness gate 성공

## 결론

발견한 차단 결함은 없다. 이 문서와 오늘할일을 포함한 trailing head의 GitHub Actions가 성공하고 merge 직전에 최신 head·mergeability를 재확인한 뒤 squash merge한다. `Closes #4765`의 자동 종료 상태와 branch 정리는 merge 후속 절차에서 확인한다.
