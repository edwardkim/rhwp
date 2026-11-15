# 최종 제품 8ad1c5c08a의 직접 판독

동일 영역·동일 배율의 독립 한컴 Print, Native, 최적화 fresh WASM을 직접 대조했다.
[원본 artifact 영수증](receipts.json)에는 원 페이지 PNG의 SHA-256과 표시용 crop을 기록한다.
전체 쪽 TSV의 측정 영역·2px 반경·raw 판정은 변경하지 않았다.

- [괘선 specimen](linechar-specimen.png): 7집합의 가는 선·굵은 선·겹줄과 교차점, 일반 기호 7개의 의미·순서·전진을 확인했다. Skia와 브라우저의 안티앨리어싱 색상 차이는 남지만 다른 선 종류로 바뀌거나 글자가 빠지는 현상은 없다.
- [바탕쪽 2쪽](master-both-even-odd-page2.png): 짝수 쪽의 실제 물리 용지 기준 막대, MP-EVEN과 본문 page 2의 소속·위치를 확인했다.
- [표 뒤 문단 3쪽](color-page3.png): 두 문단의 3쪽 소속·순서·줄 간격과 배경색 없음의 보존을 확인했다.
- [영문 스타일](helvetica-styles.png): regular·bold·italic·bold italic의 글리프 모양·간격을 확인했다. 실제 대체 face 공급이 네 스타일에 대응한다.
- [그림 외곽](picture-frames.png): 두 그림의 네 변과 내부 이미지·crop, 그림 위에 그려지는 소유 테두리를 확인했다. 독립 PDF의 물리 띠·외곽 pen과 공통 Print 장치 소비 근거는 [조사 이력](../../protected-environment-review.md)에 있다.
- [중첩 표 8쪽](nested-symbol-page8.png)·[10쪽](nested-natural-page10.png)·[12쪽](nested-overlap-page12.png): 원문 기호·본문·밑줄·줄바꿈·셀 포함과 내용 보존을 확인했다. 4·11·14쪽의 별도 확대 판독은 [TTF 형식 증거](../../supplied-ttf-frame/README.md)에 있다.

## 최신 devel 기능의 감소 쪽 대조

[CBTA 45쪽](incoming-cbta-045.png), [영문 시험 3쪽](incoming-exam-003.png),
[RowBreak 1쪽](incoming-rowbreak-first-fragment-001.png)은 독립 Print, exact base의 Native,
후보 Native, exact base의 WASM, 후보 WASM 순서다. [영수증](incoming-review-receipts.json)은
같은 영역·배율과 원 페이지 해시를 보존한다. 차이가 큰 영역을 표시용으로
선택했으며 전체 페이지 측정에는 crop을 적용하지 않았다.

글리프의 미세한 채움·위상 차이는 있지만 새 줄바꿈·이월·누락·중복·셀 침범을 발견하지 않았다.
세 쪽의 실제 export render tree는 base와 후보가 JSON·바이트 해시 모두 같았다.
이 증거의 범위는 실제 export가 제공하는 문단·셀·글자의 bbox·내용·순서이며,
paint 색상까지 같다는 주장으로 확대하지 않는다. 정밀 raw 비교는 별도 전쪽 TSV의
Decimal 값으로 수행했고 2%p 초과 하락은 없다. 수용 예외나 기준 변경은 적용하지 않았다.


## 원 사례의 최신 기준 감소 쪽

[보도자료 5쪽](original-incoming9026-repeated-005.png)과
[계획서 1쪽](original-incoming9016-001.png)을 같은 다섯 열로 직접 확대 판독했다.
[영수증](original-decrease-review-receipts.json)과 [실제 렌더 트리 대조](original-decrease-geometry.json)를
보존한다. 두 쪽의 문단·셀·글자 bbox·내용·순서는 base와 같다. 글리프 모양·장식·줄 간격·겹침·
누락을 확인했고 새 결함은 발견하지 않았다. 안티앨리어싱 채움의 작은 차이를 RGB 동일로 기록하지 않는다.
원 사례 정밀 비교의 최대 감소는 0.15084%p이며 2%p 초과 하락은 0쪽이다.
