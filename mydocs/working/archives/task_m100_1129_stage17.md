# Task M100 #1129 Stage 17 — 쪽 테두리/배경 메뉴 추가

## 배경

작업지시자가 한컴오피스 도움말의 `쪽-쪽 테두리/배경` 문서를 기준으로 rhwp-studio에 해당 메뉴가 있는지 확인하고, 없으면 생성하도록 요청했다.

참고 문서:

- `https://help.hancom.com/hoffice130_assistant/ko-KR/Hwp/index.htm#t=format%2Fpageborder%2Fpage_border.htm`
- `https://help.hancom.com/hoffice130_assistant/ko-KR/Hwp/index.htm#t=format%2Fpageborder%2Fpage_border(border).htm`
- `https://help.hancom.com/hoffice130_assistant/ko-KR/Hwp/index.htm#t=format%2Fpageborder%2Fpage_border(background).htm`
- `https://help.hancom.com/hoffice130_assistant/ko-KR/Hwp/index.htm#t=format%2Fpageborder%2Fpage_border(facecolor).htm`
- `https://help.hancom.com/hoffice130_assistant/ko-KR/Hwp/index.htm#t=format%2Fpageborder%2Fpage_border(pattern).htm`
- `https://help.hancom.com/hoffice130_assistant/ko-KR/Hwp/index.htm#t=format%2Fpageborder%2Fpage_border(gradation).htm`
- `https://help.hancom.com/hoffice130_assistant/ko-KR/Hwp/index.htm#t=format%2Fpageborder%2Fpage_border(picture).htm`
- `https://help.hancom.com/hoffice130_assistant/ko-KR/Hwp/index.htm#t=format%2Fpageborder%2Fpage_border(odd).htm`

## 현재 확인

- rhwp-studio에는 `보기-격자 보기/격자 설정`, `셀 테두리/배경`, `문단 모양-테두리/배경`, `글자 모양-테두리/배경` UI가 있다.
- `쪽` 메뉴에는 `쪽 테두리/배경` 전용 항목이 없다.
- 내부 모델에는 `SectionDef.page_border_fill`, `extra_page_border_fills`, `DocInfo.border_fills`가 있어 쪽 테두리/배경 데이터를 표현할 수 있다.
- WASM 브리지에는 `getPageDef/setPageDef`, `getSectionDef/setSectionDef`만 있고 `PageBorderFill` 전용 조회/변경 API는 없다.
- 한컴 도움말은 메뉴 구조와 용어 확인용이며, 실제 화면 배치와 `쪽 기준/종이 기준` 판정은 작업지시자가 제공한 한컴오피스 화면을 우선한다.

## 구현 범위

1. `쪽` 메뉴에 `쪽 테두리/배경` 항목을 추가한다.
2. `page:page-border` 명령을 추가한다.
3. 한컴 도움말의 기본 구조를 따르는 `쪽 테두리/배경` 다이얼로그를 추가한다.
   - 테두리/배경 종류: 양쪽, 홀수 쪽, 짝수 쪽
   - 테두리 탭: 선 종류, 굵기, 색, 바로 적용, 사용 안 함, 미리보기 방향
   - 위치: 종이 기준, 쪽 기준, 좌/우/상/하 오프셋
   - 적용 쪽: 모두, 첫 쪽 제외, 첫 쪽만
   - 배경 탭: 색 채우기 없음, 색, 채울 영역
4. 현재 구역의 `PageBorderFill`과 연결된 `BorderFill`을 조회/변경하는 WASM API를 추가한다.
5. 적용 후 재조판/재렌더 이벤트를 발생시킨다.

## 검증 계획

- `wasm-pack build --target web --out-dir pkg`
- `npm run build`
- 로컬 Playwright 기능 검증
  - `samples/hwp3-sample16-hwp5.hwp` 로드
  - `쪽 > 쪽 테두리/배경` 메뉴가 열리는지 확인
  - 쪽 기준/종이 기준 및 오프셋 변경 후 `getPageInfo(0).pageBorder*` 변화 확인
- `cargo test --lib`
- `cargo fmt --all -- --check && git diff --check`

## 단계 보고

- 구현 전 문서 작성 완료.
- `쪽` 메뉴에 `쪽 테두리/배경` 항목과 `page:page-border` 명령을 추가했다.
- `PageBorderDialog`를 추가해 한컴오피스 `쪽 테두리/배경`의 `테두리`/`배경` 탭 구조를 제공했다.
  - 초기 구현 후 작업지시자 피드백에 따라 간이 UI 형태를 수정했다.
  - `쪽 기준` 라디오 라벨 누락을 수정했다.
  - 테두리 탭은 한컴 화면처럼 좌측 설정, 우측 미리보기/방향 버튼, 위치, 적용 쪽, 적용 범위, 대화 상자 설정 행으로 재배치했다.
  - 배경 탭은 색 채우기 없음, 색, 그라데이션, 그림, 채울 영역, 적용 쪽, 적용 범위 행을 한컴 화면과 같은 용어로 노출했다.
- WASM에 `getPageBorderFill`/`setPageBorderFill` API를 추가했다.
- `DocumentCore` 회귀 테스트 `page_border_fill_api_updates_basis_spacing_and_border`를 추가했다.

## 검증 결과

- `wasm-pack build --target web --out-dir pkg`: 통과
- `npm run build`: 통과
- 로컬 Playwright 기능 검증: 통과
  - `samples/hwp3-sample16-hwp5.hwp` 로드
  - `쪽 > 쪽 테두리/배경` 메뉴 활성 확인
  - 다이얼로그 `테두리`/`배경` 탭 확인
  - `종이 기준`, `쪽 기준`, `머리말 포함`, `꼬리말 포함`, `그라데이션`, `그림`, `채울 영역` 텍스트 노출 확인
  - `쪽 기준`/간격 변경 후 `getPageInfo(0).pageBorderTop` 변화 확인
- `cargo test page_border_fill_api_updates_basis_spacing_and_border --lib`: 통과
- `cargo test --lib`: 통과 (`1399 passed; 0 failed; 6 ignored`)
- `cargo fmt --all -- --check && git diff --check`: 통과
