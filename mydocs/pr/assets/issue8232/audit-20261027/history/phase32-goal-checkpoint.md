---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase32-goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase32-goal-checkpoint.md` · 원문 SHA-256: `3d7af8a62438c9ab16200d2a180a6c377c287779f5c6b9eb28dd13fb60b7c91f`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 32차

## 목표·권한·현재 작업장

전수 조사·독립 판정 → #8232 반영 → 부적합 검사와 필요한 공통 구현 보정 → 실제 보정 파일·함수를 `issue_8232`로 명명하고 helper/module/suite manifest/생성/스크립트/E2E 호출 갱신 → 실제 필수 검증까지 완료한다. 기존의 잘못된 판정은 유지할 의무가 없고, 독립 근거가 있는 검증 목표는 재구성한다. 정상 검사와 원본 단위를 일괄 삭제/개명하지 않으며 옛 이름의 wrapper·중복 등록은 만들지 않는다. PR은 나중이다. 현재 push/PR 승인은 없고 #8232 업데이트·필요한 MCP Print는 승인됐다. agent/task 메시지·예약·메모리 쓰기는 금지돼 있다.

R=`/Users/tsjang/rhwp`, branch=`audit/8232-px-regressions-20261025`, HEAD/base=`238de6de41a208648f4ade6b494fe433a81c1fce`. Git clean. 제품·검사 수정 0건, 개명 0건. 모든 임시 자료는 ignored `output/issue8232-audit/`에 있다. jobs 8/threads 8/캡처 합계 8/Cargo 순차, 원본·PDF·공유 target·다른 작업장 보존을 유지한다. 이번은 Node 단위 실행과 메모리 반례이며 Cargo·새 WASM·브라우저·CanvasKit surface·새 시각 캡처·MCP Print는 실행하지 않았다. 목표는 ACTIVE다.

직전 goal turn은 70함수 판정·Native 41개 실행·독립 source 대조·이슈 갱신이라는 실제 진전이었다. 이번에도 121함수 판정·Node 65개·새 검출력 반례·원 PR 추적·이슈 반영으로 실제 진전이 있다. 전체 감사·보정·개명은 완료하지 않았다.

## 이번 직접 판정

`rhwp-studio/tests/render-backend.test.ts` 1,325줄 전체와 직접 관련 TS 8파일의 함수 body를 읽었다. 실제 Node test callback 65개, 중첩 callback 11개, 공용/생산 helper 45개로 합계 121함수를 중복 없이 추가했다. record_studio_helper_batch38.py 및 non-numeric-helper-semantics-38.json이 함수별 정본이다. 최초 인덱스의 JS closure/is_test 값만으로 실제 test 등록 수를 추정하지 않고 TAP의 실행 제목 65개와 대조했다.

공용 TS 8파일은 image-replay.ts(8함수), image-crop-scale.ts(7), source-guard.ts(8), clip-replay.ts(2), replay-plane.ts(3), diagnostics.ts(1), raw-svg-prefetch.ts(4), render-backend.ts(12)다. 실제 기존 검사가 모든 helper 분기를 실행했다는 뜻은 아니다. 전체 glyph-outline status 모듈 등의 추가 의존은 실행/hash로 연결했지만 읽지 않은 함수의 직접 판정으로 세지 않는다.

## 실제 Node 실행과 독립 산술

run_phase32_studio_tests.py는 Node v24.15.0에서 현재 backend test 하나를 실행했다. phase32-studio-tests-producer.json/log: 65 PASS / 0 FAIL / 0 skipped, TAP duration 226.586125ms, wrapper 0.282280834초. 실행 import와 source guard 입력의 전후 SHA가 같다. 실제 browser·WASM·CanvasKit surface·전체 문서 검증이 아니다.

probe_phase32_image_units.mjs와 inspect_phase32_unit_contracts.py는 작성 bbox·영상 치수·crop을 production helper 결과와 독립 유리수로 대조했다. contain의 정확한 폭은 165.1325991711, x는 510.7337004144다. 기존 근사 기대값 165.16/510.72와 각각 약 0.0274/0.0137 차이로 기존 0.05 이내다. 입력 원점을 이동하면 결과 원점도 같은 만큼 이동하며 크기는 같다. 이 검사는 실제 HWP로부터 계산한 셀 원점을 고정하지 않는다. 단위 계산의 통과를 실물 페이지 98% 자격으로 인용하지 않는다.

crop은 75HU/원본 pixel, 한 축의 확인된 등방 scale, 양축 crop의 source 창 보존을 확인했다. 작성 배너 첫 높이는 70행, 두 번째는 58.1951188986행이다. PNG 2개는 모두 실제 2×1 영상이며 pixel/전체 bytes가 다르지만 base64 FNV32가 7f1d283a로 충돌한다. 기존 production BLAKE3 cache key는 둘을 구별한다. PNG bytes의 SHA와 base64 문자열의 BLAKE3는 다른 대상을 해시하므로 구분한다.

실제 HWP 원본/PDF의 90/98% 또는 별도 실제 Canvas2D/CanvasKit paint 수용을 이번 산술로 대신하지 않는다. 작성 raw SVG 포장·3자리 cache key 직렬화는 과거 실물 페이지 전체 SVG golden과 구분한다. pixel/entry/dimension 예산과 timer도 배치 좌표 허용폭이 아니다.

## profile 가드의 새 반례와 원 PR

기존 root는 render-backend.test.ts:274의 `PageLayerTree bridge verifies the returned profile instead of relabeling it`다. 원래 production getPageLayerTreeObject 메서드는 print 요청에 screen tree가 오면 거절한다.

probe_phase32_profile_guard.mjs는 정확히 한 `if (tree.profile !== profile)` 분기를 메모리에서 false로 바꾸고 원 조건 문자열만 주석에 남겼다. 같은 전체 production 메서드를 추출·TypeScript strip하여 명시 transport mock으로 실행했다. 정상 사본은 mismatch를 거절하지만 변형 사본은 screen을 그대로 반환한다. 정상 profile 요청은 양쪽 모두 정상이다. 이어서 fs.readFileSync의 정확한 wasm-bridge.ts source 읽기 한 곳에만 메모리 사본을 전달하고 기존 정식 root를 그대로 실행했다. 정상/변형이 각각 1 PASS / 0 FAIL이었다. 원본 제품·test source는 변경하지 않았고 전후 SHA도 같다. 이 반례는 한 source 문자열 guard의 검출 누락이며 제품 원본의 실제 오류나 새 정식 회귀 FAIL→PASS가 아니다. 전체 WASM transport를 실행한 것도 아니다.

원 도입 commit bd9c15b5d9aff232d6bd250ddb3716a1755797a9 → PR #2297 최종 head 7998e4f7d655e89a8270cce3ad19173a59a24550 → 실제 merge 45cd4113ed4d511c20fd670447e991b4efb2fd45 → 현재 HEAD를 확인했다. 도입 commit은 PR head의 조상이고 도입/merge 모두 현재 조상이다. 실제 merge tree에도 같은 source guard가 있다. phase32-pr2297.json과 phase32-profile-origin-chain.json을 따른다. 이 항목은 보정 대상으로 원장에 기록했다. 이후 이름/주석 존재 대신 정상 profile 보존과 실제 mismatch 거절을 실행 검사로 재구성해야 한다.

## 원장·제목 대조·원격 이슈

직접 판정 6,688개/740소스 파일, 원장 10,980그룹/2,116파일이다. 미분류 3,962개와 메모만 있는 330개를 합친 분류 미확정은 4,292개다. 독립 근거 pending 508개는 직접 판정 기록 안에 있으므로 미분류에 단순 합산하지 않는다. 8개 helper 파일의 새 연결로 원장 파일 수가 증가했으며 이를 결함이나 PASS 증가로 세지 않는다. 전체 직접 기록의 현재 source SHA와 원장 연결을 validate_phase32_ledger.py로 확인했다.

처음 단순 TAP 이름 비교는 #를 포함한 4개 제목의 이스케이프 때문에 중단됐다. 원래 제목에 역슬래시가 없음을 확인한 뒤 해당 표기만 해석해 65개 전체와 정확히 대조했다. phase32-semantic-ledger-validation.json/log에 정정 이유와 원래/해석된 제목을 보존했다. 제품·검사 실패가 아니며 끝난 65개를 다시 실행하지 않았다.

#8232 본문에 이번 결과 5개 bullet을 추가하고 API 재조회에서 말미 개행 포함 byte 동일을 확인했다. 상태는 OPEN이며 updated_at·본문 SHA는 phase32-remote-verification.json을 따른다. 완료 checkbox·close·PR 생성은 하지 않았다. 제품·검사 보정과 개명은 여전히 0건이다.

## 다음 조사

ai-write-tools.test.ts와 support/overwrite-mode.runner.mjs는 초기에 경로·인덱스·길이만 확인했으며 전체 직접 판정/실행하지 않았다. 이들을 완료 파일로 세지 않는다. 나머지 미분류와 source cfg·전체 tests·Studio E2E·동적/생성/공용 경로 및 508건의 독립 근거 pending을 계속 조사한다. 기존 phase30/31의 pending과 원본·PDF 근거는 각 checkpoint를 따른다. 특히 암호 HWP3 p3의 vpos 4개·배경 차이와 전 24쪽/WASM 자격은 아직 미해결이다.

끝난 이번 65개, 분석용 기존 root의 정상/변형 1개씩, 이전 41·37·60개를 자동 재시작하지 않는다. 실제 보정·issue_8232 개명·호출/등록 갱신과 최종 검증까지 완료해야 전체 목표가 끝난다.
