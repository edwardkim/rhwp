# #9133 최신 devel 반영 검증

- 새 base: `ae9277dda0ca17be279cba89c4deea74fb75dcae` (#9134·#9135).
- 실제 검증 source: `b57fb3d5ef8ea924e5cd37fed3798ed1b18e54e7`.
- 초기 승인 후보 `e27e724802`의 네 patch가 충돌 없이 동일하게 rebase됐다. [이력 대응·의존성·입력](source-and-inputs.json)의 range-diff와 blob 확인을 참조한다.
- 원장과 로그는 2026-11-16 KST 실제 실행이다. 로그의 줄 끝 공백만 정리했다.

## 최종 실행

[9개 검증 묶음](checks.json)이 모두 exit 0이다. TypeScript 두 경로·[Studio 2,805 PASS/기존 2 SKIP](npm-test.txt)·[production build](build.txt)·[공통 창 66개 조합](dialog-viewport-issue9133.txt)·[제품 정보](about-dialog-issue8905.txt)·[PDF](pdf-import-dialog-issue8411.txt)·[실제 복구](autosave-recovery.txt)·[반응형 2,666 PASS](responsive.txt)를 포함한다.

#9134의 새 CanvasKit 선 패턴·브라우저 종료 검사는 실제 CanvasKit과 로컬/원격 browser 분기를 검사하며 새 전체 검사에 포함됐다. 해당 제품 파일은 base와 후보에서 같다. 기존 devel의 Native/보호 검증을 이번 UI-only source의 새 실행으로 바꾸어 주장하지 않는다.

[WASM 최적화](wasm-build.txt)와 [프로그램 해시](package.json)를 확인했다. 실제 `pkg/`와 Studio public의 JS/WASM이 각각 일치하며 타입 선언과 glue는 초기 빌드와 같고 WASM은 새 Rust 의존성으로 다시 생산됐다.

## 실제 화면과 측정

[후보 66개 측정](after-measurements.json)과 직접 판독용 [편집 용지](after-page-800x360.png)·[환경 설정](after-options-800x360.png)·[글자 모양](after-char-800x360.png)·[그림 속성](after-picture-320x240.png)·[복구 목록](after-recovery-320x240.png)을 보존했다. Chromium 152.0.7946.0의 실제 앱이며 Android Firefox 실기기 결과가 아니다.

[수정 전 66개 측정](before-measurements.json)은 같은 새 package·DOM에 base CSS만 적용한 음성 대조다. [진단 종료 코드](base-checks.json)의 exit 1은 제목/버튼 잘림·복구 목록 높이 0 등 67개 조건의 의도된 RED이며, 최종 9개 검증의 실패가 아니다. [진단 로그](base-dialog-viewport-issue9133.txt)와 후보의 실패 0건을 구분했다. 복구·PDF·확인/취소의 의미와 문서 엔진을 UI geometry 판정으로 대신하지 않는다.
