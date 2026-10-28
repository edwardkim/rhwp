# 편집 후 저장 좌표계와 TAC 줄 프레임 소유 보정

Issue: #2185, #7114, #7336. 시작 head `ee4523938ea42d7c10675d53745e28b756a53018`,
base `d9fd85ed5bc3c4d9ce6b2981422e48bf9bb90867`.

## 선행 분석과 독립 근거

- #2185 원 해결 PR #2194, merge `cb51cdfccb1f3f3d2d33958c247766660d5a3e96`은
  한글 글자/어절 매핑을 정정했다. 현재 한 글자 입력은 네 줄 시작점·다음 문단 원점을
  보존하지만 전체 쪽수에서 실패한다. 줄바꿈 매핑을 다시 바꾸거나 기대값을 완화하지 않는다.
- #7114 원 해결 PR #7404, merge `08257e569c2a4614b453bcea7fb35771541fe8de`는
  실제 pagination cut을 저장 스냅숏의 LineSeg에 썼다. 통합 #7568의
  `731de9e1b4`와 현재 코드를 함께 읽었다. 실제 `RHWP_DIAG_WRITEBACK`에서
  live/render의 유닛 수·anchor가 모두 같다. 매핑이 다르다는 가정으로 저장기를
  다시 쓰지 않는다. source/provenance와 재열기의 예산 차이를 먼저 보정한다.
- #7336 원 해결 PR #7368, merge `b3e3d4e2170a43ca449e3d832440a9274e4e8ee4`는
  실제 소비 컷을 덮는 조각 상자와 큰 저장 표의 행 분할을 고쳤다. 원본 두 문서의
  최종 Native/fresh 전쪽 증거는 원 PR 메인터너 보정과 #7902의 `84764fa21c`에 있다.
  현재 동의서 표 검사는 높이 오차가 아니라 앞 표가 한 쪽 늘어 노드를 못 찾아 실패한다.

원문과 timeline은 `remaining-origins-2584.json`·`2585.json`, 세 원 PR은
`pr2194-origin-2587.json`·`pr7404-origin-2587.json`·`pr7368-origin-2587.json`에
보존했다. 설치된 gh가 `--slurp`를 지원하지 않아 첫 timeline 요청은 실패했다.
기존 실패 로그를 보존하고, 페이지별 JSON 배열을 직접 읽는 수정 요청으로 완료했다.

## 같은 공개 API의 실제 관측

`remaining-edit-snapshots-2586-result.json`은 동결 Native165에 공개 즉시/지연 편집과
저장 API를 한 번 링크해 실행한 결과다. 0·1·55·56·61·200자 및 HWP/HWPX 한 글자
편집을 함께 기록했다. 한 글자 편집은 LineSeg가 원래 네 줄 그대로인데 live 116쪽이다.
55/56/61자는 live 116쪽·재열기 117쪽이며, 200자는 둘 다 116쪽이지만 104쪽 소속이 다르다.
무편집 저장은 115쪽·본문 차이 0이다. live IR의 필드는 저장 전후 변하지 않는다.

단계별 실제 원점·컷·물리 예산은 `remaining-edit-trace-2588-result.json`과
`remaining-edit-first-frames-2591.json`, 저장 매핑은
`logs/remaining-edit-trace-2592.log`에 있다. 같은 root의 글줄 예산이 편집 여부만으로
1001.600→1000.267px가 되고, 개체를 수용하는 조각은 1005.373px의 기존 예약 대신
같은 1000.267px를 적용한다. 이 숫자를 구현 조건으로 쓰지 않는다.

`mcp-edited-controls-2590/results.json`은 위 API가 실제 저장한 동일 SHA의 파일을
PrintMethod 0으로 출력했다. 무편집·56자·200자·지연 한 글자 HWP는 한컴 2024,
지연 한 글자 HWPX는 한컴 2020이며 전부 115쪽이다. 버전은 각 저장본 `info`의
lastSavedWith와 원본 제품 세대로 확인했다. 빈 HWP 스크립트 placeholder는 알려진
해시를 확인했고 HWPX에는 스크립트가 없다. 기존 56자 PDF를 새 저장본의 출력으로
대체하지 않았다. 입력·PDF SHA와 실제 job은 해당 receipt에 보존한다.

## 적용할 공통 규칙과 소비 경로

1. 저장된 조각 좌표계를 쓴다는 사실과 옛 컷을 그대로 수용할 수 있다는 사실을 구분한다.
   다른 문단의 편집만으로 남은 저장 좌표계까지 단조 재생성 사다리로 바꾸지 않는다.
   원본 줄·개체 프레임의 기하가 남아 있는지는 그 자료에서 확인하며, 기존 저장 컷의
   유효성을 판정하는 provenance 검사는 유지한다.
2. 프레임 근거는 두 갈래다. 원본 부모 줄의 실제 되감김은 페이지 상대 좌표계를 보이고,
   유효한 저장 줄과 선언 개체보다 큰 원본 행 상자의 합은 셀 상대의 분할 저장 프레임을
   보인다. 원본 거대 셀의 선언 셀 높이는 패딩뿐이므로 후자 하나를 필수 전제로 삼지 않는다.
   단조 분할 저장본은 둘 다 없으므로 기존 바깥 물리 조각 예산을 유지한다.
3. 일반 유닛처럼 표현된 TAC 줄도 실제로 개체의 줄 상자를 소유할 수 있다. #7336의
   마지막 문단은 `9395+141+141=9677HU`로 원본 LineSeg와 같다. 제어문 슬롯이
   현재 컷에 속하고 같은 소유 기하를 확인한 줄은 개체 viewport의 예약을 소비한다.
   빈 일반 글줄이나 다른 쪽의 TAC 존재로 현재 예산을 바꾸지 않는다.

생산은 원본 LineSeg·행/개체 기하 → 저장 좌표계 종류와 실제 TAC 소유 줄 판정이다.
소비는 `rowbreak_fragment_bounds`의 내용/빈 띠 하단과 `row_step`의 현재 컷 예산이다.
실제 컷·요구 높이·paint는 기존 공통 CellUnit 원장을 계속 소비한다. 저장 좌표계의
종류를 확인한 것을 옛 강제 컷의 새 인증으로 쓰지 않는다.

## 검증 계획

Native166을 공용 target·8잡으로 한 번 빌드한다. 기존 189개 검사, 편집 전 HWP/HWPX
230쪽, 고정 분할본 196쪽, 35문서/66쪽과 #7840·현대 수식을 함께 대조한다.
같은 5개 편집 저장본도 해당 새 Print와 확인한다. 현재 API의 저장 바이트가 바뀌면
그 출력은 별도 MCP Print를 확보한다. 새 회귀·기대값·fixture는 시각 선행 조건을
충족하기 전에는 추가·변경하지 않는다. 전체 Rust와 fresh WASM 완료를 이 선행 확인으로
대신하지 않으며, 실패나 미달이 남으면 원인을 계속 보정한다.

## Native166 중간 검증 결과

- 빌드: `char-overlap-native-build-166-result.json`, 8잡·공용 target, exit 0.
  기본 Native Clippy도 `logs/native166-clippy-2603.log`에서 exit 0을 확인했다.
- 기존 189개: `reviewed-contract-tests-2594-result.json`의 **184 PASS / 4 FAIL / 1 ignored**.
  #2185 한 글자 편집과 #7336 두 검사가 다시 통과했고 이전 실제 정상 181개와 제외 1개 상태를 유지했다.
  남은 네 건은 #7114의 55/56/61/200자 저장·재열기 차이다. 검사를 바꾸지 않았다.
- 원본 HWP/HWPX: `issue1949-exact-originals-native166-2595-result.json`, 각각
  115/115쪽·최저 95.69614%. 두 경로 모두 Native이며 fresh WASM을 대신하지 않는다.
- #7336: `issue7336-native166-2602-result.json`, HWP 6/6쪽 최저 97.07162%,
  HWPX 7/7쪽 최저 96.87735%. 기존 한컴 2020 Print를 같은 입력 SHA로 재사용했다.
- 고정 분할 저장본: `issue4138-native166-boundary-batch-2596-result.json`,
  196/196쪽·최저 38.91518%. 177~196쪽의 기존 미달은 남아 있으며 통과가 아니다.
  #7840 9쪽 최저 93.43362%, 현대 수식 v5 5쪽 99.98497%, v6 2쪽 100%를 유지했다.
