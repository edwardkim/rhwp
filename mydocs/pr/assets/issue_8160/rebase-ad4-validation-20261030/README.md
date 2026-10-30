# #8188 리베이스 이후 전쪽 출력 대조

2026-10-30에 렌더링 코드 `5c33032fe1`을 Native Skia와 표준 최적화 fresh WASM Canvas로
실행해, 기존 독립 한컴 Print PDF와 96dpi PDFium 래스터를 비교했다. 입력 및 PDF 경로·SHA는
[기존 증거](../rebase-8231/raw100-proof.json)와 동일하다. 각 입력은 64쪽이며 아래 수치는
2px 내용 실루엣 `silhouette_raw_match_percent`를 소수 둘째 자리로 표시한 값이다.

| 입력 | Native 전체 TSV | WASM 전체 TSV | 표시값 100.00% 미만 쪽 |
| --- | --- | --- | --- |
| HWPX | [TSV](hwpx-native.tsv) | [TSV](hwpx-wasm.tsv) | Native 18·35쪽 99.99%, WASM 18쪽 99.99% |
| HWP 2024 | [TSV](hwp2024-native.tsv) | [TSV](hwp2024-wasm.tsv) | Native 18·35쪽 99.99%, WASM 18쪽 99.99% |
| HWP 2022 | [TSV](hwp2022-native.tsv) | [TSV](hwp2022-wasm.tsv) | 없음 |

세 입력 모두 Native와 WASM의 23·59쪽은 100.00%이며, HWP 2022의 기존 해당 좌표 검사는
독립 Print 좌표 기준 ±1px로 검사한다. HWP 2022 23쪽은 같은 PDF와 비교한 [Native review](hwp2022-p023-native-review.png)·
[Native overlay](hwp2022-p023-native-overlay.png), [WASM review](hwp2022-p023-wasm-review.png)·
[WASM overlay](hwp2022-p023-wasm-overlay.png)를 보존한다. 점수는 2px 이웃 안의 실루엣 판정이며 픽셀 색상의 완전 일치를
뜻하지 않는다.

Native 실행 파일 SHA-256: `4db09178e15359b6cbd2f0601e31f9217b77a4451d027065c7f92973b40d90b4`.
최적화 WASM SHA-256: `d6dd0b6ce157ea50b6291b541854b2fec8437d2104e51e0597dccff7634df408`.
빌드 병렬도 8, WASM `codegen-units=16`을 사용했다. 이후 커밋은 Rust 서식, lint 범위와
회귀 주석을 정리했으며 렌더링 동작을 변경하지 않았다. 검증용 폰트는 로컬 시스템에서 공급했고
폰트 파일을 이 증거에 포함하지 않았다.
