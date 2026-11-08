# Native·fresh WASM 실제 API 산출물 대조

이 기록은 새 base `7fdd5441ff` 반영 후보의 실제 API 재검증 결과다. 전쪽 시각 검증과 최종 공통 검증은 별도로 확인한다. 원시 JSON·로그는 ignored output에 보관한다.

제품 코드 `4b149b6cdfc514b9858ed644be2cf8120c731cd4`, 실제 최적화 WASM 빌드 head
`7bb9e66fd006109ca9f7ee4c4d3f752c82e60fae`, WASM SHA-256
`0fcc2a5d219ade9de64ce9657de95ce8769ce21d3bed4062835dc15e205ae3e1`.
2026-11-04, 추가 글꼴 프로그램 등록 없이 실행했고 네 산출물 모두 바이트가 같다.

추가 글꼴 프로그램 등록 없이 PDF 가져오기와 실제 편집 API를 실행했다.

| 동작·입력 | 쪽수 | Native HWPX SHA-256 | WASM HWPX SHA-256 | 바이트 동일 |
| --- | ---: | --- | --- | --- |
| PDF 가져오기 · `samples/21_언어_기출_편집가능본-2010.pdf` | 16 | `54df47aa980966679a67f579916972d26d6147a08d327368398abc2552d84412` | `54df47aa980966679a67f579916972d26d6147a08d327368398abc2552d84412` | True |
| PDF 가져오기 · `samples/21_언어_기출_편집가능본-2020.pdf` | 15 | `f1929a65e3901b90087fe9ccb4b0e446ceed32926b1479d4307e22474d23f49e` | `f1929a65e3901b90087fe9ccb4b0e446ceed32926b1479d4307e22474d23f49e` | True |
| PDF 가져오기 · `samples/21_언어_기출_편집가능본.pdf` | 15 | `5a13ddd9b7fe2a51c95009e34c8e0a27337d35f145c637a3193025e9c455b1c6` | `5a13ddd9b7fe2a51c95009e34c8e0a27337d35f145c637a3193025e9c455b1c6` | True |
| 실제 편집 · `samples/issue6123/3112461_railway_emc_criteria.hwpx` | 14 | `8009a8f11dd1f53545ed457fdee47326310caf26c56f2eda9bde7d226d7ab9e9` | `8009a8f11dd1f53545ed457fdee47326310caf26c56f2eda9bde7d226d7ab9e9` | True |

PDF는 `HwpDocument.importPdf(bytes, {layout: "editable"}) → exportHwpx()`를, EMC는 `deleteText(0,114,0,1) → exportHwpx()`를 실제 호출했다.

현재 PDF 무표기판 저장본은 `mydocs/pr/assets/issue_8381/integration_20261103/editable-plain.hwpx`다.
이전 `7a54ea97…` 저장본을 현재 파일의 증거로 연결하지 않는다. 원시 실행은
`output/pr-review/planet6897-20261103/logs/rebase-7bb9e66fd0-wasm-contracts-producer-parity.log`다.
