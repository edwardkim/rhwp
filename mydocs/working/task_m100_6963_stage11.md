# 단계 11 — 하이퍼링크 끝 이어 쓰기 경계

- Issue: #6963 / PR: #6984
- 실행일: 2026-09-10
- 기준: `112a69566`
- 재현: 링크 등록 직후 끝 캐럿에서 입력하면 링크 범위가 늘고 파란색/방문 보라색·밑줄도 상속되었다.

## 원인과 수정

공통 `Paragraph::insert_text_at`은 모든 필드의 끝과 같은 위치에서도 end를 늘렸다.
또한 서식 범위 적용은 문단 끝의 원래 모양을 남기지 않고, 삽입은 경계 모양을 오른쪽으로
밀어 왼쪽 링크의 모양을 새 글자에 상속했다.

하이퍼링크 끝은 이어 쓰기 위치로 처리하여 기존 링크 범위를 늘리지 않는다.
링크 내부에서 입력하면 기존처럼 범위를 늘린다. 다른 종류의 필드 끝 규칙은 유지한다.
링크 서식을 적용할 때 문단 끝에도 원래 글자 모양을 복원하고, 끝에서 입력할 때 그 모양을
삽입 위치에 유지한다. 색상·밑줄을 일괄 초기화하지 않고 원래 모양을 재사용한다.

## 검증 기록

- 링크 관련 native 회귀 14개 통과: 문단 끝 입력, 방문 색 변경, 한글 조합처럼 delete/insert를
  반복하는 local 편집, 후속 일반 삽입, HWP/HWPX 저장 왕복, 링크 내부의 Unicode 삽입 포함.
- UI E2E에 등록 직후 실제 키 입력으로 링크 범위·일반 서식을 검사하는 사례를 추가했다.
- 잠긴 native wrapper로 진단용 WASM을 재빌드했다(`--no-opt`; Docker를 사용하지 못하는 기존 로컬 환경).
- 실제 WASM snapshot 11그룹 통과. Chrome UI E2E에서 등록 직후 `XYZ`를 실제로 입력해
  기존 링크 끝(9자)·검정·밑줄 없음 확인. 이어 쓰기 뒤 고치기/지우기·undo/redo도 통과했다.
- [이어 쓰기 화면](../pr/assets/issue6984/typing-after-link.png): 에이전트가 직접 이미지 확인.
  기존 한글 링크는 파란 밑줄, 이어 쓴 XYZ는 검정이며 밑줄이 없다.
- fmt, native/WASM/workspace all-targets Clippy, workspace build, suite manifest 검사 통과.
- 전체 Rust nextest 9,407 통과(46 skipped). native-skia 주 lib 3,930 통과(13 ignored),
  보조 lib 182 통과.
- Chrome PDF 회귀의 HWP/HWPX 저장·재열기·실제 링크 클릭·5개 PDF 캡처 완료.
  기본 Python에 pypdf가 없어 마지막 검증 스크립트가 실패했으나, 준비된 runtime Python으로
  동일 출력의 검증을 재실행해 34쪽·81주석, 최대 오차 0.381pt 미만을 확인했다.
- 추가 Skia integration: placeholder 2개·직접 PDF 출력 4개 통과.

이미 잘못 확장되어 저장된 링크 범위는 사용자 의도를 추측해 자동 축소하지 않는다.
