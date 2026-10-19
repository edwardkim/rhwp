# Stage 90 — 미주의 저장 내부 컷과 실제 앞부분 수용

## 독립 근거와 수정 범위

변경 전 source는 `9605623089dd2540e63abc3782efa129351a2fc3`다.
Stage 89의 저장 표 보정은 all-20의 13쪽을 개선했지만 all-24의 12·13쪽과
all-20의 16·17쪽 경계는 해결하지 않았다. 전체 원본·회귀 범위를 유지한다.

실패 원본은 `samples/3-11월_실전_통합_2024-구분선위0미주사이7구분선아래20.hwp`다.
대응 자매 HWPX의 저장 LineSeg와 독립 한컴 Print
`mydocs/pr/assets/pr_7600_control_print_20261006/input-13-3-11월_실전_통합_2024-구분선위0미주사이7구분선아래20-2024.pdf`를
같은 내용으로 대조했다. 기준 PDF를 새 입력으로 대체하지 않는다.

- 미주15의 내부 문단16은 6줄 본문이다. 저장 줄의 두 번째 뒤에서
  `43380 → 0HU`로 좌표가 리셋된다. 명시적 문단/단 나눔 속성은 없다.
- 현재 Native는 12쪽 왼쪽 단에서 해당 문단을 통째로 넘긴다. Print는
  `(ii)`와 그 뒤 두 줄, 총 세 줄을 왼쪽 단에 남긴다. 오른쪽 단은 그 다음
  수식 줄에서 시작한다.
- 실제 앞 항목의 흐름 끝은 933.08, 단 높이는 1001.56이다. 세 줄의 흐름
  합은 약 58.83이고 실제 점유는 그 후행 간격보다 작다. 독립 Print에서
  동일 세 줄이 본문 안에 있음을 확인했다. 이 좌표는 진단 관측값이며
  정식 테스트의 고정 px 기대값으로 사용하지 않는다.

## 잘못된 가정과 소비 경로

1. `paragraph/initial_fit.rs`는 일반 자동 컷을 세 줄로 계산하지만 저장
   vpos가 내부에서 되감기면 `query_reflowed_endnote_fit`을 호출하지 않는다.
   실제 실행 로그는 `before_style=Some(3) actual=None cut=Some(3)`이다.
2. `fit.rs::judge_internal_rewind_full_advance_needed`는 저장 컷보다 수용
   컷이 크고 특정 미주 간격이면 문단 전체 이동을 요구한다. 저장된 자동
   조판 컷이 현재 가용 공간에서 들어가는 내용 컷을 금지한다는 근거는 없다.
3. `paragraph.rs`의 `advance_for_fit`은 이 결과로 단을 넘긴다. 그 뒤에는
   이전 단의 실제 앞부분 수용 조회를 하더라도 그 단 소속을 복원할 수 없다.
4. 기존 `query_reflowed_endnote_fit`은 같은 column items·표 원장·최종 layout
   경로에서 Whole/PartialParagraph의 실제 점유를 측정한다. 그 결과는 이동
   전과 최종 split candidate에서 같은 단·항목 수일 때만 소비된다.
5. `emit_endnote_split`은 이 확정 내용 컷으로 앞부분과 이어받기를 방출한다.
   실제 배치도 같은 소유 문단·줄 범위를 소비한다.

따라서 일반 inline 미주 본문의 실제 수용 조회에서 저장 vpos 되감김에 따른
호출 차단을 제거한다. 문서 ID·미주 간격·화면 좌표 조건을 추가하지 않는다.
유효한 저장 컷은 진단 정보이며 현재 출력의 내용 수용을 대신하지 않는다.

## 비적용 경계와 검증 계획

- 명시적 단 나눔, keep-with-next, keep-lines, widow/orphan 및 별도 개체
  예약은 기존 조회의 계약을 따른다. Table/Picture/Shape와 비inline 수식,
  저장/조판 줄 수 불일치 등은 기존 guard로 분리한다.
