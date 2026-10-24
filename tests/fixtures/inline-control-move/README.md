# 인라인 개체 이동 저장 표본

- 기본 합성본과 차트 HWPX 생성 엔진 소스: `5f7a4fca71f816c2fb72154639c31e3826423a28`.
- `before/after`: `saved/blank2010.hwp`에서 공개 합성 본문을 만들고 표·그림·수식·사각형·글상자를 각각 한 문단에 둔다. 그림은 `assets/logo/logo-16.png`다.
- `after`: 문단 1~5의 인라인 개체를 차례로 문단 6의 논리 캐럿 7, 8, 9, 10, 11에 `move_inline_control_native`로 옮겼다. 본문·문단 수를 바꾸지 않았다.
- `chart-before`: 공개 원본 `samples/chart/세로막대형/묶은세로막대형.hwpx`의 차트를 글자처럼 취급하고 받는 문단을 마지막에 추가했다. 차트 내용·BinData를 새로 만들지 않았다.
- `chart-after`: 위 차트를 마지막 문단의 논리 캐럿 9로 옮겼다.
- 차트 HWP 두 파일은 `133e09e154d0d496850d64deb7e823b2599cceba`의 공개 WASM API로 HWPX를 `exportHwp()`한 뒤, 재열어 `setShapeProperties(0, para, controlIndex, '{"treatAsChar":true}')`로 인라인을 지정하고 저장했다. 실제 `getChartDataByIndex(0)`의 범주·계열·값 보존을 확인했다.
- 위 fresh WASM SHA-256: `d3145a4efad160c4c35a2a21b82838017ce5f45df2261a9e5d0a2cf7993169b4`.
- HWP/HWPX 두 저장본을 재열어 본문 동일, 각각 인라인 개체 5개/1개, 각 1쪽을 확인했다. 논리 캐럿은 Unicode scalar와 인라인 개체를 한 칸씩 센다.
- 파일은 저장/재열기·위치 계약과 독립 오피스 비교의 입력이다. 자체 생성본은 시각 정답지가 아니다. 동일 편집 저장본의 한컴 Print PDF·Native/fresh WASM Sweep는 아직 미검증이다.

SHA-256:

- `before.hwp`: `b8decfd11fd58848177392bb98bec35b066e7f35e3d548b3470ad662842b1731`
- `before.hwpx`: `7fe95b0b2db799adecc29ecc7eb6dca9777da0177312c1b8b669ffbf0e5ec7c3`
- `after.hwp`: `41214502530ec160494d7c6041aeea8795ba930251003fefdafa8dc3c1b30a20`
- `after.hwpx`: `bc83ae03baafed90c8d6b30c6227a86ea5b37d53ae298f663ff8183df0c37d9e`
- `chart-before.hwp`: `f4ad7a61bda9a19e1ef5a059fbeb8fd8851c53e1584db36e0e6dd6c9e7939375`
- `chart-before.hwpx`: `196c868bdc03e2888744e35740b0a7bbca96a9d3c2b999ba759b7e4355ae4ed2`
- `chart-after.hwp`: `8ff56111b683005b7d4cd34990e7214debc7785d9bef6bc76a2772d036ace617`
- `chart-after.hwpx`: `8ea852aacd448701811fbbd89a807c9321d171300bdba67c822f9a561910a308`
