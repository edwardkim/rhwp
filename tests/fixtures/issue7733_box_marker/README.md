# 실제 제공 프로그램의 glyph hit 대조군

`program-present.ttf`는 FontBuilder로 만든 MIT 합성 TrueType이다.
U+257A와 `.notdef` 두 glyph를 가지며 em 1000, 표식 전진은 750이다.
검사에서 `BatangChe` 별칭으로 직접 공급하여 실제 프로그램의 glyph hit가
정적 기준 프로그램의 cmap 결측 정보보다 우선하는지 확인한다.

`venv/bin/python tests/fixtures/issue7733_box_marker/generate.py`로 재생성한다.
독립 한컴 Print나 실문서의 좌표·글리프 원본을 대신하는 대조군은 아니다.
