# 직접 판독과 최종 재출력의 대응

확인 시각 `2026-11-04T00:25:05.934988+09:00`. 최종 Native와 fresh WASM은 각각 31개 입력·71쪽을 새로 출력했다. 직접 판독한 이전 출력과 각 페이지의 RGB 픽셀을 대조해 62개 입력/backend 조합·142쪽 모두 차이가 0이다. 이전 build head는 `25607fc28d0555caf0ef9b053e181c7236569c18`, 최종 검증 구현 head는 `c6ad3eea59`다.

최종 출력 실행과 대표 패널 재사용을 구분한다. 아래 패널은 이전 출력으로 합성해 직접 읽었으며, 최종 출력 픽셀이 같아 같은 판독 근거를 유지한다. 원시 raster와 render-tree·run JSON을 중복 커밋하지 않는다.

| 대표 패널 | 직접 확인한 의미 |
| --- | --- |
| [form-qualified.png](zoom/form-qualified.png) | 명령·선택 상자·계절 선택·라디오 단추의 바탕 글꼴, 배치와 경계를 같은 프로그램으로 확인했다. 작은 AA/획 두께 차이는 별도 raw 측정에 남긴다. |
| [forms-edited.png](zoom/forms-edited.png) | 본문 5+3개, 셀 4+4개 라디오 단추가 네 물리 행에 한 번씩 보인다. 부모 테두리·각 줄 순서·양식 내부의 원을 독립 Print와 대조했다. |
| [polygon-90.png](zoom/polygon-90.png) | 열린 3점 다각선의 방향·꼭짓점·중심과 열린 변을 독립 Print와 대조했다. |
| [polygon-resize.png](zoom/polygon-resize.png) | 30도·너비 변경의 긴 두 선과 만나는 꼭짓점, 본문과의 상대 위치를 확인했다. |
| [spacing.png](zoom/spacing.png) | 중간 빈 문단과 후속 문단의 세로 간격·앞뒤 문단 순서를 확인했다. |
| [header.png](zoom/header.png) | 70자 두 줄과 빈 문단 뒤의 마지막 머리말이 본문 첫 줄 위에 겹치는 원본 의도와 내용 보존을 확인했다. |
| [split-row-p16.png](zoom/split-row-p16.png) | 분할 칸 끝의 공공데이터 포털 관련 두 줄과 칸 경계, 다음 내용의 순서·누락·중복을 확인했다. |
| [notice-p12.png](zoom/notice-p12.png) | 공통 적용사항 제목·본문 줄바꿈·작은 괄호 글자와 별표 간격, 다음 동시수행과제 제한 제목을 확인했다. |
| [notice-p13.png](zoom/notice-p13.png) | 공통 사항과 지원 기준 사이의 본문 줄바꿈·번호·문단 간격·내용 순서를 확인했다. |
| [notice-tail-p21.png](zoom/notice-tail-p21.png) | 가점 사항 표의 산·학·연 본문과 작은 별표 행·다음 본문의 순서·셀 안 표시를 대조했다. |
| [grow-after.png](zoom/grow-after.png) | 가운데 행만 성장하고 위·아래 행과 표 외곽이 독립 Print와 같은 관계를 유지한다. |
| [grow-nested.png](zoom/grow-nested.png) | 중첩 표의 늘어난 행·다음 아래 행과 부모 clip 경계를 독립 Print와 대조했다. |
| [nested-wide.png](zoom/nested-wide.png) | 선언 폭을 줄이지 않은 여러 겹 표의 가로 경계와 조상 셀에 의한 clipping을 확인했다. |
| [no-lineseg-wrap.png](zoom/no-lineseg-wrap.png) | 그림 단독 마지막 줄의 세로 밴드와 본문 순서, Print의 계단형 sampling과 backend 보간을 구분했다. 끝 캐럿은 문단 글자 모양과 같은 높이를 소비한다. |

변경 영역에 수식은 없다. 작은 문자·괄호·별표·숫자·줄 사이·후속 본문 간격을 같은 배율에서 읽었다. 100% raw도 같은 픽셀이라는 뜻은 아니므로 독립 PDF와의 육안 판독을 점수와 별도로 기록했다. 위 0픽셀 차이는 기준 PDF와의 차이가 아니라, 판독한 후보 출력과 최종 재출력 사이의 차이다.

## 실제 저장 입력

| 입력 | Native/WASM SHA-256 | 연결한 독립 Print |
| --- | --- | --- |
| [notice-edited.hwpx](saved/notice-edited.hwpx) | `567ec4aabba351a3936285479ff31d5e8d11570a033e770e1662347f7b7ce3c0` | 기존 table-insertion-flow/notice-hancom-print.pdf, 24쪽 |
| [grow-nested-edited.hwpx](saved/grow-nested-edited.hwpx) | `4eb9db7414cd35336ec1a965e62b45783af477bcb150fae05fe7ce86b0e5717c` | grow-nested-2024.pdf, 1쪽 |

중첩 대조군의 원래 편집 입력은 잘못된 child 선언 높이 8502 HU를 가진 상태로 보존했다. 독립 한컴 Print는 그 내용의 실제 성장 배치를 보여 준다. 최종 저장본은 같은 내용·배치를 유지하면서 부모·자식 common height를 각각 11748 HU로 기록한다. source cellSz와 무편집 8502 HU는 유지한다. 실제 저장본을 양쪽 backend로 교차 재열기해 같은 독립 Print와 전쪽 비교했다.
