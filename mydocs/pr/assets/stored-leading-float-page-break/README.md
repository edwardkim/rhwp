# 같은 문단에 저장된 부동 표의 쪽 경계

두 입력은 첫 표가 저장 호스트 글줄 위에서 끝나는 구조를 공유한다. 두 표의 크기와 앵커를
같게 두고 제목 글자 크기만 바꿨다. [입력과 제작 근거](../../../../tests/fixtures/stored_leading_float/README.md)

제목의 글자 모양과 줄 간격을 확인해 저장 전진 높이를 확정한다. 첫 표 바깥 상자와 공백
호스트 글줄까지 본문에 들어가면 제목이 있는 쪽에 놓는다. 둘째 표는 원래 앵커가 쪽을
벗어나고 다음 쪽에 본체와 위쪽 여백이 들어갈 때, 새 쪽의 원점에서 배치한다. 두 표의 실측
높이, 현재 흐름과 저장 위치, 각주 유무가 이 경계를 뒷받침할 때 쪽 결정과 배치에 같은
프레임을 사용한다.

Windows 한컴 2020 `11.0.0.9136`의 Hancom PDF 프린터에서 전처리 없이 A4 1-up으로 출력했다.
Native Skia와 최적화 WASM Canvas는 같은 함초롬바탕·함초롬돋움을 사용했다. 96 DPI·호환 2022·
Print 프로필에서 PDFium 기준의 색상 경계 보정 전 2px 실루엣을 비교했다.
[원본·Print 해시와 영수증](print-receipt.json)

| 입력 | RHWP devel | 수정 후와 한컴 | Native 전쪽 TSV | fresh WASM 전쪽 TSV |
| --- | --- | --- | --- | --- |
| 13pt 제목 | 1쪽 | 3쪽. 제목, 첫 표, 둘째 표가 각각 1·2·3쪽 | [3쪽 TSV](windows-2020/stored-negative-title-anchor/native/silhouette.tsv) | [3쪽 TSV](windows-2020/stored-negative-title-anchor/wasm/silhouette.tsv) |
| 10pt 제목 | 1쪽 | 2쪽. 제목과 첫 표가 1쪽, 둘째 표가 2쪽 | [2쪽 TSV](windows-2020/stored-negative-title-fits/native/silhouette.tsv) | [2쪽 TSV](windows-2020/stored-negative-title-fits/wasm/silhouette.tsv) |

두 입력의 모든 쪽에서 Native와 fresh WASM 실루엣 일치율이 100.00%였다.

- 13pt 예제 [Native 2쪽](windows-2020/stored-negative-title-anchor/native/review-page-2.png) · [Native 3쪽](windows-2020/stored-negative-title-anchor/native/review-page-3.png) · [WASM 2쪽](windows-2020/stored-negative-title-anchor/wasm/review-page-2.png) · [WASM 3쪽](windows-2020/stored-negative-title-anchor/wasm/review-page-3.png)
- 10pt 예제 [Native 1쪽](windows-2020/stored-negative-title-fits/native/review-page-1.png) · [WASM 1쪽](windows-2020/stored-negative-title-fits/wasm/review-page-1.png)

앞서 2쪽으로 출력되던 공개 대조본 세 개는 양쪽 렌더러에서 수정 전후 표 좌표와 PNG가 같다.
[편람 Native 383쪽](protected/handbook-native/silhouette.tsv) · [기여자 공통 보호 검사](protected/contributor/)

기준 devel은 `2c0c1d205a54b8d5bd262241bac1391cffe0d801`, 증적 생성 소스는
`8bd7e63ec88efa48e224cc15c52cbbb22a9dcb25`이다. Rust 1.93.1과 PDFium 153.0.7999.0에서
검증했다. Native는 `cargo build --locked --profile release-test --features native-skia --bin rhwp`로
빌드했다. WASM은 `scripts/wasm-pack-locked.sh --target web --out-dir <pkg>`와 Binaryen 133으로
빌드하고 Chrome 158에서 실행했다.
