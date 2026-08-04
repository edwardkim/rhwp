# M100 #2656 완료 보고 — Chrome/Edge 확장 설정 보존

- 이슈: [#2656](https://github.com/edwardkim/rhwp/issues/2656)
- 브랜치: `codex/issue-2656-extension-settings`
- 작성일: 2026-07-21

## 결과

Chrome/Edge 확장에서 사용자가 끈 `한글파일 자동보기(autoOpen=false)`가 storage key 부분
누락, storage 읽기 실패, Service Worker 재시작, 확장 설치·업데이트 수명주기 이후에도 보존될
수 있는 이중 저장·복구 구조를 구현했다.

수정 빌드와 스토어 v0.2.8 모두 정상 수명주기 테스트를 통과했으므로 Chrome 업데이트 자체가
설정을 초기화하는 원 제보는 재현되지 않았다. 이번 작업은 설치 시 기본값 재기록, options 초기화
경쟁, 저장 오류 무시, local 복구 부재에 대한 방어 로직과 관측 가능한 오류 처리로 범위를 명확히
한다.

## 주요 변경

- sync의 기존 flat key를 권위 값으로 유지하고 local last-known-good snapshot을 추가했다.
- sync key가 없거나 sync 읽기가 실패한 항목만 local 값으로 복구한다.
- 저장 실패를 성공으로 표시하지 않고 UI를 실제 저장 상태로 되돌린다.
- options 입력은 storage 로드가 끝나기 전까지 비활성화한다.
- 설치·업데이트 이벤트는 사용자 설정을 쓰지 않고 최소 local 진단 메타데이터만 기록한다.
- options, message router, download interceptor가 하나의 settings adapter를 사용한다.
- 저장소·수명주기·options 경쟁/오류·기존 다운로드 동작 회귀 테스트를 추가했다.
- sync 상태를 읽지 못하면 local `true`만으로 자동 탭을 열지 않는 fail-closed 정책을 적용했다.
- 동일 download id 동시 이벤트를 in-flight 선점으로 직렬화하고 handled/terminal 최신 상태를
  보존해 탭 1개 불변식을 강화했다.
- sync 저장을 권위 성공 조건으로 삼고 local snapshot 실패는 복구 백업 경고로 분리했다.
- update/Chrome-update 시점의 유효한 설정을 local snapshot으로 선보존하고, 기존 설치의 partial
  sync를 clean install과 구분해 근거 없는 자동 열기와 default snapshot 생성을 막았다.

세부 원인, 저장소 우선순위, 수동 체크리스트는
[Stage 1](../working/task_m100_2656_stage1.md)에 기록했다. 과거 탭 다발 맥락과 사후 보강은
[Stage 2](../working/task_m100_2656_stage2.md), PR review partial-sync 보강은
[Stage 7](../working/task_m100_2656_stage7.md)에 기록했다.

## 검증

- 변경 JavaScript `node --check`: 통과
- Chrome 확장 테스트: 37 passed, 0 failed
- Chrome 옵션 UI 테스트 CI gate: 4 passed, 0 failed
- 공통 다운로드 판정·상태 머신 테스트: 40 passed, 0 failed
- 확장 dist 계약 테스트: 3 passed, 0 failed
- locale JSON parse: 통과
- source/dist 핵심 모듈 byte 비교: 통과
- `npm --prefix rhwp-chrome run build`: 통과
- `git diff --check`: 통과

PR review 보강 후 focused 재검증 결과는 다음과 같다.

- Chrome options + service worker 테스트: 41 passed, 0 failed
- shared + Firefox 다운로드 회귀 테스트: 76 passed, 0 failed
- Chrome/Firefox dist 계약 테스트: 3 passed, 0 failed
- 최신 `upstream/devel` 동기화 후 Chrome/Firefox 확장 빌드: 각각 169 modules transformed, 성공
- source/dist `background.js`, `settings-store.js`, `extension-lifecycle.js`, `options.js` byte 비교: 통과

자동화된 Chrome 제어는 `chrome://extensions` 내부 URL 접근이 보안 정책으로 차단되어 중단했고,
우회나 사용자 기본 프로필 설치는 수행하지 않았다. 대신 작업지시자가 별도 Chrome 프로필에서 옵션
재진입, Service Worker 재시작, 비활성화/재활성화, 브라우저 재실행, 압축해제 확장 Reload, 실제
HWP/HWPX 다운로드를 직접 확인했다.

실제 Chrome Web Store 선배포는 필요하지 않다. 업데이트 이벤트의 설정 무변경 계약은 자동
테스트로 검증하며, 동일 경로 압축해제 Reload로 배포 전 수명주기 smoke test를 할 수 있다.
Web Store 업데이트 뒤의 확인은 별도 배포 인수 항목이다.

작업지시자의 설정 수명주기 1~7 테스트가 통과했다. 보강 후 최신 빌드에서도 자동 열기를 켠 상태로
과거 HWP 기록을 둔 채 Reload/Chrome 재실행 시 기존 문서 탭 0개, 새 HWP 한 건 다운로드 시 뷰어
탭 정확히 1개를 확인해 최종 수동 인수를 완료했다.

Chrome 완전 종료 뒤 프로필 선택 화면이 나타나기까지 느린 현상도 관찰했으나, 확장을 제거한 뒤에도
동일하게 재현됐다. 이 화면은 특정 프로필의 확장이 활성화되기 전 단계이므로 #2656 변경과 무관한
Chrome/OS 시작 지연으로 판정했으며 이 PR의 회귀로 다루지 않는다.

## 배포 상태

로컬 구현, 테스트, dist 빌드, 수동 인수, 문서화까지 완료하고
[Draft PR #2658](https://github.com/edwardkim/rhwp/pull/2658)을 `devel` 대상으로 생성했다.

PR 생성 뒤 CI의 기존 확장 테스트 명령이 `rhwp-chrome/sw/*.test.mjs`만 실행해 저장 실패와 초기 로딩
경쟁을 검증하는 루트의 `options.test.mjs`를 누락한다는 점을 확인했다. 별도 옵션 UI 테스트 단계를
추가해 4개 테스트를 PR gate에 연결했다. 실제 Chromium에 압축해제 확장을 로드하는 E2E는 브라우저
바이너리·영속 프로필·다운로드 fixture를 도입하는 별도 범위이므로 후속 이슈/PR로 분리한다.

이후 [PR #2658 requested-changes review](https://github.com/edwardkim/rhwp/pull/2658#pullrequestreview-4745744846)와
[메인테이너 재실증](https://github.com/edwardkim/rhwp/pull/2658#issuecomment-5040233424)에서 local snapshot이
없는 기존 사용자의 partial sync 공백이 확인됐다. update/Chrome-update 선보존과 partial-sync
fail-closed를 함께 적용하고 reviewer의 전체 재현 절차를 회귀 테스트로 고정했다. 보강 구현과 로컬
검증은 완료했으며, commit·push·재검토 요청은 작업지시자 승인 뒤 수행한다.
