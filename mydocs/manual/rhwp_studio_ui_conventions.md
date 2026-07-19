---
kind: reference
status: active
canonical: mydocs/manual/rhwp_studio_ui_conventions.md
last_verified: 2026-07-17
---

# rhwp-studio UI 명칭과 CSS 접두어

코드, 이슈, PR, 검증 문서에서 rhwp-studio의 UI 영역을 아래 명칭으로 통일한다.

| 한국어 명칭 | HTML id | 설명 |
| --- | --- | --- |
| 메뉴바 | `#menu-bar` | 파일·편집·보기·입력·서식·쪽·표 메뉴 |
| 도구 상자 | `#icon-toolbar` | 명령 아이콘과 라벨 버튼 모음 |
| 서식 도구 모음 | `#style-bar` | 스타일·글꼴·크기·정렬 등 서식 제어 |
| 편집 영역 | `#scroll-container` | 문서 페이지 렌더링과 스크롤 영역 |
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

새 UI 영역이나 접두어를 도입할 때는 기존 DOM과 CSS에서 실제 사용 여부를 확인하고 이 표를 함께
갱신한다.
