# planet6897 통합 최종 검증

Native Skia 23조건·2,023쪽과 macOS의 `wasm-opt` 포함 fresh WASM 23조건·2,023쪽을 각각
대조했다. 4,046개 값 모두 기준과 같았고 누락·하락은 0쪽이다. 후보 전쪽 TSV의 SHA-256은
정본 `1f7ac7a52290acef829c03e15f2dd27e23c6a3450415266d4cc4067b70dfdcc8`과 일치한다.
편람 HWPX Native 383쪽도 이 집계에 포함된다.
[정본 TSV](../../visual-protected-baseline/final.tsv) · [문서별 검증과 실행 신원](protected-audit.json)

sample10 Native는 동일한 바이너리와 공급 순서로 산출한 763쪽을 재사용했다. 임시 실행기가
Native에 전달한 WASM 전용 연결 그림 옵션은 명령에서 거절됐다. 해당 시도의 출력을 집계에서
제외하고, Native에서 원본 경로의 연결 그림을 읽어 완료한 전쪽 결과를 사용했다. 시행령 Native는
휴먼고딕 TTF를 포함한 74쪽 결과를 사용했다. [시행령 공급 확인](../protected-native-mac/decree-ttf/README.md)

중첩 표를 한컴 2020 `11.0.0.9172`에서 17쪽으로 다시 인쇄했다. 최종 검증과 같은 출력 PNG를
새 인쇄본에 대조한 최저는 16쪽의 Native 98.60%·fresh WASM 98.61%다. 기존 보호 PDF의
같은 쪽은 각각 99.81%·100.00%다. 새 인쇄본의 16쪽에서 Type0 글꼴이 Type1 하위 개체를
참조하는 오류가 확인되어, 보호 판정에는 기존 인쇄본을 적용했다.
[중첩 표 인쇄 영수증과 비교](nested-reprint.json) ·
[Native 17쪽 TSV](nested-reprint-native.tsv) · [fresh WASM 17쪽 TSV](nested-reprint-wasm.tsv)

기존 정본 경로와 PDF SHA-256은 문서별 집계에 있고, 보호값과 인쇄본 재현 조건의 변경은
정본의 갱신 영수증에 남겼다.
[보호 자료 갱신 이력](../../visual-protected-baseline/README.md) ·
[KAF 최종 전쪽·Studio 실측](../kaf-print-repair/README.md)
