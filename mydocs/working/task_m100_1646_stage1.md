# Task #1646 Stage 1 완료 보고서 — README 확장 스토어 배지 추가

## 1. 작업 개요

- 이슈: <https://github.com/edwardkim/rhwp/issues/1646>
- 브랜치: `local/task1646`
- 기준 브랜치: `upstream/devel`
- 작업일: 2026-06-29

루트 `README.md` 상단에 브라우저 확장 스토어 배지 줄을 추가했다. 기존 CI/Demo/npm/VS Code/License/Rust/WebAssembly
배지 줄은 수정하지 않고, 바로 아래 별도 `<p align="center">` 문단으로 Chrome Web Store, Microsoft Edge Add-ons,
Firefox Add-ons 배지를 배치했다.

## 2. 최신 반영

구현 전 `git fetch upstream devel` 후 `local/task1646`를 `upstream/devel` 최신으로 fast-forward 했다.

- 이전 HEAD: `5bb80cd5`
- 최신 HEAD: `6d113a5a`

fast-forward 중 upstream에 `mydocs/orders/20260629.md`가 새로 추가되어 로컬 할일 문서와 경로가 충돌했다.
로컬 #1646 행을 보존한 뒤 upstream의 #1521 행이 포함된 최신 파일에 #1646 행을 다시 병합했다.

## 3. 구현 내용

수정 파일:

- `README.md`
- `mydocs/orders/20260629.md`

추가한 README 배지:

- Chrome Web Store: `chrome-web-store/v` 동적 버전 배지, `googlechrome` 로고 사용
- Edge Add-ons: 정적 `Edge Add-ons | Store` 배지
- Firefox Add-ons: `amo/v` 동적 버전 배지, `firefoxbrowser` 로고 사용

`README_EN.md`는 상단 구조만 확인했고 이번 이슈 범위에 따라 수정하지 않았다.

## 4. 검증

실행한 확인:

- `git diff -- README.md`
  - README 변경은 기존 배지 문단 아래 새 확장 스토어 배지 문단 6줄 추가에 한정됨을 확인
- `git diff --check`
  - 공백 오류 없음
- `rg -n "chromewebstore|microsoftedge|addons.mozilla.org|chrome-web-store/v|Edge%20Add--ons|amo/v" README.md`
  - Chrome/Edge/Firefox 링크와 Shields.io 배지 URL이 `README.md`에 존재함을 확인
- `sed -n '1,45p' README_EN.md`
  - 영어 README 상단 배지 구조 확인, 변경 없음

## 5. 테스트 생략 사유

이번 변경은 Markdown/HTML 배지 문단 추가와 작업 문서 갱신만 포함한다. Rust 코드, WASM 코드, 확장 manifest,
패키지 설정을 변경하지 않았으므로 cargo 빌드/테스트는 실행하지 않았다.

## 6. 남은 작업

- 최종 보고서 작성
- 필요 시 작업지시자 승인 후 커밋