- 35문서/66쪽 정책·원점·괘선 대조군: `reflow-*-native166-2597/results.json`,
  6개 driver exit 0, 각 쪽 raw 100%와 쪽수 일치를 확인했다.
- 공개 편집 API: `remaining-edit-snapshots-2598-result.json`. 무편집·즉시/지연
  한 글자 HWP 및 지연 한 글자 HWPX는 115쪽과 쪽별 본문이 저장·재열기 전후 같다.
  55/56/61자는 live 115·재열기 116쪽, 200자는 live 116·재열기 117쪽이다.
- 새 저장본의 독립 Print: `mcp-edited-controls-2600/results.json`. 실제 Native166
  56/200자·지연 한 글자 HWP 바이트를 한컴 2024 PrintMethod 0으로 출력했다.
  이 PDF를 이전 Native165 저장본의 기준으로 바꾸어 보고하지 않는다.

현재 source의 전쪽 결과와 정책 대조군은 개선·보존됐지만 저장 왕복 네 실패와 분할본 미달,
최종 전체 Rust·lint 묶음·fresh WASM은 남았다. PR 준비 완료나 시각 승인으로 판정하지 않는다.

## 다음 보정의 확인된 경계

`edited-unit-differences-2608.json`과 `logs/remaining-edit-trace-2601.log`에서
56자 편집의 live/reload는 유닛 수와 각 높이가 같지만 저장 원점과 종료 줄 소유가 다르다.
3쪽 컷 115 앞뒤의 빈 문단 59는 원본 다음 프레임(문단 60) 바로 앞의 종료 줄이다.
live의 다음 조각에서는 무높이로 지나가나, 재열기는 `owns_stored_closing_line`으로
다시 소유해 같은 컷의 요구가 한 줄(25.6px) 커지고 추가 쪽이 생긴다.
종료 줄 자체의 앞 조각 점유는 보존하고, 다음 조각의 시작에서는 앞 프레임 소유 줄을
중복 계상하지 않아야 한다. 원래 값이나 테스트 허용치를 변경할 근거가 아니다.

## Native167: 종료 빈 줄의 앞 조각 소유

`is_free_leading_spacer`의 공통 술어에서 `owns_stored_closing_line` 유닛은 다음
조각의 시작 전진으로 다시 예약하지 않는다. 앞 조각 안의 같은 유닛 높이는 그대로다.
컷 선택·물리 범위 측정·저장 시작 컷은 기존 공통 술어를 함께 소비한다.

- `char-overlap-native-build-167-result.json`: 8잡·공용 target 빌드 exit 0, 247.36초.
- `reviewed-contract-tests-2614-result.json`: 기존 **189개 중 187 PASS / 1 FAIL / 1 ignored**.
  55/56/61자의 세 실패가 통과했고 기존 실제 정상 184개 검사도 유지했다. 새 검사나 기대값 변경은 없다.
  남은 실패는 200자 편집의 중첩 표 내부 컷 뒤 저장·재열기 내용 소속이다.
- `remaining-edit-snapshots-2615-result.json`: 55/56/61자 모두 live/reload 115쪽과
  쪽별 본문이 같다. 200자는 live 116·재열기 117쪽, 첫 내용 차이는 103쪽이다.
  무편집·즉시/지연 한 글자 HWP·지연 한 글자 HWPX도 115쪽과 본문을 보존한다.
- 8개 API 저장 바이트는 Native166과 모두 SHA-256이 같다. 읽기·이어받기 처리가 바뀐 것이며
  새 저장 바이트를 Print했다고 보고하지 않는다. 같은 SHA의 `mcp-edited-controls-2600`
  Print를 재사용한다. `edited-print-pagecounts-2625.json`의 실제 PDF는 세 파일 모두 115쪽이다.
- `issue1949-exact-originals-native167-2616-result.json`: 원본 HWP/HWPX 각 115쪽,
  최저 95.69614% 유지. `issue4138-native167-boundary-batch-2617-result.json`의 고정
  분할본 196쪽은 최저 38.91518%로 미달 유지, #7840·현대 수식 대조군도 이전 값을 유지했다.
- `logs/native167-clippy-2622.log`: 기본 Native Clippy exit 0. 전체 Rust·WASM lint·
  workspace lint·fresh WASM 완료를 뜻하지 않는다.

### 쪽수·본문 왕복 통과와 독립 Print의 시각 미달

`edited-print-native167-2623-result.json`은 같은 API 저장본·같은 SHA의 독립 Print로
전쪽을 확인했다. 56자 저장본은 115/115쪽이지만 **102쪽 미달·최저 11.38223%**이며,
한 글자 저장본은 **8·104·105쪽 미달·최저 19.93037%**다. 200자 저장본은 쪽수가
다르므로 TSV를 산출하지 못했다. 두 측정 가능 문서의 gate는 `re_review_required`이며,
쪽수 불일치도 통과가 아니다. 관계 검사 통과를 독립 출력 일치로 승격하지 않는다.

새 공식 PNG는 `native166-7336-reviews-2612`의 #7336 6쪽 review/overlay와
`native166-edited-reviews-2618`의 한 글자 저장본 105쪽 review/overlay다. 후자는
그림이 다른 쪽에 놓이고 본문 원점도 크게 달라 직접 판독에서도 실패했다.
`untouched-one-character-print-boundaries-2626.json`에서 원본·무편집 저장본·한 글자
저장본의 실제 한컴 Print 104/105쪽 이미지 bbox는 같다. 반면
`one-character-all-source-lines-2621.json`에서 rhwp 저장기는 편집하지 않은 부모 문단
54개의 LineSeg 원점을 바꿨다. 빈 시작 문단 158과 TAC 그림 호스트 2290의 원본 재시작이
그 뒤 문단으로 옮겨졌다. 단순히 페이지 수·낱말만 비교하면 이 배치 차이를 검출하지 못한다.

다음 분석은 줄 기하가 같은 편집까지 전체 표의 저장 프레임 되쓰기를 요구하는 출처 표식과,
부모 사다리의 실제 쪽 경계·개체 로컬 원점의 구분이다. 원본 줄이 남아 있는데 다른 호스트의
생성 tag 하나 때문에 셀 전체의 저장 좌표계를 기각하는 조건도 200자 재열기와 함께 확인한다.
같은 높이만으로 불변을 판정하지 않고 줄별 원점·높이·간격·소유 변화를 함께 대조한다.

`native167-focused-summary-2628.json`에서 전체 producer source hash 일치, 8개 저장 바이트
불변, 기존 실제 187 PASS / 1 FAIL / 1 ignored와 35문서/66쪽 raw 100%를 묶어 확인했다.
정식 회귀 추가·변경과 PR 제출의 시각 선행 조건은 충족하지 않았으며 보정을 계속한다.

## 다음 보정: 실제 줄 기하 변경과 저장 좌표계의 국소 유효성

현재 head `466c57122dd868d5bae9eb55bdc59320c3c663a9`에서 다음 두 생산 지점을 함께 확인한다.

1. 셀 텍스트 삽입/삭제는 실제 줄 기하가 그대로여도 `mark_table_text_reflowed_after_edit`를
   무조건 기록한다. 저장기는 이 표식을 보고 전체 표의 원본 사다리를 되쓴다.
   한 글자 입력의 원본 줄별 원점·높이·간격은 그대로인데 편집하지 않은 54개 문단의
   원점을 바꿨고, 독립 Print는 원본·무편집·한 글자 저장본의 그림 위치가 같다고 확인했다.
   삽입/삭제가 같은 원본 수직 줄 프레임과 소유 분류를 보존하면 새 프레임 무효화 표식을
   추가하지 않는다. 기존에 실제 재조판으로 기록한 표식은 지우지 않는다.
   전체 높이 하나 대신 줄별 원점·높이·간격, 원본 줄의 유효성, 빈 줄·제어 소유를 함께 비교한다.
2. 저장 좌표계의 종류를 찾는 판정은 지금 셀의 모든 문단이 원본이어야 되감김 하나도
   인정한다. 200자 저장본의 호스트 81에 생성 tag가 붙어도 다른 원본 줄들의 페이지 상대
   좌표계는 남아 있다. 생성·dirty 줄은 근거에서 제외하되, 연속한 실제 원본 줄 쌍의
   되감김은 그대로 인정한다. 이것을 옛 컷 전체의 재인증으로 사용하지 않는다.

