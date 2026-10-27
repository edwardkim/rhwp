---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase92-glyph-bounds-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase92-glyph-bounds-contracts/goal-checkpoint.md` · 원문 SHA-256: `4a3817d9209eb9a8a5d976664f375cd15f16eee9c8fb189650c4d6994d7492ed`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 92차 체크포인트

전체 목표는 ACTIVE이며 미완료다. 전수 조사와 독립 판정을 완료한 뒤 #8232를 최종 갱신하고, 현행 지침과 다른 기존 검사를 필수 시각 조건 및 독립 기대값에 맞게 실제 보정한다. 보정한 파일·함수는 issue_8232로 명명하고 helper/module/suite/manifest/생성/스크립트/E2E 참조를 함께 갱신한다. 정상 검출 목적을 보존하고 이전 이름의 중복 wrapper는 만들지 않는다. 제품·검사 수정과 개명은 아직 0건이다. PR은 나중이며 push/PR 승인은 없다. 다른 task/subagent/예약/메모리 쓰기는 하지 않는다.

작업장은 /Users/tsjang/rhwp, 브랜치는 audit/8232-px-regressions-20261025, HEAD는 01ced3d42724285c8f9765a453393ab6af326617다. 유일한 dirty는 감사 전부터 존재한 rhwp-studio/public/rhwp.js이며 SHA c1c8d1820cc4375ab668427fa0a42c87d71c3d0d83955f4d300880559023ddc6와 97줄 추가를 보존했다. 작성자를 추정하거나 감사 변경으로 표시하지 않는다. 사용자 원본/PDF/다른 작업장/공유 target/Studio 서버를 보존했다. 요청 없는 fetch/rebase, Cargo/prepare/추가 빌드는 모두 0회다. 마지막 실제 프로세스 조회에서 Cargo/rustc/nextest/wasm-pack 및 91~92차 감사 프로세스는 없었다. 검사 session 40531, recorder 31921, 검사 29382, recorder 51268은 실제 exit 0으로 종료했다.

정본은 active-audit.json → phase92-glyph-bounds-contracts/audit-ledger.json 및 scope-summary.json이다. 현재 21,421그룹 / 2,501파일, 직접 판정 10,120함수, 그중 Rust 검사 6,479개 / 837파일이다. 미분류 11,032그룹 + 의미 메모 95그룹 + 파일 문맥 165그룹 = 11,292그룹은 추가 판정이 필요하다. 직접 판정 안의 근거 pending 511개도 미완료다. 보정 필요 87개는 유지했으며 실제 보정 완료 수나 모두 같은 종류의 실물 페이지 핀이라고 부르지 않는다. AST/원장 함수 수와 실제 실행 testcase 수를 구분한다.

80차에서 초기 수치/렌더 검색의 누락 후보 8,517개를 연결한 범위 교정은 유지한다. Rust 2,887개와 scripts/tools/bindings 5,630개를 포함하고 workspace 명시 제외 vendor 6,496개는 별도 보존한다. 사용자 지시에 따른 삭제 Rust 3검사(원장 4그룹), 신규 Rust 95검사/48파일, 신규 JS callback 및 관련 helper는 감사/PASS에 넣지 않았다. is_test=false callback·매크로·동적 순회·생성 경로·feature별 등록 대조도 미완료다. 이전 #8266 및 실물 미달·잘못된 핀의 발견을 삭제하지 않는다. 세부 선행 근거는 90차 goal-checkpoint.md와 그 이전 연결을 따른다.

91차는 scripts/tests/test_gym_build_baseline.py의 전체 127함수(124검사·3helper)를 신규 판정했다. 전체 함수 원문과 build_baseline.py의 자리표/경로/step/파일 생성/답안/채점/집계/CLI 생산 경로, runner.score_task와 eval_check 소비를 읽었다. 작성 자리표 개수·문자열 인덱스·exit code·과제 건수와 실물 문서 px를 구분했다. 같은 pack의 제출 루트를 채점하는 호출 계약, 중간 sub를 submit.files로 승격하지 않는 계약, 생성 성공과 실제 채점을 분리하는 경로를 확인했다. 실제 pack/reference/check의 모든 기대값은 이 파일의 완료로 대신하지 않는다.

91차의 기존 Python 단위 검사는 실제 124 PASS / 0 FAIL / 0 skip, 0.559초, wrapper exit 0이다. score_task/find_bin/process_pack 등은 각 기존 검사 본문의 mock이며 실제 rhwp/한컴 출력이나 전체 Gym 왕복은 실행하지 않았다. 임시 파일은 phase91의 runtime-temp/work 아래에서만 만들었다. 기존 검사로 출력한 X 메시지는 의도된 작성 실패 대조군이며 unittest FAIL이 아니다. source 전후 SHA와 HEAD 바이트 일치 및 실제 등록 ID 124개를 확인했다. producer.json·existing-unittest.log·function-class-map.json·generator-consumer-scope.json이 근거다.

91차 함수 도입은 3d86b5309b544f4aec97e6c978dcef4d9253fb7c와 ccb163ca25f0348b701c2c8c8f5c64f27e1bfc64의 현재 조상 관계를 확인했다. GitHub commit-PR 연관과 기존 보고서에서 통합 PR #5425 및 #6677에 연결했다. #6677 원문은 self-live와 독립 oracle, benchmark와 제품/한컴 정확성을 명시적으로 분리한다. 과거 전수 수치나 #5425의 과거 단위 결과를 현재 PASS로 재사용하지 않았다. legacy main의 failed==0 종료와 엄격 JSON 보고서의 taskCount>0/built 일치/failed·skip 0/모든 row.ok 조건이 다름을 기록했다. source 파일 상단의 과거 세 CLI 옵션 설명과 현재 네 옵션도 구분했다. 이 조사에서 일반 Gym 기능 보정으로 범위를 넓히지 않았다.

92차는 tests/cases/issue_536_glyph_visual_bounds.rs 전체 22함수(10검사·12helper)를 신규 판정했다. 작성 사각형·곡선·affine·COLRv0/COLRv1·shared DAG·조상 clip·page-space bitmap/SVG·NaN/cycle fallback·binary32 반올림을 읽고 현재 paint_bounds 생산 경로와 연결했다. contains의 1e-9는 작성 산술 포함 여유이며 실제 한컴 문서에 대한 ±1e-9px 좌표 잠금이 아니다. contains가 임의로 팽창한 bounds를 모두 검출하지 않으며 fallback 대조가 같은 제품 결과 간 관계라는 한계도 유지했다. 작성 diagnostics의 strict_visual_eligible=true를 실제 페이지의 98% 자격으로 취급하지 않는다.

92차 independent-authored-geometry.json은 제품 함수를 호출하지 않고 사각형 모서리와 두 단계 행렬로 monochrome [96,188,125,208], COLRv0 [116,140,120,152], clip 교집합 [120,236,122,239]를 계산했다. Bezier t=1/2, miter tip 및 binary32의 2^24+1/4097² 반올림·clip의 순차 변환 0..4도 독립 산술로 확인했다. 실제 HWP/PDF의 관측 위치를 복사해 기대값으로 만드는 경우와 구분했다. 이 기록은 실제 backend ink의 최소 영역이나 한컴 피델리티를 인증하지 않는다.

현재 HEAD와 같은 phase62 producer의 regression_suite_024 바이너리 SHA 55a40af0208c83a75202cc1986ab10ca440e6b23e2efa3be998093d3513dd4bd를 실행 전후 확인해 재사용했다. 보유 metadata와 실제 바이너리 --list에서 정확한10개 이름을 대조하고 --exact/threads8로 그10개만 실행했다. 실제 결과 10 PASS / 0 FAIL / 0 ignored, exit 0이다. 새 빌드·전체 suite/nextest·실물 캡처는 실행하지 않았다. source와 실행 파일 hash가 같으며 기존 생성 suite의 module 연결도 확인했다. producer.json·existing-rust-tests.log·current-binary-list.log에 연결한다.

원 PR #7894는 직접 merge되지 않고 #7969로 체리픽 통합됐다. 원 cb6d0861...→현재 abbf4d8a..., 원 1a8711c1...→현재 3ca4e4e9... 매핑은 Git -x 본문과 커밋된 개별 review/적용 기록으로 확인했다. 원 PR의 보수적인 visual envelope·logical 보존·fallback 한계와 현재 테스트 목적이 맞는다. 누적 이력 때문에 commit 조회에 나타난 #7930/#8196을 원 도입 PR로 오인하지 않는다. 과거 83개/10779개 검사 및 과거 페이지 TSV는 현재 PASS로 재사용하지 않았다. 원 PR/통합 PR 원문 snapshot과 origin-contract-map.json을 보존했다.

이번 목표 turn의 신규 판정은 총149함수 = Python 검사124 + Rust 검사10 + helper15다. 실제 실행134 PASS / 0 FAIL / 0 skip이며 새 빌드0, 제품/검사 수정0, 개명0이다. 직접 판정 함수와 독립 testcase 수를 합쳐 세지 않는다. 두 파일에서 새로 발견한 지침 위반 문서 px 핀은 없고 기존 보정 필요87과 pending511은 그대로다. 아직 전수 조사·필수 독립 판정·실제 보정/issue_8232 개명·참조 갱신·필수 검증이 남아 있으므로 완료로 표시하지 않는다.

90차 wrapper의 집계 JSON 오류는 기존 테스트168PASS/1skip과 구분한 채 유지한다. 당시 원 driver exit1과 재실행 없는 결과 복구 근거를 현재 두 실행의 exit0으로 덮어쓰지 않는다. 이전 실제 검출 한계 및 미달 페이지·근거 부족 항목도 90차 및 선행 원장에 보존했다.

#8232 본문은 80차 게시 당시 exact readback(updated_at 2026-10-26T14:23:00Z, UTF16 65,534)이 마지막 확인이다. 이는 현재 live 본문이 그 뒤 바뀌지 않았다는 뜻이 아니다. 81~92차는 로컬 기록만 추가했고 이번 원격 게시0건이다. 전수 조사 후 최종 이슈 갱신 지시를 유지한다. 게시 시 fresh GET·UTF16 한도·다른 작업 및 실제 발견 근거 보존·exact readback을 따른다.

다음은 남아 있는 renderer/image/geometry 검사와 scripts/tools/bindings의 미분류 및 helper/생성/동적 참조를 계속 판정한다. 동일 HEAD의 검증된 기존 바이너리를 재사용하고 단순 감사 때문에 Cargo/prepare를 반복하지 않는다. 모든 완료 요건을 실제로 충족하기 전 goal을 complete/paused/blocked로 바꾸지 않는다.
