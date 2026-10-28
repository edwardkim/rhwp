# 중첩 표의 저장 첫 조각과 현재 물리 컷 소유

Issue: #4138, #1949. 시작 head `f74f887afe`, base
`d9fd85ed5bc3c4d9ce6b2981422e48bf9bb90867`.

## 현재 실행 경계

전체 Rust 회귀 `output/pr-review/semanticist21-20261019/current-full-rust-2532/`는
위 head의 Rust/Cargo 소스를 해시 고정해 종료했다. 컴파일 8잡·테스트 12스레드,
고정 `target/pr-review`를 재사용했으며, 실행 중 제품·테스트 소스는 변경하지 않았다.
종료 코드 100과 전체 결과는 아래 확정 결과에 기록했다.

## 실제 출력과 원시 근거

현재 196쪽 저장본의 Native161에서 177~196쪽은 90% 미달이다. 176쪽의 부모
조각은 본문 아래 공간이 남아도 자식 표의 로터리 조종기 둘째 줄을 177쪽으로
넘긴다. 177쪽 표·뒤 문단·그림이 함께 내려간다. 공식 PNG는
`issue4138-native161-tail-review-2537`의 review/overlay이며, 원본 표 호스트는
왼쪽 거대 셀의 문단 2280, 자식 표는 14×2다.

모델을 거치지 않은 `read_tail_nested_raw_frames_2544.py`는 CFB의 Section0
레코드를 직접 읽는다. `tail-nested-raw-frames-2544.json`과 로그에서 확인했다.

| 대상 | 자식 9행 0열의 둘째 문단 vpos | 9행 1열의 둘째 줄 vpos | 10행 0열의 둘째 문단 vpos | 자식 common.height |
| --- | ---: | ---: | ---: | ---: |
| 원본 115쪽 HWP | 0 | 0 | 1600 | 33150HU |
| 현재 분할 저장본 | 0 | 0 | 1600 | 33150HU |
| 같은 분할의 한컴 재저장본 | 1600 | 1600 | 0 | 36884HU |

문자와 원본 크기의 실제 변경 여부는 각 레코드·입력 SHA와 함께 대조한다. 자식
CTRL_HEADER의 너비는 모두 43720HU이며, 위 높이는 해당 레코드에서 직접 읽었다.
현재 저장본 SHA는 `0b3fe8886ad136e2c16d8da7095ce7330c9c2090ce14e2e4bcb91e655686928c`다.
한컴 재저장본은 `mcp-issue4138-hancom-save-2256`의 독립 출력이다. 이전 분할
저장본의 Print와 현재 Print는 196쪽 전부 96dpi RGB가 같다. 연결 근거는
`issue4138-native159-native160-independent-print-comparison-2512.json`과
`issue4138-native160-native161-independent-print-comparison-2529.json`이다.

`tail-cut-trace-2539`의 실제 root 컷은 `[6751,6770)`이다. 유닛 6770은 문단
2280·중첩 9행의 둘째 내용 컷 `[1,1]→[2,2]`이며, `hard_break`·
`stored_frame_break`·`page_frame_reset`가 모두 켜진다. 뒤 간격 용량 부족의
`PHYSICAL_REFIT`가 이 지점에서 발생한 것은 아니다. 저장 경계가 컷을 먼저
닫는 경로와 실제 물리 예산을 구분해 검토한다.

## 원 해결과 현재 호출 경로

- #4069는 원래 큰 중첩 행을 통째로 넘기고, 첫 등장과 continuation이 다른 컷을
  쓰면서 중복하던 문제였다. 원본 이슈는 `issue4069-origin-2543.json`에 보존했다.
- 원 해결 PR은 #4122다(`pr4122-origin-2546.json`). `7c9ce05e629`는 재귀 RowCut을 투영했고, `81777a6a99b`는 원본 문단의
  저장 프레임 경계를 보존했다. 원 저장 문서의 실제 이어받기를 지우는 변경으로
  이번 현상을 고치지 않는다.
