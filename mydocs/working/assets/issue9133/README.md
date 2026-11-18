# #9133 실제 Studio 대화창 검증 증거

- 최종 code head: `80bf4c9e1b01ca6b621a7bc5d49d7fd64a1f7ad7`.
- CSS 기준: `6269cb0f1707a1c5afcae4840ac875f928c4a03d`의 공통·글자 모양·반응형 CSS. 같은 DOM/실제 앱·같은 최적화 WASM으로 전후를 실행했다.
- 측정: 2026-11-16 KST, Chromium `152.0.7946.0` / revision `1660786`, WSL Ubuntu. Android Firefox 실기기 결과가 아니다.
- 검사: `npm --prefix rhwp-studio run e2e:dialog-viewport`. 화면 좌표의 포함·본문 끝·실제 버튼 클릭을 검사하며, 소스 정규식이나 문서 조판 golden으로 판정하지 않는다.

## 전체 측정과 실행

[수정 전 66개 조합](before-measurements.json)의 67개 실패 조건이 [수정 후](after-measurements.json)에는 0건이다. 창 11종 각각 `800×360`, `768×320`, `767×320`, `320×240`, `360×640`, `1280×900`을 실행했다. 데스크톱 11종의 외곽 크기·위치는 전후 모두 같다.

[전체 검사 명령·종료 코드](checks.json)는 최종 code head에서 실행한 9개 묶음 모두 성공이다. [프로그램 해시와 source](package.json), [WASM 최적화 로그](wasm-build.txt), [Studio 전체 검사](npm-test.txt), [production build](build.txt)를 포함한다.

브라우저 실행 로그: [공통 창](dialog-viewport-issue9133.txt), [제품 정보](about-dialog-issue8905.txt), [PDF 가져오기](pdf-import-dialog-issue8411.txt), [자동 복구](autosave-recovery.txt). 반응형은 [검사별 HTML 보고서](responsive-report.html)의 PASS assertion 2,666개·FAIL 0개와 실제 종료 코드로 확인했다. 중복 진단 실행의 stdout 경합이 있어 그 혼합 로그는 증거로 사용하지 않았다.

기준 실행의 [종료 코드](base-checks.json) 중 공통 창의 실패는 목표 결함의 RED 증거다. [기준 자동 복구 정체](base-autosave-recovery.txt)는 글꼴 선택에 응답하지 않던 기존 E2E 셋업 문제이며 공통 CSS 회귀가 아니다. 실제 선택 보완·원 복구 assertion 보존은 [결과 보고](../../../report/task_m100_9133_report.md)에 구분했다.

## 직접 판독용 화면

아래 PNG는 각 본문 끝으로 스크롤한 뒤 캡처했다. 제목/조작과 본문 끝을 함께 확인한다. 후보 복구 창의 마지막 항목을 실제 좌표로 선택해 `restore/viewport-11`을 반환하는 검사도 통과했다.

| 화면 | 수정 전 | 수정 후 |
| --- | --- | --- |
| 편집 용지 800×360 | [전](before-page-800x360.png) | [후](after-page-800x360.png) |
| 환경 설정 800×360 | [전](before-options-800x360.png) | [후](after-options-800x360.png) |
| 글자 모양 800×360 | [전](before-char-800x360.png) | [후](after-char-800x360.png) |
| 그림 속성 320×240 | [전](before-picture-320x240.png) | [후](after-picture-320x240.png) |
| 복구 목록 320×240 | [전](before-recovery-320x240.png) | [후](after-recovery-320x240.png) |

HTML·텍스트 로그는 생성기의 줄 끝 공백만 정리했으며 assertion·수치·임베딩된 화면은 원본과 같다. 텍스트 실행 로그는 `.txt` 확장자로 보존했다.
