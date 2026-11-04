# Native·fresh WASM 실제 API 산출물 대조

이 기록은 명시한 리베이스 전 제품의 관측 결과다. 이후 head의 최종 검증 통과를 대신하지 않는다. 원시 JSON·로그는 ignored output에 보관한다.

제품 `dc87f4fa2e`, WASM SHA-256 `c7e166e1b087641e727d326dd5ff504653f96134808242f9d1fd83228bbae903`.

추가 글꼴 프로그램 등록 없이 PDF 가져오기와 실제 편집 API를 실행했다.

| 동작·입력 | 쪽수 | Native HWPX SHA-256 | WASM HWPX SHA-256 | 바이트 동일 |
| --- | ---: | --- | --- | --- |
| PDF 가져오기 · `samples/21_언어_기출_편집가능본-2010.pdf` | 16 | `54df47aa980966679a67f579916972d26d6147a08d327368398abc2552d84412` | `54df47aa980966679a67f579916972d26d6147a08d327368398abc2552d84412` | True |
| PDF 가져오기 · `samples/21_언어_기출_편집가능본-2020.pdf` | 15 | `f1929a65e3901b90087fe9ccb4b0e446ceed32926b1479d4307e22474d23f49e` | `f1929a65e3901b90087fe9ccb4b0e446ceed32926b1479d4307e22474d23f49e` | True |
| PDF 가져오기 · `samples/21_언어_기출_편집가능본.pdf` | 15 | `7a54ea971e88cb9e4d946288beb28a131439640f8015dbdbb44a64189b6f9e7d` | `7a54ea971e88cb9e4d946288beb28a131439640f8015dbdbb44a64189b6f9e7d` | True |
| 실제 편집 · `samples/issue6123/3112461_railway_emc_criteria.hwpx` | 14 | `8009a8f11dd1f53545ed457fdee47326310caf26c56f2eda9bde7d226d7ab9e9` | `8009a8f11dd1f53545ed457fdee47326310caf26c56f2eda9bde7d226d7ab9e9` | True |

PDF는 `HwpDocument.importPdf(bytes, {layout: "editable"}) → exportHwpx()`를, EMC는 `deleteText(0,114,0,1) → exportHwpx()`를 실제 호출했다.