공개 편집 8종·기존 189개, 원본 230쪽과 고정 분할본 196쪽을 같은 Native168 빌드로
일괄 확인한다. 35문서/66쪽·#7336·#7840·현대 수식의 기존 정상 경계도 유지한다.
새 저장 바이트는 기존 Print와 SHA를 먼저 대조하고, 달라졌으면 같은 제품 세대의
MCP Print를 새로 확보한다. 쪽수·본문 왕복만으로 시각 검증이나 새 회귀 자격을 판정하지 않는다.

## 제외 검사의 집계 정정

`focused-terminal-audit-2635.json`에서 Native165/166/167의 189개 원 로그를 전수
재독했다. 진단 드라이버는 test binary의 exit 0만으로 PASS를 집계해 기존 ignored 검사
`issue_2308_short_rowbreak_child_wraps_where_the_authority_pdf_wraps` 1개를 잘못 포함했다.
정확한 Native165/166/167 결과는 각각 181/7/1, 184/4/1, 187/1/1
(PASS/FAIL/ignored)이다. 기존 JSON·로그는 원 증거로 보존하고 이 감사를 연결한다.
수정한 다음 진단 드라이버는 실제 실행 수와 ignored 수를 따로 보고한다.
이 검사를 활성 회귀나 실행 성공 증거로 승격하지 않는다.

Native168의 실제 집중 검사는 188 PASS / 0 FAIL / 1 ignored, 공개 API 14종의
저장·재열기 쪽별 본문은 모두 같다. 200자는 여전히 116쪽이므로 독립 Print의 115쪽과
다르며, 시각 통과가 아니다. 다음 Native169에서는 원본 쌍을 찾는 도중 저장 줄이 없는
문단을 만나도 이전 줄과의 연결을 끊는다. 생성·dirty 문단뿐 아니라 정보가 없는 문단을
가로질러 가짜 원본 쌍을 만들지 않는 같은 판정의 보수적인 경계다.

## Native168의 실제 실행과 독립 Print 개선

`native168-focused-summary-2650.json`은 실제 기존 188 PASS / 0 FAIL / 1 ignored,
공개 API 14종의 저장·재열기 쪽별 본문 일치, 35문서/66쪽 raw 100%를 기록한다.
직접 삭제와 짧은/줄바꿈 삽입 후 삭제를 HWP/HWPX에서 실제 실행했다.

한 글자 HWP 저장본 SHA는 `9e76b3b462e5c6f5e4adade2356f6f7dca6cb513be1c9fb6327dccec902a0107`로
바뀌어 이전 PDF를 재사용하지 않았다. 한컴 2024 MCP job
`bd1ccbc8-a5ab-4862-9445-4b8cda06f6aa`의 PrintMethod 0 출력은
`mcp-edited-controls-2637/results.json`에 원본·PDF SHA와 함께 있다.
`edited-print-native168-2638-result.json`의 Native 115/115쪽 최저는 95.69614%,
90% 미만 0쪽이다. `one-character-preserved-vertical-frames-2642.json`에서
원본 부모 문단 2,507개의 원점·높이·줄간격 차이도 0개다.

공식 `native168-one-reviews-2643`의 105쪽 review와 standalone overlay를 직접 읽었다.
이전 19.93037%에서 raw 100%로 개선됐고 그림과 뒤 제목·본문의 같은 쪽 소속,
상하 순서와 원점이 Print와 일치한다. `not_evaluated` 실루엣 보조 gate를 최종 승인으로
바꾸어 보고하지 않으며 fresh WASM과 확대 판독 범위의 최종 증거는 아직 남아 있다.

200자는 116쪽으로 저장·재열기는 같지만 독립 Print의 115쪽과 다르다.
56자 편집의 전쪽 시각 미달과 고정 분할본 177~196쪽 미달도 해결 완료가 아니다.
다음 전체 Rust 재실행과 별개로 이 물리 배치 차이를 계속 보정한다.

## Native169 결과

`native169-focused-summary-2654.json`에서 producer source hash를 재확인했다.

- 기존 실제 188 PASS / 0 FAIL / 1 ignored, 35문서/66쪽 전쪽 raw 100% 유지.
- 공개 API 14종은 저장·재열기 본문이 같고, Native168과 각 저장 바이트 SHA도 같다.
  새 저장 바이트를 별도 출력한 것처럼 보고하지 않고 동일 입력 Print를 재사용한다.
- 원본 HWP/HWPX 각 115쪽·최저 95.69614%, 고정 분할본 196쪽·최저 38.91518%로
  기존 뒤 20쪽 미달 유지. #7840와 현대 수식 정상 대조군도 유지했다.
- 전체 format과 Native root Clippy는 각각 exit 0,
  `logs/native169-format-2651.log`·`logs/native169-clippy-2652.log`에 보존했다.
- 다음 전체 Rust 로그는 `current-full-rust-2644/nextest.log`다. 전체 결과 전에는
  과거 68 FAIL을 새 전체 통과로 바꾸지 않으며, fresh WASM 등 나머지 필수 gate도 남아 있다.

## 11563건 전체 실행과 줄바꿈 편집의 독립 출력 재검토

`current-full-rust-2644/terminal-inventory.json`의 head `7e3be0a3fb`는 실제
11,516 PASS / 47 FAIL / 45 skipped, nextest exit 100이다. 실행 중 source 변경은 없다.
이전 전체 실행의 68 FAIL과 함수명으로 대조하면 21개 해결·새 실패 0개·47개 지속이다
(`full-rust-2532-2644-difference-2663.json`). shard 번호 이동을 새 실패로 세지 않았다.
2026-10-24 다시 fetch한 upstream/devel도 `d9fd85ed5b`로 같아 리베이스는 필요하지 않았다.

`native169-wrap-reviews-2662`의 56자 저장본 4쪽은 raw 40.81625%다. 공식 review와
standalone overlay에서 제목부터 본문까지 한 줄 위에 있고 마지막 중첩 행 전체가 앞 쪽에
남는다. 독립 Print는 제목을 아래에 두고, 중첩 표의 마지막 행 안에서 나눠 마지막 항목을
5쪽에 배치한다. 저장·재열기 일치와 기존 집중 검사 통과만으로 정확한 조판을 입증하지 못했다.

수정할 공통 규칙과 실제 소비 경로는 다음과 같다.

- 앞 쪽에 들어가지 못한 실제 빈 줄은 다음 조각에서 자기 높이를 점유한다.
  `stored_closing_empty_line_advance_hu`의 국소 원본 줄 닫힘 → `CellUnit` 높이·소유 →
  `is_free_leading_spacer`를 공유하는 컷 선택·물리 시작점·요구 높이·실제 배치를 연결한다.
  표의 다른 문단이 재조판됐다는 이유로 이 줄 소유를 지우지 않는다. 반대로 dirty·합성 줄은
  원본 근거로 수용하지 않는다. `466c57122d`의 종료 줄 무조건 건너뛰기는 독립 Print와
  어긋나므로 이 술어를 보정한다. 기존 실제 종료 줄의 소유를 유지한 채 예산에 반영한다.
- 자리차지 RowBreak 중첩 표의 일반 글줄은 선언 높이가 있어도 셀 안에서 나뉠 수 있다.
  자식 자체가 성장하지 않고 부모 앞 내용만 늘어난 경우도 같다. 기존 per-driver CellUnit →
  모든 셀의 start/end RowCut → `NestedTableUnitCut` → `row_cut_content_height` →
  실제 부분 표 배치를 재사용한다. 별도 높이 추정기는 추가하지 않는다. rowspan·TAC·
  개체 혼재는 기존 계약을 유지하며, 일반 글줄의 물리 높이는 같은 컷의 실제 요구 높이를 쓴다.

Native170 한 번 빌드 후 기존 집중 검사와 공개 API 편집·저장을 먼저 묶어 확인한다.
줄바꿈 저장 바이트가 달라지면 기존 PDF와 점수를 재사용하지 않고 같은 제품의 새 MCP
Print를 확보한다. 원본 HWP/HWPX 230쪽·35문서/66쪽과 분할본·정상 대조군도 유지해야 한다.
기대값·새 회귀는 이 단계에서 바꾸지 않는다.

