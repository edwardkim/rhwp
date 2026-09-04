# Task M100 #6107 — 6단계 최초 로딩 focus 보정 보고서

- **이슈**: [#6107](https://github.com/edwardkim/rhwp/issues/6107)
- **PR**: [#6116](https://github.com/edwardkim/rhwp/pull/6116)
- **보정 commit**: `775106d0d`
- **후속 이슈**: [#6149](https://github.com/edwardkim/rhwp/issues/6149)
- **완료일**: 2026-08-26 KST

## 결함과 원인

문서를 처음 연 직후 `InputHandler.activateWithCaretPosition()`은 복원된 캐럿 DOM만 표시하고
`cursor-rect-updated`를 발행하지 않았다. 따라서 CanvasView의 편집 focus는 `null`로 남았고, 눈금자는
첫 쪽을 확정 focus가 아닌 viewport fallback으로만 사용했다. 줌 아웃으로 페이지 배치와 viewport 중심
페이지가 바뀌면 눈금자도 다음 페이지로 이동했다.

## 보정

- 최초 캐럿 표시와 같은 물리 쪽 focus 발행을 `showInitialCaretAndPublishFocus()` 관문으로 묶었다.
- 정상 복원과 오류 복구 경로가 모두 실제 `CursorRect.pageIndex`를 발행한다.
- 첫 쪽을 강제로 지정하지 않으므로 저장된 중간 쪽 캐럿도 그대로 복원한다.
- focus만 확정하며 스크롤·배율·문서 데이터는 변경하지 않는다.

## 검증

```text
$ node --test rhwp-studio/tests/initial-caret-focus.test.ts \
    rhwp-studio/tests/active-page.test.ts \
    rhwp-studio/tests/active-page-integration.test.ts
tests 15, pass 15, fail 0

$ npx --prefix rhwp-studio tsc -p rhwp-studio/tsconfig.json --noEmit
exit 0

$ npm --prefix rhwp-studio test
tests 1155, pass 1154, fail 0, skipped 1

$ npm --prefix rhwp-studio run build
231 modules transformed, build success

$ cargo fmt --all && cargo fmt --all -- --check
exit 0

$ git diff --check
exit 0
```

실제 115쪽 복구 문서를 새 탭에서 열어 페이지를 클릭하지 않은 채 100%에서 10%까지 10%p씩 축소했다.
가로 이동과 세로 이동 모두 상태 표시줄이 `1 / 115`를 유지했고, 가로·세로 눈금자도 최초 1쪽에 남았다.

## 범위 분리

저배율에서 눈금이 뭉치고 페이지 간격이 지나치게 작아지는 현상은 최초 focus 정확성과 독립된 화면 식별성
문제다. #6116에는 포함하지 않고 #6149에서 화면 픽셀 기준 눈금 LOD와 최소 페이지 간격을 함께 다룬다.
