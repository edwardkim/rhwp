# #8174 T08 — source unit 정책과 공개 계약 대응

2026-10-28. 원본 source는 `dd2d4cfb13`이며 T07 이후 계획 문서만 추가한 상태다.
비교 base는 최신 fetch에서 고정한 `db5cd5bcc60a69e42f419f18a7f57a33b6fb7fdc`다.
실행 원장과 제거 전 source 전체·Git blob·SHA-256·함수별 대응은 ignored
`output/pr-review/issue8174/t08-completion-20261028/`의 `migration-originals.json`,
`original/`, `public-contract-mapping.json`에 보존한다.

## 범위 구분

이슈에서 추가한 source unit은 **33개**, 신규 cfg 지원 항목은 2개다. 이전 TODO의 34개와
최신 base 비교의 35개에는 이슈 밖에서 변경한 기존 검사 차이가 섞여 있었다. 비교 base를
HEAD로 바꾸거나 policy 최대값을 올리지 않고 최신 base의 **4,099개**와 일치시켰다.

| 이전 항목 | 현재 활성 검사의 계약 |
| --- | --- |
| owned_work 1·value_allocation 10 | 실제 PDF 진단의 숫자/문자열/배열 schema·정확한 backing 계상·한도 이전 거부·실패 고정·복구를 기존 `issue_8174_public_storage_boundaries`·session과 새 frontend admission으로 검사 |
| allocation 9 | 기존 문서/문단/탭·프레임·메타데이터·lazy asset의 공개 session 검사와 새 실제 HWP DocInfo 이식/긴 UTF-8 font name의 snapshot 거부·live IR 보존·복구. 문자열/slot을 helper에서 직접 만들어 재인용하지 않음 |
| recognition 2·text_analysis 1·wrap_width 2 | 실제 원본 PDF 세 개의 공개 parse/인식에서 작은 요청을 거절하고, 새 요청의 전체 저장 IR·span/인식 metadata가 정상 대조군과 같음을 확인. 기존 CFF 원문/지표·공통 문단/표 요청 수명 검사도 유지 |
| pdf_stroke 1 | 기존 PDF path/대시 자원 한도와 실제 SVG 출력·정규화·공유 요청 거부/복구 검사 유지 |
| budget_cursor 1 | 기존 HWP/HWPX 출력 한도·요청 실패 고정·live IR/기존 출력 보존·정상 복구 검사 유지 |
| layout_frame 4 | 기존 stored/body frame·metric probe·paginator/paragraph/table state의 실제 초기 요청 거부/복구, active 문서 보존을 검사. helper의 임의 Storage 거부/재개를 제품의 재시도 계약으로 설명하지 않음 |
| static_svg 2 | 기존 Native glyph replay의 실제 paint 검사에 XML 선언·제한 subtree 뒤 gradient/그림 보존, 참조 cycle·지수 확장 거부와 정상 후속 대조를 이관. 실제 WASM 공개 raster 경로도 최종 브라우저 검증에서 실행 |
| composer 지원 2개 | 삭제한 private token 검사만 사용하던 cfg use/fn 제거. 일반 token/reflow 생산 경로는 동일 |

원래의 임의 `Serialize`/`Clone` 구현·무한 길이 힌트·가변 private Storage는 공개 PDF 입력이나
공개 session API에 주입할 수 없다. 그 원본 합성 검사 전체를 보존했지만, 공개 검사가 그
가짜 callback을 문자 그대로 재실행한다고 주장하지 않는다. 실제 생산자는 고정된 문서/진단
타입이며 공개 입력으로 관측하는 schema·예약 실패·부분 결과 금지·live IR 보존·새 요청 복구를
검사한다. private helper를 테스트 때문에 공개하거나 구현을 복사한 별도 harness는 만들지 않았다.

## 이슈 밖의 기존 검사 두 건

- `hwp3::test_hwp3_sample16_load_alignment`는 #877 원 구현 `dcf64b4da0`에서 도입됐고
  upstream #8228 통합 `3e02fe2749`에서 제거됐다. 이번에는 같은 원본과 기존 1,000개 이상
  문단의 정렬 계약을 `issue_8174_hwp3_record_alignment`로 보존했다. 파일 존재 시에만
  실행하던 분기 대신 커밋된 입력을 실제 실행했다. 사용자 지정 page-border 제외와 다른 검사이며
  새 제외를 추가하지 않았다.
- `oversized_row_width_outlier_does_not_expand_base_columns`는 #2512의 기존 100HU 합성이
  최소 내용 폭보다 작아 실제 열 차이를 가리지 못한다는 upstream 보정 `c9225530b3`을 추적했다.
  최신 base의 `column_width_contract_tests.rs`와 연결을 그대로 적용해 1,000HU의 지배 행 격자·
  같은 열 폭·원본 총폭·원시 값 보존 관계를 유지한다. 해당 생산 column resolver는 바꾸지 않았다.
  단순히 기존 unit 하나를 지워 숫자를 맞춘 처리가 아니다.

## 생산 코드와 실행 경로

`production-cohort.json`에서 13개 생산 파일의 cfg(test) 영역·공백을 제외한 텍스트가 이전과
같음을 확인했다. 나머지 layout test 파일과 새 column contract는 test-only 경로다.
field/API·계산·조건·기본 feature·자원 한도·policy 기준선은 바꾸지 않았다.

공개 frontend admission 2·HWP3 1·Native glyph replay 10개를 기존 자동 수집 경로로 실행해
통과했다. Native glyph replay는 기본 feature의 전체 실행에서 cfg로 빠지고 기존 Native CI
호출 목록에도 없었다. 기존 Native job의 release-test/release 분기에 한 호출씩 추가하고,
해당 source만 바뀌어도 job이 실행되도록 impact classifier의 소유 목록에 연결했다.
기존 workflow 계약이 이 목록 누락을 실제로 검출했으며 연결 보정 후 재검증을 통과했다.

위 내용은 G01의 집중 검증 기록이다. T08 전체 완료는 최종 source의 전체 lint/회귀·표준 제품·
실제 브라우저·전쪽 시각·비용과 포장 영향 대조까지 완료한 뒤 별도 완료 보고에서 판정한다.

## 최신 base 대비 integration source 위치

고정 base `db5cd5bcc6`의 #8228 (`3e02fe2749`)은 기존 integration 원본 19개를
활성 경로에서 제거했다. 현재 branch에서는 기존 검사를 보존하고 있어 정책 비교가 이 경로를
신규 root source로 검출했다. assertion·입력·기대값은 바꾸지 않고 같은 바이트의 원본을
`tests/cases/`로 옮겨 자동 수집 규칙을 적용한다. 기존 검사 제거·허용치 변경은 하지 않는다.
각 이전/이후 경로와 SHA-256은 로컬 `legacy-source-relocations.json`에 기록했다.
이 이동은 최신 devel의 조판 구현을 통합하거나 기존 검사의 시각 판정을 승격하는 작업이 아니다.
