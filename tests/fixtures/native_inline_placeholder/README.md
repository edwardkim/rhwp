# Native inline 자리표시자 합성 계약

기존 OFL A subset에서 윤곽·전진폭은 바꾸지 않고 네 codepoint를 같은 A에 대응했다.
U+FFFC가 실제 보이는 글리프를 갖게 하여 host fallback에 무관하게 잘못된 재생을 검출한다.
U+FFFD는 숨기면 안 되는 대조군, U+0915는 복합 shaping 소비 경로를 검증한다.
실제 한컴 문서의 글꼴·좌표·픽셀 정답지가 아니며 제품 글꼴로 공급하지 않는다.
원본·산출 SHA와 이름·매핑은 provenance.json, OFL은 OFL.txt에 있다.