### 정상 대조군 producer 경로 감사 정정

Native170 검증 스크립트를 준비하며 과거 6종 정상 matrix의 실제 `--rhwp-bin`을
전수 확인했다. `reflow-*-native169-2649`라는 폴더명과 달리 실제 실행은 Native165였다.
`matrix-producer-audit-2671.json`에 원 명령을 보존한다. 따라서 앞 절의 35문서/66쪽
raw 100%는 Native165 대조 증거이며 Native169 재출력·기능 보존 증거로 수용하지 않는다.
이전 기록의 같은 주장은 이 정정으로 대체한다. 원 로그·JSON은 변경하지 않는다.
현재 6종 드라이버는 실제 Native170 경로와 실행 파일 SHA를 검증하도록 고쳤으며
최신 후보의 같은 전체 대조군을 새로 실행한다.

### Native170의 반례와 종료 컷 소비 차이

Native170은 기존 실제 178 PASS / 10 FAIL / 1 ignored로 후퇴했다
(`reviewed-contract-tests-2666-result.json`). 원본 HWP/HWPX도 각각 103·104쪽이
90% 미만이며 최저 55.91873%·55.92440%로 수용하지 않는다. 반면 정상 matrix
35문서/66쪽은 실제 Native170으로 재출력해 전쪽 raw 100%를 확인했다.
편집 저장·재열기의 일치는 14종 모두 유지하지만 55·56·61·200자는 116쪽이다.

56자 기존 저장본의 4쪽 raw는 40.81625%에서 95.07363%로 개선됐다. 그러나 5쪽은
15.30899%이고 이어받을 마지막 항목이 없다. `native170-wrap-reviews-2676`의
공식 review·overlay와 실제 render tree를 함께 읽었다. 트리에는 마지막 항목이
4쪽 본문 아래에 존재해, 내용이 사라진 것이 아니라 잘못 수용한 조각에서 잘린 것이다.
이 결과를 단순 점수 통과로 완료하지 않는다.

원인은 일반 글줄 컷을 투영할 때 끝 컷을 모두 소비한 인덱스 벡터로 재지만,
`nested_table_split_from_cut_units`는 같은 `terminal`을 빈 end_cut으로 바꾸어
자식 행 전체를 그리는 데 있다. 온전한 행의 선언 높이·정렬 띠가 상위 예약에서는
빠지고 배치에서 되살아났다. 컷 생산 → 상위 예약 → 실제 자식 컷을 같은
`NestedTableUnitCut`로 연결하고, 종료 시 배치와 같은 빈 end_cut 및 온전한 행 높이를
예약한다. 프레임 시작 전에는 소비 높이를 0으로 두며, 이어받기 시작 컷은 실제 컷으로
유지한다. 첫·마지막 행의 캡션 예약도 기존 통째 행 경로와 일치시킨다.

Native171에서 같은 기존 검사·실제 API·원본·정상 대조군·동일 SHA 편집본을
묶어 재검증한다. Native170 실패와 원본 증거는 보존하며 기대값을 완화하지 않는다.

## Native171의 확인 결과와 남은 실제 소비 차이

- `char-overlap-native-build-171-result.json`: 공용 target, 8잡, exit 0.
- `reviewed-contract-tests-2678-result.json`: 실제 181 PASS / 7 FAIL / 1 ignored.
  #7114 3개, #7518 3개, 중첩 프레임 컷 검사 1개가 실패한다. 정수 컷 번호의
  의미도 직접 원본·Print와 확인해야 하며, 단순 번호 차이로 검사부터 바꾸지 않는다.
- `issue1949-exact-originals-native171-2681-result.json`: HWP/HWPX 각 115/115쪽,
  양쪽 최저 95.69614%, 90% 미달 0쪽으로 원본의 Native 보존은 회복했다.
- 실제 Native171로 다시 실행한 정상 matrix는 35문서/66쪽 raw 100%다
  (`reflow-*-native171-2682/results.json`). 파일명만으로 producer를 판단하지 않고
  실제 명령의 경로와 실행 파일 SHA를 함께 확인했다.
- `issue4138-native171-boundary-batch-2683-result.json`: 고정 분할본 196쪽은
  뒤 20쪽 미달·최저 38.91518% 유지, #7840 9쪽 최저 93.43362%, 현대 수식
  v5 5쪽 99.98497%·v6 2쪽 100%다. 분할본 미달은 해결 완료가 아니다.

`remaining-edit-snapshots-2679-result.json`의 기본 API 14종은 저장·재열기 본문이
같지만 55·56·61·200자 편집은 116쪽이다. API 기본 조판 세대가 2022인 점을
확인해 원본 제품에 맞게 HWP 2024·HWPX 2020을 명시한 진단도 별도로 실행했다
(`remaining-edit-snapshots-2687-result.json`). 같은 116쪽 차이가 남으므로
환경 차이만으로 실패를 해소하지 않는다. 기존 테스트·기대값은 변경하지 않았다.

Native170이 저장한 실제 55·56·200자 입력은 `mcp-edited-controls-2686/results.json`의
한컴 2024 13.0.0.3901·PrintMethod 0에서 각각 115쪽이다. 각 입력·PDF SHA와
실제 job을 보존했다. Native171의 새 저장 바이트와의 비교는
`native170-171-edited-bytes-2692.json`에 있으며 달라진 입력에 이 PDF를 재사용하지 않는다.

### 다음 공통 보정에 필요한 실제 호출 경로

`native171-wrapped-cut-trace-2690/receipt.json`과 로그는 같은 56자 저장본을
`--compat 2024`로 다시 렌더한 실행 증거다. 재조판된 빈 줄의 점유와 별도로 다음 차이가 있다.

1. `advance_row_cut_for_physical_band`에서 현재 컷의 요구 높이가 물리 예산을
   넘는다고 확인한 뒤 예산을 줄여도, `absorb_tail_before_stored_frame_break`가
   옛 저장 reset까지 같은 컷을 다시 선택한다. 반복 컷이면 알려진 초과 상태로
   반환되고 상위 fallback이 훨씬 앞 컷으로 물러난다. 실제 trace는
   start `[115]`·end `[155]`·물리 예산 1005.373·요구 1027.973을 반복한 뒤
   end `[151]`로 물러났다. 숫자를 구현 조건으로 쓰지 않는다. 물리 초과를 확인한
   경계에서 저장 꼬리 허용을 재사용하지 않고 현재 컷을 실제 예산 안에서 선택해야 한다.
2. `nested_table_split_from_cut_units`가 자식의 같은 재귀 start/end cut을 갖고 있어도
   viewport/흐름 높이는 유닛 합을 쓴다. `reflow_recursive_run_extra`의
   실제 자식 상자 대조는 NoStoredLineSeg 호스트에 제한되어 원본 호스트의 새 글줄 컷은
   건너뛴다. 또한 양수 추가분만 반영하므로 온전한 행에 예약한 물리 띠를 부분 컷이
   전부 소유하지 않을 때의 감소분을 측정과 실제 배치에 공유하지 못한다.
   같은 자식 컷의 실제 상자와 첫/끝 호스트 여백 소유를 생산·예약·viewport가
   공유하도록 보정할 필요가 있다. 아직 이 후속 보정을 구현하거나 통과했다고 기록하지 않는다.

`native171-wrap-reviews-2688`은 쪽수 116/115 불일치 상태에서 기존에 만든 PNG의
4·5·6쪽만 진단한 공식 review/overlay다. 전쪽 gate 통과 증거가 아니며, 5쪽
raw 1.69110%의 큰 이어받기 차이가 남는다. 작은 경계와 이 직접 차이를 먼저 고친 뒤
전체 Rust·fresh WASM·남은 45개 PR 범위의 최종 검증을 완료해야 한다.

## 다음 후보 Native172의 공통 보정과 검증 계획

Native171의 실제 55·56·200자 저장본도 한컴 2024 Print에서 모두 115쪽이다
(`mcp-edited-controls-2695/results.json`). 기본/명시 세대 API 모두에서 같은
116쪽 문제가 남아 환경 차이만의 문제가 아님을 다시 확인했다.

