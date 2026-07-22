---
kind: working-note
status: complete
issue: 2656
stage: 1
---

# Task #2656 Stage 1: Chrome/Edge 설정 보존 구현·검증

## 확인한 원인 경계

정적 검사만으로 정상 Chrome 업데이트가 `chrome.storage.sync` 값을 직접 삭제한다고 단정할
근거는 없었다. 대신 확장 자체에는 설정 원복으로 관찰될 수 있는 다음 결함이 있었다.

- 설치 이벤트에서 기본 설정 전체를 다시 기록하는 경로
- options DOM 기본값과 비동기 storage 로드 사이의 경쟁 조건
- 한 항목 변경 시 아직 로드되지 않은 다른 항목까지 함께 저장하는 동작
- `runtime.lastError`를 무시하고 저장 성공을 표시하는 동작
- sync key 누락·읽기 실패 시 사용자 값의 복구 경로가 없는 구조

따라서 수정 범위는 Chrome 업데이트 원인을 추정하는 대신, 사용자가 저장한 `false`가 설치·업데이트
수명주기와 저장소 부분 장애 뒤에도 보존되는 계약을 만드는 것으로 정했다.

## 구현

- `sw/settings-store.js`
  - 기존 flat sync key를 호환 권위 값으로 유지했다.
  - local에 schema version과 timestamp가 있는 last-known-good snapshot을 둔다.
  - sync key 누락 또는 sync 읽기 실패 항목만 local snapshot에서 복구한다.
  - 신뢰할 수 있는 저장값이 전혀 없는 정상 clean install에서만 기본값을 적용한다.
  - local/sync 저장 오류를 호출자에게 전달해 거짓 성공을 차단한다.
- `sw/extension-lifecycle.js`, `background.js`
  - `install`, `update`, `chrome_update`에서 사용자 sync 설정을 쓰지 않는다.
  - reason, 이전/현재 버전, 시각만 local 진단 메타데이터로 기록한다.
- `options.js`, `options.html`
  - storage 로드 전에는 입력을 disabled로 유지하고 HTML 초기 `checked`를 제거했다.
  - 로드된 snapshot을 기준으로 변경 항목만 합성해 저장한다.
  - 저장 실패 시 실제 저장값으로 UI를 되돌리고 오류를 표시한다.
- message router와 download interceptor는 공통 settings loader를 사용한다.
- 한국어/영어 오류 문구, 개인정보 문서, 브라우저 확장 개발 문서를 갱신했다.

## 자동 검증

2026-07-21 최종 재검증 결과:

- 변경 JavaScript 4개 `node --check`: 통과
- Chrome 확장 단위·통합 테스트: 31 passed, 0 failed
- Chrome/Firefox/Safari dist 계약 테스트: 3 passed, 0 failed
- locale JSON parse: 통과
- source/dist `options.js`, `settings-store.js`, `extension-lifecycle.js` byte 비교: 통과
- `git diff --check`: 통과
- `npm --prefix rhwp-chrome run build`: 통과
  - Vite 8.1.4, 167 modules transformed
  - manifest, background/content/options, `sw/`, locales, WASM, fonts 복사 확인
  - theme-init, CanvasKit externalization, runtime SVG, chunk size 관련 기존 warning만 발생

최초 빌드는 저장소 루트의 공유 Vite 설치를 참조하려다 실패했다. `rhwp-chrome`에
`npm ci`로 잠금 파일 기준 의존성을 설치한 뒤 실제 빌드가 통과했으며, 추적 파일 변경은 없었다.

## 브라우저 검증 시도와 제한

Chrome 제어 세션에서 새 탭으로 `chrome://extensions` 접근을 시도했으나 브라우저 자동화 보안
정책이 내부 URL 접근을 차단했다. 정책을 우회하거나 사용자의 기본 Chrome 프로필에 확장을 설치하지
않았다. 따라서 다음 항목은 자동 검증과 분리한 별도 테스트 프로필 수동 인수 항목이다.

1. `rhwp-chrome/dist`를 압축해제 확장으로 로드한다.
2. 옵션에서 한글파일 자동보기를 해제하고 설정 페이지 재진입 뒤 해제 상태를 확인한다.
3. Service Worker 재시작, 확장 비활성화/재활성화, Chrome 재실행, 동일 경로 Reload마다 상태를
   확인한다.
4. HWP/HWPX 다운로드 때 뷰어가 자동으로 열리지 않는지 확인한다.
5. 확장 Service Worker DevTools에서 sync의 `autoOpen`만 제거한 뒤 옵션을 다시 열어 local
   snapshot의 `false`가 복구되는지 확인한다.

압축해제 확장 Reload는 실제 Chrome Web Store 업데이트와 완전히 같은 유통 경로는 아니다.
그러나 업데이트 이벤트의 설정 무변경 계약은 단위 테스트로 고정했으므로 스토어 선배포가 merge의
필수 조건은 아니다. 스토어 버전 업데이트 후 확인은 배포 인수 smoke test로 수행한다.

## 판정

코드 결함 수정, 회귀 테스트, 실제 dist 빌드는 완료했다. 실제 Chrome 프로필 수명주기와 다운로드
동작은 정적 검증만으로 완전히 증명할 수 없으므로, 별도 테스트 프로필 수동 확인을 배포 전 인수
항목으로 남긴다.