- `429dee416e6`(#7704), `d9befb30381`(#7877 통합 보정)은 일반 원본 행의
  저장 컷과 흐름 소유를 함께 확장했다. 해당 정상 입력·보존 계약도 재검증 대상이다.
- 생산 경로는 `cell_units`의 `stored_zero_origin_text_frame_break_witness`와
  `stored_plain_zero_origin_line_boundary` → 자식 `CellUnit`의 경계 표식 →
  선언 행 분해의 `driver_unit` → 부모 `CellUnit` 투영 → 실제 컷 선택이다.
  그 뒤 `nested_table_fragment`의 같은 자식 커서를 paint가 소비한다.

`SavedZeroOriginWitness`는 첫 프레임과 나중 프레임을 이미 구분하지만, bool
경계 표식으로 바뀐 뒤 부모 투영에서도 같은 의미가 유지되는지 확인해야 한다.
자식의 `common.height`는 이 사례에서 현재 쪽 나눔에 따라 변하는 첫 조각 높이다.
과거 첫 조각과 현재 부모 흐름의 대응을 확인하지 않고 독립적인 불변 기하로
취급할 수 없다. 실제 본문 순서·겹침·누락과 현재 밴드의 점유까지 함께 대조한다.

다른 쟁점은 선언 높이가 있는 CELL 행의 합법적인 줄 컷이다. 원본의 10행은 과거
캐시에서 줄이 0→1600HU로 이어지지만 현재 Print는 그 두 문단 사이에서 나뉜다.
과거 reset을 제거하는 것만으로는 새 합법 컷의 존재를 입증하지 못한다. 선언 행
높이·패딩·빈 물리 띠를 버리지 않으면서 측정·선택·paint가 같은 유닛을 소비하는
규칙과 반례를 먼저 확인한다. 행/열 수·문서 ID로 대상을 좁히지 않는다.

## 올바른 출력 환경의 원본 회귀 확인

현재 `info --json`에서 HWP는 한컴 2024 `13.0.0.3622`, HWPX는 한컴 2020
`11.0.0.8362` 저장본이다. 확장자만으로 같은 환경을 가정하지 않았다.
MCP의 각 engine을 명시해 전체 Print를 새로 만들었다
(`mcp-issue1949-exact-originals-2542/results.json`).

| 원본 | 실제 출력 | 쪽수 | Print job | PDF SHA-256 |
| --- | --- | ---: | --- | --- |
| HWP | 2024 `13.0.0.3901` | 115 | `ae492a86-fbf1-403e-a0c9-4e07e3ec16cc` | `88706653cafe77001bcaa3f0ecfb4e64f08987c8d2caa08a66673822cc0fc52a` |
| HWPX | 2020 `11.0.0.9136` | 115 | `d3f25cae-9be4-4501-ada1-04da03fbb04a` | `d3af0cbddf49bce84d25083775ed4e05f7f9bde0fe01af8b772fb7b8dd36a08e` |

둘 다 PrintMethod 0이다. 2024는 `frame_print_to_pdf_ex_one_up`, 2020은
`hancom2020_pdf_driver_one_up`이다. HWP는 알려진 빈 스크립트 placeholder를
해시 대조한 후 비활성화했고, HWPX에는 스크립트가 없다. 원본은 변경하지 않았다.
Native의 세대는 각각 `--compat 2024`, `--compat 2022`로 전쪽 대조 중이다
(`logs/issue1949-exact-originals-native161-2545-driver.log`).

전체 Rust에서 #2214 두 건과 #1949는 HWPX 117쪽과 기대 115쪽의 차이로 실패했다.
단순한 캐시 통계 실패 또는 기준 PDF 버전의 잘못으로 결론 내리지 않는다. 새 독립
Print도 115쪽이므로 출력·페이지 소유를 먼저 보정하고, 검사의 정확한 원본·전체 쪽수·
내용 순서 계약을 지킨다. 미달 상태에서 쪽수 기대값을 117로 바꾸지 않는다.

## 구형 수식의 별도 관측

`equation-version-print-comparison-2526.json`의 버전 한 속성 대조는 원본 빈 버전이
HFT Type3, Version 60이 HYhwpEQ TTF/PUA를 소비함을 보여 준다. 현재 설치 face
1610개를 전진폭·C의 윤곽 경계로 대조한 `legacy-equation-font-scan-2534.json`에는
정확히 일치하는 face가 없다. 87개 읽기 오류가 있어 모든 프로그램을 완전히 해석한
결과도 아니다. 가까운 경계의 글꼴을 원래 face로 선택하거나 이 관측만으로 글꼴 예외를
신청하지 않는다. 첫 진단은 fontconfig family 안의 줄바꿈 처리로 실패했으며, 파일·
index만 읽는 수정 실행의 로그와 결과를 따로 보존했다.


### 새 원본 Print의 실행 결과

Native161과 새 MCP 원본 Print를 같은 세대로 전쪽 대조했다. HWP는 115쪽이
일치하고 113쪽이 90% 이상, 103쪽 35.38844%·104쪽 33.66437%다. 높은 일부
선택 쪽의 기존 결과만으로 이 원본 전체를 통과한 것으로 볼 수 없다. HWPX는
Native 117쪽·Print 115쪽이어서 전쪽 TSV 산출이 중단됐다. 이 실행의 페이지별
비율은 측정 불가이며, 처음 115쪽만 잘라 통과율을 만들지 않는다. 자세한 결과는
`issue1949-exact-originals-native161-2545-result.json`과 해당 파일별 로그에 있다.

읽기 전용 provenance API의 원 구현/프레임 보정 PR 연결 결과는
`issue4069-implementation-pulls-2546.json`·`issue4069-frame-pulls-2546.json`에
보존했다. 현재 모델 검사에 잘못 `convert`로 HWPX 출력을 요청한 한 번의 CLI
실패가 있었다. 원본과 실패 로그는 유지했고, CLI 정본을 확인한 뒤 `export-hwpx`로
고쳐 실행한 `recursive-budget-model-inspection-2536/results.json`을 사용한다.
이 HWPX는 속성 검사 전용 RHWP 산출물이며 독립 한컴 기준을 대신하지 않는다.


## 전체 Rust의 확정 결과와 다음 보정 우선순위

head `f74f887afef11f1b4cba3f49741a0a28185be43e`의 전체 실행은 종료 코드 100이다.
총 11563건, 11495 PASS / 68 FAIL, 기본 ignored 45건이다. 컴파일 포함 1436.11초,
실제 테스트 구간 882.197초다. Rust/Cargo 소스의 전후 해시는 모두 같다. 정확한
명령·로그·receipt는 `current-full-rust-2532/result.json`에 있다.

`terminal-inventory.json`은 1~11563번의 고유 terminal event를 모두 확인했다.
nextest가 뒤 요약에서 같은 FAIL을 다시 나열하므로 단순 정규식 행 수는 136이다.
ordinal·binary·함수·결과가 같은 중복임을 검증하고 68건으로 집계했다. 요약 없는
coverage나 프로세스 종료 추정만으로 완료를 표시하지 않았다.

이전 head `17d4346fcda58e0243be8729d982357dada680ca`의 11554건 universe와 실제
실패 56건을 대조했다. 파생 suite 번호는 달라져 함수 identity로 사용할 수 없다.
통합 binary family와 완전한 함수 이름을 맞추고 이전 universe도 조회한 결과,
47건의 기존 실패·9건의 실제 PASS 전환·21건의 이전 PASS→현재 FAIL이다. 새로
실행 대상이 된 함수가 실패해 생긴 증가는 아니다. 근거는
`full-rust-1975-2532-failure-difference-2547.json`이다.

21건은 원본 HWPX 거대 셀, #7114 저장/재열기, #7336 저장 표, #1488와
`rowbreak-problem-pages`, #7902의 완료 페이지·원본 내용 보존, 쪽수 baseline 등에
집중된다. 현재 수식 130쪽과 분할 저장본의 개선만으로 이 후퇴를 수용하지 않는다.
정확한 기존 검사·98% 완료 페이지의 독립 좌표를 유지하고 구현을 고친다.

재빌드 없이 해시 고정 producer를 같은 원본에 실행한
`frozen-pagecount-comparison-2548/results.json`의 관측은 다음과 같다.

| Native | 원본 거대 셀 HWPX | rowbreak-problem-pages HWPX | #7336 stored-frame HWPX |
| --- | ---: | ---: | ---: |
| 109 | 115 | 18 | 7 |
| 148 | 116 | 18 | 7 |
| 155 | 116 | 19 | 8 |
| 156·158·161 | 117 | 19 | 8 |

이는 기본 profile의 쪽수 진단이며 전쪽 Visual Sweep을 대신하지 않는다. 각
producer의 source receipt·파일 SHA와 입력 SHA를 함께 고정했다. 모든 후퇴를
마지막 줄 경계 커밋 하나의 탓으로 돌리지 않고, 중간 producer를 같은 조건으로
추가 대조해 실제 원인 변경을 좁힌다. 이전 구현으로 통째로 되돌려 현재의 독립
Print 개선을 없애거나, 기대값을 올려 실패를 숨기지 않는다.


### 보존 producer로 좁힌 세 경계

`frozen-pagecount-comparison-2549`·`2551`에서 같은 세 원본을 중간 producer에
일괄 적용했다. 재컴파일이나 테스트 기대값 변경은 하지 않았다.

- 거대 셀 HWPX는 Native143까지 115쪽, Native144부터 116쪽이다. 두 producer의
  제품 해시 차이는 `equation_line_metrics.rs`, `table_layout.rs`,
  `deferred_flow_units.rs`다. 144의 혼재 발행 초안은 이후 철회됐지만 145~155에서도
  116쪽이 남으므로 수식 변경 하나로 원인을 단정하지 않는다. 당시 보고서는
  `task_semanticist21_20261023_deferred_frame_scope.md`에 있다.
- rowbreak-problem-pages와 #7336은 Native148에서 각각 18·7쪽이지만 Native149부터
  19·8쪽이다. 차이 소스는 `rowbreak_frame.rs`, `table_layout.rs`,
  `row_physical_band.rs`, `stored_span_content.rs`, scan runner의 `row_step.rs`다.
  바깥 물리 프레임·자식 viewport와 마지막 슬롯의 소비 위치를 함께 대조한다.
- 거대 셀 HWPX는 Native155의 116쪽에서 Native156의 117쪽으로 다시 늘었다.
  이 경계의 차이 소스는 지역 float frame·table units·block host와 부분 paint다.
  같은 보정이 현재 분할 문서에서 개선한 원점·뒤 공백 소유도 보존한다.

`rowbreak-before-after-trace-2550`은 같은 원본과 `--compat 2022`의 실제 17쪽
레이아웃을 Native148과 Native161로 추적했다. 148은 pi28의 이어받은 컷
`(11, end)`를 17쪽 위에 배치하지만 161은 같은 쪽 아래에서 첫 컷 `(0,11)`을
시작한다. 현재 17쪽의 표만 따로 옮겨 맞출 문제가 아니라 앞 조각의 실제 수용·점유가
먼저 바뀐 결과다. 이 위치 차이를 clamp하거나 그림을 숨기지 않는다. 전후 로그는
`logs/rowbreak-before-after-trace-2550-native148.log`·`...native161.log`다.

## 양수 원점 재시작 앞의 그림 소유 보정

최신 fetch에서도 `upstream/devel`은 `d9fd85ed5bc3c4d9ce6b2981422e48bf9bb90867`이며
현재 head의 조상이다. 새 리베이스는 수행하지 않았다. 실제 같은 HWPX를 세 producer에
실행한 `original-hwpx-boundary-trace-2552`에서 최초 그림 유닛 순서 차이를 확인했다.

원본 HWPX의 부모 셀 문단 315는 마지막 줄 원점 72960HU, 문단 316은 17678HU다.
그림을 앵커한 문단 310의 지연 띠는 17678HU(235.706667px)다. 따라서 다음 본문이
그림 뒤에서 양수 원점으로 시작하는 실제 저장 프레임이다. 기존
`stored_deferred_picture_frame_before`가 이를 `hard_break_before`로 인증하지만,
설계상 `stored_frame_break_before`로 승격하지는 않는다. 줄 내부 재시작과 그림/본문이
함께 소유한 프레임은 같은 신호가 아니다.

반면 `deferred_flow_units::same_cell_stored_frame_boundary`는 저장 프레임 플래그만
받아서 이 경계를 놓쳤다. Native143은 문단 316 앞에서 그림 띠를 닫지만 Native155·161은
문단 335의 네 번째 줄까지 넘긴다. 원본 ZIP의 직접 XML과 실제 유닛 순서로 대조했으며
문서 번호나 좌표를 구현 조건으로 쓰지 않았다.

후보는 이미 인증한 hard break도 검사하되 같은 셀의 인접 비합성 원본 줄에서 실제
원점 되감김을 계속 요구한다. 자식 reset·단조 재조판 사다리·먼 명시적 쪽 나눔만으로는
그림을 이동하지 않는다. 경계 플래그 자체를 바꾸지 않고 그림 띠를 기존 컷 앞에 닫는다.
동일한 정렬 유닛 원장은 기존 컷·측정·paint가 소비한다. Native162를 한 번 빌드한 뒤
정확한 원본 쪽수와 기존 물리 프레임·보존 검사 묶음을 함께 재실행한다. 결과를 받기
전에는 115쪽 회복이나 시각 통과를 주장하지 않는다.

### Native162의 실제 결과

빌드는 265.50초·종료 코드 0이며 생산 소스 해시가 그대로임을 확인했다
(`char-overlap-native-build-162-result.json`). `native162-focused-validation-summary-2560.json`에서
기존 133개 계약과 검사 소스 해시를 대조해 133 PASS를 확인했다. 전체 실행의 새 실패
모듈까지 확장한 기존 검사 묶음은 189건 중 171 PASS / 18 FAIL이다. 실패 18건은 이미
확인한 실패이며, 이 묶음만으로 전체 Rust의 68 FAIL이 해소됐다고 보고하지 않는다.
새 테스트나 기대값은 추가·변경하지 않았다.

원본 거대 셀 HWPX는 기본 조판 117→116쪽이다. 아직 Print 115쪽과 달라 전쪽 TSV는
산출되지 않았고 통과가 아니다. rowbreak-problem-pages HWPX 19쪽, #7336 HWPX 8쪽도
그대로다. 원본 HWP는 새 Print와 전쪽 비교해 115쪽·113쪽 90% 이상·103/104쪽 미달이
같으며, 이전 Native161과 115개 PNG의 파일 SHA까지 전부 같다. 증거는
`issue1949-exact-originals-native162-2555-result.json`이다.

HWPX의 첫 15쪽 PNG는 이전과 같고 16쪽부터 달라졌다. 15쪽 진단을 새 개선 증거로
표시하지 않았다. 공식 같은 16쪽의 raw는 29.13116→19.47829%이며
`issue1949-native162-frame-review-2559`의 review와 standalone overlay를 직접 판독했다.
현재 16쪽의 그림·3.4.7 및 다음 본문은 Print 15쪽에 속할 내용이다. 쪽수가 하나 줄었다는
이유로 이 큰 내용 귀속 차이를 면제하지 않는다. 앞쪽의 추가 쪽과 현재 물리 프레임 소유를
함께 보정할 중간 상태로 보존하며, PR 준비 완료·시각 통과로 판정하지 않는다.

`stored-frame-boundary-trace-2557`의 #7336 전후 출력에서도 일반 행 수용 하단과 빈 종료
띠 하단을 합친 뒤 내용 예산이 추가로 줄어든 것을 확인했다. 실제 저장 프레임을 쓰는
개체/중첩 유닛과 새로 구성한 일반 글줄의 예산 소유를 구분해 다음 보정에서 대조한다.

## 현재 컷의 일반 글줄과 개체 viewport 구분

다음 후보는 행 전체에 있는 먼 개체를 보고 예산을 고르지 않는다. 공통 물리 컷 계획으로
현재 밴드에 들어갈 실제 유닛 범위를 구한 뒤 그 범위가 일반 글줄뿐일 때 바깥 내용
프레임을 적용한다. 개체/중첩 유닛을 실제로 소비한 범위는 그 유닛의 기존 viewport
예약을 유지한다. 같은 행의 다음 쪽에만 있는 개체 때문에 현재 일반 글줄의 경계가
사라지는 문제와, 현재 개체에 이미 예약한 물리 몫을 다시 빼는 문제를 함께 대조한다.

또한 내용 수용 하단과 빈 종료 띠 하단의 기존 구분을 복원한다. 단독 셀의 조각 프레임은
기존 단독 셀 계약을 유지하고, 일반 격자의 내용 컷에 빈 띠 inset까지 추가하지 않는다.
이는 앞서 읽은 `budget.rs`의 원본 HWPX·병합 HWP5 내용 경계 계약과도 일치해야 한다.
인증한 하단을 빈 띠와 합칠 때는 내용 하단이 아니라 빈 띠 하단을 각각 소비한다.
Native163에서 기존 189개 검사, 세 원본 쪽수, 현재 분할 저장본의 경계와 독립 Print를
같이 대조한다. 실제 결과를 받기 전에는 이 가설을 완료 규칙으로 채택하지 않는다.

### Native163의 개선과 반증

248.15초·종료 코드 0으로 빌드했고, 기존 189개 검사는 184 PASS / 5 FAIL이다.
기존 133개 물리 계약은 모두 유지했다. #1949·#2185의 원본 쪽수/내용 검사,
#1488·rowbreak의 18쪽·#7902 완료 페이지 검사와 #7114의 중첩 컷 저장 검사가
다시 통과했다. #7114 저장/재열기 세 건과 #7336 두 건은 남아 있다. 정확한
각 함수의 결과는 `reviewed-contract-tests-2563-result.json`에 있다.

새 MCP Print와 원본 HWP/HWPX를 각각 전쪽 대조해 둘 다 115쪽·최저 raw
95.69614%·90% 미달 0쪽을 확인했다(`issue1949-exact-originals-native163-2564-result.json`).
`not_evaluated`는 직접 판독·fresh WASM·최종 승인 완료가 아니다. #7840 전 9쪽
최저 93.43362%, 현대 수식 v5/v6 전 7쪽 최저 99.98497/100%도 유지했다.

반면 고정한 #4138 분할 저장본은 196쪽이 같아도 12쪽만 90% 이상이고 13쪽부터
크게 달라진다(`issue4138-native163-boundary-batch-2565-candidate-result.json`).
따라서 격자의 내용/빈 띠 분리를 모든 사다리에 적용한 후보는 채택하지 않는다.
실제 원본 부모의 저장 쪽 프레임과 API가 재생성한 단조 부모 사다리를 구분해야 한다.
앞의 원시 분석에서 분할본의 경계는 자식에서 전파됐으며 부모 자체의 되감김은 없다.
원본 격자의 이전 컷 계약을 그 재생성 사다리의 수용량으로 옮긴 가정이 잘못됐다.

보완 후보는 기존 `original_stored_table_frame_is_usable`을 확인한 뒤,
부모 셀의 비합성 원본 줄이 실제로 되감긴 경우만 저장 격자 프레임으로 사용한다.
자식 reset·0/0·편집/재조판 상태는 그 증거가
아니다. 같은 인증을 내용 하단과 현재 개체 viewport의 바깥 예산 선택이 함께 소비한다.
재생성 사다리는 Native162와 같은 실제 바깥 조각 하단을 유지하고, 원본은 Native163의
전쪽 개선을 보존해야 한다. 두 문서군과 기존 189개 검사를 다시 대조한다.

### 부모 원본 인증에서 선언 셀 높이의 잘못된 가정 제거

Native164는 빌드 243.61초·종료 코드 0이지만 원본 HWPX가 다시 116쪽이다
(`native164-original-hwpx-info-2573.json`). 이 후보를 그대로 전체 검증에 넘기지 않았다.
원본 부모의 선언 행 합이 개체 높이를 넘어야 한다는 추가 전제는 잘못됐다.
`read_raw_parent_frames_2574.py`는 모델 없이 원본 HWP의 CFB·HWPX의 XML을 읽었다.

| 입력 | 부모 개체 높이 | 부모 셀 높이 | 부모 원본 줄 되감김 |
| --- | ---: | --- | ---: |
| 원본 HWP | 73326HU | 세 칸 각각 282HU | 114 |
| 원본 HWPX | 73326HU | 세 칸 각각 282HU | 114 |
| 고정 분할 저장본 | 846HU | 네 칸 각각 282HU | 0 |

`cell.height`에는 패딩만 저장될 수 있다는 기존 계약을 그대로 적용한다. 작은 선언 셀
높이의 합으로 실제 원본 줄의 쪽 재시작을 기각하지 않는다. 반대로 온전한 행 합만으로
원본 쪽 재시작이 없는 재생성 사다리를 인증하지도 않는다. 원본 source SHA·원시 값은
`raw-parent-frames-2574.json`·`logs/raw-parent-frames-2574.log`에 있다.
Native165는 그 잘못된 전제를 제거한 동일 범위의 보완 후보다.

## Native165에서 원본 개선과 기존 분할 출력의 동시 보존

빌드 245.97초·종료 코드 0이며, 생산 당시의 실제 Rust/Cargo 소스는
`char-overlap-native-build-165-result.json`에 해시 고정했다. 문서만 다른 과거
head의 결과를 현재 제품 소스 검증으로 재명명하지 않았다.

| 확인 | 실제 결과 | 근거 |
| --- | --- | --- |
| 기존 물리 프레임 계약 | 실제 132 PASS / 1 ignored, 검사 소스 해시 동일 | `native165-focused-summary-2582.json`; 실행 집계는 `focused-terminal-audit-2635.json`으로 정정 |
| 기존 실패 모듈 포함 | 실제 181 PASS / 7 FAIL / 1 ignored; Native162의 실패 11건 PASS 전환, 새 실패 없음 | `reviewed-contract-tests-2576-result.json`; `focused-terminal-audit-2635.json` |
| 원본 HWP와 새 2024 Print | 115/115쪽, 전쪽 raw 최저 95.69614%, 90% 미달 0쪽 | `issue1949-exact-originals-native165-2577-result.json` |
| 원본 HWPX와 새 2020 Print | 115/115쪽, 전쪽 raw 최저 95.69614%, 90% 미달 0쪽 | 같은 파일의 HWPX 원본 행 |
| 고정 분할본과 2024 Print | 196/196쪽, 176쪽 90% 이상·177~196쪽 미달; 최저 38.91518% | `issue4138-native165-boundary-batch-2578-candidate-result.json` |
| 분할본의 기존 개선 보존 | 이전 Native161과 196개 Native PNG의 파일 SHA가 전부 같음 | `native165-focused-summary-2582.json`에 연결한 전쪽 산출물 |
| 합성 Print 대조군 | 35문서/66쪽 전부 raw 100%·쪽수 일치 | `reflow-*-native165-2579/results.json`, `logs/native165-matrix-2579-driver.log` |
| #7840·현대 수식 | #7840 전 9쪽 최저 93.43362%; 현대 수식 v5/v6 전 7쪽 최저 99.98497/100% | `issue4138-native165-boundary-batch-2578-result.json` |
| 형식·Native root Clippy | 각각 종료 코드 0, 29.24/79.81초 | `native165-checks-2580-result.json` |

같은 104쪽의 원본 HWP review·standalone overlay를 직접 판독했다. 원본 Print와
Native의 이어받은 표, 로터리/보조 제어장치 줄, 후속 본문과 두 조종 손잡이 그림이
같은 순서·위치에 있다. raw는 Native161의 33.66437%에서 99.99053%다. HWPX
16쪽도 직접 대조해 잘못 넘어온 그림·3.4.7 대신 Print와 같은 3.5.3 이후 본문이
같은 쪽에 있고, 줄 끝·문단 간격을 유지함을 확인했다(raw 29.13116→100%).
네 공식 비교 결과는 `native165-original-reviews-2581/results.json`에 연결했고 사용자에게
각각 review와 overlay를 함께 표시했다. 새 Native 출력이며 과거 PNG 재사용이 아니다.

남은 7건은 #2185의 한 글자 편집, #7114의 네 저장/재열기 검사와 #7336의 두 검사다.
이들은 기존 Native162에서도 실패했던 항목이며 검사·기대값을 변경하지 않았다.
Native163의 5 FAIL과 비교하면 두 편집 경로는 다시 실패한다. 해당 개선만으로 분할본의
184쪽 시각 후퇴를 수용하지 않았고, 양쪽을 함께 만족시키는 보정을 계속한다.

#7336의 두 번째 실패는 실제 행 높이 40.5px 위반이 아니라 6쪽 동의서 표가 다른 쪽으로
이동해 노드를 찾지 못하는 실패다. `stored-frame-boundary-trace-2570`에서 마지막
15행의 앞 예약은 552.187px로 같은데, Native148은 개체를 품은 마지막 행의 기존
예산을 쓰고 Native163은 일반 글줄 예산을 다시 적용하는 차이가 남았다. 원본 마지막
문단의 9677HU LineSeg가 TAC 표를 소유하는 경로를 다음에 추적한다. 쪽수 기대 7과
정확한 기존 행 높이를 바꾸지 않는다.

이번 결과는 Native 선행 보정 기록이다. 마지막 전체 Rust의 68 FAIL을 새 전체 결과로
갱신하지 않았고, fresh WASM·전체 Rust·WASM/workspace lint·나머지 통합 범위의 필수
검증이 아직 완료되지 않았다. 분할본 뒤 20쪽도 계속 보정한다. 새 회귀·fixture 추가,
PR 생성·원격 push·승인·merge는 수행하지 않았다.

### 집중 검사 집계 정정

후속 `task_semanticist21_20261024_edit_frame_coordinate_space.md`의 실행 감사에서
진단 드라이버가 ignored 검사 1개의 exit 0을 PASS로 센 오류를 확인했다. 위 Native165
표는 실제 terminal 결과로 정정했다. 원 JSON·로그는 그대로 보존하며 제외 검사는
실행 성공 증거가 아니다. Native168의 후속 결과는 실제 188 PASS / 0 FAIL / 1 ignored다.
