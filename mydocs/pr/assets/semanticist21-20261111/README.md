# semanticist21 #8913·#8915 검증 원장

검증일 2026-11-11 KST. 원 head 두 개를 보존한 누적 source는
`18623a34e8a6bea48d2908ee85c34ae786f4feab`, base는
`e118de8abadd2dffb2798036b9ffd381b96ed7d5`다. 추가 제품 보정은 없다.
원 PR의 CI와 current-base 로컬 실행을 구분한다.
작업지시자 승인 뒤의 [병합 직전 재확인](receipts/approved-premerge-snapshot.json),
[#8915](receipts/pr8915-immediate-premerge.json)·[#8913](receipts/pr8913-immediate-premerge.json)의
정확한 CI/정책 상태·merge tree와 [확정 병합](receipts/merged-prs.json)을 추가했다.

## 기록과 판정

- [집계](validation-summary.json), [입력 생성·선택 범위](inputs.json).
- [원 head CI·job](receipts/original-head-ci.json), [접수](receipts/intake-minimal.json),
  [reviewer 지정](receipts/reviewer-assignment.json), [개별 병합 시뮬레이션](receipts/merge-simulations.json).
- [최신 원격 재확인](receipts/final-remote-check.json)은 작성 시점 참고값이다. merge 직전 재조회한다.
- [실행 명령·종료값](receipts/runtime-validation.json), [최소 글꼴 실제 목록](logs/visible-fonts.log),
  [focused Rust 로그](logs/selection-focused.log).
- [이번 소스의 dev WASM producer](receipts/fresh-wasm-producer.json), [빌드 로그](logs/wasm-dev.log).
  최적화된 release producer가 아니며 API/UI 동작 검증에만 사용했다.
- [WASM API 전후 결과](receipts/selection-wasm-probe.json), [로그](logs/selection-wasm-probe.log).
- [Studio 실제 API/DOM](studio/selection-overlay.json), [실행 로그](logs/selection-studio-browser.log),
  [400문단·10쪽 기존 대조군 로그](logs/existing-selection-layout.log).
- [확장의 고정 엔진 producer](receipts/extension-wasm-producer.json),
  [실제 확장 전후 실행](receipts/extension-browser-validation.json),
  [기준 예상 실패](logs/extension-smoke-base-negative.log), [후보 성공](logs/extension-smoke-candidate.log),
  [Node 100건](logs/chrome-node.log), [build](logs/chrome-build.log).

검증 영수증의 `/home/edward/.../output/` 절대 경로는 실행 당시 로그 위치다. 종료 정리 뒤에도
실제 결과를 읽을 수 있도록 필요한 로그·JSON·PNG·SVG는 이 경로에 별도 보존했다.
텍스트 로그 사본의 줄 끝 공백만 정규화했으며 [원문/보존본 해시](receipts/preserved-log-hashes.json)를 남겼다.
진단용 임시 스크립트나 생성된 WASM 바이너리를 제품 소스로 추가하지 않았다.

## 재현 명령과 범위

```sh
node scripts/rust-test-suite-manifest.mjs --prepare
# runtime-validation.json에 기록한 bwrap 최소 글꼴 환경 안에서 실행
node scripts/run-rust-test.mjs body_selection_tree_reuse -- \
  --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --test-threads 6
scripts/wasm-pack-locked.sh --target web --out-dir pkg --dev
node --test rhwp-chrome/*.test.mjs rhwp-chrome/sw/*.test.mjs \
  rhwp-chrome/e2e/{failure-diagnostics,page-budget,png-integrity,report-browser,report,run,tab-monitor}.test.mjs
node rhwp-chrome/build.mjs
node rhwp-chrome/e2e/extension-smoke.test.mjs
# fresh WASM을 공급한 rhwp-studio에서 실행
VITE_PORT=7771 node e2e/run-with-vite.mjs -- \
  node e2e/selection-layout-issue8103.test.mjs --mode=headless --renderer=canvas2d
```

Chrome smoke는 fresh dev WASM 공급 **전** 동일 엔진으로 고정한 실제 dist에서 실행했다.
소유한 임시 dist의 content script만 base/candidate로 바꿔 비교하고 복원했다. 임시 브라우저·서버는
각 실행이 종료했으며 기존 사용자의 실행 환경은 중지하지 않았다.

새 선택 probe는 [inputs.json](inputs.json)의 본문 5문단·셀 3문단·자동 줄바꿈 입력을 생성했다.
동일 텍스트와 선택 범위에서 공개 WASM API 사각형을 얻고 실제 Studio의 CursorState·
InputHandler·WasmBridge·SelectionRenderer로 투영했다. 본문 zoom 1·0.66, 셀 zoom 0.66을
확인했다. 기존 자동화는 400문단·10쪽의 선택 유지·Undo/Redo·Backspace·스크롤·비동기 복원을 보호한다.

## 선택 화면과 문서 출력 대조

- 실제 후보 [본문](studio/body-zoom1.png), [축소](studio/body-zoom0.66.png), [셀](studio/cell.png).
- 독립 Hancom GUI [본문](../issue_8908_selection_paragraph_mark/hancom-paragraphs.png),
  [셀](../issue_8908_selection_paragraph_mark/hancom-cell.png),
  [자동 줄바꿈](../issue_8908_selection_paragraph_mark/hancom-wrapped.png).
- 선택 전후 문서 SVG 불변과 producer 사이 동일성:
  [기준 본문](api-svg/base/body.svg), [후보 본문](api-svg/candidate/body.svg),
  [기준 셀](api-svg/base/cell.svg), [후보 셀](api-svg/candidate/cell.svg).

선택 overlay는 인쇄되지 않는다. Print PDF 실루엣·Native Skia·편람/정본 보호 TSV를 적용하지
않으며 화면 전체 RGB 동일성도 주장하지 않는다. 이번 변경은 문서 배치·저장에 영향을 주지 않는다.

## 병합 후 메타데이터

[실제 갱신 결과](receipts/duration-refresh-results.json)를 확인했다. 두 workflow는 성공했으며
#8915는 Rust worker가 없어 `no-verified-pr-duration-measurements`로 갱신을 보류했다.
#8913은 검증된 PR 실측을 읽고 duration policy를 갱신했다. 병합 후 제품 CI를 새로 실행하지 않았다.
