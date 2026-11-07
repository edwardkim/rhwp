# #7728 — Native·fresh WASM 전 18쪽 각각 98% 이상

사용자가 지정한 이 문서의 98% 목표를 충족했다. 브랜치 전체 nextest·Skia·다른 문서/실제 소비 경로 검증 및 PR 준비 완료를 뜻하지 않는다.

## 입력과 조건

- 원본: `samples/issue6854/70833-electrical-safety-rule-regulatory-analysis.hwp`, SHA-256 `a80b09754ce4f8e73398af9a9467f570c02be237815fec203ee8ff2db1bb8cbb`.
- 독립 PDF: `pdf/issue7728/70833-electrical-safety-rule-hancom2020-print-20261106.pdf`, SHA-256 `6cec821ff5897714669602c8707d66f669cfcb1187362c69101a0255c8d17956`, 한컴2020 Print,18쪽.
- Native/fresh WASM Print·compat2022·96DPI·PDFium·보정 전 2px raw. 동일 원본/PDF/검증 글꼴로 전쪽을 새로 출력했다.
- Mac locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다. Docker 최적화 검증으로 보고하지 않는다.

## 결과

| 항목 | Native | fresh WASM |
| --- | --- | --- |
| 실제 페이지 | 18 | 18 |
| 최저 raw | **98.83905% (9쪽)** | **98.83565% (15쪽)** |
| 98% 미만 / 누락 / 중복 쪽 | 0 / 0 / 0 | 0 / 0 / 0 |
| 사용자 지적 6쪽 | 99.58584% | 99.26284% |
| 행 이어받기 7쪽 | 99.70863% | 99.71627% |
| TAC 래퍼 9쪽 | 98.83905% | 98.87584% |
| 선언 프레임 정렬 12쪽 | 99.83837% | 99.83778% |

원자 행도 자식 행 커서를 보존해 같은 부분 표 경로를 사용한다. 이어받은 행의 바깥 프레임 소유를
공통 원점과 물리 높이에 전달했고, 저장 줄 없는 닫힌 TAC 래퍼는 측정의 기존 원본 등식을 paint도 소비한다.
문서명·쪽 번호 예외나 보정 px 상수를 추가하지 않았다.

6·7·9쪽 N/W와12쪽 W의 최종 review/overlay를 직접 열었다.12쪽 Native는 같은 최종 PNG를 확인했다.
심한 부모 하단 역전·소속 누락은 없고 미세한 글리프/글줄·가는 선의 차이는 남는다.100% 동일 출력으로 보고하지 않는다.

## 회귀와 빌드 출처

- 정식 `issue_7728_print_column_frame_guards` 8 PASS, `issue_7728_original_nested_frames` 4 PASS. 총12 PASS / 0 FAIL.
- 새 핵심 원본 여백·프레임 관계는 실제 FAIL→PASS이며 6쪽 독립 하단 괘선 ±2px도 실제 FAIL→PASS다.
- 6쪽 고정형은 양쪽98% 이상 및 독립 PDF 확대 판독에 근거한다. 나머지 새 여백·정렬 검사는 관계형이며 위치를 임의로 고정하지 않는다.
- source HEAD `433be55253` + 이 단계의 공통 조판 diff. source patch SHA-256 `0e002bb0b0fd3810fc64b33c2a2704a814e2612225f1d55e83226e84e841197c`.
- Native binary SHA-256 `568425db1a799a3e60d539e03244faac32577bd94d78bfcdc25a714c0bd6503a`.
- fresh WASM SHA-256 `beaf2ae6c555d44bdec4fcfe07d056d4f18bfae71975b28b83c1e77d4fd45492`.
- JS SHA-256 `370ddff0fd1a1cf6f889edb1b2a42f420fac62f7f57df4b9cff19829e60cdd5f`.
- Root pkg/Studio public/frozen package의 JS/WASM hash 일치를 확인했다. 실제 Studio 초기화 응답 검증은 별도 단계다.

빌드/실행 로그·진단 driver·중간 JSON 및 변형 입력/PDF는 ignored output에만 보존한다.

## 제출 자료 SHA-256

- `native-silhouette.tsv`: `428a7f0bf674114ad332922ae47fc8dbed1d9bd3b8056628a01c288e90ac0036`.
- `native-p006-review.png`: `9a65a84a30ca60592d40ed64c608e76cd79497d8ccf1a1ad589298c8d1437a49`.
- `native-p006-overlay.png`: `f6e28cce22942098fb02f6bf08291597a8fc81c0411e9672a56b09163dcbbb79`.
- `native-p007-review.png`: `ca899c2b0ce4efc7388294c6f6abad7e5ff889c22494efb2f43642a816525a9b`.
- `native-p007-overlay.png`: `6ef1fdde7c57fcac8193a0d5839dc922e621a1d3fff95902903362cd51a3d959`.
- `native-p009-review.png`: `7458384dfc6f20e2d41b376580ab7c48e3ff5e7968d696aa88206b00fa6a1df6`.
- `native-p009-overlay.png`: `013b5bd52b1569e6840582a8362aae2168d96d31b5b09298324a9731506636b1`.
- `wasm-silhouette.tsv`: `df853ac4e82cd1da17e63c6b18d61a1ec6c25a8209c58b21789046616b5a7a8f`.
- `wasm-p006-review.png`: `76420085bf120dc9444941bc934ad580fbf9d3c4a53eab47d551d8c57873634b`.
- `wasm-p006-overlay.png`: `49b3257f0610a83b9ba06a6604a6f39d3b7a443afb36fe7fdfd0e55f44e9b799`.
- `wasm-p007-review.png`: `2e05187c60bfa8583b1f4aff481b6ae7a9d7ccce340e525d88b096d5b4d3be5f`.
- `wasm-p007-overlay.png`: `d5eb0e19e707e4fbaea12637ad977afeab89255972202d0d5cf3703477a6f8f0`.
- `wasm-p009-review.png`: `9d89bdba1e384f64479bdc741ceb87231dc3fcb4be2c3787a74ae450807a6435`.
- `wasm-p009-overlay.png`: `5fb7420ac727e969a896cd76f673ffce700dafd37f9eb9f7eb2536394018590e`.
- `wasm-p012-review.png`: `cd26e6ac74ac2b78d0bcd09dd55781c901a4d665794e20eac8226361954f2259`.
- `wasm-p012-overlay.png`: `742a5fd14013f12d04e24a9136f4d767f8b9ee990a0e001ef4f7f775352df8cd`.
