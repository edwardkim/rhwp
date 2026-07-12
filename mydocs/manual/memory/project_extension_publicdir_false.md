---
name: project_extension_publicdir_false
description: 확장 빌드는 vite publicDir:false라 public/ 자산이 자동 복사 안 됨 — build.mjs에 개별 copy 필수. 인라인 script는 확장 CSP가 차단
metadata: 
  node_type: memory
  type: project
  originSessionId: 2560b31a-9f1c-4764-bbf1-7ba5fc27c7ce
---

rhwp-chrome/rhwp-firefox 확장 빌드의 두 함정 (Task #1444, 2026-06-20):

**① 확장 vite 는 `publicDir: false`**: `rhwp-chrome/vite.config.ts`·`rhwp-firefox/...` 가
public/ 을 통째 제외(samples/images 대용량 회피)한다. 그래서 viewer 가 참조하는 새
public/ 자산(이미지·스크립트)은 **`build.mjs` 에 개별 `copy(...)` 라인을 추가해야** dist 에
들어간다. 안 하면 viewer 에서 404. 웹앱(rhwp-studio)은 publicDir 기본값이라 자동 복사 →
**웹앱 정상 = 확장 정상 아님**. chrome·firefox build.mjs 는 별도 파일이라 둘 다 정정.

**② 확장 CSP 는 인라인 스크립트 금지**: manifest `extension_pages: "script-src 'self'
'wasm-unsafe-eval'"` (unsafe-inline 없음). rhwp-studio/index.html 에 인라인 `<script>` 를
넣으면 확장에서 차단된다(웹앱은 통과). 외부 파일 + `<script src="...">`(동기, FOUC 필요
시 module/defer 금지)로 두고, 그 파일을 build.mjs 로 개별 copy 한다. (dev-tools-inject.js
src 주입이 동형 선례.)

**검증**: 웹앱 e2e/빌드만으로는 확장 회귀를 못 잡는다. 확장 새 기능/자산 추가 시
chrome://extensions unpacked 로드 → viewer 콘솔 CSP/404 확인을 메인테이너에게 요청.

**확장 버전 정책**: 스토어는 동일 버전 재업로드 불가. 배포 후 확장 회귀 발견 시 같은
버전 재제출 불가 → 다음 버전(0.2.x+1)에 묶어 올린다. (라이브러리 버전과 이원화: [[publish_guide 참조]])

관련: [[project_lfs_quota_full]], PR #1420(다크테마) 회귀.
