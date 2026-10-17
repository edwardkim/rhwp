# Task #7957 — PDF 열기 일곱 유형 공동 구현계획

- Issue: #7957; 공동 #7958·#7959·#7960·#7961·#7962·#7963
- 작성일: 2026-10-17; 상태: F1–F5 및 추가 색상·글꼴/간격 보정의 로컬 검증 완료, 원격 반영 전
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

### 2026-10-17 사용자 범위 확장: operators 12쪽

사용자가 남은 12쪽도 이번에 보정하도록 승인했다. PDF의 영문/PostScript 이름과
설치 family의 동일성을 먼저 확인하고 canonical font-rule v2 절차로 연결한다.
이름이 없는 Type3는 별도 fallback 범위로 구분한다. PDF TJ의 위치 기반 단어 간격은
기존 표 변환의 공통 복원을 일반 positional 텍스트에서도 소비하도록 한다. 전쪽 Native/
fresh Docker WASM TSV와 영향 영역 판독으로 확인하고 기존 색상 통과 증거와 구분한다.

최종 시각 판정과 313쪽 Native/fresh WASM 결과, 10,599 PASS / 45 skip,
Rust/Node/browser 검증 범위는 [7단계 결과](../working/task_m100_7957_stage7.md)에 연결했다.
Type3 원 glyph·전체 parser 할당 증명의 미완료 경계는 유지한다. 원격 반영/PR을 준비하는
시점에 최신 devel 통합 여부를 확인하며 이번 로컬 검증을 미실행 원격 CI의 성공으로 표시하지 않는다.


### 2026-10-17 추가 범위 — #7966 이미지 배치 예산

사용자가 `exceeds_32.pdf`도 이번 공동 처리에 포함하도록 승인했다. 19쪽 중 6쪽에는
18개 Image XObject가 35회 실제 배치된다. `handles.len() > 32`라는 페이지 고정 제한을
요청 전체의 `max_document_image_placements`로 대체한다. 기본값은 기존 XObject 방문의
유한한 요청 규모(10,000)와 연결하며, 신뢰 호스트는 더 낮은 값을 지정할 수 있다.
개별·누적 픽셀 및 decoded/codec·XObject·연산·pool 제한은 유지한다. 정적인 개수의
상향만으로 자원 제한 전체를 해결했다고 주장하지 않는다.

실제 소비 경로는 page/Form/inline의 paint handle → 요청 누적 예약 → decode/clip →
이미지·source_order → 공통 lowering/최종 paint다. 같은 자원의 실제 반복 배치는 모두
보존한다. invisible handle도 열거 작업으로 보수적으로 계상하며 기존 숨김 동작은 바꾸지 않는다.
열거 전 할당 자체의 완전한 경계는 이번 보정의 증명이 아니다.

새 원본 19쪽과 32회 초과 생성 대조군의 전쪽 Native/fresh Docker WASM을 먼저 비교한 뒤,
관계형 출력 회귀와 요청 누적·경계/초과·실패 뒤 복구 계약을 검증한다. 기존 여섯 원본의
기준 TSV 불가 예외는 새 원본도 기준 거절을 확인한 뒤 같은 신규 지원 입력으로 기록한다.
이미 진행된 f6096ff5 빌드·검증은 변경 전 증거로 보존하고 추가 소스에 재인용하지 않는다.


#### #7966 추가 실패 — 7쪽 Form BBox의 affine clip

32회 제한 보정 후에는 7쪽만 `non-rectangular image clipping`으로 거절됐다. 나머지 18쪽과
XObject/Form/inline 33회 생성 대조군은 실제 공개 WASM으로 가져왔다. 이 결과를 19쪽 전체
완료로 표시하지 않는다. 7쪽 Form 239의 BBox는 `[0 0 180.01 50.018]`, 호출 CTM의 두
비대각 성분은 `-1.88e-7`이다. 축에 정확히 평행한 경계만 인정하던 ClipPath가 이 실제
변환을 Unsupported로 분류했다. epsilon으로 회전 성분을 지우거나 bbox만을 clip으로 쓰지 않는다.

이미지 경로에 한해 최대 32 꼭짓점의 직선 볼록 clip과 교집합을 보존하고, 원래 CTM의
역변환·실제 clip 내부 판정으로 RGBA를 구성한다. 투명도/마스크를 먼저 합성하고 새로운
축 정렬 저장 프레임·matrix를 함께 생산해 lowering의 crop이 동일한 결과를 소비한다.
프레임은 정확한 물리 경계이며 ceil은 저장 픽셀 수에만 적용한다. 기하학적 clip 품질은
기존 compound-path와 같은 최소 3 pixels/point, 더 높은 원본 해상도는 보존한다. 기존의
RGBA·codec·decoded 예약, 16M 픽셀 및 유한한 dimensions 한도를 적용한다.

직사각형 crop은 기존 편집 가능한 crop 경로를 유지한다. 별도 alpha compositor가 없는
vector paint의 clip 해석은 기존의 원래 경로 처리를 유지한다. 곡선·오목·복합 clip과
32 꼭짓점 초과는 미지원으로 남긴다. bbox 하나로 내용 유출을 숨기는 지원으로 확대하지 않는다.
양·음 winding, reflection, 실제 회전 Form BBox, 저해상도 원색 그림의 대각 clip 대조군으로
동일 프레임·내용·투명도를 확인한 뒤 최종 producer의 전체 gate를 수행한다.

#### #7966 직접 판독 뒤 추가 승인 — 연결선·글꼴·간격

작업지시자는 19쪽의 남은 출력 차이도 이번에 보정하도록 승인했다. 6쪽의 `s`
close-and-stroke 누락은 페이지·Form interpreter 모두에서 같은 path extractor로 전달한다.
Bold PostScript 이름은 설치된 full face 정체성을 보존하고, 측정의 metric alias도
별도 canonical 결정면에서 연결한다. Regular로의 무조건 치환은 하지 않는다.
독립 경계와 실제 6·7쪽부터 대조한 뒤 고정 소스의 최종 gate를 진행한다.
실행 근거·결과는 [9단계](../working/task_m100_7957_stage9.md)에 연결한다.
