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

## CSS 접두어

| 접두어 | 대상 |
| --- | --- |
| `tb-` | 도구 상자 요소 |
| `sb-` | 서식 도구 모음 요소 |
| `stb-` | 상태 표시줄 요소 |
| `md-` | 메뉴바 드롭다운 요소 |
| `dialog-` | 대화상자 공통 요소 |
| `cs-` | 글자 모양 대화상자 |
| `ps-` | 문단 모양 대화상자 |
| `chart-data-` | 차트 데이터 편집 대화상자 (#4694) |

새 UI 영역이나 접두어를 도입할 때는 기존 DOM과 CSS에서 실제 사용 여부를 확인하고 이 표를 함께
갱신한다.

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
