# Task #6961 — 파일명 수정 결과

- Issue: #6961, code commit: `65e4b32e0`.
- 수정: Chrome/Firefox viewer URL의 filename에 경로 대신 basename을 전달한다.
- 상태 머신·다운로드 이벤트·Studio/Rust/WASM source는 무변경이다.

## 검증 결과

- 결정적 red: 74개 중 30 실패. 수정 후 shared/sw 및 두 adapter 171/171 통과.
- Firefox/Chrome production build, extension dist 계약 3/3 통과.
- Chrome 실제 패키지 다운로드 4/4: XLSX 두 건 탭 0, HWP 두 건 탭 1, filename basename 확인.
- Firefox 155.0.1 임시 프로필+production package, 기존 공개 HWP sample을 loopback에서 다운로드.
- 실제 viewer URL·편집기 상태 표시 이름: `신청서(SW) (개인)_1.hwp`.
- 입력 이벤트 경로로 `ISSUE6961_EDIT` 삽입 후 실제 파일 메뉴의 저장 명령 실행.
- 저장본: `신청서(SW) (개인)_1(1).hwp`.
- 다른 이름으로 저장 두 단계 모달의 기본 이름: `신청서(SW) (개인)_1`.
- 다른 이름 저장본: `신청서(SW) (개인)_1(2).hwp`.
- 출력의 (1)/(2)는 Firefox의 동일 파일명 충돌 회피다. 원본 경로 접두사는 없다.
- 두 저장본을 동일 WASM HwpDocument로 새로 파싱하여 편집 marker 보존을 확인했다.

## 실제 검증이 드러낸 별도 결함

Firefox 다운로드 ID 1건에 탭 ID 2개가 열렸고, 저장/다른 이름으로 저장의 blob 다운로드마다
탭 2개가 추가됐다. 같은 패키지의 launcher를 upstream/devel 원본으로 바꾼 기준본에서도
HTTP 다운로드 1건에 탭 2개가 재현돼 #6961 도입 회귀가 아님을 확인했다.
사용자 승인으로 #6964를 별도 등록하고 별도 branch/commit으로 수정한다.
따라서 #6961의 파일명 정확성은 확인했으나 탭 안전성을 포함한 배포 완료 판정은 #6964 검증 뒤로 보류한다.

## 환경과 한계

Node v24.15.0, Puppeteer/Firefox WebDriver BiDi. 임시 프로필에서 확장 페이지 관측에 필요한
`--remote-allow-system-access`를 사용했다. 기존 사용자 프로필이나 브라우저 설정은 변경하지 않았다.
BiDi Page.url()은 확장 탭을 about:blank로 표시하여 실제 location.href와 browser.tabs.query ID로
검증했다. privileged page screenshot은 도구가 지원하지 않아 화면 캡처 성공으로 기록하지 않는다.
입력·메뉴는 production DOM 이벤트 경로를 사용했고 OS 네이티브 저장 창을 조작한 검증은 아니다.
이번 작업은 Rust/WASM/Studio source가 없어 기존 pkg WASM을 재사용했으며 Rust lint/전체 검증은 대상 밖이다.

로컬 원시 로그: `/private/tmp/issue6961-tests.log`, `issue6961-chrome-e2e.log`,
`issue6961-firefox-baseline.log`, `issue6961-firefox-save-as.log`.
이 로그는 세션 증적이며 public source test가 아니다. 실제 저장본과 프로필도 /private/tmp에만 있다.
원격 push, PR 생성, 배포는 아직 수행하지 않았다.

## #6964 결합 검증 후속

#6964는 독립 브랜치 `codex/issue-6964-download-duplicates`에서 구현했다
(구현 `c4c757a18`, 첫 이벤트 호출 시점 보정 `5de285adb`).
공통 상태 머신 정책은 그대로 두고 Firefox의 같은 ID 이벤트를 직렬화하며,
자체 extension Blob 저장의 자동 열기를 제외한다.

첫 browser API 호출은 기존처럼 이벤트 수신 중 시작하도록 보정한 뒤,
#6961의 launcher/helper를 결합한 Firefox 155.0.1 패키지가 새 프로필 3회와 최종 재빌드에서
정상 파일명·뷰어 1개·편집 내용 보존을 확인했다. 다운로드는 입력 1개와 저장 출력 2개이며,
저장/다른 이름 저장 파일은 `신청서(SW) (개인)_1(1).hwp`, `신청서(SW) (개인)_1(2).hwp`였다.
첫 큐 구현에서 관측된 간헐적 headless 누락과 호출 시점 보정은 #6964 결과보고에 기록했다.

#6964 최종 관련 테스트 148/148, Chrome 기존 4종 + 자체 Blob 저장, 두 build와 dist 계약이
통과했다. 두 브랜치는 merge-tree에서 충돌 없이 결합된다. 배포 시 두 수정의 포함 여부를 확인한다.
원격 push/PR/배포는 아직 수행하지 않았다.


## 사용자 직접 검증

사용자가 제공한 #6961 + #6964 결합 Firefox dist를 직접 테스트하고 다음 세 항목이 모두
문제없다고 확인했다. 에이전트 자동 검증과 구분한 사용자 보고다.

1. HWP 다운로드 시 뷰어가 1개만 열린다.
2. 편집 후 저장/다른 이름으로 저장에서 파일명에 `_Users_…` 경로 접두사가 붙지 않는다.
3. 저장 시 추가 뷰어가 열리지 않으며 저장본에 편집 내용이 남는다.


## Draft PR 제출

사용자의 명시 승인 후 upstream 작업 branch에 push하고 [PR #6965](https://github.com/edwardkim/rhwp/pull/6965)를 devel 대상으로 draft 생성했다.
[작성자 검토 기록](../pr/archives/pr_6965_review.md)을 같은 PR에 포함한다. 최초 head는
`6e957ebfed0fa3f527026244c72fc6ddca0bc943`이며, 이후 기록 commit도 같은 branch에 push한다.
생성 후 API에서 base/head/draft와 한글 본문을 재확인했다. CI는 작성 시점 실행 중이며,
ready 전환·merge·issue close·배포는 수행하지 않았다.


## 추가 Chrome packaged smoke 검증

- 검증 head: `a20442ea2133ebc0ba7732e37dc6c3ee3d2730e0` (기존 PR source 변경 없이 실행)
- 실행: `PUPPETEER_EXECUTABLE_PATH='/Applications/Google Chrome.app/Contents/MacOS/Google Chrome' npm --prefix rhwp-chrome run test:e2e:smoke`
- 결과: production build 성공, page-budget/proxy 계약 4/4 통과,
  `PASS: viewer/options/print/service worker/content script` 확인, exit 0.
- 로컬 Chrome을 headless·격리 프로필로 실행했으며 기존 pkg WASM을 재사용했다.
- MV3 background 시작과 메시지 정책, HWP3 문서 canvas, 다크 아이콘 자산,
  settings hydration, print.html 로드, content script 배지를 확인했다.
  console/page/worker 오류와 예상 밖 탭은 관측되지 않았다.
- smoke는 autoOpen=false이므로 다운로드 감지·편집 후 저장 회귀 테스트를 대체하지 않는다.
  기존 다운로드 E2E와 Firefox/사용자 저장 검증을 보완하는 결과다.
- 로그: `/private/tmp/pr6965-chrome-smoke.log`.
