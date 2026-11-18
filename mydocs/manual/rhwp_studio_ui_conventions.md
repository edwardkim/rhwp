---
kind: reference
status: active
canonical: mydocs/manual/rhwp_studio_ui_conventions.md
last_verified: 2026-08-17
---

# rhwp-studio UI 명칭과 CSS 접두어

코드, 이슈, PR, 검증 문서에서 rhwp-studio의 UI 영역을 아래 명칭으로 통일한다.

| 한국어 명칭 | HTML id | 설명 |
| --- | --- | --- |
| 메뉴바 | `#menu-bar` | 파일·편집·보기·입력·서식·쪽·표 메뉴 |
| 도구 상자 | `#icon-toolbar` | 명령 아이콘과 라벨 버튼 모음 |
| 서식 도구 모음 | `#style-bar` | 스타일·글꼴·크기·정렬 등 서식 제어 |
| 편집 영역 | `#scroll-container` | 문서 페이지 렌더링과 스크롤 영역 |
| 눈금자 | `#h-ruler`, `#v-ruler`, `#ruler-corner` | 가로·세로 눈금과 여백·들여쓰기 핀 (#4977) |
| 상태 표시줄 | `#status-bar` | 쪽·구역·편집 모드·확대 배율 표시 |
| 오른쪽 패널 | `#side-panel-host` | 플러그인이 등록하는 도킹 패널 자리 (AI 도우미 등) |

## CSS 접두어

| 접두어 | 대상 |
| --- | --- |
| `tb-` | 도구 상자 요소 |
| `sb-` | 서식 도구 모음 요소 |
| `stb-` | 상태 표시줄 요소 |
| `md-` | 메뉴바 드롭다운 요소 |
| `dialog-` | 대화상자 공통 요소 |
| `dialog-pdf-import` | PDF 자원 프로필·유효 한도·진행·취소·재시도 (#8174) |
| `dialog-link-destination` | 로컬 파일·등록 앱 링크의 호스트 위임과 브라우저 안내 (#8174) |
| `dialog-hyperlink-` | 하이퍼링크 연결 대상·입력·확인 대화상자 |
| `cs-` | 글자 모양 대화상자 |
| `ps-` | 문단 모양 대화상자 |
| `skin-onboarding-` | 첫 실행 스킨 선택 대화상자 |
| `chart-data-` | 차트 데이터 편집 대화상자 (#4694) |
| `hf-` | 머리말/꼬리말 대표 편집 영역·타겟 표시 |
| `spl-` | 오른쪽 패널 자리의 머리·경계·본문 |
| `ai-` | AI 도우미 패널과 대화상자 |

새 UI 영역이나 접두어를 도입할 때는 기존 DOM과 CSS에서 실제 사용 여부를 확인하고 이 표를 함께
갱신한다.

## 대화상자의 화면 경계와 스크롤

`dialog-wrap`은 viewport 안에서 세로 flex로 배치한다. 제목과 확인/취소 영역은 축소하지 않고,
`dialog-body`가 남은 높이에서 스크롤한다. 모바일의 하단 시트와 `90dvh` 상한도 같은 계약을 따른다.
글자·문단·개체 모양처럼 옆에 버튼이 있는 창은 `cs-main-row`와 `cs-left-col`의 `min-height: 0`을
유지해 실제 본문까지 축소가 전달되게 한다. 탭은 가로 스크롤하며 버튼 영역을 덮지 않는다.

복구 창은 기본적으로 후보 목록만 스크롤한다. 안내문과 후보 한 줄도 함께 들어가지 않는 낮은
화면에서는 본문 스크롤로 안내문과 목록에 접근한다. 표·그림 속성의 전용 패널 스크롤은 유지한다.
새 창은 제목/footer를 본문 안에 넣거나 wrap 전체 스크롤로 바꾸지 않는다. 배경 클릭 닫힘은
기존 `ModalDialog`의 명시 opt-in 정책이며 화면 크기 처리와 별개다.

`npm --prefix rhwp-studio run e2e:dialog-viewport`로 실제 Studio에서 작은 높이·767/768px 경계·
화면 회전·입력 보존·확인/취소를 검증한다. 이 검사는 문서 조판 Visual Sweep을 대신하지 않는다.

## 눈금자 핀 명칭

눈금자 핀은 **자기가 바꾸는 문서 필드의 이름**으로 부른다. 같은 축에 같은 모양의 핀이 여러 개
있어 모양이나 위치로 부르면("아래 핀", "왼쪽 삼각형") 어느 값이 움직였는지 말로 구분되지 않는다.

| 핀 | 모양·위치 | 바꾸는 값 |
| --- | --- | --- |
| 쪽 왼쪽/오른쪽 여백 | 가로 눈금자 아래쪽 △ | `PageDef.marginLeft` / `marginRight` |
| 첫 줄 들여쓰기 | 가로 눈금자 위쪽 ▽ | `ParaShape.indent` |
| 쪽 위/아래 여백 | 세로 눈금자 ▽ / △ | `PageDef.marginTop` / `marginBottom` |

문단 좌우 여백(`ParaShape.marginLeft`/`marginRight`)은 눈금자에 두지 않는다 — 쪽 여백과 같은 축·
같은 모양이라 눈금자만 봐서는 주인을 알 수 없다. 문단 모양 대화상자에서 조정한다.
