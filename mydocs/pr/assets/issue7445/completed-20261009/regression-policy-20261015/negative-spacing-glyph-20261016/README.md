# 확인된 글리프 폭과 음수 배분 간격 (2026-10-16)

Issue: #7445. 현재 기준 `cdef0073bb24512daf0622dec5051ac62cb8d57e`, 수정 전 `79181f759d8008f43dd2f092947bc0185ac07762`.

## 원인과 소비 경로

원본 `samples/task2097/75544_pii_bunseok.hwpx`의 표219/셀2/문단9는 글자 자간0,
배분 간격−0.393939CSSpx다. 독립 한컴2020 Print
`pdf/task2097/75544_pii_bunseok-hwpx-2020.pdf`20쪽의 H2mjsM '개'는 폭18.72CSSpx,
다음 원점 전진18.56087CSSpx로 글자 폭과 시작 위치의 간격이 다르다.
기존 Native는 전진18.272727CSSpx로 글리프까지 압축했다.

`glyph_fit_positions`는 음수 간격의 최소 전진폭 clamp를 역산하지 않고 기존 측정기에
간격0인 스타일을 전달해 폭 맞춤 경계를 다시 만든다. 원래 문자 원점·줄 폭·조판은 보존한다.
SVG `textLength`, Canvas2D `scaleX`, Native Skia의 최종 글리프 폭은 이 결과를 소비한다.
양수 배분 간격 제거·커닝 전 경계 계약은 유지한다. 문서/글꼴 이름별 분기는 없다.

## 시각 검증

동일 원본·독립 Print·명시 글꼴 환경으로 Native/fresh WASM 각각 전66쪽을 재출력했다.
양쪽 쪽수66/66, 최저93.77937%,90% 미만0쪽이다. Mac WASM은 locked wrapper의
`--target web --out-dir pkg --no-opt` 로컬 대체 빌드다.

|20쪽|수정 전|수정 후|고정 devel|
|---|---:|---:|---:|
|Native|98.16041%|98.18414%|98.19083%|
|fresh WASM|98.16108%|98.18481%|98.16902%|

Native의 devel 대비0.01%p 이상 하락은 전66쪽에서0쪽이다.
WASM은 다른 원인의9/10/28/29/50/52/53쪽 하락이 남아 있어 PR 완료로 판정하지 않는다.
전쪽 TSV/명령/입력·PDF·글꼴 hash 정본은 ignored output의
`regression-policy-audit-20261015/rebase-cdef0073-20261016/c72-whole-pii-*`에 보존한다.
Native binary SHA256 `198f77a34cd9eaf0b9130c52ac555f7c760149e184e39741405519a4dffb9f64`,
WASM SHA256 `b17f2fde41c551d8033e6e3300532697a4b1b762ac84be075245c429252c0605`.

18쪽 rowbreak 문서의 선택4쪽 캡처에서는 숫자 모양 차이가 나타났지만,
같은 전쪽18/작업자8 조건으로 재출력한 WASM은 수정 전과 전18쪽 수치가 동일했다.
선택 캡처와 전쪽 캡처를 직접 비교해 소스 회귀로 수용하지 않았다.
현재 canonical 방법/허용치/글꼴 공급을 변경하지 않았다.

## 정식 회귀

`issue_7445_negative_spacing_glyph_shape::negative_distribution_keeps_the_native_glyph_shape`는
실물의 해당 원문/스타일로 실제 Native paint를 실행한다. 간격0과 음수 간격의 같은 글리프 모양을
비교하며 고정 PNG/hash·임의 좌표 golden을 사용하지 않는다. 수정 전 실제1FAIL(801픽셀 차이,
0.221초) → 수정 후1PASS(0.222초)다. 쪽수·배치의 기존 관계 검사는 보존했다.
SVG의 기존 음수 간격 검사는 자연 글리프 폭 유지와 다음 원점 이동을 함께 검사하도록 교정했고
실제1PASS다. 내부 crate의0건 필터 출력은 통과 건수로 더하지 않는다.

새 테스트 추가 전에 위 Native/fresh WASM 전66쪽의90% 선행 조건을 확인했다.
20쪽은 양쪽98% 이상이며 실제 래스터 픽셀에서 자연 글리프 모양을 보호한다.
기존11쪽의 끝점 좌표 px 보호는 별도 계약으로 유지했다. 지침은 변경하지 않았다.

## 비교 이미지

|출력|수정 전|수정 후|
|---|---|---|
|Native review|[review](before-native-review.png)|[review](after-native-review.png)|
|Native overlay|[overlay](before-native-overlay.png)|[overlay](after-native-overlay.png)|
|WASM review|[review](before-wasm-review.png)|[review](after-wasm-review.png)|
|WASM overlay|[overlay](before-wasm-overlay.png)|[overlay](after-wasm-overlay.png)|

실제 Studio 기본 ‘짤림보기’는 셀 클리핑을 끈 편집 보기다. 기본값과 제품 메뉴로 잘림을 적용한
화면을 각각 보존했다. 7쪽99.19733%,10쪽97.85426%, 원본 HWPX4쪽99.45016%는
이전 C71 패키지의 실제 CanvasView 대조이며 새 C72의 실제 소비 검증 완료로 재인용하지 않는다.
이 작업의 새 전체 회귀·실제 소비 경로 및 남은 문서별 하락 보정은 계속 필요하다.