- 물리 높이가 예산을 넘는 컷은 저장 꼬리 흡수를 허용한 채 다시 선택하지 않는다.
  `RowCutCapacity::Physical`에 실제 수용 모드를 전달하고 첫 물리 초과 뒤 원래의
  물리 예산에서 저장 꼬리 허용 없이 재선택한다. 알려진 초과분을 근거 없이 수용하거나
  허용 px·baseline을 늘리지 않는다. 큰 첫 atom의 별도 진행 계약은 유지한다.
- 행 유닛의 생산자가 실제 더한 호스트 앞뒤 공간을 `NestedRowFlowEdges`에 기록한다.
  임의 상수나 문서 ID로 다시 추정하지 않는다. 기존 소유 줄·캡션·후행 간격은 그때
  실제로 계상한 값이며, 같은 값이 자식 내용 컷의 물리 높이와 분리되어 소비된다.
- `nested_row_projection_cut`이 예약과 실제 viewport의 자식 커서를 함께 만든다.
  `reflow_recursive_run_extra`는 원본 호스트의 명시적인 행 내부 컷도 같은 실제
  자식 상자로 대조하며 부분 컷이 소유하지 않는 물리 띠의 감소분도 전달한다.
  반복 머리행은 상위에서 이미 예약한 몫을 다시 더하지 않는다.
  `nested_table_split_from_cut_units`도 같은 조정을 받아 내용 컷·높이·배치가 연결된다.

기존 검사 이름·기대값·baseline은 그대로 두고 Native172 한 번 빌드 후 집중 경계,
실제 API 14종, 원본 230쪽, 실제 35문서/66쪽, 분할본과 정상 대조군, 동일 SHA의
Native171 편집본 Print를 묶어 대조한다. 수정 뒤 결과가 나오기 전에는 개선 완료로
판정하지 않는다. 새 저장 바이트는 SHA 대조 후 필요한 경우 다시 Print한다.

## Native172 결과와 전진·점유 구분 보정

Native172는 실제 174 PASS / 14 FAIL / 1 ignored로 아직 수용할 수 없다
(`reviewed-contract-tests-2698-result.json`). 원본 HWP/HWPX 각 115쪽의 Native
최저 95.69614%는 유지했지만, 55·56·200자 동일 저장본은 Print 115쪽과 쪽수가 다르다.
새로 실패한 #7418의 70833 검사 3개와 auto-height 정상 대조군에서도 의미 있는 내용 귀속
차이를 검출했다. 기대값·baseline을 바꾸지 않고 실패 증거를 보존한다.

소비 경로를 다시 읽어 높이 계약의 차이를 확인했다. `CutHeight::Occupied`는
마지막 줄 상자를 닫지만 `Capacity`는 자식 viewport에서 다음 내용으로 전진할
줄간격을 유지한다. Native172는 같은 조정을 `visible_height`와 `flow_height`에
모두 적용했고, 부모 컷에 후속 내용이 있어도 자식의 Occupied 높이를 썼다.
빈 후속 문단도 같은 내용 순서의 소유자이므로 이 전진을 지우면 안 된다.

Native173은 같은 자식 커서·같은 앞뒤 공간의 결과를 유지하면서 viewport에는
Occupied, 흐름 전진에는 Capacity를 소비한다. 같은 부모 컷 안에서 뒤 유닛이
있으면 중간 자식도 Capacity를 예약하고, 마지막 자식만 부모의 점유 끝 판정을
받는다. 가시 글자의 유무를 흐름 전진의 대용으로 쓰지 않는다. 기존 집중 검사·
실제 API·원본·정상 matrix·독립 Print의 같은 전체 범위를 다시 확인한다.

항공교통관제사 CBTA 원본은 Hancom 2018 저장본이다. 과거 engine 2020 명칭의
실제 Hancom 2022 PDF와 실제 HOffice110 11.0.0.9136의 새 Print를 분리했다
(`mcp-cbta-source-2708/results.json`, 원본 204쪽·PrintMethod 0).
Native172는 새 Print에서도 204쪽 중 12쪽 미달·최저 37.96169%로 아직 전쪽
시각 자격을 충족하지 않는다(`cbta-native172-2710-result.json`). 단순 내부 컷
번호가 바뀌었다고 회귀부터 고치지 않는다. 과거 PDF 비교는 2707, 새 실제
한컴 2020 비교는 2710이며 라우팅 정정은 `cbta-baseline-routing-2709.json`에 있다.

## Native173 결과와 후속 소비 지점 확인

Native173도 기존 실제 174 PASS / 14 FAIL / 1 ignored이며 수정 완료가 아니다.
동일 원본·새 실제 한컴 2020 Print의 70833은 기준 18쪽, Native173 19쪽으로
불일치한다. 단순 내부 컷 번호나 오래된 PDF 차이만으로 해소할 수 없는 구현 차이다.
`mcp-70833-source-2713/results.json`은 HOffice110 11.0.0.9136의 같은 원본 Print다.
기본 스크립트는 저장 형식의 두 길이 지정 UTF16 문자열·말미 12바이트를 해석해
읽기 전용 문서 별칭과 빈 OnDocument_New만 있음을 확인했다
(`70833-inert-script-proof-2725.json`). 서비스의 scripts_disabled는 이 기본
템플릿만 비활성화했으며 본문은 변경하지 않았다. 초기 로컬 검증·다운로드 실패와
같은 완료 job에서 회수한 이력은 원 로그로 보존했다.

다음 실제 소비 차이를 함께 보정한다.

- 행 유닛의 첫·끝 여백만 전달해도, 재조판 자리차지 상자는 이어받는 중간
  viewport마다 바깥 여백을 연다. 새 metadata 경로가 기존 공통
  `reflow_nested_table_has_outer_frame` 계약을 건너뛰어 이 반복 여백을 빠뜨렸다.
  첫·끝 유닛이 소유한 여백과 중간 조각의 반복 여백을 구분해 동일 계약으로 예약한다.
- `advance_row_cut_inner`는 편집 provenance가 옛 저장 컷을 무효화했어도
  원점 되감김에서 만든 hard/reset flag를 물리 쪽 경계로 재사용할 수 있었다.
  좌표계·되감김 정보는 남기되 현재 컷의 수용은 같은 table_text_reflowed 출처로
  판정한다. 모델의 명시적인 구역·쪽·단 나누기는 기존 권한을 유지한다.
  본문 닫힘 보조 인증도 같은 provenance를 확인하며, 생성 줄이 보조 인증의
  조기 반환으로 원본 경계에 승격되지 않게 한다.
- `DIAG_RESET_OWNERS`는 환경 변수로 선택한 셀에서만 실제 cut 소유 근거를
  기록한다. 일반 실행의 기하·수용 결과를 바꾸지 않는 진단이며 추가 gate가 아니다.

Native174에서 기존 189개 항목·실제 API·원본·정상 matrix·독립 Print를 일괄
재실행한다. CBTA 204쪽도 새 Print에 대해 대조하고 미달을 감추지 않는다.

## Native174 결과와 devel 확인

`upstream-rebase-check-2749.json`에서 fetch 종료 0, 기존/새 base
`d9fd85ed5bc3c4d9ce6b2981422e48bf9bb90867` 동일과 HEAD의 조상 관계를 확인했다.
작업지시자의 동일 HEAD에서는 리베이스하지 않는 지시에 따라 추가 리베이스는 하지 않았다.
동결 Native174와 실행 소스의 SHA-256도 모두 일치한다. 이 기록은 최종 PR 검증 완료가 아니다.

- 집중 검사: `reviewed-contract-tests-2732-result.json`, **178 PASS / 10 FAIL /
  1 ignored**. 기존 189개 항목을 모두 확인했으며 기대값을 바꾸지 않았다.
  #7114 저장·재열기 3건은 회복했다. #7418 3건, #7518 4건, #7902 원본 후속 줄
  2건, CBTA 내부 컷 1건은 실패한다.
- 실제 공개 API: `remaining-edit-snapshots-2733-result.json`, 14종 모두 live/재열기
  115쪽이다. 13종은 쪽별 본문도 일치하지만 HWPX의 56자 입력 후 56자 삭제에는
  6쪽부터 본문 소속 차이가 남는다. 쪽수만 같다는 이유로 보존 완료라고 하지 않는다.
- 원본 HWP/HWPX: `issue1949-exact-originals-native174-2734-result.json`, 각
  115쪽, 최저 95.69614%. fresh WASM 결과를 대신하지 않는다.
