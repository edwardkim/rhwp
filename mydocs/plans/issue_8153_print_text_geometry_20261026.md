# #8153: Print 글리프의 원본 기하와 래스터 격자

## 사양·원본·독립 출력

[HWP 5.0 사양 설명본](../tech/한글문서파일형식_5.0_revision1.3.md)의 기본 단위는
1/7200인치이며, 출력 해상도가 가변적이므로 크기를 장치 도트로 표현하지 않는다고
명시한다. 이 설명이 glyph hinting의 금지 명령이라고 확대하지 않는다. 원본의
줄·글자 소수 좌표를 Print에서 보존할 수 있는 래스터 소비 계약을 별도로 확인한다.

두 편람 239쪽의 실제 저장 글꼴은 KoPub돋움체 Light, 10pt·장평 98%·자간 -3%다.
현재 source의 실제 TextStyle은 13.333333px·ratio 0.98·letter_spacing -0.4px,
글자 `편`의 원점은 (142.48,336.92)다. Native trace의 원 프로그램은 KoPubDotum,
문자 폭은 원본 872HU에 장평·음수 자간을 적용한 828HU다.

고정 `b8ca19e50d`의 표준 최적화 WASM은 같은 글꼴을 실제 공급해도 `편`·`반`의
아래 획에 2픽셀이 남는다. 실제 exporter의 사본에서 기본 auto 출력이 기존 결과와
RGB 전 픽셀까지 같음을 먼저 확인했다. 좌표·글꼴·clip을 유지하고 표준
geometricPrecision을 적용하면 해당 획이 복원돼 2→0픽셀이다. HWPX 전 383쪽
진단은 321→324쪽이 미대응 0, 총 미대응은 3124→2948이다. 기존 100% 쪽의 이탈은
없지만 5·59·80·136·145·280쪽에 작은 증가가 있어 최종 검증으로 승격하지 않는다.

[HTML Canvas textRendering](https://html.spec.whatwg.org/multipage/canvas.html#dom-context-2d-textrendering)은
[SVG text-rendering](https://www.w3.org/TR/SVG2/painting.html#TextRenderingProperty)의 의미를 따른다.
기하 정밀도 모드는 윤곽의 기하를 우선한다. Chromium의 Linux/Cros 참조 구현은
subpixel positioning을 켜고 hinting을 끈다. 브라우저 실제 실행과 원문 snapshot은
ignored `p239-text-rendering-prototype/`에 있다. main snapshot을 설치본의 정확한
revision이라고 주장하지 않는다.

Native도 별도로 조사했다. 고정 `cb8d552a7d`의 공개 Native renderer로 실제 원본을
재생한 PNG는 CLI와 RGB 전 픽셀이 같다. 같은 실제 공급 글꼴을 읽기만 한 작은
Skia 진단의 기본 모드는 `편`·`반`의 y=326..337 foreground crop과 정확히 같다.
y=338은 뒤에 그리는 원본 아래 채움 그림의 영역이므로 단일 glyph 진단과 다르다.
수정·복사한 폰트는 없다. 실제 음수 자간에서는 기존 폭 fit이 None이라는 조건도
확인하며, 이 조건을 빠뜨린 초기 no-hint fit 진단을 최종 결과로 사용하지 않는다.

## 공통 소비 계약과 반례

`RenderProfile::Print → 원본 기하 보존 정책 → Native raster Font/Canvas2D
textRendering → 기존 glyph 원점·폭·clip·그리기 순서`로 연결한다. Native는
해당 raster Font의 hinting과 baseline snap을 해제하고, Canvas는 같은 Print 정책을
표준 geometricPrecision으로 전달하는 방향을 검증한다. 소스 줄 구성·스타일·좌표·
메트릭 DB를 바꾸지 않는다. 화면의 기존 grid 계약과 벡터 기록 경로는 유지한다.
문서·쪽·글꼴·크기에 따라 이 정책을 골라 켜지 않는다.

먼저 공개 Native raster에서 일반 TTF의 Print 1/4px 이동과 화면 격자 유지,
실제 WASM Canvas의 Screen/Print 반복 전환을 검사하는 정식 원문의 수정 전 FAIL을
확보한다. 합성 API 계약 검사를 한컴 문서 정확도의 증거로 대신하지 않는다.
수정 뒤 같은 원문·기존 105건·기존 소수 원점 대조와 실제 두 원본의 영향·전쪽
Native/fresh WASM, 알려진 작은 증가 쪽을 모두 대조한다. 기존의 정확한 화면 검사를
Print에 맞춰 바꾸지 않는다. 새로 드러난 차이는 원인 보정과 독립 출력으로 해결한다.

현재는 구현 전 계획이며, 383쪽 진단만으로 Native/HWP/최종 WASM을 완료 판정하지
않는다. 원본·PDF·임계값 232·이웃 반경 2를 유지하고, 전 1532조건의 raw 100%·
정수 미대응 0과 필수 테스트를 마친 후 PR 준비까지만 진행한다. upstream/devel
동기화·원격 게시·push·PR 제출은 하지 않는다.
