# RHWP 호스트 글꼴 검증 픽스처

`scripts/generate_host_font_fixture.py`가 생성한 독창적인 합성 윤곽선이며 저장소의 MIT
라이선스로 배포한다. A와 가에 직사각형/기울어진 평행사변형을 매핑한다. 한컴의 출력 정답지가
아니라 face 선택, TTC index, 합성 기울임 중복 방지를 확인하는 계약 입력이다.

TTC의 0/1/2번 face는 각각 Regular/Italic/Oblique다. 별도 TTF는 TTC 추출 구현을 거치지 않는
독립 입력이며, 실제 CanvasKit의 glyph 출력으로 TTC 선택 결과와 비교한다.
