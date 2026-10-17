# Task #7957 — PDF 열기 일곱 유형 공동 구현계획

- Issue: #7957; 공동 #7958·#7959·#7960·#7961·#7962·#7963
- 작성일: 2026-10-17; 상태: 사용자 계획 승인, F1–F4 Native 검증 완료·F5 통합 검증 진행
- 수행·개별 범위: [공동 수행계획](task_m100_7957.md)
- 기준 SHA: `b7a5388034ee66f58a7e1d5363974e83b6374f57`

## 독립 근거와 현재 관측

현재 제품 WASM의 `HwpDocument.importPdf`를 각 독립 Node Worker에서 실행했다. 실물 6개와
크기 거절용 합성 입력 1개에서 사용자 오류를 모두 재현했다. 매 호출은 기존 Docker bundle
SHA-256 `6dc88d238132563e40d332c53898e96ce1272df997d099923e9ec69adf9ee35b`를
사용했다. 이 결과는 수정 후 검증이 아니며 Native 실행이나 새 worktree 빌드라고 표시하지 않는다.

실물 6개는 Poppler가 쪽수·비암호화 상태를 읽는다. `extraction_error.pdf`의 qpdf 검사도
syntax/stream encoding 오류를 검출하지 않았다. 이 사실을 모든 PDF 의미의 정상성 증명으로
확대하지 않는다. 원본 PDF의 독립 Poppler 출력을 가져오기 paint의 기준으로 사용한다.
한컴 재저장/재조판과 PDF 원본의 일치는 별도 계약이다.

현 제한은 frontend 입력 16 MiB·원본 128쪽, backend input 16 MiB·operators 250,000·
decoded 누적 256 MiB, frontend 개별 path 20,000 operations·문서 누적 80,000 operations다.
`MAX_PAGES`는 주석상 선택 상한이지만 전체 원본과 WASM 선택 인덱스에도 사용되고 있다.
문서 누적 decoded 비용은 heap/RSS와 같지 않고 실패한 시도·복사/codec scratch도 포함한다.

이미지 입력에는 ICC 기반 RGB·SMask·Flate/DCT 객체가 관측됐다. 이를 실제 Unsupported 원인으로
확정하지 않는다. container/extraction 오류도 현재 catch-all 문구만으로 원인을 확정하지 않는다.
초기 계상량·거절 객체·내부 오류 분류는 미검증이며 F1에서 확인한다.

## 실제 소비 경로와 변경 계약

| 대상 | 생산 → 소비 → 최종 영향 | 변경 계약·반례 |
| --- | --- | --- |
| 입력 크기 | CLI/Studio 읽기 → Worker 복사 전 검사 → Rust frontend 복사 → backend 생성 | 같은 유한한 요청 정책. 정상 큰 PDF·경계/초과·실패 후 기존 문서 보존. 호스트의 더 낮은 상한은 명시적으로 구분 |
| 쪽 선택 | 원본 page tree → 원본 count → WASM sourcePages/selected_pages → section/metadata | 전체 원본과 선택량 구분, 원본 쪽 대응 보존. 128 이후 쪽·불연속 선택·범위 밖·빈 선택 |
| operators | stream parse/Form 호출 → immutable parsed ops/cache → 각 extractor/paint 해석 → IR | parser 중복 비용과 실제 반복 paint 구분. 캐시로 paint 의미·좌표·scope·한도 계상을 없애지 않음 |
| decoded | stream/codec output·scratch/복사 예약 → cache/그림/마스크 → lowering 소유 bytes | 실제 데이터 공유로 줄인 비용만 제거. 같은 자원과 다른 Decode/색 공간/마스크 반례. sticky 오류·요청 격리 유지 |
| paths | 추출 path.operation → 개별·누적 계상 → path_paint 벡터/bitmap → 최종 paint | 실제 operation·paint·할당 소비를 반영. clip/opacity/순서·구멍 보존, 유한한 복잡도 제한 |
| image | handle.decode → 실제 filter/color/mask → bounded pixel/PNG → IR picture/최종 paint | 실제 거절 분기부터 구현. alpha/crop/색·요청 예약 유지, 무제한 legacy fallback 금지 |
| parse 오류 | backend 실제 오류 → frontend 분류 → Native/WASM/Studio 오류 | 최초 원인·단계/쪽·객체를 진단에 보존. 사용자 공개 메시지와 로컬 상세 관측 구분, 부분 성공·무시로 바꾸지 않음 |

