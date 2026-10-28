# 완료 보고서 — Task M100 #3126 브라우저 인쇄/PDF 저장 경로 안정화

- Issue: #3126
- 작성일: 2026-07-23
- 브랜치: `codex/3126-print-pdf-ux`
- 상태: **구현·Chrome 검증 및 Draft PR 준비 완료, Windows Edge 수동 검증 대기**

## 1. 완료 내용

직접 PDF bytes 생성은 #2657 결정에 따라 v1 범위에서 보류하고, 브라우저 인쇄 기능을 이용하는
PDF 저장 경로를 발견 가능하고 재현 가능한 사용자 흐름으로 정리했다.

- `PDF로 저장…`
  - 파일 메뉴의 저장 영역에서 별도 PDF 아이콘으로 노출
  - 브라우저 인쇄 대상을 PDF로 선택하는 방법을 실행 전에 안내
  - 확인 뒤 같은 모달에서 페이지별 준비 진행률 표시
  - 준비 완료 뒤 hidden same-origin iframe에서 native print UI 자동 호출
  - 반복 안내를 생략하고 환경 설정에서 다시 켤 수 있는 사용자 설정 제공
- `인쇄`
  - 기존의 별도 미리보기 의미를 same-origin `print.html` 창으로 유지
  - 미리보기에서 사용자가 `인쇄`를 눌렀을 때 native print UI 호출
- 공통 출력 계약
  - 명시적인 `print` render profile 사용
  - 페이지별 named `@page`, SVG id namespace, 폰트 준비와 layout 확정 공유
  - 파일 handle, 파일명, dirty/save 상태 불변
  - `about:blank` 의존 제거

## 2. 주요 구현

- Rust/WASM에 기존 SVG API를 보존하는 opt-in `renderPageSvgWithProfile` 추가
- Studio print pipeline과 same-origin iframe/window surface 분리
- `public/print.html`에 인쇄 surface의 초기 loading 문서 제공
- PDF 안내·진행·오류 모달과 안내 표시 사용자 설정 추가
- 파일 메뉴를 `저장 → 다른 이름으로 저장 → PDF로 저장 → HWP/HWPX 형식 저장` 순서로 정리
- #2524/#2525 fixture 기반 Chrome/CDP 인쇄 회귀 E2E 추가

## 3. 검증 결과

통과:

- Studio/editor 전체 테스트: 516개
- focused 설정·인쇄 계약 테스트: 11개
- production TypeScript/Vite build
- `cargo test --lib`: 2,530 passed / 7 ignored
- `cargo clippy --lib -- -D warnings`
- #2524/#2525 Rust focused regression
- E2E manifest 검사
- Chrome headless #3126 E2E
  - embedded font와 검색 가능한 SVG/PDF text
  - 7쪽 다중 페이지와 named `@page`
  - PDF 안내 생략 설정의 저장과 재실행
  - PDF iframe과 별도 인쇄 미리보기 surface
  - 실행 전후 문서 저장 상태 불변
- macOS Chrome 사용자 수동 확인
  - 안내, 진행 상태, native print UI 자동 호출
  - 별도 인쇄 미리보기
  - PDF 아이콘과 저장 영역 메뉴 배치

## 4. 호환성과 리스크

- 웹 API로 native print UI의 대상을 PDF로 강제하거나 저장 성공과 취소를 구분할 수 없다.
  따라서 실행 전 남은 단계를 안내하고 `PDF가 저장되었습니다` 같은 성공 메시지는 표시하지 않는다.
- 브라우저 인쇄 결과는 브라우저·OS·printer backend의 영향을 받는다. Studio는 출력용 SVG, CSS,
  폰트와 layout 준비까지만 통제한다.
- 현재 macOS 환경에는 Microsoft Edge가 없어 Windows Edge native dialog와 named `@page` 호환성은
  Draft PR의 merge 전 잔여 게이트로 둔다.

## 5. Windows Edge 잔여 게이트

PR head를 Codespaces 또는 동등한 원격 미리보기로 열어 다음을 확인한다.

1. `PDF로 저장…` 확인 뒤 진행 상태와 native print UI 자동 호출
2. PDF 대상, 7쪽 페이지 수와 혼합 용지 크기·방향
3. 저장 PDF의 텍스트 검색·선택
4. 취소 뒤 파일명과 dirty 상태 불변
5. 안내 생략 설정과 환경 설정 복원
6. 별도 `인쇄` 미리보기의 열기·인쇄·닫기

Edge에서 iframe print 또는 named `@page` 결함이 실증될 때만 same-origin 전용 창 fallback을 검토한다.

## 6. 결론

#3126의 구현과 Chrome 자동·수동 검증은 완료했다. Draft PR에서 CI와 Windows Edge 수동 검증을
마지막 merge gate로 확인한 뒤 Ready 전환과 이슈 close를 진행할 수 있다.
