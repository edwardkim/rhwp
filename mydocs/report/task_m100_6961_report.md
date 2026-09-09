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
