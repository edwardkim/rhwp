# #9133 공통 대화상자 화면 경계 보완 결과

## 범위와 현재 상태

작업지시자의 2026-11-16 신규 이슈 등록·메인테이너 구현 지시에 따라 [#9133](https://github.com/edwardkim/rhwp/issues/9133)을 등록하고 `edwardkim`에게 할당했다. [수행 계획](../plans/task_m100_9133.md)을 `5e9f208486`에, 공통 CSS·회귀·가이드를 `4b72435a0765fdc1d38698ac46f0556339986c05`에 기록했다.

로컬 구현·필수 검증을 완료했다. 복구 E2E의 실제 글꼴 선택 보완까지 포함한 최종 code head는 `80bf4c9e1b01ca6b621a7bc5d49d7fd64a1f7ad7`이다. 원격 push·PR·공개 댓글·merge·issue close는 수행하지 않았다. 기존 #8905의 제품 정보 수정은 유지하고 공통 창의 후속 범위를 별도 이슈로 처리한다.

## 원인과 보완

기존 공통 `dialog-wrap`에는 viewport 최대 높이가 없었다. 모바일에서는 wrap 전체가 스크롤해 확인/취소가 밀렸고, 옆 버튼 구조의 중간 flex 항목과 글자 모양의 본문 최소 높이는 내용 축소를 막았다.

- 공통 wrap을 viewport 안의 세로 flex로 제한하고 제목/footer를 고정했다. 직접 본문은 남은 공간에서 양축 스크롤한다.
- `cs-main-row → cs-left-col → dialog-body`까지 축소를 전달한다. 기존 380px은 최소 높이 대신 선호 flex 크기로 소비한다.
- 탭은 줄어들거나 여러 세로줄로 늘어나 옆 버튼을 덮지 않고 가로 스크롤한다.
- 모바일 하단 시트·90dvh·PDF/제품 정보·표/그림의 전용 패널은 유지한다.
- 복구 창은 기본 후보 목록 스크롤을 유지하되, 안내와 후보 한 줄도 못 담는 높이에서는 본문도 스크롤한다. 목록 높이 0px을 막고 마지막 항목의 실제 선택·복구를 검증한다.
- 기존 복구 소스 검사 중 `overflow:hidden`·`min-height:0`을 고정하던 두 assertion만 제거했다. viewport·DOM hook·모바일·긴 이름 보호는 유지하고, 실제 스크롤과 선택은 새 브라우저 회귀로 검증한다.

문서 엔진·조판·저장 API·CI 실행 정의는 변경하지 않았다. 조판 Visual Sweep·383쪽 편람·정본 보호 TSV는 이 UI-only 변경에 비해당이다.

## 실제 브라우저 대조

기준 devel `6269cb0f1707a1c5afcae4840ac875f928c4a03d`의 CSS와 후보를 같은 실제 Studio·최적화 WASM·Chromium 152.0.7946.0 / revision 1660786에서 실행했다. 기준 CSS는 66개 조합을 끝까지 수집하고 67개 조건이 실패했으며, 후보는 모두 통과했다. `1280×900`의 11종 창 외곽 크기·위치는 기준과 모두 같다.

| 800×360의 창 | 수정 전 세로 범위 | 수정 후 세로 범위 |
| --- | --- | --- |
| 편집 용지 | -70~430px | 12~348px |
| 환경 설정 | -62.09~422.09px | 12~348px |
| 글자 모양 | -39.5~399.5px | 12~348px |

11종은 편집 용지·환경 설정·제품 정보·글자/문단 모양·대표 글꼴·스타일·복구·PDF 가져오기·표/셀 속성·그림 속성이다. 각각 `800×360`, `768×320`, `767×320`, `320×240`, `360×640`, `1280×900`을 검사했다.

창/제목/조작의 viewport 포함, 본문·목록·패널 끝 스크롤, 스크롤 뒤 고정 조작, 실제 좌표 취소, 마지막 복구 후보 선택·복구, PDF 취소 시 읽기 0회, 열린 창의 resize와 입력 값 보존을 확인했다. 실제 F7 명령에서 제본 여백 1.5mm 확인 → Undo → Redo, 취소 버튼의 Enter, 배경 클릭 기본 비닫힘과 Escape, 세로 시트의 실제 CDP 터치 취소도 통과했다.

스크린샷을 직접 판독해 제목·확인/취소와 마지막 복구 후보가 읽히는 것을 확인했다. 브라우저 에뮬레이션이며 Android Firefox 실기기 검증으로 주장하지 않는다.

## 최종 검증

| 최종 검사 | 실제 결과 |
| --- | --- |
| Docker locked release WASM + wasm-opt | 성공, 종료 코드 0 |
| `npx tsc --noEmit`, `npx tsc -p tsconfig.ci-unit.json` | 각각 종료 코드 0 |
| Studio `npm test` | 2,798건 PASS·실패 0·기존 skip 2건 |
| `npm run build` | production/PWA build 성공 |
| 새 공통 창 E2E | 66개 조합·resize·입력·실제 확인/Undo/Redo·취소·터치 통과 |
| 기존 제품 정보 E2E | 19개 검사 통과 |
| 기존 PDF 가져오기 E2E | 두 테마·입력/키보드·실제 Worker·제한·취소/재시도 통과 |
| 기존 자동 복구 E2E | 새 문서·HWP·HWPX 복구 통과 |
| 기존 반응형 E2E | 2,666 PASS·실패 0 |
| E2E manifest·문서 링크·diff 검사 | 통과 |

초기 로컬 `pkg/`는 JS에 API가 있지만 타입 선언에 `getMemoList`가 빠진 불일치가 있었다. 현재 Rust 소스로 Docker에서 최적화까지 빌드해 해소했다. 공용 `/home/edward/mygithub/rhwp/target/pr-review`를 사용했으며 별도 Cargo 캐시를 만들지 않았다. 최종 검증 source와 빌드 source 사이에는 E2E 파일만 추가 변경됐고 Rust/Cargo 입력은 같다.

최적화 WASM SHA-256은 `806bc205fb3a23d1018007d9bdf6bec1e778c7e5a01e483c3fd8622f5ca22f82`이다. 실제 `pkg/`와 Studio public의 JS/WASM 해시가 각각 일치한다. [패키지 정보](../working/assets/issue9133/package.json)와 [검사별 명령·종료 코드](../working/assets/issue9133/checks.json)를 보존했다.

자동 복구 E2E의 최초 정체는 기준 CSS에서도 동일하게 재현했다. `11c6fbaa85`의 복구 검사는 선행 [PR #8397](../pr/archives/pr_8397_review.md)·통합 `99a176b7cb`의 실제 글꼴 선택에 응답하지 않았다. `local-font-prompt-issue8371.test.mjs`의 지원된 선택 계약과 대조한 뒤, 글꼴 미공급 환경에서 실제 ‘대체 글꼴로 보기’ 버튼을 누르는 셋업만 추가했다. 문서 복구·dirty·원본 draft 삭제 assertion은 모두 유지했다. 제품 글꼴 선택·복구 로직을 우회하거나 수정하지 않았다.

증거는 [원장](../working/assets/issue9133/README.md)에 보존한다. 현재 로컬 작업 branch는 `task/9133-dialog-viewport-containment`이며 원격 제출 대기로 작업 worktree를 유지한다. 최종 devel과 본인 열린 PR·제출 후보 병합 시뮬레이션을 재확인한 뒤, 별도 승인된 원격 절차를 진행한다.
