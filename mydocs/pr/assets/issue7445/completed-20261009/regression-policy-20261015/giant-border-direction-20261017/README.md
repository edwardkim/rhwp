# giant 원본80쪽의 devel 하락 원인 조사

Issue: #7445. 기준 `b7a5388034ee66f58a7e1d5363974e83b6374f57`,
후보 `07a656abfbcb2163a748ae0ea857ebfe33bdc8ee`.

원본 `samples/issue1949_giant_cell_nested_tables_perf.hwp`
SHA256 `ef10261cd29325116028e4f4f3e6be1a72c675eb771bddfd8484e7fe5aa94b4e`,
독립 Print `pdf/issue1949_giant_cell_nested_tables_perf-hwp-2024.pdf`
SHA256 `9fb08dea604813eed90a0449f29682d400be54dacd413352cc4f680f37cd4a44`다.
compat2024·Print·192DPI·현재 canonical PNG 생산·raw2px를 유지했다.
두 코드의 frozen Native/fresh WASM producer와 같은 실제 공급 글꼴을 사용했다.

|80쪽|기준|후보|기준−후보|
|---|---:|---:|---:|
|Native Skia PNG|94.40109%|96.82269%|−2.42160%p|
|fresh WASM Canvas PNG|97.54273%|96.81942%|+0.72331%p|

각 생산자의 쪽수는115/115지만 이는 선택80쪽의 집중 비교다. 전쪽 수용으로 확대하지 않는다.
WASM 하락은0.01%p를 초과하므로 PR을 보류한다.80쪽은 양쪽98% 조건을 충족하지 않아
새 실물 px 회귀를 추가하지 않았다.

## 기준 WASM의 기본 글꼴 환경 호환

현재 canonical helper는 실제 공급 프로그램의 자동 선택 API를 요구한다. 고정 기준에는
그 API가 없어 첫 WASM 호출은 종료코드1로 중단됐다. 이를 시각 실패로 집계하지 않았다.
기존 공개 `setFontEnvironment(JS null)`가 기본 None을 그대로 유지함을 기본 환경46조건에서
확인했다. 반환값은 모두 false이고 전후 쪽수·첫 쪽 text-layout SHA가 같았다.
canonical helper가 이미 지원하는 명시 None 경로로 기준 WASM을 다시 실행했다.
base package·helper·원본·PDF·글꼴은 수정하지 않았다.

이 원본은 현재 자동 계획에서도 명시 대체 요청0개다. 실제 기준/후보 export metadata의
환경은 양쪽 모두 None이었다. 원본에 선언한 명시 환경은 다른 대상에서도 그대로 유지하며,
후보 코드의 자동 선택을 과거 기준의 None으로 강제하지 않는다. 선택 결과가 코드 변경으로
달라지는 대상은 실제 metadata와 함께 구분해 기록한다.

## 생산 좌표와 최종 래스터 소비

원문 표 → 공통 render tree의 하단 `Line` → Print profile의
`raster_line_endpoints` → Web Canvas/Skia·CanvasKit adapter가 소비하는 장치 선을 추적했다.
기준/후보 tree의 해당 선 bbox는 동일하다. 새 장치 중심 정렬이 최종 paint 원점을 바꾼다.
추가 raw 불일치의 최다 행은 PNG1846행이며, 후보1177픽셀·기준0픽셀이다.
고정된 작은 글자 폭 차이만으로 이 긴 가로줄의 불일치를 설명하지 않았다.

원인 분리용 getter 대조에서 실제 Canvas transform·원문·조판·글꼴을 유지하면서
이 직교 정렬 분기만 우회했다. render-tree SHA와 준비한 글꼴 프로그램·환경은 원 실행과
동일했고 raw97.50476%였다. **미채택 진단**이며 정상 출력·보정 완료·새 golden의
근거가 아니다. 기준보다0.03797%p 낮은 잔여도 유지한다.

독립 PDF의 하단 선 두 경로는 Y692.6019897pt, 폭0.3600000pt이며,
600DPI 텍스트 content matrix는 X0.119935/Y0.119869pt다. PNG1588×2246과
PDF raster1587×2243의 차이도 남아 있다. 인쇄 좌표·장치 coverage의 일반 계약과
정상 대조군을 더 확인해야 하며, 문서별 좌표 이동·색·DPI·허용치 보정은 적용하지 않았다.

후속으로48조건의 원본 용지 HWPUNIT와 PDF 첫 쪽의 실제 content matrix를 대조했다.
33조건은 `floor(용지HU/100) / ceil(용지HU/12)`와 소수6자리에서 일치했다.
7조건은0.12pt의 명목 변환이고8조건은 해당 텍스트 행렬을 관측하지 못했다.
같은59528×84188HU에서도 두 변환이 모두 관측되므로 용지 크기만으로 일괄 축소를
적용하지 않았다. 이는 출력 장치 조사이며 생산 코드나 비교 입력의 보정이 아니다.
ignored `print-device-diagnosis/independent-print-matrix-audit.json`에 전48조건을 보존한다.

ignored `rebase-b7a53880-20261017/giant80-current-direction/`에 producer·원 PNG·TSV·
정상/미채택 대조·부분 영역 진단을 보존한다. 첫 부분 영역 crop의 작은 PDF 바깥을
검정으로 채운 진단은 폐기했고, canonical 흰색 padding 후 다시 계산한 자료만 사용했다.
원문48조건의 원본/PDF96개 참조는 실제 실행 바이트와 현재 commit blob을 모두 대조했다.
output의 tab-period 원본도 커밋된 `tests/fixtures/issue4138/tab-period-split.hwp`와 같다.

## canonical 비교 이미지

두 WASM review와 overlay를 직접 열어 전체 내용·표 하단·앞뒤 글자를 대조했다.

|출력|review|overlay|
|---|---|---|
|기준 fresh WASM|[review](base-wasm-review.png)|[overlay](base-wasm-overlay.png)|
|현재 fresh WASM|[review](head-wasm-review.png)|[overlay](head-wasm-overlay.png)|

전체 회귀10646PASS/0FAIL/45skip과 release build는 별도 통과했다. 남은 local gate,
전쪽 시각·실제 소비 검증, WMF raw 미달·다른 devel 하락은 아직 완료가 아니다.
