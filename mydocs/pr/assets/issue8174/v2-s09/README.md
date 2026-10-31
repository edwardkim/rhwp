# #8174 T03 증거

제품 source는 `1e7282ad64fae37bbf9684c1e77008e132473075`다. `before`는 `9cabbff39d`이다.
Native는 `release-test` 최적화 + `native-skia`, WASM은 locked 기본 feature의 release·wasm-opt 빌드다.
검증 명령·폰트 환경·확대 판독과 비용은 [T03 기록](../../../../working/task_m100_8174_t03_font_fidelity.md)을 따른다.

공개 입력 16개·34쪽의 TSV는 산출 원문 그대로 보존한다. 별도 maintainer 문서의 15쪽은
익명 집계만 보존하고 원본·시각 자료·경로는 포함하지 않는다. 합계는 환경마다 17개·49쪽이다.
CFF의 같은 actual Nimbus family와 TTF clip 대조군의 같은 actual DejaVu family를 명시해
원본 프로그램과 동일한 폰트를 사용한다. 공급 환경은 기존 S07/S08 기록에 있다.

CFF의 독립 PDF는 PyMuPDF와 실제 Nimbus Roman CFF/OTF 프로그램으로 생성했고,
Type3 기준은 저장소의 독립 한컴 PDF다. PDF → 편집 IR → HWP/HWPX 저장·재열기의
내용·줄 소유를 별도로 검사했다. 점수는 직접 판독을 대신하지 않는다.

대표 PNG는 CFF 전체와 Type3 1·3·11쪽의 Native/fresh WASM review·overlay다.
모든 비교는 96 dpi·compat 2022이며, raw 점수는 색상 경계 보정 전 2 px 실루엣이다.
잔여 글꼴 윤곽·raster AA 차이를 exact pixel 일치로 주장하지 않는다.

`native-perf-summary.json`은 동일 호스트의 각 3회 관측이다. child median은 CFF 16→16 ms,
Type3 338→368 ms, maintainer 문서 2133→2150 ms다. 전체 RSS 상한이나 일반 성능 보증이 아니다.