- 앞부분이 들어갈 때만 이동을 취소한다. 아무 줄도 못 들어가면 실제
  Overflow를 소비하며 빈 조각을 방출하지 않는다. 묶음 속성 때문에 앞부분을
  나눌 수 없는 경우에도 전체의 실제 초과 판정을 소비한다. 단/항목이 바뀌면
  이전 결과를 재사용하지 않는다.
- 네 실패 원본의 같은 Print PDF에서 컷 앞뒤 내용·문단 소유·누락·중복,
  다음 문항과 전체 쪽수를 확인한다. 변수·미주 대조군도 같은 source로
  재검증한다. 작은 직접 비교에서 방향을 확인한 뒤 fresh WASM과 전체
  검증을 진행한다.
- 관련 모든 페이지의 Native/fresh WASM 90% 이상과 확대 판독 전에는
  정식 회귀·기대값을 변경하지 않는다. 55개 원본·853쪽, 기존 91개 파일과
  #1139 전체의 관계형 전환 및 전체 게이트·PR은 미완료다.

## 현재 상태

ignored `stage89-original-diagnostics/all24-dump-receipt.json`, `all24-pages.json`,
`all24-flow.log`와 `stage89-table-source-ladders.json`에 수정 전 입력·명령·해시·
실행 결과를 보존한다. 위 원인을 구현하기 전 확인했으며 수정 후 출력은
아직 미검증이다.

## 구현과 선행 검증

source `b770b1d2e01ddc3570771c313f9e952a8df4e97a`에서 일반 inline 본문의
실제 Whole/Partial 수용 조회가 저장 되감김에도 호출되도록 연결했다. 같은
단과 항목 수가 유지되면 `can_stay`와 `Overflow` 모두 실제 이월 판정에서
소비한다. 최종 split candidate도 그 공통 결과를 사용한다. 후속 선행 방출이나
단 전환으로 문맥이 달라진 경우에는 기존 조회를 재사용하지 않는다.

fmt, Native Clippy, WASM Clippy, Skia check가 순차로 모두 exit 0이다.
`stage90-endnote-prefix-occupancy-type-checks.json`에 명령·실행 source·파일
해시·실제 종료 코드를 보존했다. workspace 전체 lint와 전체 nextest는
이 결과에 포함하지 않는다.

현재 같은 source의 Native 실행 파일을 고정 `target/pr-review`에서 빌드한다.
`stage90-native-validation-pipeline.json`의 소유 프로세스와 실제 Cargo PID를
확인했다. 성공하면 네 실패 원본의 전체 쪽을 한 번씩 Skia 출력해 독립
Print와 비교하고 대표 PNG를 생성한다. all-24의 실제 문단 소유·앞부분·이어받기도
`dump-pages`로 기록한다. 새 후보의 시각 결과는 아직 미검증이다.

선택 37개 입력·97쪽과 전체 55개 원본·853쪽의 Native/fresh WASM 드라이버를
준비했다. 전체 드라이버는 아직 실행하지 않았다. 이전 source의 중복 WASM·
선택 Native 검증은 소유를 확인하고 종료했으며 기존 queue 스냅샷과 산출물을
`stage90-superseded-processes.json`에 연결했다. 해당 결과를 현재 source의
통과로 보고하지 않는다. Cargo 작업이나 공유 산출물은 중단·삭제하지 않았다.

## 실제 Native 빌드와 원본 전체 직접 비교

source `b770b1d2e01ddc3570771c313f9e952a8df4e97a`의 Native 빌드는 exit 0,
454.886초로 종료했다. 동결 binary SHA-256은
`7e786e1bd3d5bb5ce00cf193b6fd4c5541ea0460e246e0b4bcd56ef10ef0a54e`다.
`endnote-prefix-occupancy-native-provenance.json`에서 빌드 전후 소스 해시가 일치한다.

기존 실패 원본 9개의 전체 191쪽을 독립 Print와 직접 Skia 비교했다.
6개 원본은 전쪽 90% 이상으로 개선됐으며, 다음 3개는 여전히 미달이다.
이 결과는 방향 확인용이며 Native SVG/fresh WASM 전쪽 TSV의 최종 통과를 대신하지 않는다.

