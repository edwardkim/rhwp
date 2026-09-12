# Task #6961 — 구현 및 결정적 회귀 검증

- Issue: #6961
- 기준: 계획 commit `7e386b8f6`, upstream/devel `a3cd825c2`.
- 단계: 구현 완료, 실제 패키지 저장 검증 진행 중.

## 변경

`documentFilename()`은 POSIX/Windows 경로 구분자 뒤의 leaf만 반환한다.
Chrome/Firefox `buildViewerUrl()`의 filename 파라미터에만 사용하며 원본 item/URL/상태 머신은
무변경이다. 빈 leaf는 filename 파라미터를 생략해 Studio의 기존 URL fallback에 맡긴다.
URL decode, trim, Unicode normalize를 추가하지 않아 정상 basename의 문자 자체를 보존한다.

## 수정 전후

실제 launcher의 새 탭/빈 탭 재사용 × 두 브라우저 × 경로·문자 10종을 검증한다.
기존 adapter의 정상 HWP metadata도 absolute filename으로 바꿨다.
수정 전 74개 중 30개 실패, 44개 통과. 수정 후 shared/sw 및 Chrome/Firefox adapter 전체
171개 통과. 기존 과거 다운로드·background 재시작·중복 이벤트·설정 OFF 검사를 포함한다.

Chrome packaged download E2E에는 실제 viewer URL의 filename이 다운로드 basename인지 검사한다.
기존 XLSX 두 건 탭 0, HWP 두 건 탭 1 검증과 함께 통과했다.
두 production build와 extension dist 계약 3개도 통과했다.

## 검증 환경

Node v24.15.0. 기존 pkg WASM 재사용(이번 작업 Rust/WASM source 무변경).
Firefox build는 기존 설치 의존성을 연결했다. Chrome은 기존 node_modules가 없어 전용 worktree에
lockfile 기반 npm ci(PUPPETEER_SKIP_DOWNLOAD=true)를 실행한 뒤 빌드했다.
의존성 없는 최초 Chrome build 실패는 설치 후 해소했다. 임시 symlink·dist·node_modules는 커밋하지 않는다.
Firefox UI 연결은 동일 bundle ID의 사용자 세션을 선택하므로 해당 세션에서 조작하지 않았다.
별도 프로필을 쓰는 Firefox WebDriver BiDi 패키지 E2E로 실제 다운로드·저장 검증을 진행한다.
