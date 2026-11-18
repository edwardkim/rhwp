# 글머리표의 문자 칸과 실제 인스턴스 폭

공식 사양 표 40 bit2는 실제 인스턴스 문자열의 너비를 사용할지 구분한다.
기존 공통 기하는 이 bit를 글머리표에서 무시하고 비ASCII 글자를 모두 1em,
ASCII는 항상 실제 glyph 폭으로 처리했다. 공식 형식 문서 67쪽의 `·` 목록은 bit2=0,
상대 거리 50%인데 본문이 19px 뒤에서 시작해 독립 PDF의 약 12.67px보다 밀린다.

원본 공식 PDF는 변경하지 않았다. 별도 공개 합성 HWPX의 `·`/`-`/`○` × bit2 0/1,
같은 글꼴·크기·50% 거리의 6문단을 한컴 2024 MCP Print로 출력했다. 영수증의 실제 제품은
13.0.0.3901, direct DLL host, Print method 0, frame_print_to_pdf_ex_one_up, preprocess none다.
유효한 생성 입력과 1쪽 PDF를 정식 경로에 보존한다. 앞선 비연속 paraPr id 생성기 오류의
PDF는 output의 INVALID로 분리했으며 대조 증거로 쓰지 않는다.

독립 Print에서 bit2=0의 ·/-는 본문을 1em(반각 칸+50% 거리), ○는 1.5em 앞으로 둔다.
bit2=1은 같은 glyph의 자연 폭에 50% 거리를 더한다. 표시 glyph와 예약 칸의 소유를 구분해
공통 줄 채움·배치가 동일한 결과를 사용하도록 보정 중이다. 기존 Wingdings PUA의 전각
예약은 유지한다. Native/fresh WASM의 새 후보 전쪽 결과와 회귀 검증은 아직 진행 중이다.
