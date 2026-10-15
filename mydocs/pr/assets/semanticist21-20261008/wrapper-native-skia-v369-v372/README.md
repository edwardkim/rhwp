# 실제 Native Skia 래퍼·각주 출력

생산 소스940ea132의 실제 `native-skia` CLI를 고정 target/pr-review에서 빌드했고
exit0을 확인했다. 빌드590.681초, 바이너리 SHA 및 같은 입력/Print의 해시는
[출처 원장](provenance.json)에 기록한다. SVG를 raster한 PNG를 Native Skia로 부르지 않는다.

96dpi / print profile / compat2022 / 기존 일곱 공급 font path로11개 원본·대조군14쪽을
모두 출력했다. 독립 한컴 Print의 같은14쪽에 대해 전체 TSV를 확인했으며
최저91.21234%, 90% 미만·누락0이다. 래퍼7건은 모두100%다.
Native/fresh SVG의 기존17쪽 검증과 별도의 backend 관측이다.

[child-flow review](child-flow-p001-review.png),
[child-inline review](child-inline-p001-review.png),
[원본 review](current-05-17a174162fd3-p001-review.png),
[각주 뒤 본문2쪽 review](current-04-6b4f6a375029-p002-review.png)를 직접 열었다.
자식 폭을 유지한 채 흘림/글자취급 경로가 부모 밖을 그리지 않고,
일반 자식은 자기 표시 영역을 유지한다. 본문·각주·footer의 쪽 소속이 대응한다.
Latin 글자의 획·자간 차이는 남으며90% 통과로 면제하지 않는다.
세 래퍼의 오른쪽 괘선과 앞/뒤 글자를3배 확대해 같은 Print와 직접 대조했다.
부모에서 끝나는 흘림/글자취급 경계와 제한 없는 원본의 자기 표시 영역이 대응한다.
Print stroke0.36pt는 원본0.12mm와 대응하며 raster AA의 농도 차이는 남는다.
다른 변경 경로와 Latin 글자의 상세 backend 수용은 별도로 확인한다.

최초 overlay caption의0-based page 인자 오류를 바로잡고 네 review를 다시 생성·판독했다.
보존하는 이미지는 모두 compare와 overlay의 실제 쪽 번호가 같다. 입력·raster·TSV는 변하지 않았다.

실행·전체 TSV·로그·14개 actual PNG는 ignored
`output/pr-review/semanticist21-20261008/wrapper-native-skia-{png-v369,compare-v372}/`에 있다.
이 결과를 차트 내부·직접3문서의 최종 전쪽·전체 회귀 완료 또는18개 PR 수용으로 확대하지 않는다.
