# Task M100 #6107 — 5단계 PR 리뷰 보정 보고서

- **이슈**: [#6107](https://github.com/edwardkim/rhwp/issues/6107)
- **PR**: [#6116](https://github.com/edwardkim/rhwp/pull/6116)
- **검토**: [pullrequestreview-5028382095](https://github.com/edwardkim/rhwp/pull/6116#pullrequestreview-5028382095)
- **보정 code candidate**: `f16b1fed8`
- **완료일**: 2026-08-26 KST

## 판정

리뷰 10건을 실제 결함 6건, 부분 수용 2건, 의도된 계약 유지 2건으로 분류했다. 사용자와 한글 2024 대조로
확정한 눈금자 focus와 가로 이동 계약은 되돌리지 않고, 안전성·가시성·선택 lifecycle·행위 테스트의 좁은
보정만 적용했다.

| 리뷰 항목 | 판정 | 보정 |
| --- | --- | --- |
| focus 페이지가 화면 밖이면 눈금자를 가시 페이지로 재지정 | 미반영 | 마지막 클릭·편집 focus의 물리 좌표에 남는 계약을 유지한다. |
| 가로 PageUp/PageDown에 Y overflow 혼합 | 미반영 | 가로 이동 모드의 키 이동은 X축 전용으로 유지한다. |
| 세로 이동의 여러 쪽 배치에서 X축 가시성 누락 | 반영 | 모든 이동 모드에서 페이지와 viewport의 X/Y 교차를 함께 검사한다. |
| source 문자열 정규식 테스트 | 반영 | `VirtualScroll`·resolver를 실제 실행하는 행위 테스트로 교체한다. |
| WASM/layout page count 불일치 | 반영 | 두 page count의 최솟값으로 눈금자 후보와 drag 페이지를 제한한다. |
| 활성 페이지·focus 이벤트 중복과 편집 문맥 | 부분 반영 | 서로 다른 의미의 이벤트는 유지하고, 문단 핀 문맥은 마지막 focus만 사용한다. |
| 다중 그림 선택의 render/focus 페이지 혼용 | 반영 | 기존 마지막 bbox 렌더 기준과 첫 유효 bbox 편집 focus를 분리한다. |
| 가로 페이지 경계 탐색 비용 | 부분 반영 | 캐시된 전체 폭 조회를 loop 밖으로 이동했다. binary search는 계측 근거가 없어 보류한다. |
| PageUp 테스트의 약한·항진 assertion | 반영 | 실제 `scrollX` 변화량과 `deltaX` 일치, `scrollY === 0`을 검증한다. |
| 그림·표 선택 해제 뒤 stale editing page | 반영 | clear 경로와 선택 해제 이벤트에서 `editing-page-changed(null)`을 보장한다. |

## 구현 보정

- `VirtualScroll.getVisiblePages()`를 이동 방향과 무관한 2D 교차 판정으로 통일했다.
- CanvasView의 viewport fallback도 항상 viewport 중심의 X/Y에서 `getPageAtPoint()`를 사용한다.
- 눈금자 페이지는 문서 page count와 확정 레이아웃 page count에 모두 존재해야 한다.
- 개체 선택의 overlay 페이지와 편집 focus 페이지를 별도 helper로 계산하고, 선택 해제 시 stale focus를
  지운다.
- source 문자열을 읽는 눈금자 테스트를 삭제하고, 다중 열·단일 열·page count 불일치·개체 선택 lifecycle을
  실제 함수 실행으로 검증한다.

## 검증

```text
$ node --test rhwp-studio/tests/active-page.test.ts \
    rhwp-studio/tests/active-page-integration.test.ts \
    rhwp-studio/tests/object-selection-page.test.ts \
    rhwp-studio/tests/page-scroll-step.test.ts \
    rhwp-studio/tests/virtual-scroll-page-arrangement.test.ts
tests 36, pass 36, fail 0

$ npx --prefix rhwp-studio tsc -p rhwp-studio/tsconfig.json --noEmit
exit 0

$ npm --prefix rhwp-studio test
tests 1152, pass 1151, fail 0, skipped 1

$ npm --prefix rhwp-studio run build
230 modules transformed, build success

$ CHROME_PATH=... VITE_URL=http://127.0.0.1:7700 \
    npm --prefix rhwp-studio run e2e:page-key-scroll
PASS: 6쪽 문서 TC1~TC7 전체 통과

$ node scripts/rust-test-suite-manifest.mjs --prepare
32 harnesses, 9 exceptions 생성·확인 완료

$ cargo fmt --all && cargo fmt --all -- --check
exit 0

$ git diff --check
exit 0
```

첫 E2E 시도는 Chrome 실행 경로 미지정, 두 번째는 Vite 서버 미실행, 세 번째는 worktree 밖 WASM symlink가
Vite allow list를 벗어나 중단됐다. 설치된 Chrome 경로를 지정하고 worktree 안에 검증용 WASM 산출물을
복사해 서버를 다시 띄운 최종 실행은 TC1~TC7이 모두 통과했다. 검증용 `pkg/`, generated suite와 E2E
산출물은 PR source에 포함하지 않는다.

## 원격 처리와 남은 게이트

보정 commit과 이 문서를 PR head에 push하고 10개 리뷰 스레드에 반영·부분 반영·미반영 판정과 근거를
각각 답변한 뒤 모두 resolve했다. 최신 head의 CI는 코드 변경을 포함하므로 Frontend package gate를
포함한 일반 CI로 판정하며, merge는 별도 승인 전 수행하지 않는다.