| 원본 | 비교 쪽수 | 최저 일치율 | 90% 미만 쪽 |
|---|---:|---:|---|
| `all-24` · `samples/3-11월_실전_통합_2024-구분선위0미주사이7구분선아래20.hwp` | 21 | 96.42088% | 없음 |
| `all-20` · `samples/3-11월_실전_통합_2024-구분선위0미주사이20구분선아래2.hwp` | 22 | 95.61623% | 없음 |
| `all-25` · `samples/3-11월_실전_통합_2024-구분선위0미주사이7구분선아래20.hwpx` | 21 | 96.42080% | 없음 |
| `all-21` · `samples/3-11월_실전_통합_2024-구분선위0미주사이20구분선아래2.hwpx` | 22 | 30.54841% | 19, 20, 21 |
| `all-28` · `samples/3-11월_실전_통합_2024-구분선위20미주사이7구분선아래2.hwp` | 21 | 96.42088% | 없음 |
| `all-29` · `samples/3-11월_실전_통합_2024-구분선위20미주사이7구분선아래2.hwpx` | 21 | 96.42080% | 없음 |
| `all-39` · `samples/exam_math.hwp` | 20 | 77.46046% | 1, 2, 9, 13, 17, 18 |
| `all-41` · `samples/exam_math_no.hwp` | 20 | 76.50795% | 1, 9, 13, 15, 16, 17, 18 |
| `all-10` · `samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwpx` | 23 | 90.53366% | 없음 |

증거는 ignored output의 `stage90-original-skia/receipt.json`,
`stage90-additional-skia/receipt.json`에 입력/PDF 해시와 페이지별 PNG 해시로 보존했다.
실행 당시 첫 프로세스는 원본 4개만 로드했으며 추가 5개는 별도 프로세스로 실행했다.
review 생성 시 zero-based page index 오류를 고쳐 실제 쪽 번호와 캡션을 일치시켰다.
`stage90-review-label-correction.json`은 재생성 PNG와 쪽 번호를 기록한다.

all-24의 12·13쪽은 각각 98.09573%, 98.21438%다. 독립 Print와 직접 판독한
미주15 문단16의 왼쪽 0..3 / 오른쪽 3..6 컷을 확인했다.
all-20의 16·17쪽은 각각 98.18284%, 96.90130%다.

## 다음 보정의 실제 원인

- all-21 HWPX의 첫 차이는 미주 내부 pi864다. 앞 항목 흐름 끝 863.813에서
  실제 한 줄 점유 끝 891.4는 가용 높이 1001.56 안이다. HWP는 남기지만 HWPX는
  `paragraph.rs`의 선행 tail overflow 분기에서 먼저 넘긴다. 공통 실제 fit 조회가
  한 줄 문단을 제외해 그 결과가 선행 분기에 전달되지 않는다. 저장 좌표 정규화의
  문제로 추정하지 않는다. `stage90-twin-pagination-diagnostic/`에 실제 실행을 보존했다.
- all-39/41의 셀 쪽 번호 도형은 원본에서 가로/세로 모두 PAPER 기준이다.
  `layout_cell_shape_with_parent_path`가 가로는 셀, 세로는 PARA 외 모든 경우 셀을
  기준으로 삼아 용지 정렬을 무시한다. `stage90-math-header-diagnostic/`의 RHWP
  재직렬화는 파싱 속성 확인용이며 독립 한컴 저장본/Print로 승격하지 않는다.

## 회귀 전환 범위 재확인

기존 17개 파일 후보 외에 10개 파일의 22개 검사 함수를 직접 읽고
`stage90-additional-regression-geometry-classification.json`으로 분류했다.
고정 좌표 선택과 소유 관계 검사를 구별한 조사이며, 변환 완료 기록이 아니다.
이번 단계에서는 정식 회귀 테스트와 기대값을 추가·변경하지 않았다.
55개 원본/853쪽, 기존 91개 파일과 #1139 전체 관계형 전환의 완료 판정은 보류한다.