- 실제 Native174의 35문서/66쪽 대조는 `native174-visual-batch-driver-2738.json`과
  `logs/check_native174_batch_2735-driver.log`에서 종료 0을 확인했다.
  고정 분할본의 기존 미달은 `issue4138-native174-boundary-batch-2736-result.json`에 남는다.
- CBTA: 새 실제 한컴 2020 Print와 204쪽 대조를 완료했으며 최저 37.96169%·12쪽
  미달이다(`cbta-native174-2745-result.json`). 2740의 최초 진단 스크립트는 PDF 해시의
  복사 오타로 실행 전 중단됐다. PDF 바이트를 바꾸지 않고 변환 완료 manifest에서
  해시를 직접 읽도록 고쳐 2745로 재실행했다. 최초 실패 로그도 보존했다.
- 70833: 새 실제 한컴 2020 Print는 18쪽, Native174는 19쪽이다
  (`70833-native174-2741-result.json`). TSV를 산출할 수 없는 쪽수 불일치를 통과로
  집계하지 않는다.
- 정상 이어받기 대조: `continuation-controls-native174-2743-result.json`.
  auto/재저장 auto는 각 8쪽·최저 98.56458%다. terminal-tail은 3쪽·최저
  57.39116%, picture-band는 2쪽·최저 80.65412%다. 동일 원본의 Native169 비교는
  2742이며 base devel의 비교가 아니다. 실제 Native171 기하 비교는 2753에 있다.
- `cargo fmt --all --check` 종료 0: `logs/native174-fmt-check-2752.log`.
  전체 Rust·Clippy·fresh WASM의 최종 통과를 뜻하지 않는다.

### 현재 후보가 저장한 바이트의 독립 Print

과거 Native171 저장본은 Native174에서 다시 열어도 116쪽이다(2737).
현재 Native174의 새 55·56·200자 저장본은 SHA를 고정해 각각 새 한컴 2024 Print를
요청했다(`mcp-edited-controls-2746/results.json`, 실제 13.0.0.3901, PrintMethod 0).
셋 모두 한컴과 Native가 115쪽이지만 시각 검증은 통과하지 못했다.

| 현재 저장본 | Native/Print 쪽수 | Native 최저 raw | 90% 미만 쪽수 |
| --- | --- | --- | --- |
| 55자 | 115/115 | 11.38223% | 109 |
| 56자 | 115/115 | 11.38223% | 109 |
| 200자 | 115/115 | 15.47782% | 108 |

원본·PDF SHA, 실행 명령과 전쪽 TSV는 `edited-print-native174-2747-result.json`과
해당 출력 폴더에 보존했다. 55자 5·8쪽의 공식 review/standalone overlay는
`edited-native174-reviews-2754/results.json`에서 연결하고 두 종류를 직접 열었다.
5쪽 첫 중첩 표의 마지막 행은 PDF의 수평 괘선이 96dpi의 y=84~85/113~114에
있는데 현재 표 상자는 y=85.0, h=17.3으로 더 짧다. 뒤 본문도 위로 이동한다.
이는 관측 좌표이며 구현 상수나 회귀 기대값으로 사용하지 않는다. 같은 쪽수·같은
본문 소속만으로 시각 개선을 선언할 수 없음을 확인했다.

terminal-tail 2쪽의 raw 100%도 3쪽의 57.39116%를 상쇄하지 않는다. 공식 패널
2748·2750에서 2쪽 내용과 3쪽 빈 물리 띠를 각각 확인했다. 후속 보정은 자식의
내용 컷과 미소비 물리 상자, 실제 paint의 행 높이 소비 경로를 함께 대조해야 한다.
명시적인 하위 쪽 나누기 보존도 확인해야 하며, 새 컷 유효성 조건을 부모 문단의
명시적 속성만으로 완결된 계약이라고 판정하지 않는다. 기존 검사의 기대값과
baseline을 유지한 채 같은 입력·Print에서 보정을 계속한다.

## Native175: 잘못 확장한 물리 높이 재해석을 제거

새 저장본의 5쪽에서 확인한 3×3 자식 표는 첫 조각의 끝 컷 [2,2,2]와 다음 조각의
시작 컷 [2,2,2]가 같다. 그러나 실제 배치는 마지막 행을 38.42667px/17.09333px로
만들었고, 부모 생산자는 마지막 유닛에 36.42667px의 물리 예약을 이미 기록했다
(`native174-wrapped-cut-trace-2755/receipt.json`). 새로운 signed 조정이 그 예약을
단순 내용 높이로 낮췄다. 내용 커서가 끝났다는 사실만으로 미소비 물리 띠를 없앨 수 없다.

- 저장 호스트에 실제 자식 RowCut이 있다는 이유로 NO_LS 재조판용 예약 계산에
  넣었던 조건을 제거한다. 저장 행 유닛의 원본 물리 프레임과 내용 커서를 구분한다.
  예약과 viewport가 호출하는 같은 helper에 적용하며, 실제 NO_LS 재조판은 유지한다.
- 재조판에서도 자식 최소 높이는 추가 요구만 보완한다. 생산자가 이미 기록한
  물리 예약을 음수 조정으로 없애는 실험을 제거한다. px·문서 ID나 기대값을 보정하지 않는다.
- 옛 강제 컷의 유효성과 원본 좌표계를 분리한 Native174 보정은 유지한다.
  동일 source Print·189개 집중 검사·실제 편집 API·230쪽 원본과 정상 대조군을
  묶어 Native175에서 확인하며, 이 단계의 변경만으로 시각 완료를 주장하지 않는다.

별도 출력의 명시적 하위 쪽 나누기 진단(2758)은 기여 입력의 UNIT010에 Page 속성을
주고 실제 편집 API를 실행했다. Native174는 편집 전후 모두 UNIT000~017을 4쪽에
두었다. 이 진단만으로 새 provenance 조건의 후퇴라고 판정할 수 없으므로, 원래부터
무시된 경로와 현재 변경의 영향을 구분하기 전에 새로운 helper를 추가하지 않는다.
진단은 임시 output이며 정식 회귀 추가나 한컴 시각 일치 증거가 아니다.

### Native175 결과와 독립된 전체/부분 프레임 근거

Native175 빌드 종료 0·250.11초. 전체 189개의 재실행에 앞서 이전 실패와 #7114의
15개 경계를 확인해 10 PASS / 5 FAIL을 관측했다(2771). #7418 3건과 #7902 2건은
회복했고 70833은 새 Print와 같은 18쪽이다(2768). 70833의 시각 미달 7쪽은 남는다.
원본 HWP/HWPX 230쪽·최저 95.69614%와 정상 auto 16쪽·98.56458%도 유지했다.
동일 Native174 저장본의 55자 5쪽 raw는 25.20047%에서 74.59554%로 올랐지만
8·66쪽과 200자 편집은 미달이다(2772). 이는 새 Native175 저장 바이트의 검증이 아니다.

실제 API 14종(2761)에서 HWP의 56자 입력 후 삭제는 live 115/재열기 116쪽으로
102쪽부터 다른 새 차이가 생겼다. HWPX의 같은 동작은 기존 6쪽 소속 차이가 남는다.
이 후보를 완료로 수용하지 않으며, 기대값·테스트를 변경하지 않는다.

같은 3×3 자식의 원본/55자 편집 Print 벡터를 직접 읽었다(2773).
원본의 통째 표는 전체 101.65px·마지막 행 67.45px이며, 분할본의 마지막 행은
첫 조각 41.87px + 다음 조각 29.09px = 70.96px다. 선언 7635HU/5331HU와
Print의 배율·반올림을 고려하면 각각 전체 표 높이/분할 행 높이에 대응한다.
기존 `fit_measured_table_declared_tail_to_declared_height`의 빈 꼬리 회수 규칙은
전체 표의 관측을 설명한다. 측정이 없는 경로에 새 수식을 복제하지 않고 같은 계산을
공유하되, 실제 부분 내용 컷에는 전체 표 회수 규칙을 적용하지 않아야 한다.

## Native176: 전체 저장 프레임의 회수를 측정·예약·배치에서 공유

