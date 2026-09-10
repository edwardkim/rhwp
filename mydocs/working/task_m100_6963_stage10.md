# 단계 10 — PR 갱신과 동일 문서 링크 보존 증거

- Issue: #6963 / PR: #6984
- 실행일: 2026-09-10
- 제품·테스트 검증 후보: `fd06747f6`
- 범위: 설명 문자열은 사용자 요청으로 이번 PR에서 제외한다. 단계 9의 설명 문자열 대기는 후속 과제이며 이 PR의 미완료 조건이 아니다.
- 승인: 기존 PR 업데이트·push·스크린샷 게시 및 Viewer 보안 경고의 ‘한 번 허용’을 사용자에게 승인받았다. merge·issue close·댓글은 범위 밖이다.
- 문서 라우팅: collaborator_self_merge + intake_and_review, local_validation, visual_fixture_evidence, rework_and_exceptions. 상위 PR workflow와 선택표, 개발 환경·시각 검증 정책을 확인했다.

## 회귀에서 보완한 사항

전체 검사에서 표시 문자열 API의 mutation registry 등록, snapshot 호출 개수 가드,
마우스 이벤트 helper 위임 검사와 native passthrough 분류 누락을 발견했다.
`fd06747f6`에서 실제 변경 경로를 등록하고 helper 위임을 검증하도록 보완했다.
가드 자체를 건너뛰거나 실패를 허용하지 않았다.

## 동일 문서 재현

[원본 HWPX](../pr/assets/issue6984/hyperlink-preservation.hwpx)는 rhwp의 기존 링크 문서를
현재 WASM API로 편집하여 생성한 작은 합성 검증 문서다. 표시 문자열은 `한컴 공식 홈페이지`,
주소는 `https://www.hancom.com`이다. 개인정보나 사용자의 작업 문서를 포함하지 않는다.
자동 회귀 입력을 추가하는 samples fixture가 아니라 PR 수동 재현 첨부물이다.

1. Studio의 파일 열기로 이 HWPX를 열고 기존 링크 고치기에서 표시 문자열·주소를 확인했다.
2. **같은 HWPX 파일**을 macOS 한컴오피스 한글 Viewer로 열었다. 파란 밑줄 링크를 클릭하고,
   보안 경고에서 사용자 승인 후 ‘한 번 허용’을 눌러 Firefox의 한컴 공식 홈페이지 이동을 확인했다.
3. Studio에서 같은 HWPX를 PDF 인쇄 경로로 내보낸 [PDF](../pr/assets/issue6984/hyperlink-preservation.pdf)를
   macOS 미리보기로 열었다. 파란 밑줄과 링크 열기/링크 복사하기 메뉴를 확인했고,
   접근성 트리도 `link https://www.hancom.com/`를 보고했다.
4. pypdf로 PDF의 실제 `/Link` → `/URI` 주석 1개와 목적지를 확인했다.
   [검증 JSON](../pr/assets/issue6984/verification.json)에 원본·PDF SHA-256과 주석 사각형을 보존한다.

세 스크린샷은 직접 앱에서 캡처한 원본이다. Studio·Viewer는 동일 HWPX, PDF는 그 문서의
Studio 출력물이다. 화면은 링크 표시의 증거이며, 실제 이동·PDF 주석 검사는 별도로 구분한다.
Viewer는 기존 macOS 앱을 사용했고 브라우저의 개인 탭·즐겨찾기 화면은 게시하지 않았다.

- [Studio](../pr/assets/issue6984/rhwp-link.png)
- [한글 Viewer](../pr/assets/issue6984/viewer-document.png)
- [PDF 미리보기](../pr/assets/issue6984/pdf-link.png)

## 검증

- review worktree에서 integration suite prepare 후 fmt, native/WASM/all-targets Clippy,
  workspace build, manifest check를 순차 통과했다.
- 전체 nextest: 9,405 통과, 46 skipped.
- Studio 전체 Node: 1,502 통과, 2 skipped.
- native-skia lib: rhwp 3,930 통과(13 ignored), workspace 보조 lib 182 통과.
- native-skia placeholder 2개·직접 PDF 4개 통과. Studio TypeScript/Vite build 통과.
- source-side unit tier 정책 검사 통과(4,205 tests / 298 modules).
- 단계 9의 실제 WASM 11그룹, UI E2E, PDF 5개·34쪽·81주석 검증도 유지된다.
- 최신 upstream/devel과 merge-tree 검사에서 충돌이 없었다.

HTTP/HTTPS 텍스트 링크 범위의 검증이다. 설명 문자열, 북마크·파일·메일 연결,
회전 텍스트를 포함한 모든 레이아웃의 링크 정확도까지 일반화하지 않는다.
