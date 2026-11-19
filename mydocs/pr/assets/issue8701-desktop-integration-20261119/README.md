# 데스크톱 SDK P131–P136 통합 검증

검증한 코드 SHA는 `64ba35858d4f511bdd993f24eae9337880a7f2b2`, 비교용 devel SHA는
`c526dd402bcf55d09cb8d0993537698a25d4740a`다. 해당 통합은 원 PR #9158, #9159, #9161,
#9162, #9163, #9166을 포함한다.

## P136 Native 글꼴 대체

한컴 데스크톱 2022에서 만든 Hancom PDF Print를 96 DPI로 출력하고 2px raw 실루엣으로
비교했다. 1쪽 양쪽 표에 있는 한자 6글자와 표 경계, 2쪽의 내용을 직접 확인했다.
원본 HWP·기준 PDF·생산 바이너리·WASM package·TSV·PNG 해시는 [설명](p136-validation.json)에 있다.

| 출력 경로 | 1쪽 review | 1쪽 overlay | 전쪽 TSV |
| --- | --- | --- | --- |
| Native | [review](native/review_001.png) | [overlay](native/overlay_001.png) | [TSV](native/silhouette.tsv) |
| fresh WASM | [review](wasm/review_001.png) | [overlay](wasm/overlay_001.png) | [TSV](wasm/silhouette.tsv) |

- Native 2쪽의 [review](native/review_002.png)와 [overlay](native/overlay_002.png).
- fresh WASM 2쪽의 [review](wasm/review_002.png)와 [overlay](wasm/overlay_002.png).
- 고정 devel의 [Native TSV](base-native.tsv)와 [WASM TSV](base-wasm.tsv). 두 출력 경로의 2쪽 점수와 PNG 바이트가 통합 후보와 일치했다. [페이지별 대조](p136-devel-comparison.json).

## 보호 문서와 편람

- [Native 23종·2,023쪽 기록](protected-native-validation.json). 새 전쪽 TSV 23개가 저장소의 수용 TSV와 바이트가 같았다.
- [WASM 23종·2,023쪽 기록](protected-wasm-byte-identity.json). fresh JS·WASM의 바이트 일치와 수용 TSV의 현 기준 충족을 확인했다.
- [편람 HWPX Native 383쪽](handbook-webfonts.json). 격리 Mac에서 KoPub 바탕·돋움의 Light, Medium, Bold와 함초롬바탕을 웹에서 공급했다. 최저 99.97%, 보호값 하락 0쪽이었다.

## 데스크톱 SDK

- [Ubuntu Xvfb 소프트웨어 GL의 실제 SDK 검증](linux-sdk-validation.json). GTK·GL 검사 10단계와
  단일 화면 및 두 타일의 메모리 압박 시험에 해당한다. 물리 VRAM과 소비자의 GPU 완료 확인은 이 환경에서 측정하지 않았다.
- [릴리스 Native FFI 반복 저장 2건](repeat-save-validation.json). HWP·HWPX에서 조합한 ‘한’을 저장하고 두 번 다시 열어 보존을 확인했다.
- [Mac Rust 검증 기록](macos-rust-validation.json). 전체 회귀 13,781건과 Clippy 3종, 워크스페이스 빌드, Native Skia 검사의 통과 결과를 기록했다.
