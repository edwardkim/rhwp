# #7998 PDF 의미 표·문단 구현 결과

상태: **원격 devel 통합·필수 로컬 검증 완료, 승인된 Open PR 등록 후보**.
작업지시자는2026-10-21 최신 원격 devel 병합 후 PR 등록을 지시했다.
현재는 원격 등록 전이며 self-review·merge·이슈 종료를 완료한 것으로 표시하지 않는다.

## 구현 결과와 기준

PDF의 문자·경계·지역 격자에서 확인한 소유를 실제 Table/Cell과 Paragraph/LineSeg로
연결했다. 셀 테두리·채우기, 검증한 병합·중첩과 행열/문자 편집, HWP/HWPX 저장·재열기를
소비 경로까지 연결한다. 불확정 구조는 원문을 단일 소유로 보존하고 보류 이유를 진단한다.
모든 PDF 작성 객체나 원래 Excel 셀 구조를 완전히 복원했다는 주장은 하지 않는다.

- 기능 보정 commit: `d1d3091c5`; 전체 테스트 head: `168e96b86`;
  동작 불변 lint 정리 head: `979188617` (`c448e70a4`의 후속).
- 최신 통합 devel: `2e3b4f880bd11d35398e6ea0100846611f13a72d` (#7930).
- 구현 보존 `e58323511` → 충돌 없는 merge `49cd26522` → rustfmt 기록 `d921ccac8` →
  실제 저장 공백 보정·회귀 `1f24b1526` 이후 실제 그림 배치/독립 선택 페이지 자원 계상과
  세 쪽 보정을 추가했다. 소스가 달라진 전체 Rust·lint·출력 경로는 새 제품으로 검증한다.
- 유입 Qt 참조 창·Native clipboard 공백/emoji·빈 clipboard 거절·geometry revision은
  원 PR과 실제 diff를 대조했다. Qt adapter25검사를 확인했다.
- 같은 제품 동작의 빌드/판정을 재사용했다. lint-only 차이와 실행 head는 아래에 구분한다.
  로그·중간 JSON·서비스 응답은 ignored
  `output/pr-review/issue7998/pr-registration-20261021/`에 보존한다.

구현·독립 근거·반례·사용자 판정의 연결은 단계별 기록을 유지한다.
최근 통합과 실제 새 제품의 명령/결과는 [stage38](../working/task_m100_7998_stage38.md),
실사용 복구·성능·추가 입력/캐럿은 [stage37](../working/task_m100_7998_stage37.md)을 따른다.

## 실제 문서 개선

메인테이너의 실제162쪽 문서에서 처음에는 표 인식이 실패하고 지연이 과도했다.
수용된 복구본은 실제 Table162/Cell54,938을 생성한다. 공통 정확 메트릭 재사용과 중복
인식 증거 복사 제거로 같은 조건의 Native 가져오기·저장24.13/25.08→19.90/18.45초,
headless Chrome 첫 표시28.27→17.82초를 관측했다. 당시 저장/문자 계약을 유지했다.
메인테이너는 표 인식과 “이제 기다릴만하네요”로 속도 개선을 수용했다.
새 최종 제품의 동시 빌드/캡처 중25.33초는 통제된 성능 재측정이 아니므로 이 값과
비교해 속도 개선/회귀를 주장하지 않는다.

추가 실사용 입력의 font metric·Table 소유·그림 마스크·XObject·collection 자원 경계와
회전 글자 열기도 보완했다. 자원 상한/중복 예약·캐시의 실제 범위를 구분했고,
`parser_allocation_bounds_proven=false`와 나머지 parser 할당 감사 백로그는 유지한다.
모든 비신뢰 PDF에 대한 안전성 증명이나 배포 준비 완료를 주장하지 않는다.

PDF 글상자 더블클릭은 원래 hit의 path·offset·page를 유지하고 실제 paint와 같은 frame
변환을 cursor/hit에 적용한다. boolean15쪽과 회전2010글자의 실제 입력·삭제·양 저장을
확인했으며 메인테이너가 **편집 동작 통과**로 판정했다. 정지 클릭을 resize로 처리하는
경계와 선택된 반복 바탕쪽의 페이지 이동도 대조했다.

## 실제 저장·독립 한컴 Print

제출 전 처음 확보한162쪽 저장본의 한컴2020 Print에서43쪽 전각 공백 장평197%가
부서명 마지막 글자를 다음 줄로 넘기는 결함을 검출했다. 같은 원본의 devel 저장·새 Print는
정상이라 현재 변환 경로의 결함으로 판정했다. 미수록 U+3000 폭을 반각으로 재고 fit한
것이 원인이었다. 공통 측정의 미수록 전각 공백만 전각으로 처리하며 실제 metric hit와
ASCII·NBSP·HWP 고정폭 빈칸 계약을 보존했다. 문서 ID·특정 수치 예외를 추가하지 않았다.

`GlyphWidths::measure → observed_line_text fit → char_width_decision → CHARSHAPE →
실제 글줄`을 추적했다. 보정 후 literal 공백과 한 문단의 문구를 유지하면서 장평100%를
저장한다. 독립2020 Print43쪽에서 한 줄 보존을 확인했고 Native99.55895/WASM99.57078%
및 직접 판독 뒤 정식 관계·Worker 회귀를 추가했다. 동일 실제 Worker 검사에서 수정 전
동결 WASM은 정확히197% 저장 단언에 실패하고 새 WASM은138검사·실제7 Worker를 통과한다.

새 실제 Studio에서 셀 X 삽입·직접 삭제 후 HWP/HWPX를 저장·재열기했다. Native 무편집
저장과 section0.xml이 달라 독립 Print를 새로 요청했다. 실제 편집 복원 양 형식 모두
한컴2024 `13.0.0.3901`·전처리none·Print method0·원본 용지/100%/한 쪽씩162쪽을
인쇄했다.2020 전체 작업의 worker 종료는 실패로 보존하며2024 성공으로 덮어쓰지 않는다.

| 실제 편집 복원 저장본 | Native 전체 | fresh WASM 전체 | 독립 Print 쪽수 |
|---|---|---|---|
| HWP | 162쪽 최저97.08677% | 162쪽 최저97.12104% | 162쪽 일치 |
| HWPX | 162쪽 최저97.08677% | 162쪽 최저97.12104% | 162쪽 일치 |

누락·90% 미만·쪽수 차이는 없다.43쪽과 최저162쪽을 직접 대조했다. 원본 PDF·rhwp의
외형과 한컴 재인쇄의 머리행 위 선·일부 제목 줄 표현/선 농도 차이는 B26에 명시한다.
작업지시자의 잔여 범위 백로그 지시를 유지하며 한컴 완전 호환으로 표시하지 않는다.

## 전쪽 시각·실제 제품 검증

| 고정 범위 | 새 제품 결과 |
|---|---|
| 기존 Native158입력169쪽 | 일반163쪽 최저92.17391%; 정식 Helvetica 글꼴 예외6쪽 raw최저89.54211% |
| 같은 fresh Docker WASM169쪽 | 모두90% 이상; 최저90.70673% |
| 추가 실사용 원본138쪽/backend | 모두 직전 수용 PNG와 동일; 기존 출력 예외 유지 |
| 실제 편집 복원 양 저장 형식324쪽/backend | 최저97.08677/97.12104%; 누락·쪽수 차이0 |
| 추가 승인된 tac-img-02 전체66쪽/backend | 최저96.39090/95.51053%;26·45·53쪽 및 직접 확대 판독 통과 |
| 실제 Worker | 7 Worker/138검사; 원본→양 저장의 공백/문단 계약 포함 |
| 실제 CanvasKit 강제 경로3쪽 | fallback 없이99.77925/100/99.87775%; 실제 host font provider 사용 |
| 기존 front-end2348 PASS/2 SKIP·집중6건·TypeScript | 해당 소스 바이트를 확인해 재사용; 새 Worker는 다시 실행 |

169쪽은 새 Native/fresh WASM 각각 이전 수용 PNG·공개 원문 TSV와 모두 같다.
최신 devel2e3의 실제 유입은 Native clipboard/Qt이며 렌더링에 사용한 기준5e2의 경로는
같아 기준11입력22쪽/backend 증거를 source 범위 대조로 재사용한다. 전체 바이너리가
같다고 주장하지 않는다. 비공개 견적서 Native1쪽의 devel 대비99.79020→95.16159%
하락은 기존 승인된#8110 후속이며 devel 무회귀0건으로 바꾸지 않는다.

추가 입력 중 XObject18쪽의 기존2쪽 글꼴/무늬, collection69쪽의 기존 Native 한글 미표시,
회전 원본7쪽의 기존7쪽은 사용자가 열기/회전 범위로 한정해 출력 백로그를 승인했다.
전체 미달 쪽·원문 TSV를 그대로 제출하며 일반90% 통과와 구분한다.
개인 이력서·비공개 견적서 원문/식별 가능한 이미지는 로컬에만 유지한다.

[기존 전쪽 TSV·정식 글꼴 예외·입력 대응](../pr/assets/issue_7998/final-p5/README.md)와
[추가 전쪽 TSV·실제 저장 원본/Print·대표 PNG](../pr/assets/issue_7998/submission-20261021/README.md)를
보존했다. 원문 TSV의90% 미달·기준값·허용치를 바꾸지 않았다.

## 추가 승인된 기존 세 쪽 보정

사용자는 그림 기준선 검사 원본의 기존26·45·53쪽도 이번에 수정하도록 지시했다.
뒤쪽 번호 재시작 제어가 앞쪽 쪽번호 위치까지 숨기던 상태 처리를 고쳤다. 실제 원본
굴림 프로그램의 하강 메타데이터를 메트릭 별칭을 거쳐 공급하고 Typesetter/Paginator가
같은 물리 쪽의 제어 이벤트를 소비한다. 표지 감추기와61쪽 번호1 재시작은 보존한다.
45·53쪽은 선행 글줄을 이미 배치한 뒤 표 줄만의 상한으로 문단 끝을 되감던 결함이다.
formatter·원본 저장 줄·실제 표 높이·다음 문단의 닫힌 점유가 일치하는 경우 전체 선행
글줄부터 표 줄까지의 점유를 공유한다. 수정/분할/무효 저장 정보에는 적용하지 않는다.

독립 한컴2020 Print66쪽 대비26/45/53쪽은 Native100/99.76853/99.97208%,
fresh WASM100/99.81509/99.97212%다.66쪽 모두90% 이상이며 실제 굴림·맑은고딕 bold를
추가 공급한 대조에서도 같은 전쪽 최저를 유지한다. 메인테이너가 **시각 판정 통과**를 주었다.
글꼴 공급은 기존64MiB 한도 안에서 원본 glyph advance를 보존한 추출/부분 집합으로 준비했고
상용 글꼴은 커밋하지 않는다. 최초 수용본과 추가 공급의 PNG 차이를 구분해 공개했다.

두 정식 회귀가 수정 전 실제 producer에서 의도한 원인으로 실패하고 수정 후 통과했다.
26쪽은 양쪽 raw100%·직접 판독에 근거해 독립 기준선만±2px로 보호하고, 나머지는 내용
소유·순서·비겹침 관계를 검사한다. 기존#6708의 무효한 TAG 수동 입력 가정을 원 계약과
독립 출력에 맞춰 수정했으며,98% 미달1쪽의 절대 위치 pin은 원본 줄 정렬 관계로 바꿨다.
[전체66쪽 TSV·원본 SHA·대표 PNG·공급 근거](../pr/assets/issue_7998/submission-20261021/tac-baseline/README.md).

## CI 글꼴·검증 경계

새 공백 회귀는 OS 글꼴 설치 대신 저장소의 공통 메트릭, literal 공백·문단 단일 소유와
저장 CHARSHAPE를 검사한다. CI workflow의 중복 호출이나 개인 PC 절대 글꼴 경로를
추가하지 않았다. 정식 Native Helvetica6건은 source glyph program 부재, 실제 Native의
Noto Sans KR과 한컴 대체 Batang, 동일한 geometry·해시 고정 증거를 유지한다.
`font_mismatch_exception`을 일반 통과나 자동 승인으로 표시하지 않는다.
예외6건은 최종 lint head의 JSON/해시로 실제 도구 판정을 모두 통과했다. 기존169쪽과
추가138쪽은 RGB 세 채널을 직접 비교해 이전 수용본과 동일함을 확인했다.

전체 회귀 이후의 변경은 attribute 뒤 빈 줄 제거, Copy인 bool/Option의 동등한 패턴 매칭,
동일한 데이터 helper 본문을 테스트 앞에 이동, 고정110,000 개수 검사의 동등한 부등호 정리다.
여섯 조건 조합·함수 본문·좌표·소유·자원 계상과 검증 강도는 변하지 않는다. 기존 성공한
전체 실행과 동결 producer의 시각 증거를 재사용하며 실제 실행 head와 후속 head를 구분한다.

CI와 같은 Studio/Chrome/Firefox/VS Code 배포 빌드를 완료한 뒤 글꼴/라이선스8검사,
실제 Canvas font binding8검사와 확장 배포3검사를 통과했다. 첫 실행의 dist 부재·Node
의존성 미설치는 환경 실패로 구분하고 동일 lockfile로 준비한 뒤 같은 검사를 통과했다.

필수 로컬 검증을 완료했다. 실제 실행 head·명령·경과 시간은 [stage38](../working/task_m100_7998_stage38.md)의 최종 게이트 표에 연결한다. 동작 불변 lint-only 차이는 위에 구분했다.

| 검증 | 결과 |
|---|---|
| release build / release lib | 성공 / 3,984 PASS·실패0·기존12 ignored (workspace package 합계) |
| 전체 integration nextest | 11,209 PASS·실패0·기존45skip |
| Native Skia lib / 그림 placeholder / 직접 PDF | 4,041 PASS·실패0·기존12 ignored / 2 PASS / 14 PASS |
| Rust fmt / Native·WASM·workspace all-target·root all-target Clippy | 모두 성공, 경고 무시나 기준 완화 없음 |
| workspace build / doc tests / Native FFI build / 실제 Qt smoke | 성공 / 8 PASS·기존3 ignored / 성공 / 성공 |
| 정식 Native 글꼴 예외 | 최종 lint head의 실제 도구 판정6건 모두 font_mismatch_exception |

## 후속 범위와 원격 처리

남은 내용 성장 격자#8106, 보고서6쪽 glyph 폭#8107, 본문1/4쪽 승격#8108,
Base14 Helvetica#8109, 선·작은 글자 coverage#8110과
[백로그](../plans/task_m100_7998_backlog.md)를 유지한다.5pt 표준 셀 전체 채움 수용,
비사각 clip의 rhwp 왕복, 작은 clip 경계와 B26 재인쇄 차이도 해결로 표시하지 않는다.
미완료 범위는 새 기능을 계속 확장하지 말고 후속으로 처리하라는 지시에 따른다.

이 결과보고는 승인된 원격 push·Open PR 등록 직전에 확정했다. 등록 후 정확한 PR head의 CI와
실제 본문/시각 URL을 다시 확인한다. self-review·merge·관련 이슈 close는 이번 등록
완료와 구분한다.
