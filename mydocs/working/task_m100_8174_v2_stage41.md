# #8174 v2-S41 — 초기 본문 조판의 요청 예산 연결

Issue: #8174. 2026-10-26, 기준 `ec28fd3da9`.

PDF Session이 전달하는 같은 Storage를 Core의 초기 본문 composition까지 연결했다. 일반
문서와 PDF는 같은 줄 구성·표시 투영을 소비하며, 일반 문서의 공개 API는 Unbounded 경로로
유지한다. **T04 전체와 PR 제출 준비는 아직 완료가 아니다.**

## 구현과 적용 경계

`PdfImportSession::open` → `from_parsed_document_with_storage` →
`try_recompose_all_with_horizontal_shaping` → `try_compose_section_with_horizontal_shaping` →
문단·저장 줄·글자모양/언어 run 분할·컨트롤 위치·PUA/동적 표시 투영으로 같은 Storage를
전달한다. 배열·문자열의 실제 생성/증가 전 예약과 실패 반환을 사용한다. 실패한 후보를
활성 문서에 설치하지 않으며, Core의 기존 composed 배열도 후보 전체가 성공한 뒤 교체한다.

- 문자·줄/run·표시 문자열, 인라인 컨트롤과 TAC/각주 위치, 탭 배열과 누락 마커용 문단
  복사에 예약을 연결했다. 필드 부수 슬롯 HashMap도 새 키의 capacity 증가 전에 검사한다.
- 원본 char_offsets·LineSeg 소속·명시적 개행·TAC 표 분리·기준선/높이 규칙은 유지한다.
  구성 결과는 기존 Core composed에 전달되어 측정과 실제 배치가 소비한다. 좌표 보정이나
  새 문서별 조건을 추가하지 않았다.
- 수식 메트릭 경로는 수식 없는 앞 텍스트의 후속 그림 예약 높이도 보정한다. 이를 건너뛰지
  않고 원래 알고리즘에 위치·TAC 배열과 누락 줄의 Storage를 전달했다. 수식 전용 소유 계산,
  supplemental/exact-font shaping, pagination 내부의 중첩 문단 등 미감사 경로까지 완료로
  확대하지 않는다. PDF lowering은 Table/Shape/Picture를 발행하며 AutoNumber/Equation을
  생성하지 않는다. 이 관측을 모든 외부 IR의 안전성 보증으로 바꾸지 않는다.
- 한글 접두부 뒤에 다른 언어가 있는 run에서 `all(default language)`를 글자마다 반복하던
  탐색을 한 번만 수행한다. 순수한 같은 조건을 재사용하므로 초기 언어와 경계는 유지한다.
  Unbounded collect는 표준 iterator collect를 사용해 기존 재사용·specialization을 보존한다.
  PUA 테두리 문자가 없는 줄은 폐기될 run 사본을 만들지 않는다.

## 실제 검증

생산 source의 파일별 SHA-256은 ignored
`output/pr-review/issue8174/v2-s41-20261026/production-freeze.json`, 실제 Native 제품은
`native-producer.json`에 고정한다. 아래는 이 source의 집중 검증이며 최종 T08 전체 suite와
최적화 fresh WASM·Studio의 결과로 표시하지 않는다.

| 검증 | 결과 |
| --- | --- |
| 실제 Native CLI/example 빌드 | release-test, root opt-level=1 / codegen-units=64, 성공 |
| 공개 composition 변경 전후 대조 | 저장/무저장 합성 대조와 HWP/HWPX 4개, 본문·직접 표 셀 문단 **7,258개 구성 결과 동일** |
| 공개 PDF Session·저장 경계 | **11 PASS**. 초기 composition 잔여 예산 거절·sticky 실패·다른 활성 문서 보존·fresh 요청 복구 포함 |
| PDF 17개 전체 HWP/HWPX 저장·재열기 | **34개 저장·98쪽 재열기**와 상세 진단 3건, 사용량 이외 진단·재열기 render tree 동일 |
| 저장 파일 대조 | 위 일반/상세 저장 **37개 bytes 동일** |
| 선택 쪽 7·9의 HWP/HWPX | 추가 **4개 왕복**, bytes·진단·재열기 render tree 동일 |
| Native 영향 페이지 | CFF **100.00000%**, Type3 11쪽 **99.97900%**, 새 review/overlay 직접 판독으로 글자·표 외곽·본문/그림·캡션 간격 유지 확인 |
| Native Clippy·feature-off | `--features native-skia --lib --bin rhwp -- -D warnings`와 `--no-default-features --lib` 통과 |
| WASM target Clippy | `--locked --target wasm32-unknown-unknown --lib -- -D warnings` 통과. 실제 fresh WASM 실행 증거와 구분 |
| source unit 정책 | 직전 HEAD 대비 **4,134 tests / 304 modules**, 증가 없음. 전체 PR의 기존 잔여 이관 차단은 유지 |

초기 연결 검사에서 sibling 모듈 메서드의 visibility 오류가 발생했고 `pub(super)` 범위를
수정했다. Native/WASM Clippy와 실제 제품 실행은 그 보정 뒤 source에서 수행했다. ignored
대조 probe의 최초 private walker 사용은 컴파일 실패했으며 공개 sections/paragraphs/cells
순회로 고쳤다. 새 test-only 생산 API는 추가하지 않았다. 실패 로그도 보존한다.

## 성능 측정 범위

공개 `compose_paragraph`의 **한 저장 줄**에 한글 접두부와 마지막 영문 1자를 둔다. 같은
프로필의 변경 전/후 라이브러리에 독립 probe를 연결하고, 각 길이를 3회 합성한 시간을
3라운드 교차 순서로 측정한 중앙값이다. 당시 다른 Cargo/시각 작업은 종료했다.

| 한글 접두부 길이 | 변경 전, 3회 합계 | 변경 후, 3회 합계 |
| --- | --- | --- |
| 8,000자 | 420.415 ms | 1.698 ms |
| 16,000자 | 1,679.311 ms | 3.402 ms |
| 32,000자 | 6,738.825 ms | 6.908 ms |

반복 탐색이 입력 길이에 대해 제곱으로 늘던 경로를 줄였다. 이 집중 결과를 전체 앱의
지연·일반 문서 속도·최적화 배포 제품이나 peak RSS의 개선율로 해석하지 않는다. 원자료는
`composition-performance.json`과 라운드별 TSV에 남겼다.

## 남은 작업

Core pagination·내부 중첩 composition과 첫 저장의 BodyText/control payload·format adapter/
XML/context scratch는 T04에 남는다. Native 접근 제한·Worker 요청 수명의 기존 S36/S37과
구분해 T05의 나머지 복사·동기 재열기 경계도 계속 해결한다. 최종 조합에서 fresh WASM·Studio·
전체 시각/회귀·성능을 T08로 수행한다. 이전 S37 제품을 이 source의 fresh 결과로 재사용하지
않는다. 현재 `parserAllocationBoundsProven=false` 계약과 host 한도 책임도 유지한다.