모든 값의 적용/비적용 경로를 실제 코드에서 대조한다. 공통 helper 호출 뒤 재계산·우회가 남으면
공통 정책 완료로 보지 않는다. HWP/HWPX 원본 의미·조판 엔진을 이 PDF 입력에 맞춰 바꾸지 않는다.

## 절편별 구현·검증

### F1 — 원인 관측과 공통 요청 정책

- backend 원인 분류·거절 예약 지점·쪽/객체/요청량·한도·cache hit/miss의 진단을 마련한다.
  source 내용·사용자 경로를 기본 오류 메시지에 노출하지 않는다.
- frontend/Worker/backend에 흩어진 입력·쪽·연산 한도를 목록화하고 하나의 요청 계약으로
  연결한다. 확장값은 실제 입력·비용 측정과 호스트 조건으로 정하고 `0=무제한`을 만들지 않는다.
- 동일 Worker에서 실패 요청 뒤 정상 PDF·기존 세션 보존을 먼저 확인한다.
- #7961·#7959의 실제 최초 실패 계층을 특정한다. 아직 원인 미확정인 사례에 codec 이름을
  추측해 분기를 추가하지 않는다.

### F2 — 큰 입력과 원본/선택 쪽 계약 (#7957·#7962)

- 162쪽 전체와 129번째 이후 선택 쪽을 같은 제품 경로로 처리한다. sourcePages는 JS 숫자
  유효성·증가·실제 원본 범위를 검증하며 128이라는 상수를 인덱스의 대용으로 사용하지 않는다.
- 입력 크기 검사를 복사·structured clone·할당 전에 적용하고 Native·WASM·Worker가 같은
  계약을 소비하게 한다. 정상 큰 PDF 생성기와 16 MiB 및 새 상한의 직전/정확한 경계/초과를 검증한다.
- 정상 생성 PDF의 미사용 stream으로 크기를 조절한다. 초기 합성 거절 입력의 통과만으로
  정상 대형 PDF 지원을 주장하지 않는다. style/resource ID·section 등 다음 제약도 확인한다.

### F3 — 실제 작업·재사용·복잡도 (#7958·#7963·#7960)

- content operator의 중복 해석과 실제 Form paint 호출, decoded 누적 작업·live 소유 데이터를
  구분한다. 기존 #7953 cache를 먼저 재사용하고 확인된 중복 지점만 보정한다.
- 캐시는 요청·object/filter/Decode/color/mask identity를 반영한다. CTM·clip·opacity 등 paint
  상태가 다른 실제 표시를 합쳐 없애지 않는다. 해석 절감 전후에도 정상 픽셀·원본 순서를 유지한다.
- 개별 path와 문서 누적 요구량을 구분하고 실제 lowering/벡터·raster scratch 소비를 한도와
  연결한다. 문서 크기와 무관한 상수 일괄 상향이나 카운터 초기화로 통과시키지 않는다.
- 소진 지점·요청별 계상·시간/메모리 관측 범위를 전후 비교하고 한도 실패의 sticky 상태와
  새 요청 초기화, cache hit/miss·공유/비공유 대조를 확인한다.

### F4 — 필요한 image/container/extraction 호환 (#7959·#7961)

- F1에서 특정한 실제 미지원 조합·오류 경로에 필요한 해석/codec/color/mask 처리를 구현한다.
- 정상/손상·마스크 유무·색 성분/길이·실제 예약 한도 반례를 같은 제품 호출 경로에서 실행한다.
- 전쪽을 가져오고 변경 쪽의 색·투명도·crop·paint 순서와 내용 소유·누락·중복을 직접 대조한다.
  첫 오류를 넘긴 뒤 드러나는 다른 미지원은 기록하고 해결 범위를 재판정한다.

