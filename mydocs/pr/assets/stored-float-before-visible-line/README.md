# 글줄 앞 부동 표와 이어지는 표의 쪽 배치

[공개 합성 HWP](../../../../tests/fixtures/stored_float_before_host/float-with-single-column-marker-a4.hwp)는
앞 줄과 호스트 글줄 사이에 작은 부동 표가 들어갈 공간을 저장한다. 칸의 첫 문단에서 단을
하나로 선언한다. [공개 입력 제작 근거](../../../../tests/fixtures/stored_float_before_host/README.md)

앞 줄 끝과 호스트 글줄 사이의 저장 거리에서 부동 표의 원점과 바깥 높이를 구한다.
`stored_visible_table_before_host_frame`이 승인한 높이를 `cell_units_uncached`의 호스트 줄에
한 번 예약한다. 전체 표와 분할 표는 같은 프레임에서 표를 호스트 줄 앞으로 옮기고 다음 행은
호스트 글줄 끝에서 이어 간다. 단이 두 개 이상이거나 저장 좌표에 빈틈이 있는 대조 입력은 기존 경로를 탔다.

[기준 Print PDF](../../../../pdf/stored-float-before-visible-line/float-with-single-column-marker-a4-2020.pdf)는
Windows 한컴 2020 `11.0.0.9136`의 Hancom PDF 프린터에서 1-up으로 출력했다. 같은 HWP 바이트를
전처리 없이 출력한 결과와 SHA-256은 [Print 영수증](print-receipt.json)에 있다.

## 2026-11-24 전체 두 쪽 결과

두 엔진에 같은 함초롬바탕과 함초롬돋움을 제공했다. 96 DPI, 호환 2022, Print 프로필로 렌더링하고
PDFium 기준의 색상 경계 보정 전 2px 실루엣을 계산했다. 표 테두리와 앞뒤 글줄은 각 쪽 대표
대조 이미지와 겹침 이미지에서 판독했다.

| 출력 | 수정 전 1쪽 / 2쪽 | 수정 후 1쪽 / 2쪽 | 전체 TSV |
| --- | --- | --- | --- |
| Native Skia | 59.36% / 48.70% | 100.00% / 100.00% | [수정 전](base-windows-2020/native/silhouette.tsv) · [수정 후](windows-2020/native/silhouette.tsv) |
| 최적화 WASM Canvas | 59.35% / 48.73% | 100.00% / 100.00% | [수정 전](base-windows-2020/wasm/silhouette.tsv) · [수정 후](windows-2020/wasm/silhouette.tsv) |

수정 전에는 A행이 1쪽, B–D행이 2쪽에 놓였다. 수정 후에는 작은 표가 호스트 글줄 위에 놓이고,
A–C행은 1쪽, D행과 뒤 문단은 2쪽에 놓인다. [Native 1쪽](windows-2020/native/review-page-1.png) ·
[Native 2쪽](windows-2020/native/review-page-2.png) · [WASM 1쪽](windows-2020/wasm/review-page-1.png) ·
[WASM 2쪽](windows-2020/wasm/review-page-2.png)

단 선언이 없는 대조 입력은 B·C행이 1쪽으로 복구됐고 D행은 여전히 2쪽에 놓인다. 다른 대조 입력
6개의 Native PNG, WASM 렌더 트리의 표 경계와 쪽별 텍스트는 전후 동일했다.

기준 엔진은 `9ee9a5b0b12115f06b31e07ec89b94e454c1a744`,
수정 엔진은 `ecbbadb02c59088e706c2bd4253c01dd4a141a86`이다.
Linux x86_64, Rust 1.93.1, Chrome 158.0.8127.88, PDFium 153.0.7999.0을 사용했다.
Native는 `cargo build --locked --profile release-test --features native-skia --bin rhwp`,
WASM은 Binaryen 133을 지정해 `scripts/wasm-pack-locked.sh --target web --out-dir <pkg>`로 빌드했다.
[편람 Native 383쪽](protected/handbook-native/silhouette.tsv) · [기여자 공통 검사](protected/contributor/)
