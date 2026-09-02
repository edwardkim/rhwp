# PR #4745 검토 기록 - Studio 개발용 핫패치 경계 보강

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#4745](https://github.com/edwardkim/rhwp/pull/4745) |
| 제목 | `fix(studio): 개발용 핫패치 경계를 보강한다` |
| 작성자 | `jangster77` |
| base | `devel` |
| 초기 code head | `125b49b2c3fb7a966fe4ecd9704b83842b78b0d1` |
| 규모 | 12 files, +192/-30 (문서 포함) |
| 작성 시점 상태 | `MERGEABLE`, CI·CodeQL·Render Diff preflight 진행 중 |

## 라우팅

```text
base route: collaborator_self_merge.md
modifiers: intake_and_review.md, local_validation.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
collaborator_self_merge.md, intake_and_review.md, local_validation.md
current head: 125b49b2c3fb7a966fe4ecd9704b83842b78b0d1 (문서 작성 시점의 code candidate)
```

`gh pr edit --add-reviewer jangster77`는 GitHub CLI가 요청과 무관한 deprecated Projects classic
GraphQL 필드를 조회하면서 실패했다. 외부 reviewer를 추정해 요청하지 않았고, 이 PR은 collaborator
자체 검토 기록과 최신 required check를 근거로 판단한다.

## 관련 이슈와 변경 범위

- `#4635`: 주석 처리된 자동 투명선 구독과 소비자 없는 이벤트 발행을 제거했다. 번들 감시에서는
  개발 전용 식별자가 아닌 `rebuildDerivedState`를 표지 목록에서 제외하고 문서 줄 참조를 고쳤다.
- `#4643`: 적용 요청 결과를 진단 표와 누적 계수가 공유하는 상수로 만들었다. 개발용 dynamic import와
  devtools 연결 실패는 경고 뒤 일반 WASM 초기화를 계속하며, CanvasView 생성 순서가 틀리면 원인을
  개발 콘솔에 남긴다.
- 렌더러, 페이지네이션, HWP/HWPX fixture, WASM 공개 ABI와 CI workflow는 변경하지 않았다.

## 로컬 검증

- `npm --prefix rhwp-studio test`를 실행해 922/922건 통과했다.
- `npm --prefix rhwp-studio run build`를 실행해 TypeScript 검사와 Vite production build를 통과했다.
- `node --test scripts/frontend-studio-dist.test.mjs`를 실행해 4/4건 통과했다.
- `npm --prefix rhwp-studio run dev -- --host 0.0.0.0 --port 7702`를 foreground로 기동하고,
  `http://127.0.0.1:7702/`와 `/src/main.ts`가 모두 HTTP 200인 것을 확인한 뒤 종료했다.
- `git diff --check`를 통과했다.

Studio 개발용 런타임·도구·문서만 변경했으므로 Cargo 전체 회귀, Native Skia, WASM 재빌드와
PDF/SVG 시각 대조는 적용 대상이 아니다.

## 권고

로컬 검증상 차단 이슈는 없다. 문서 trailing commit이 포함된 최신 PR head에서 GitHub Actions,
CodeQL, Render Diff와 mergeability를 다시 확인하고, 작업지시자 승인 후에만 merge한다.
