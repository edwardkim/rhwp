---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase17-goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase17-goal-checkpoint.md` · 원문 SHA-256: `cf9c52c1e446f424fa37d50cf517f50e95600526460cac085ccc3fda1773193c`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 17차 체크포인트

목표는 ACTIVE이다. 이전 16차 턴은 실제 감사·반례·원장 보정·이슈 갱신을 했으므로 progress였다. 이번도 직접 분류, 원본 CFB 독립 확인, 새 전쪽 캡처, 기존 실제 검사, 원장/이슈 갱신을 수행했으므로 progress이다. 완료/차단/일시 정지로 바꾸지 않는다.

## 작업장과 승인

- R=/Users/tsjang/rhwp, branch=audit/8232-px-regressions-20261025.
- HEAD/base=238de6de41a208648f4ade6b494fe433a81c1fce. 마지막 tracked status는 clean이다.
- 제품·테스트 소스 수정과 개명은 여전히 0건이다. 이번 새 도구/PNG/TSV/로그/JSON/보고서는 모두 ignored output/issue8232-audit 아래다.
- 전체 목표는 전수 조사 후 이슈 정리, 현행 지침과 독립 근거에 따른 실제 보정/issue_8232 새 명명/실행 등록·helper·module·manifest·생성·E2E 호출 갱신/필수 검증이다. 감사-only로 완료하지 않는다.
- 독립적으로 유효한 결함 검출을 보존하며 잘못된 기존 단언/대상/기대값을 보존하지 않는다. 원 이름 wrapper/중복 등록은 만들지 않는다.
- PR/push는 나중이다. #8232 갱신과 필요한 MCP는 승인돼 있지만 새 PR/push 승인은 없다. subagent·다른 task 메시지·예약·메모리 쓰기는 하지 않는다.
- Cargo 순차/jobs8/tests8/capture 총8을 지키고 공유 target·원본/PDF·다른 작업을 보존한다.

## 직접 읽고 기록한 범위

record_integration_batch30.py와31.py → integration-semantics-batch-30/31.json, non-numeric-helper-semantics-21/22.json.

- 6023 halfwidth 115줄,6172 PUA 77줄,6699 terminal tracking88줄/inline pictures210줄,6708 baseline195줄,6712 row units658줄 전체: test33/helper35.
- 7642 forced break190줄/ingest content407줄 전체: test22/helper11.
- 합계8파일/test55/helper46=101함수. functions.json 전수 목록·source SHA·판정·원장 연결을 대조했다. phase17-audit-record-validation.json.
- 기존 부분 분류를 읽고 갱신했다. 특히6708 helper48의 source baseline/그림 ascent 독립 근거 pending은 없애지 않았다.6712 p340의5297HU 출처는 원본 바이트로 확인했다.
- 6699 절대 핀 x128.70/123.10/334.73,외부 그림 x/y±0.06,표 폭430.9/590.2 고정 선택,큰 배경600×800 선택을 보정 대상으로 기록했다.334.73은 주석에서도 현재 엔진 잔차다. 단순±2 변경으로 해결하지 않는다.
- 6023의 glyphs는 x/y만 읽고 변환 글자를 건너뛸 수 있다. 원본 반각/다음 음절 소유는 pending이다.6172의 em 관계는 절대 위치가 아니지만 독립 글꼴 전진과 현재 전체 시각은 pending이다.
- 6708의66쪽/작성한3조건 font 변형과 과거3쪽 통제 Print의 연결은 pending이다. source 단위/같은 줄 관계를 무조건 px 핀으로 취급하지 않는다.
- 6712의 visible_svg_lines는 y/rect clip만 읽는다. 실제 모든 변환 글자의 가시성까지 입증했다고 쓰지 않는다. 정상 원본 HU/내용/소유 관계는 유지한다.
- forced break의 EPSILON 공통 축은 산술 관계이고 tac vec0/0/2/2/4는 문자 사이 컨트롤 위치다. ingest의 같은 프로젝트 serializer/parser 왕복을 독립 한컴 oracle로 부르지 않는다.

## 독립 CFB 단위

inspect_phase17_raw_units.py를 parser-venv Python으로 실행했다. phase17-independent-raw-units.json.

- table-in-tbox input SHA02b0ced8b96cec5b6e1cad6dd1926b701a2875ea9591c589e57edf7653daf4af.
- DocInfo CharShape20의 원본 base1600HU,7개 언어 slot spacing10%. 대상 충남중부권지사장은8글자/내부 간격7개다.
- 가정통신문 외곽 table의 직접 LIST_HEADER 셀3은 row2,col0이다. Korean71개/Chinese66개 직접 문단이며 중첩 내부 문단을 섞지 않았다.
- Chinese p35..38의 vpos56079/57839/59599/61359HU는1760HU 간격이다.
- Korean p41/42의 column_start5297HU, p43은0이다.
- 두 셀 padding850HU 네 면, margin flag1281로 지정 bit를 확인했다.
- rhwp 모델을 거치지 않은 CFB 압축 스트림/레코드 읽기다. 원본 단위 확인을 실제 출력 전체 수용으로 승격하지 않는다.

## 새 전쪽 시각 실행

run_phase17_current_layout.py, 로그 phase17-current-layout-driver.log, O=phase17-current-layout-evidence.
exec38984/PID88307은 terminal exit0,87.391106초로 끝났다. 중복 시작하지 않는다.

기존 DocumentCore::from_bytes 기본과 같은 compat2022, canonical PDFium5.13/96CSSdpi/2px raw, Native Skia/fresh WASM PrintCanvas PNG, capture8 배타 lock을 사용했다. frozen Native/fresh WASM은 코드 무변경을 확인한 기존 빌드 재사용이다. Mac web/pkg/no-opt 대체 빌드이며 Docker 최적화가 아니다.

| 원본 | Native1/2쪽 raw | WASM1/2쪽 raw |
| --- | --- | --- |
| table-in-tbox.hwp |92.05166/92.55344 |92.02897/92.59857 |
| 한국어 가정통신문 |98.57113/93.68608 |98.99134/93.89238 |
| 중국어 가정통신문 |80.87359/93.76267 |81.26138/93.92760 |

각 backend6쪽/총12쪽, 세 원본 각각2/2쪽·누락/추가0이다. Chinese1쪽은 capture와 silhouette TSV 모두exit1/re_review_required로 보류했다. driver 자체 exit0은 모든 시도 수집 완료일 뿐 시각 PASS가 아니다. state.visual_acceptance_verified=false는 미완료를 숨기지 않는 값이다. 자세한 판정은 existing-pins-qualification.json이다.

직접 판독:
- 양쪽12개 review(각각 overlay 패널 포함)를 전부 읽었다. standalone Native table p1/Chinese p1/Korean p2도 읽었다.
- make_phase17_details.py로 원본 PNG를 보존한2배 NEAREST 상세9개를 만들고 모두 직접 읽었다. detail/producer.json에 crop/원본/결과 SHA를 남겼다. 넓은3패널만 축소해서 본 것을 확대 판독으로 주장하지 않는다.
- table-in-tbox1쪽 외곽선이 rhwp에서는 마지막 안내 뒤에서 닫혀 로고가 밖에 있지만 PDF는 로고 아래까지 이어진다. raw90%로 시각 완료를 선언하지 않는다. 두 쪽 모두98% 미만이라 절대 핀 자격이 없다.
- Korean p1만 양쪽98% 수치 자격이 있지만 독립 좌표/주요 동작의 최종 px 자격은 새로 승인하지 않았다.2쪽의 그림/표/본문 미세 위치·글리프 차이는 남는다.
- Chinese는 제목 자형/굵기와 내용/표/그림 위치 차이가 남는다. 눈병 제목은 그림 옆에 있고1쪽 마지막 예방수칙4줄·손씻기 그림·2쪽footer가 보인다. 원 이슈의3→2쪽 흐름 결함 재발을 단정하지 않는다. 현재 미달과 기존 올바른 관계 검사를 구분한다. 미달 때문에 정상 검사를 일괄 제거하지 않는다.

입력/PDF SHA는 state와 qualification에 있다. PDF는 table-in-tbox-hwp-2020.pdf/Korean-2020.pdf/Chinese-2024.pdf의 기존 독립 자료를 재사용했다. 현재 이슈6712 종료 댓글에서 원본 저장 제품2020/2024와 두 기준의 대응을 다시 확인했다.

## 원 이슈 계보

6699 전체 본문/11개 댓글과6712 전체 본문/6개 댓글을 읽었다. 초기 표 폭·그림만 이동·로고 중복 폭 가설과 후속 정정을 구분한다. 6699는6813 부분 보정→6818 마지막 자간 잔여 종료,6712는6813에서 그림 옆 흐름/쪽 밀림/말미 소실·제목 겹침 종료다. 당시 pixel-perfect 종료가 아니었고 과거 pixel/proxy는 현재 raw와 달라 직접 하락량으로 빼지 않는다. merge6a193a648/07bc5e549의 현재 조상을 확인했다. phase17-original-issue-history.json과 기존 작업 보고서/PR archive를 읽었다. 실제 코드 변경 전에는 필요한 PR diff/후속 commit의 나머지도 따라간다.

## 기존 실제 Native33개

run_phase17_existing_tests.py가 nextest 실제 등록 목록에서6모듈/33개를 계산해 실행했다. suite004/008/010/016/019를 사용했다. exec53040/PID17370은 terminal exit0,컴파일 포함31.335858초다. Summary3.574초33PASS/0FAIL,선택 target1285미실행이다. phase17-existing-tests.log/producer.json. 새 검사 FAIL→PASS·전체 회귀·전체 지침 적합이 아니다. Cargo는 종료됐다.

## 원장/목록/이슈

- 최신10227그룹/2103파일,unreviewed4613,direct4829,evidence_pending446,semantic_notes127,file_context212.
- 직접 판정5275개가 원장에 모두 연결되고 누락0이다. source/full conformance false와 call closure pending을 적합/완료로 오인하지 않는다.
- paired catalog는96조건/각 backend2505쪽/입력+PDF189파일. 새3조건을 미달까지 포함해 기록했다. numeric 조건은 최종 자격이 아니다.
- catalog를 처음 갱신할 때 exit0만 요구해 Chinese 보류 결과에 assertion이 났고, 이어 보조 manifest의 not_evaluated를 passed로 가정한 오류도 검출됐다. 실제 capture summary와 auxiliary gate를 분리하고 완료/쪽수/누락/문턱90/보류쪽/exit1을 검증해 정정했다. 재캡처하거나 guard/문턱을 바꾸지 않았다. 이 과정의 도구 오류를 시각 실패 해소로 보고하지 않는다.
- #8232 본문/댓글 https://github.com/edwardkim/rhwp/issues/8232#issuecomment-6403555581 갱신 후 원격과 로컬 본문을 정확히 대조했다. phase17-final-issue-body.md/phase17-survey-report.md/phase17-published-receipt.json.
- 마지막 프로세스 확인에서 Cargo/rustc/driver/capture가 없고 tracked clean이었다. 다음 긴 실행 전 다시 실제 상태를 확인한다.

## 다음 작업

4613 미분류와 독립 pending/동적 helper·fixture 소비를 계속 조사한다. 이번8파일과 전쪽 캡처를 다시 처음부터 중복 실행하지 않는다. phase16/15 이전 체크포인트의 실제PUA셀19/22 오류, MCP response owner, Hyperlink 드롭, relative size, Recipe, qualified px 보정 등도 여전히 source미수정이다. 전수 조사 후 실제 보정·issue_8232 새 명명/등록/호출 갱신·필수 검증까지 마쳐야 완료다. 전역 memory 검색은 hits0이어서 관련 memory를 판단 근거로 쓰지 않았다.
