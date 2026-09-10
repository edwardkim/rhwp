## 문제와 수정

다운로드로 자동 열린 문서를 저장하면 전체 로컬 경로가 파일명 앞에 붙었다.
Chrome/Firefox의 viewer URL 조립 경계에서 basename만 추출해 한글·공백·괄호·밑줄을 보존한다.
다운로드 상태 머신과 Studio 저장 로직은 변경하지 않는다.

Closes #6961

## 검증

- 사용자 직접 검증: 결합 dist에서 다운로드 탭 1개, 정상 저장 파일명, 저장 시 추가 탭 없음과 편집 보존을 모두 확인.

- shared/sw·두 adapter: 171/171, 두 확장 production build, dist 계약 3/3.
- 실제 Chrome 다운로드 4/4: 파일명 및 탭 수 검증.
- 실제 Firefox 155.0.1: 저장/다른 이름으로 저장 정상 basename, 저장본 재파싱으로 편집 보존 확인.
- Firefox의 기존 탭 중복/자체 저장 재오픈은 수정 전 기준본과 대조하고 #6964로 별도 분리했다.
- Rust/WASM source 무변경, 기존 WASM 재사용.

#6964의 최종 보정과 결합한 Firefox 패키지에서 탭 1개 유지·정상 파일명·편집 보존을 확인했다.
배포에는 두 수정이 함께 포함되어야 한다.

## 제출 기준

- 검증한 code commit: `65e4b32e0bde1fdb0bc99300e37d85d5f57af5e7`
- 최초 제출 head: `6e957ebfed0fa3f527026244c72fc6ddca0bc943` (code 검증 이후 변경은 `mydocs/` 기록뿐)
- 최신 devel `0d36da409`와 merge-tree 충돌 없음.
- 변경 범위: 확장 JavaScript·회귀 테스트·작업 문서. Rust/WASM/Studio source와 CI 설정은 변경하지 않았다.
- [x] 변경 범위의 로컬 검증과 실제 브라우저 검증을 수행했다.
- [x] `git diff --check`와 최신 base 결합을 확인했다.
- 캡슐은 문서 편집 산출물 생성 작업이 아니므로 미첨부.

사용자 승인에 따라 draft로 제출한다. ready 전환·merge·배포는 별도 단계다.

관련 PR: #6966. [작성자 검토 기록](https://github.com/edwardkim/rhwp/blob/codex/issue-6961-download-filename/mydocs/pr/archives/pr_6965_review.md).


## 추가 Chrome smoke

- `a20442ea2133ebc0ba7732e37dc6c3ee3d2730e0`에서 `npm --prefix rhwp-chrome run test:e2e:smoke` 통과.
- production build, 보조 계약 4/4, 실제 Chrome headless의 viewer/options/print/service worker/content script 모두 PASS.
- 기존 다운로드 E2E에 추가한 검증이며, 소스 수정 없이 실행했다. 기존 WASM 재사용.
