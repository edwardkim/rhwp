# rhwp v0.8.6

v0.8.6은 v0.8.4 이후 2,214개 커밋과 262개 PR provenance를 통합한 누적 PATCH
릴리스입니다. HWP/HWPX 문서의 조판·저장 충실도, Studio 편집, CLI·agent 표면,
입력 안전성과 배포 신뢰성을 함께 보강했습니다. 기존 JSON 봉투 major는 유지합니다.

## 주요 변경

- exact font instance API와 guarded kerning·common shaping·세로쓰기를 추가했습니다.
  RowBreak, LineSeg, 표·글상자·자리차지 개체와 특수 글립 배치도 한컴 오라클에 맞게
  연속 보정했습니다.
- HWPX OLE shape component, curve, BinData storage id와 HWP3·OWPML·HWPML 본문을
  보존하는 열기·저장 경로를 넓혔습니다. 중첩 표 검색·치환은 깊은 셀에서
  `cellPath`를 제공하면서 깊이 1의 기존 `cellContext`를 유지합니다.
- 문서 전체 HTML·Word `.doc` 내보내기, 차트 숫자 데이터 편집, 한글 IME,
  머리말·꼬리말, 배율·표 크기 조절을 개선했습니다. Chrome 확장이 `.xlsx`
  다운로드를 HWP로 잘못 여는 경로도 차단했습니다.
- 편집·조회 CLI, `rhwp-q-pack`, 문서 agent bridge와 HWP 2024 원격 MCP client를
  확장했습니다.
- HWPX container·parser 재귀 깊이, 입력 경계와 WMF 초기화를 보강했습니다.
  폰트·shaping 자원은 제한된 소유자 범위에서 재사용하고, CI의 반복 작업을
  줄였습니다.
- 공식 CLI 릴리스 매트릭스에 native Linux AArch64 target을 추가했고, Rust,
  `@rhwp/core`, `@rhwp/editor`, Studio, VS Code와 Chrome/Edge/Firefox/Safari 버전을
  0.8.6으로 맞췄습니다.

상세 변경 및 영문 변경 기록은 `CHANGELOG.md`와 `CHANGELOG_EN.md`를 참조하세요.

## 호환성과 후속 확인

- exact font instance와 `--compat 2024`는 명시적 opt-in이며 기존 기본 경로를 바꾸지
  않습니다.
- 저장 보존 결함을 복구한 문서는 재저장 byte가 달라질 수 있으나, 문서 의미를
  보존하기 위한 의도된 변화입니다.
- Linux AArch64 #5949는 실제 v0.8.6 asset의 ELF·실행권한·버전을 검증한 뒤
  닫습니다. trusted controller post-release canary #6243은 `main` 반영 후 실건에서
  검증합니다.

## 기여자

이번 사이클에 참여한 사람 20명(credit key, 대소문자 보존·알파벳순):

<!-- release-contributors:start -->
- @chrisryugj
- @coolwithyou
- @davindev
- dkh0324 — Git author credit, 공개 GitHub 계정 미확인
- @edwardkim
- @humdrum00001010
- @JamesPsh
- @jangster77
- @jeong-sik
- @johndoekim
- @keepYaoung
- @kevin9327
- @kjh0523
- @lpaiu-cs
- @planet6897
- @postmelee
- @RaghavShubham
- @Shadungi
- @t2c-lab
- @thhan74
<!-- release-contributors:end -->
