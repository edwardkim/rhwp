---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase90-gym-release-diff/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase90-gym-release-diff/goal-checkpoint.md` · 원문 SHA-256: `c759dfd617d17d0836322013c00c1572ba25e7285e4c6a5f2a9e384dced99fbb`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 90차 체크포인트

전체 목표는 ACTIVE이며 미완료다. 전수 조사와 독립 판정을 마친 뒤 이슈를 최종 갱신하고, 현행 지침과 다른 기존 검사를 실제 출력 보정/필수 시각 조건에 맞게 변경한다. 실제 보정한 파일·함수를 issue_8232로 명명하고 helper/module/suite/manifest/생성/스크립트/E2E 참조도 갱신한다. 정상 검출 목적은 보존한다. 제품·검사 수정과 개명은 아직 0건이다. PR은 나중이며 원격 push/PR 승인은 없다. 다른 task/subagent/예약/메모리 쓰기는 하지 않는다.

작업장은 /Users/tsjang/rhwp, 브랜치는 audit/8232-px-regressions-20261025, HEAD는 01ced3d42724285c8f9765a453393ab6af326617다. 기존 dirty rhwp-studio/public/rhwp.js의 SHA c1c8d1820cc4375ab668427fa0a42c87d71c3d0d83955f4d300880559023ddc6를 보존했다. 감사가 만든 변경이 아니므로 revert/stash/commit하지 않는다. 요청 없이 fetch/rebase하지 않는다. 원본/PDF/공유 target/다른 작업장/기존 Studio 서버를 보존한다. 이번 Cargo/prepare/추가 빌드는 모두 0회다.

정본은 active-audit.json → phase90-gym-release-diff/audit-ledger.json과 scope-summary.json이다. 현재 21,407그룹 / 2,501파일, 직접 9,971함수 / Rust 6,469검사 / 836파일이다. 미분류 11,167그룹 + 의미 메모 95그룹 + 파일 문맥 165그룹 = 11,427그룹은 추가 판정이 필요하다. 직접 판정 안의 근거 pending 511개도 미완료다. 보정 필요로 분류한 87개는 구현 완료 수가 아니다. AST/원장 수를 실제 실행 testcase 수로 부르지 않는다.

80차의 초기 검색 누락 연결과 사용자 제외는 유지한다. Rust 2,887개·스크립트 5,630개, 총 8,517개 기존 AST 후보를 미분류로 추가했으며 workspace 명시 제외 vendor 6,496개는 별도 보존했다. 삭제 Rust 3검사(원장 4그룹), 신규 Rust 95검사/48파일, 새 JS callback 1(schema 2건)과 관련 helper는 감사/PASS에 넣지 않는다. is_test=false callback·매크로·동적 순회·생성 경로·feature별 실제 등록 대조도 완료 전이다. 세부 범위와 이전 실제 발견사항은 phase80 및 phase89-renderer-equation-ast/goal-checkpoint.md를 따른다.

90차는 scripts/tests/test_gym_release_diff.py 전체 1,630줄, 207함수(검사 169개·helper 38개)를 신규 판정했다. 실제 함수 원문, 26개 TestCase 클래스의 목적, 중첩 fake/boom/loader 및 setUp/_row/_task, 전역 load_rd/_value와 작성 동등성 18행·분류 12행을 읽었다. 작성 pageCount 6/7, 논리 table/row/col, 종료 코드, timeout 30초, 오류 head 80글자, SHA 64자리, 요약 30행/실제 차이 40개를 실물 페이지 절대 px 기대값과 구분했다. raw 관측 동일성·분류·보고·예외 생산 경로와 checks.dig/find_cell·runner.resolve_args의 실제 소비도 확인했다. 이 도구는 한컴 정답/시각 gate가 아니라 동작 변화 관측기다.

기존 Python 검사 169개를 한 번 실행했다. 실제 요약은 Ran 169 tests in 0.698s; OK (skipped=1)이며 168 PASS / 0 FAIL / 1 skip이다. test_self_diff_report_is_stable은 저장소의 gym/release-diff.json이 없어 기존 코드대로 명시적으로 건너뛰었다. 이 root를 PASS로 세거나 파일을 만들어 우회하지 않았다. 실제 제품 CLI/probe/pack 비교는 mock이고 상대 경로 산출물은 phase90-gym-release-diff/work 아래로 제한했다. 실제 HWP 렌더링·브라우저·새 바이너리 빌드·전체 Gym pack 실행은 하지 않았다.

검사 자체가 끝난 뒤 감사 driver가 result.skipped 안의 TestCase 객체를 JSON으로 직렬화하지 못해 wrapper exit 1이었다. 원본 driver는 driver-before-skip-json-fix.py, 오류는 ../phase90-run.log, 성공/skip 원본은 existing-unittest.log에 보존했다. 직렬화 코드를 test.id()/reason 문자열로 고쳤고 recover_completed_result.py는 테스트를 재실행하지 않고 실제 등록 169개와 원 로그의 성공 168개/skip 1개 ID를 정확히 대조했다. 복구/수정은 감사 output에만 적용했다. producer.json은 실제 wrapper exit 1과 unittest 성공을 분리해 기록한다.

원 driver의 source 전후 hash 동일성 assert는 통과한 뒤 JSON 저장에서 실패했다. 당시 hash 값은 파일로 저장되지 않았으므로 복구 시점의 현재 source/HEAD 일치 값을 저장했고 사전 저장 hash인 것처럼 주장하지 않았다. 현재 source는 모두 HEAD bytes와 같고 원본 log/driver SHA도 기록했다. test 재실행/빌드는 0회다. 중단/관측 timeout 때문에 같은 검사를 새로 시작한 것이 아니다.

독립 작성 정수 진단에서는 old=2^53, new=2^53+1이 정수 산술/JSON에서는 구별되지만 release_diff의 일반 숫자 비교가 float으로 변환해 observations_equal=true, diff_task=[]가 됨을 확인했다. integer-observation-counterexample.json과 probe_integer_observation.py가 증거다. 실제 사용자 문서의 값이 손실됐다는 증거/정식 회귀 FAIL/px 지침 위반으로 단정하지 않는다. 기존 18개 동등성 표를 모든 정수 정확성의 증거로 확대하지 않도록 별도 검출 한계로 보존했다. 이 발견으로 제품/검사를 수정하거나 목표를 일반 비교기 개선으로 확장하지 않았다.

원 source 4개 커밋의 GitHub 연관 PR을 읽기 전용으로 확인했다. 기능/누락 제출물 관측은 통합 PR 4672, 순수 분류/예외 시험 보강은 통합 PR 5425에 연결된다. pr-4672.json/pr-5425.json의 목적과 과거 검증 범위를 읽고 원 source 조상 관계를 확인했다. 당시 100과제/122관측 또는 Python 1,737개 통과를 현재 PASS로 재사용하지 않는다. 커밋-PR 연관은 commit-pr-associations.json, 함수별 source는 function-origin-blame.json에 있다. 외부 게시/원격 소스 변경은 0건이다.

이번 목표 turn의 신규 판정은 207함수 / Python roots 169개 / helpers 38개다. 실제 실행은 Python 168 PASS / 0 FAIL / 1 skip, 0.698초다. 테이블 18행·12행을 별도 testcase 수로 더하지 않았다. source/검사 수정, 개명, Rust 실행, 제품 CLI 렌더링, 실물 캡처, Cargo/prepare/추가 빌드는 모두 0건이다. 직전 turn의 49함수 신규 판정과 pending 12개 재검토는 이번 수치와 중복하지 않는다.

87차의 미주 간격 원본 XML 1984HU와 경험 완충 pending 12개, 88~89차의 EqEdit/렌더러 AST 판정, 이전 실제 부적합 핀/시각 미달/검출 한계는 모두 유지했다. source 단위를 확인한 것과 추가 경험 완충/올바른 실제 위치를 입증한 것은 다르다. pending 511개를 적합/완료로 승격하지 않았다. 실제 보정은 전수/독립 판정과 필수 시각 조건을 갖춘 뒤 진행한다.

#8232 원격 본문은 마지막으로 80차 게시 당시 exact readback을 확인했다(updated_at 2026-10-26T14:23:00Z, UTF16 65,534). 이는 현재 live issue가 그 뒤 변경되지 않았다는 주장이 아니다. 81~90차는 로컬 정본에 기록했고 이번 게시 0건이다. 전수 완료 뒤 이슈를 최종 갱신하라는 지시를 유지한다. 다음 갱신은 fresh GET·UTF16 한도·다른 작업/실제 실패 근거 보존·exact readback을 따른다. 실제 실패와 보정 필요 항목을 삭제해 정상 요약만 남기지 않는다.

다음은 나머지 source cfg(test)·/tests와 scripts/tools/bindings의 미분류, helper/생성 자료/동적 호출을 계속 점검한다. test_gym_release_diff.py의 현재 207함수는 완료했으므로 같은 빌드나 단위 실행을 반복하지 않는다. 다른 Gym 도구의 score/trajectory/baseline/게이트 수치가 실제 화면 px인지 단위/독립 기준/실제 소비를 읽어 구분한다. renderer의 고정 영역·좌표·SVG hash는 독립 Print 및 현행 페이지 자격과 대조한다. 전수 조사·독립 판정·실제 보정·issue_8232 개명/참조 갱신·필수 검증이 끝나지 않았으므로 complete/paused/blocked로 바꾸지 않는다.
