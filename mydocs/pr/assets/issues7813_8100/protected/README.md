# 공개 글꼴 공통 보호 검사와 편람 HWPX

공통 검사 4개 문서의 Native·fresh WASM 8건이 모두 통과했다. 소수 둘째 자리 표시값은 각각 100.00%다.
행정업무운영 편람 HWPX Native는 전 383쪽 최저 99.97%였고 정본 표시값 미달은 0쪽이었다.
96dpi, Print, 색상 경계 보정 전 2px 실루엣으로 비교했다.
[편람 전 383쪽 TSV](handbook-hwpx-native.tsv) · [검증 원본과 SHA](manifest.json)

```bash
venv/bin/python scripts/visual_sweep_contributor.py --rhwp-bin <고정 Native> --wasm-pkg <고정 WASM 패키지> --page-workers 1 --out <출력 경로>
```

| 공통 검사 입력 | 출력 경로 | Raw 표시값 | 전쪽 TSV | 비교 이미지 |
| --- | --- | ---: | --- | --- |
| re-03-latin-only-hancom | native | 100.00% | [전쪽 TSV](re-03-latin-only-hancom-native.tsv) | [비교 PNG](re-03-latin-only-hancom-native-review.png) · [Overlay](re-03-latin-only-hancom-native-overlay.png) |
| re-03-latin-only-hancom | wasm | 100.00% | [전쪽 TSV](re-03-latin-only-hancom-wasm.tsv) | [비교 PNG](re-03-latin-only-hancom-wasm-review.png) · [Overlay](re-03-latin-only-hancom-wasm-overlay.png) |
| angle-2400 | native | 100.00% | [전쪽 TSV](angle-2400-native.tsv) | [비교 PNG](angle-2400-native-review.png) · [Overlay](angle-2400-native-overlay.png) |
| angle-2400 | wasm | 100.00% | [전쪽 TSV](angle-2400-wasm.tsv) | [비교 PNG](angle-2400-wasm-review.png) · [Overlay](angle-2400-wasm-overlay.png) |
| pic-crop-01 | native | 100.00% | [전쪽 TSV](pic-crop-01-native.tsv) | [비교 PNG](pic-crop-01-native-review.png) · [Overlay](pic-crop-01-native-overlay.png) |
| pic-crop-01 | wasm | 100.00% | [전쪽 TSV](pic-crop-01-wasm.tsv) | [비교 PNG](pic-crop-01-wasm-review.png) · [Overlay](pic-crop-01-wasm-overlay.png) |
| shape-group-02 | native | 100.00% | [전쪽 TSV](shape-group-02-native.tsv) | [비교 PNG](shape-group-02-native-review.png) · [Overlay](shape-group-02-native-overlay.png) |
| shape-group-02 | wasm | 100.00% | [전쪽 TSV](shape-group-02-wasm.tsv) | [비교 PNG](shape-group-02-wasm-review.png) · [Overlay](shape-group-02-wasm-overlay.png) |