### F5 — 통합·직접 판독·최종 검증

- 2026-10-17 사용자 지적에 따라 `bounded_import_path.pdf`의 검정 텍스트 배경도 포함한다.
  원본은 `/Cs1 cs` 뒤 `1 1 1 sc`로 흰 도형을 그리지만 path extractor가 이 색 명령을
  소비하지 않아 이전 검정을 사용했다. `ColorSpace` 자원 → q/Q에 보존하는 해석된 색 공간 →
  페이지/Form 공통 colour operator → `PathContent.fill_color/stroke_color` →
  `src/parser/pdf/path_paint.rs`의 IR fill/벡터 paint → Native/WASM 출력 순서로 보정한다.
  `rg/g/k`의 색 공간 전환, 이름 충돌이 있는 Form scope, stroke와 `SCN/scn`, 저장·복원을
  반례로 사용한다. ISO 32000-1:2008 8.6.5.5·8.6.8(Table 74)를 근거로 하며 문서별
  예외나 흰색 덮어씌우기로 출력을 숨기지 않는다. ICC CMM이 없는 bounded 제품 경로는
  명시적 Alternate/규정된 N별 기본 공간을 사용하고, 다른 색 공간 전체 지원으로 확대해
  보고하지 않는다. 변경 전 313쪽 WASM TSV·10595 PASS는 변경 전 결과로 보존한다.
- 절편별 작은 경계·영향 페이지 판독 뒤 최종 소스의 Native와 실제 Docker WASM을 만든다.
  root pkg/public과 Studio 실제 fetch의 hash를 확인한다. 기존 7700을 임의로 교체하지 않고
  별도 review 서버/탭을 사용한다.
- 실물 313쪽(162+12+4+107+14+14) 및 공개/생성 대조군을 같은 원본 PDF 기준으로 전쪽
  Native/fresh WASM TSV 산출한다. 쪽수·누락·90% 미달과 변경 영역 직접 판독을 각각 확인한다.
- 출력 fixture/golden·배치 회귀는 이 선행 조건 뒤 관계형 계약으로 추가한다. API·자원 한도
  검사는 실제 오류/복구/자원 소비의 계약을 검사하며 미달 배치를 고정하지 않는다.
- 변경 범위의 Rust lint·focused·전체 release-test·Native Skia 3종, Studio 타입/테스트·실제
  browser/Worker·패키지 소비 검증은 [local_validation 4.3](../manual/pr_review/local_validation.md#43-변경-범위별-기본-검증)을 따른다.
- 동일 source/dependency/profile의 결과를 명시적으로 재사용하고 근거 없는 재빌드를 반복하지 않는다.
  최종 devel을 필요한 시점에 통합해 영향 범위를 검증한 뒤 원격/PR 승인을 요청한다.

## 자료·미완료 범위와 단계 종료

입력은 `/mnt/e/tmp/`의 여섯 PDF다. 파일 크기·SHA·원본 쪽수·오류·WASM producer hash와 이슈
등록/재조회는 ignored `output/pr-review/issue7957/registration/`에 보존한다. 개인 PDF·글꼴·
중간 JSON/TSV/로그는 Git에 포함하지 않는다. 50 MiB 이상 입력이 추가되면 저장소 밖 보관 규칙을 따른다.

기존 #7911의 논리 문단·그림 조각 복원·B10-1·전체 codec/font/cache 할당 증명, 다른 PDF paint
지원 전체는 이번 일곱 오류의 자동 완료 조건으로 확대하지 않는다. 실제 출력에 필요한 추가 차이가
나오면 현재 범위와 후속 백로그를 구분해 기록한다. `parserAllocationBoundsProven=false`를 유지한다.

2026-10-17 사용자가 “계획 승인·구현 시작”으로 승인했다. F1부터 순서대로 구현한다.
계획 승인을 수정 후 검증·시각 통과·배포 준비의 완료로 표시하지 않는다.
단계가 바뀌면 현재 변경을 commit하고 다음 보고를 시작한다.
