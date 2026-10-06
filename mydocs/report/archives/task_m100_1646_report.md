# Task M100 #1646 최종 보고서

## 요약

루트 `README.md` 상단에 브라우저 확장 스토어 배지를 추가했다.

기존 CI/Demo/npm/VS Code/License/Rust/WebAssembly 배지 줄은 유지하고, 바로 아래 별도 가운데 정렬 문단에
Chrome Web Store, Microsoft Edge Add-ons, Firefox Add-ons 배지를 배치했다. 이슈 본문에서 확정한 방침대로
Chrome/Firefox는 로고가 포함된 동적 버전 배지를 사용하고, Edge는 정적 `Edge Add-ons | Store` 배지를 사용했다.

## 최신 반영

구현 전 `upstream/devel`을 최신으로 반영했다.

- 이전 task 브랜치 HEAD: `5bb80cd5`
- 최신 반영 후 HEAD: `6d113a5a`

fast-forward 과정에서 upstream에 `mydocs/orders/20260629.md`가 새로 추가되어 로컬 할일 문서와 경로가 겹쳤다.
upstream의 #1521 행을 유지하고 #1646 행을 다시 병합했다.

## 변경 내용

- `README.md`
  - 기존 상단 배지 문단은 그대로 유지
  - 기존 배지 문단 아래에 새 `<p align="center">` 문단 추가
  - Chrome, Edge, Firefox 확장 스토어 배지 추가

추가된 배지 링크:

- Chrome: `https://chromewebstore.google.com/detail/pgakpjflombjmehnebnbpnalhegaanag`
- Edge: `https://microsoftedge.microsoft.com/addons/detail/rhwp/nfkdfobhmanddlhdbclkpoanbccpigcn`
- Firefox: `https://addons.mozilla.org/firefox/addon/rhwp-free-hwp-editor/`

## 검증

실행한 확인:

- `git diff -- README.md`
  - 변경 범위가 README 상단의 확장 스토어 배지 문단 추가에 한정됨을 확인
- `git diff --check`
  - 공백 오류 없음
- `rg -n "chromewebstore|microsoftedge|addons.mozilla.org|chrome-web-store/v|Edge%20Add--ons|amo/v" README.md`
  - 세 스토어 링크와 세 Shields.io 배지 URL 존재 확인
- `sed -n '1,45p' README_EN.md`
  - 영어 README 상단 구조 확인, 이번 이슈 범위에 따라 변경 없음

## 테스트 생략

이번 변경은 Markdown/HTML 배지 문단 추가와 작업 문서 갱신만 포함한다. Rust 코드, WASM 코드, 브라우저 확장 코드,
manifest, 패키지 설정을 변경하지 않았으므로 cargo 빌드/테스트와 확장 빌드는 실행하지 않았다.

## 변경 범위

소스 변경:

- `README.md`

작업 문서:

- `mydocs/orders/20260629.md`
- `mydocs/plans/task_m100_1646.md`
- `mydocs/plans/task_m100_1646_impl.md`
- `mydocs/working/task_m100_1646_stage1.md`
- `mydocs/report/task_m100_1646_report.md`

## 후속

- 작업지시자 승인에 따라 스테이징, 커밋, draft PR 작성을 진행한다.
- 이슈 #1646 close는 작업지시자 승인 후에만 수행한다.
- `README_EN.md`에도 동일 배지를 노출할지는 별도 지시 또는 후속 이슈에서 결정한다.
