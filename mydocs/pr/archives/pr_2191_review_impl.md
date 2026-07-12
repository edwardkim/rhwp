# PR #2191 검토 실행 기록

## 검토 대상

- PR: #2191
- contributor 커밋: `5634bf13d5377c71bceced8463ae30a624f833e8`
- 메인터너 보완 커밋: `3120f8c18e4d427f297205dc5a99774bdb6aa369`
- merge commit: `223f34f7e1a2eec18d901d07f853ab11c42bb389`
- 관련 추적: #2188, #536

## Stage 1 - 원 변경 검토 완료

1. `displayText`/`displayPositions` 선택, printable ASCII direct glyph gate, glyph ID `0` 거부와
   CanvasKit native object cleanup을 확인했다.
2. `U+F012B` JavaScript escape가 실제 U+F012B가 아니라 U+F012 + `B`가 되는 테스트 오류를 확인했다.
3. 비-ASCII 첨자 텍스트가 CanvasKit `drawText`로 내려가면 simple-text rendering이 되어 shaping을
   수행하지 않는다는 것을 확인했다.

## Stage 2 - 메인터너 보완 완료

1. #2191을 최신 `devel` 위에 체리픽해 검증한 뒤, 보완 커밋만 contributor head 위로 재배치했다.
2. 기본 폰트 bytes에서 `FontMgr`와 family를 확보하고 dispose 시 해제하도록 보강했다.
3. 비-ASCII superscript/subscript는 `ParagraphBuilder`/`drawParagraph`로 렌더하고, 실제
   `U+F012B -> (인)` projection과 cleanup을 계약 테스트에 추가했다.
4. contributor 원 커밋은 rewrite하지 않았으며, 보완 커밋은 별도 commit으로 유지했다.

## Stage 3 - 코드 PR merge 완료

1. Rust policy focused test, Studio unit/contract test, WASM build, Studio production build,
   CanvasKit 실번들 shaping probe, `git diff --check`를 확인했다.
2. reviewer `jangster77`를 지정하고, 보완 사유와 검증을 PR 코멘트로 게시했다.
3. 최신 head의 CI, CodeQL, Render Diff 성공을 확인한 뒤 #2191을 squash merge했다.

## Stage 4 - 옵션 2 운영 기록

1. 이 archive review 문서와 오늘할일만 docs-only fast-pass PR로 반영한다.
2. 후속 문서 PR merge 후 `upstream/devel`을 다시 동기화한다.
3. #536 open 유지와 contributor 감사 코멘트를 확인하고, 로컬 review/docs 브랜치를 정리한다.
