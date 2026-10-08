# 저장 셀 프레임과 편집 후 자동 페이지 경계 대조군

비공개 문서·문구·그림은 포함하지 않는다. 저장 좌표는 XML에 손으로 입력하지 않았다.
공개 `samples/issue5818/cell_square_logo_text_wrap.hwpx`의 문서/셀/그림 구조와
공개 `rhwp-studio/public/icons/icon-128.png`를 사용해 새 셀 내용을 생성했다.
본문 대조군은 `samples/issue2527_empty_linesegs.hwpx`의 공개 스타일로 새 문구를 썼다.
생성 입력에 LineSeg를 넣지 않고 Hancom 2020에서 HWP, HWPX 순서로 저장했다.
각 최종 HWPX를 같은 엔진의 Print PDF로 인쇄했다(11.0.0.9136, method 0, one-up).

- `cell-negative`, `cell-zero`, `cell-positive`: 가운데 정렬 셀의 문단 기준
  Square/flowWithText 그림. 수직 offset -1200/0/+1200 HU. 한컴은 음수와 0의
  흐름 원점을 같게, 양수에서는 그림과 후속 제목 프레임을 함께 아래로 배치한다.
- `cell-overlay`: 같은 음수 offset의 BehindText 대조. 그림의 배경 배치와
  흐름 비점유를 유지한다. 일반 흐름 그림과 같은 높이를 예약하지 않는다.
- `cell-fragments`: 그림의 가로 offset 2000 HU로 실제 한컴 저장에서 같은
  vpos의 가로 조각 2개를 생성한다. 다음 TAC 제목은 그림 뒤에 놓인다.
- `body-original`: 첫 문단 뒤의 41줄 본문, 빈 문단 2개, 후속 문단 2개. 저장본과 Print 2쪽.
- `body-reflow`: 원문 본문만 짧게 편집하고 해당 문단 LineSeg를 제거한다. 뒤 저장
  좌표는 원문 그대로이며 명시적 나누기는 없다. 독립 Print는 빈 줄을 유지한 1쪽.
- `body-single-symbol`: 같은 편집 경로에서 선두 공백+단독 기호 줄과 별표 본문을
  재조판한다. 독립 Print는 3줄과 뒤의 빈 문단/본문을 모두 같은 쪽에 둔다.
- `body-explicit`: 짧게 편집한 대조군의 후속 문단에 pageBreak=1을 지정한다. Print 2쪽.

PDF는 위치·줄 구성의 독립 근거다. RenderTree 검사는 셀 포함·앞뒤 순서와
문단/빈 줄의 소유·내용 누락/중복을 확인하며 절대 픽셀이나 전체 SVG 해시를 고정하지 않는다.
자동 경계를 복구하는 구역 진입·whole-fit·split-entry는 같은 유효성 판단을 소비한다.
명시적 나누기는 선행 entry에서 처리한다. 여러 줄 재조판 대조군은 최초 보정의
두 번째 fit 경로 오류까지 검출했다.

해시·생성 계약 및 시각 검증 source는 `hancom-evidence.json`에 기록한다.
실행 로그·PNG·TSV는 ignored output에 보존한다. 이 자료는 원 PR의 다른 문서
시각 게이트를 대신하지 않는다.