기존 `fit_measured_table_declared_tail_to_declared_height`의 계산과 판정 조건을
`table_row_frame::declared_tail_row_height`로 옮겼다. 측정 경로는 같은 계산을 호출해
기존 누적 높이·캡션을 보존한다. 임계값·내용 하한·저장 되감김 배제 조건은 바꾸지 않았다.

측정이 없는 부분 표 경로도 전체 행 범위·빈 시작/끝 컷·별도 이월/높이 override 없음일 때
같은 회수를 적용한다. `nested_cut_height`와 저장 호스트 원장 역시 전체 자식이 선택되고
그 원본 줄·선언 상자가 회수를 입증한 경우만 같은 결과를 사용한다. 실제 부분 행의 선언
잔여 띠를 이 전체 표의 회수와 혼동하지 않는다. 첫/끝 컷과 소유 영역을 공유하며, 별도
보정 좌표나 문서 ID를 만들지 않는다.

이것은 전체 저장 프레임의 계산 경로 차이 보정이다. 부분 행의 실제 수용 띠와 다음 쪽
미소비 띠까지 검증을 마쳐야 전체 시각 완료로 판정할 수 있다. Native176에서 기존 집중
검사·공개 API·같은 Print 경계와 정상 대조군을 함께 확인하며, 실패 기대값은 유지한다.

### Native176 검증 결과

빌드 종료 0·247.56초. 기존 189개와 #5906 1개·#7095 4개를 함께 실행한 2777은
**188 PASS / 5 FAIL / 1 ignored(총 194개, 실제 실행 193개)**다. #5906과 #7095의
기존 검사 5개는 모두 통과했고 기존 #7418·#7902 회복도 유지했다. 실패는 #7518의
정렬·상자 4건과 CBTA 내부 컷 1건이며 기대값을 바꾸지 않았다.

- 원본 HWP/HWPX 각 115쪽의 Native 최저 95.77381%, 원본 4쪽은 양쪽 모두
  95.69614% → **99.99055%**다(2783). 2789의 공식 review·standalone overlay를
  함께 열어 전체 표의 아래 경계와 글줄·본문을 판독했다. 글자 잉크 굵기 차이는 남으며
  이 값은 정답률이나 엄격 RGB 일치율이 아니다. fresh WASM·px 회귀 자격은 아직 아니다.
- 실제 Native176의 정책·원점·괘선 35문서/66쪽은 전쪽 raw 100%(2784). 각 행의 실제
  실행 명령이 동결 Native176 바이너리를 사용하는지도 대조했다.
- 정상 auto/재저장 auto 각 8쪽은 최저 98.56458%. terminal-tail 3쪽은 최저
  57.39116%, picture-band 2쪽은 최저 80.65412%(2788)로 아직 미달이다.
  최초 2781은 버전 번호 문자열 치환이 PDF SHA의 `175`도 `176`으로 바꾸어 일부 실행
  후 검증에서 중단됐다. 실제 입력·PDF가 바뀐 문제가 아니며, 이전 변환 manifest에서
  바이트 해시를 직접 읽어 2788로 재실행했다. 최초 실패 로그도 보존한다.
- 70833은 18/18쪽이지만 7쪽 미달·최저 16.94452%(2782), 새 한컴 2020 CBTA는
  204/204쪽 중 12쪽 미달·최저 37.96169%(2785)다. 기존 내부 컷 기대값 변경의 근거로
  쓰지 않는다. 편집본 경계 2780의 미달도 남아 있다.
- 실제 편집·삭제·저장 API 14종(2778): 12종은 쪽별 본문까지 일치한다. HWP의 56자
  입력 후 삭제는 live 115/재열기 116쪽·102쪽부터 차이, HWPX 같은 동작은 115쪽이지만
  6쪽부터 본문 소속이 다르다. 편집 문단 자체는 원래 4줄·원점으로 돌아와도 표의 이전
  재조판 provenance는 남는다는 관측을 다음 보정에서 추적한다.
- Native174/176 저장 바이트 비교(2791): HWP 56자 입력 후 삭제만 다르다. 현재
  55·56·200자 저장본은 기존 2746 Print 입력과 바이트가 같으므로 같은 독립 PDF를
  재사용할 수 있다. 2780은 해당 동일 바이트·현재 Native176의 일부 경계 검증이며
  편집본 전쪽 완료로 확대하지 않는다.

전체 저장 프레임 개선을 별도로 보호하기 위해 동일 source의 fresh WASM을 빌드해 원본
두 문서 전쪽을 대조한다. 원본 검증의 완료가 편집본·다른 문서의 미달을 면제하지 않는다.
부분 행의 실제 잔여 상자 전달은 #7095 원 PR #7669와 실제 통합 `9ced3b518e`·#7923의
기록(2787)을 읽었다. 커서의 `start_row_height_override`로 같은 수용 띠를 scan과 paint에
전달하는 계약이며, 자식 내용 커서만으로 같은 사실을 입증할 수 없다. 이 축과 남은 전체
Rust·lint·차트·45개 PR 범위 검증이 끝나기 전에는 새 PR을 제출하지 않는다.

## fresh WASM049와 완료 원본 4쪽의 새 회귀

2026-10-24 `git fetch upstream devel`을 실행했다. `upstream/devel`은
`d9fd85ed5bc3c4d9ce6b2981422e48bf9bb90867`로 바뀌지 않았고 현재 후보의 조상이다.
사용자의 동일 head 생략 지시에 따라 리베이스하지 않았다(2795). 새로운 외부 PR은 적용하지 않았다.

동결 제품 소스 `adc54cb8847ebc06637d6f6df296be5887658e2f`의 standard release
fresh WASM049 빌드 종료 0·878.09초와 `verify_release_wasm.py` 패키지 검증을 확인했다.
공유 `target/pr-review`·빌드 잡 8개이며 pkg와 Studio public의 JS/WASM 해시가 같다.
Native176과 같은 제품 소스 바이트인지 빌드 전에 대조했고 검증 중 제품 소스를 바꾸지 않았다.
이는 최종 전체 Rust·lint·Studio 검증 완료를 뜻하지 않는다.

각 원본 115쪽, 합계 230쪽을 같은 글꼴·96dpi·Print 프로필로 비교했다.
Native 최저 95.77381%, fresh WASM 최저 95.70233%이며 미달·누락 쪽은 없다.
원본 4쪽은 각각 99.99055% / 100%다. 2801의 공식 review와 standalone overlay에서
표의 위·아래 괘선, 헤더·재료 3줄·압력 값과 앞 본문을 같은 쪽에서 직접 판독했다.
글자 잉크 굵기·색상 차이는 남으며 raw 100%를 RGB 일치나 문서 전체 정확도로 해석하지 않는다.
실루엣-only gate의 `not_evaluated`도 전체 PR 승인으로 바꾸지 않는다.

### 정식 입력과 독립 Print

기존 원본 입력은 그대로 재사용한다. 새 MCP Print PDF는 임시 진단 위치의 파일을 숨기지 않고
다음 정식 경로에 바이트 그대로 보존했다. 둘 다 원문 그대로 PrintMethod=0 전체 출력이다.

- `samples/issue1949_giant_cell_nested_tables_perf.hwp`: SHA-256 `ef10261cd29325116028e4f4f3e6be1a72c675eb771bddfd8484e7fe5aa94b4e`.
  기준: [`original-hwp-2024.pdf`](../../pdf/issue1949-original-print-20261024/original-hwp-2024.pdf),
  SHA-256 `88706653cafe77001bcaa3f0ecfb4e64f08987c8d2caa08a66673822cc0fc52a`, engine 2024, 115쪽.
  MCP job `ae492a86-fbf1-403e-a0c9-4e07e3ec16cc`.
  제출 TSV와 4쪽 대표 PNG:
  - [native TSV](../pr/assets/issue_1949_complete_stored_frame/native/hwp_silhouette.tsv)
    (최저 95.77381%, SHA-256 `5033bfd29c9372587591de536e773f0541af2e9a52312f3fb455d1fda57c2839`).
    [review](../pr/assets/issue_1949_complete_stored_frame/native/hwp_p004_review.png) ·
    [overlay](../pr/assets/issue_1949_complete_stored_frame/native/hwp_p004_overlay.png).
  - [wasm TSV](../pr/assets/issue_1949_complete_stored_frame/wasm/hwp_silhouette.tsv)
    (최저 95.70233%, SHA-256 `1b807f6866b0c1851a8a63792771ea654385eb4504fabc7600023750227db77a`).
    [review](../pr/assets/issue_1949_complete_stored_frame/wasm/hwp_p004_review.png) ·
    [overlay](../pr/assets/issue_1949_complete_stored_frame/wasm/hwp_p004_overlay.png).
- `samples/issue1949_giant_cell_nested_tables_perf.hwpx`: SHA-256 `fc6e5f156de470dfbb14aab392389491720ee7fb1bf6f03fe9a018e93b420c65`.
  기준: [`original-hwpx-2020.pdf`](../../pdf/issue1949-original-print-20261024/original-hwpx-2020.pdf),
  SHA-256 `d3af0cbddf49bce84d25083775ed4e05f7f9bde0fe01af8b772fb7b8dd36a08e`, engine 2020, 115쪽.
  MCP job `d3f25cae-9be4-4501-ada1-04da03fbb04a`.
  제출 TSV와 4쪽 대표 PNG:
  - [native TSV](../pr/assets/issue_1949_complete_stored_frame/native/hwpx_silhouette.tsv)
    (최저 95.77381%, SHA-256 `9881adb58845de4b78108cc5876c40e51ecb7b132c4f42e519b8da911b286f65`).
    [review](../pr/assets/issue_1949_complete_stored_frame/native/hwpx_p004_review.png) ·
    [overlay](../pr/assets/issue_1949_complete_stored_frame/native/hwpx_p004_overlay.png).
  - [wasm TSV](../pr/assets/issue_1949_complete_stored_frame/wasm/hwpx_silhouette.tsv)
    (최저 95.70233%, SHA-256 `76ccdb4118f839988e4d827f5294f92942f6d3076e67f015bfbfce01bc7099f0`).
    [review](../pr/assets/issue_1949_complete_stored_frame/wasm/hwpx_p004_review.png) ·
    [overlay](../pr/assets/issue_1949_complete_stored_frame/wasm/hwpx_p004_overlay.png).

### 검사와 실패 진단

새 이름 `tests/cases/issue_1949_complete_nested_stored_frame.rs`의 HWP/HWPX 2개 검사는
원본 문단 81·컨트롤 0·3×3 표를 소유 정보로 선택한다. 독립 PDF 괘선의
`[154.797333, 969.658691, 669.079997, 1071.308024]`를 96dpi에서 ±2px로 검사하고
바깥 셀 포함, 재료별 내용의 단일 소유·순서·줄 비겹침 및 각 셀의 글줄 포함을 함께 검사한다.
좌표로 검사 대상을 찾거나 전체 SVG 해시를 고정하지 않았다. 해당 쪽 양쪽 98% 조건을 먼저 충족했다.

같은 새 검사·기대값을 이전 Native175와 Native176에 각각 링크한 2803에서
**수정 전 2 FAIL → 수정 후 2 PASS**를 확인했다. 이전 아래 경계 1076.32px가
독립 Print 1071.308024px ±2px를 벗어난 것이 실패 원인이다. 다른 결함으로 인한 실패나
수정 전에도 통과하던 검사를 수정 효과로 표시하지 않는다. 정식 suite 자동 수집은 현재
정본의 `--prepare`로 생성했다(2806). 초기 legacy `--adopt`/`--generate`의 파생 manifest
오류는 로그를 보존한 뒤 현행 절차로 재생성했으며 파생 파일은 커밋하지 않는다.

편집 복원 진단은 별개다. 실제 56자 삽입·삭제 후 같은 모델의 새 Print(2794)는 두 형식 모두
115쪽이다. HWP 현재 재열기는 116쪽으로 전체 대조가 중단됐고 8·102쪽은 각각
26.27084%·39.97791%다. HWPX 재열기는 115쪽·전쪽 최저 95.77381%(2796)지만
편집 중 live와 재열기 간 6쪽 내용 소속 차이는 남는다. 이 실패를 원본 통과로 면제하지 않는다.

2797/2800에서 복원 직후 표의 직렬화 모델과 DocInfo 진단값이 원본과 같음을 확인했다.
현재 live 쪽별 내용은 원본과 다르지만 같은 문서 모델을 `set_document`로 재구성하면
두 형식 모두 원본 115쪽의 내용과 일치한다. 남은 파생 상태·재조판 provenance를 추적할
근거이며 자동 제거 규칙을 구현한 상태는 아니다. 최초 2797 진단은 DocInfo에 Serialize가
없어 컴파일이 실패했다. 제품 코드를 바꾸지 않고 진단 표현만 고쳐 재실행했다.
실행 로그·중간 JSON·진단 스크립트는 ignored output에 보존한다.

남은 편집본·부분 행·차트·전체 Rust/lint와 PR 기능 보존 검증이 끝나기 전에는
PR 준비 완료·생성·승인·통합으로 판정하지 않는다.

### 현행 suite 정책과 실행 경로 보존

정확한 devel base의 manifest 검사는 기존 커밋 `36b64bccfe`의 #4969 request contract
target 이동과 bidi 의존성 추가가 같은 PR에 섞인 것을 거부했다(2807/2808).
당시 이동은 독립/중첩 39개 검사의 동일성에 근거했으나, 현행 registry 유지보수 정책은
Cargo marker 바깥 변경과의 혼합을 허용하지 않는다. 정책·target 예산을 완화하지 않고
해당 target과 분류만 devel의 원래 독립 실행 경로로 복구했다. 기존 39개 검사의 내용은
바꾸지 않았다. 예외 20개·전체 target 48/48이며 source 1750개가 자동 수집된다.
커밋된 HEAD를 대조하는 정책 검사는 이 복구 커밋 뒤 정확한 같은 devel SHA로 다시 수행한다.
policy 단위 검사 25개는 통과했다(2810). 새 회귀 2개와 기존 shaping 39개는 같은
Cargo/nextest 실행 2809에서 함께 검증하며, 그 결과를 확인한 뒤 단계 커밋을 만든다.

4쪽의 원본 HWP/HWPX Native raster는 서로 SHA-256이 같고, WASM raster도 서로
같으며, 두 독립 Print의 해당 raster도 같다(2801). 직접 판독한 두 backend의 동일
바이트 쌍이므로 같은 그림을 형식별로 반복 표시하지 않았다. 전체 TSV는 형식별 원본을
각각 보존하며, 페이지 단위 동등성을 전쪽 동등성으로 확장하지 않는다.

추가 2813 대조에서 HWP 재열기 표는 원본보다 저장 `vertical_pos` 3473개·tag 2개가
달라졌다. 첫 차이는 문단158의 첫 줄 0→73560이며, HWPX의 차이는 raw header
바이트뿐이다. 표 JSON 비교는 serde 제외 필드의 동등성까지 입증하지 않는다. 전체
파생 상태 재구성으로 양쪽 내용이 회복된 사실과 함께 다음 원인 분석에 사용한다.
문서의 새 상대 링크 14개는 파일별로 존재를 확인했다(2812). 공통 링크 검사기의
전역 redirect 탐색은 중단했으며 이를 정식 검사 통과로 기록하지 않는다.

2809 정식 Cargo/nextest는 종료0, 266.96초이며 **41 PASS / 0 FAIL**이다.
새 표 프레임 2개와 원래 독립 shaping 39개가 실제 실행됐다. 다른 suite의 275개 필터
skip은 성공 수에 넣지 않았다. 이 실행은 전체 Rust 검증이 아니다. 제품 구현은 Native176
그대로이며 이번 단계의 추가 변경은 테스트·target 라우팅·증거와 기록이다.

단계 커밋 `7c13d215c1` 뒤 같은 고정 devel SHA의 정책 검사도 종료0으로 통과했다
(2815): 1750 sources / 7605 static test attrs / 28 suites + 20 exceptions = 48/48 targets.
`cargo fmt --all -- --check`와 `git diff --check`도 통과했다. 검증 뒤 다시 fetch한
devel은 같은 `d9fd85ed5b`이며 새 반영 PR이나 리베이스 대상은 없다. 이번 단계의
제품 소스·전쪽 시각 증거는 Native176/WASM049이고, 후속 테스트·라우팅·기록 커밋을
새 제품 빌드의 SHA로 바꾸어 보고하지 않는다. 나머지 보정과 최종 검증은 계속 필요하다.
