---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase26-goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase26-goal-checkpoint.md` · 원문 SHA-256: `4ecec6fed8b3e7b88c52f74adff5b8d619b5ec08318b419db649ef689ba6d30a`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 26차 체크포인트

목표 ACTIVE, 완료 아님. 이전24~25차와 이번26차 모두 실제 원장·실행·독립 근거·원격 이슈를 갱신한 progress다. blocker/대기 프로세스는 없다. 전수 판정 뒤 실제 부적합 검사 보정·issue_8232 새 명명·helper/module/suite/생성/스크립트/E2E 연결 갱신과 필수 재검증까지 해야 목표 완료다. 기존 잘못된 단언·검출 방식의 무조건 보존을 요구하지 않는다. 정상 원본 단위·관계·구문/저장 계약은 이름 변경만을 위해 훼손하지 않는다.

## 현재 작업장과 권한

R=/Users/tsjang/rhwp, branch audit/8232-px-regressions-20261025, HEAD238de6de41a208648f4ade6b494fe433a81c1fce. 시작/최종 Git clean, diff --check0. 제품/테스트 수정·개명0건이다. 새 자료는 전부 ignored output/issue8232-audit 아래이며 원본/PDF/다른 작업장/공유 target을 보존했다. PR/push는 하지 않는다. subagent/다른 task 메시지/예약/메모리 쓰기 금지를 유지했다. memory quick 검색8232/audit는0건으로 관련 memory를 사용하지 않았다.

## 전체 직접 읽기/판정

- tests/cases/issue_4969_shaping_shadow_context.rs994줄:19검사/4helper.
- tests/cases/issue_4969_shaping_atomic_activation.rs1832줄:13검사/13helper.
- tests/cases/split_caret_axis.rs711줄:11검사/23helper.

합계43검사/40helper=83함수다. batch41/helper32에 모두 연결했다. 최초 원장에는8개 직접 기록이 있었으므로 고유75개가 증가했다. 후속 숫자 근거 보완을 위해 같은 recorder를 재실행했을 때 previous_direct83이 나온 것은 idempotent 보완이며 새83개로 집계하지 않는다.

직접6190개/710소스 파일 모두 원장과 현재 source SHA에 연결되고 누락0이다. 원장은10676그룹/2107파일: unreviewed4156, direct meaning5701, evidence pending489, semantic notes118, file context212. 전체 call closure/full conformance는 아직 미완료다. 숫자를 결함 수/PASS 수로 표현하지 않는다.

shadow의 font10px/width0.8·UPEM 정수 advance·cluster·slot/cache/generation·상한/typed rejection과 authored source instance는 페이지 절대 위치 핀과 다르다. source 문자열만 보는 staging red guard는 실제 행동 검증으로 확대하지 않는다. atomic의 같은 호출 clear/default/rollback/ABA JSON/해시 동등은 historical whole SVG golden이 아니다. 성능 출력에 기록만 한 hash와 기대 literal을 구분했다.

caret의 도형/그림 제작 HU·16px image size와 logical index는 화면 위치 답안이 아니다. 작성 anchor에서 float0/inline·각주1/번호 placeholder1칸을 독립 순회하고 본문/셀/머리말/각주·3storage·Enter/Page/Column/merge/paste/copy를 대조한다. 공용 check_split은 head의 객체 종류/위치는 엄격히 보지만 tail은 개수만 확인한다. 실제 원본 End test의 생산 is_logical_inline 사용을 그 판별 자체의 독립 oracle로 표현하지 않는다.

## 실제 Native 실행

run_phase26_existing_tests.py session4979 terminal exit0. 현재 Native/native-skia metadata에서 generated024/026과 standalone shadow target을 확인했다. 직접43개 정의 중 기존 성능 ignored2개를 제외한41PASS/0FAIL, Summary0.180초, 선택target473skipped다. metadata build11.20초. test binary 경로/전후SHA가 같고 포함 source kerning/shaping/context3개의 SHA를 고정했다.

shadow의 직접 path source-module include와 include_str 구조 guard를 구분했다. 이번 선택에서 별도 포함 source test 등록은0이다. 실제 수집을 확인했으며 포함 source 모듈에 test가 있을 거라는 예상으로 수를 늘리지 않았다. 기존ignore2개는 실행하지 않았다. Native에서 수행한 CanvasKit/Native/SVG 등의 typed variant/plan 분석을 실제 browser/각 backend 재생으로 주장하지 않는다. 새WASM/capture/MCP/전체회귀는 실행하지 않았다. 기존paired catalog116조건/각backend2591쪽을 새26차 통과로 인용하지 않는다.

## 독립 font-unit 대조

inspect_phase26_font_contracts.py는 C HarfBuzz14.4.0 `/opt/homebrew/bin/hb-shape`, shaper=ot/font-size=upem/direction ltr/script/language/features/variations를 고정하고 fontTools 원본 UPEM/outline/metrics와15조건을 대조했다. source3개는 NotoSansKR-Regular.ttf, RHWPOldHangul-Regular.otf, HappinessSansVF.ttf이다. 파일/도구 SHA와 버전을 JSON에 고정했다. font binary를 수정/배포/커밋하지 않았다.

- Noto office liga off/on20.504/20.488px, glyph80/11819/68/70·UTF8cluster0,1,4,5 및 liga폭7.344px.
- Noto AV kern off/on9.464/9.320px, delta-0.144px.
- 옛한글 ᄒᆞᆫ글 cluster0/9·15.456px.
- 가변5instance×한글/Latin10조건 중9개가 C의UPEM 정수결과와 직접 같다. Typography650/400 한 조건은 C44.456과기존Rust44.448이 다르다.

최초 script는 C와Rust의15조건 모두 같다고 가정해 위 한 조건에서 assertion이 실패했다. 기대값/허용폭을 바꾸지 않고 최초script 및 독립 관측을 보존했다. script-first-comparison.py.txt와 phase26-variable-hb-unit-scale-observations.json을 참고한다. 이후15개 실제관측 중14직접일치/1엔진변환차이를 구분한다. 해당 차이를 숨겨15개 C일치라고 하지 않는다.

## 가변 한 조건의 실제 근거

inspect_phase26_variation_rounding.py / phase26-variation-rounding-explanation.json:

원본GDEF/GPOS lookup1의 실제T/y class pair base XAdvance=-65, variationIndex49의delta=-2.5, HVAR instanced Tadvance650을 독립 FontTools로 읽었다. normalized wght=.5/opsz=0이다. 현재 locked Rustybuzz0.20.1의 DeviceExt::get_x_delta는 variation delta에 f32.round를 적용하므로 -2.5→-3, T=650-65-3=582다. C HarfBuzz14.4.0의1000scale은T583,1000000scale은582500(582.5units)다. Crun5557, 고정밀5556.5와 달리 현재Rustrun5556→10×0.8/UPEM=44.448이며 실제NativePASS와 같다.

이것은 backend의 font-unit 반올림 계약이다. 현재 검사 오류/페이지px 배치 회귀를 뜻하지 않는다. matrix182의 수치 pending은 이 독립 원본/locked dependency 근거로 해소했지만 페이지 원점 자격/전체 호출 폐쇄는 미완료다. 같은 shape 전체 glyph들의 C/Rust 완전 일치나 물리 인쇄 위치를 주장하지 않는다.

진단 중 FontTools glyphset width만 읽은 값은 CFF glyph폭과 HVAR/GPOS를 구분하지 못하므로 최종 advance oracle에 쓰지 않았다. 최종 근거는 instanced HVAR hmtx·GPOS VarStore이다. 또 첫 script가 lookup1의 subtable0을 ClassDef라고 가정해 AttributeError가 났다. 최초copy(first-subtable.py.txt)를 남기고 실제T coverage/ClassDef의 유일 subtable을 선택해 재실행했다. 이 진단 오류를 제품/정식 검사 FAIL→PASS로 세지 않는다.

## 세로 bbox와 남은 pending

Noto 한/글의 원본 bounds=(52,-58,885,826)/(50,-67,870,788), hmtx advance920, vmtx=(1000,54)/(1000,92)이다. 작성1000HU font size13.3333CSS에서 크기11.1066667×11.7866667/10.9333333×11.4가 기대값과 같다. vertical top 차이는 (1000+92-54)×13.3333/1000=13.84CSSpx로 기대두bbox의 차이와 같다. 이는 원본 glyph 크기/상대 advance의 독립 단위 근거다.

그러나 assert_bbox390/root402의 절대x257.92/y132.986666... 원점은 독립 page/cell origin·현재 Print 자격이 없다. default public blank에 작성1x1 table을 넣었다는 이유만으로 모든 원점을 독립 수학/98% 자격으로 승격하지 않았다. 같은 origin의 이전 Q4-D2 qualified 기록/현재PASS를 현재페이지 검증으로 인용하지 않았다. 절대원점은 pending이며 source-derived glyph크기/관계까지 함께 일괄삭제하거나±2로 완화하지 않는다.

atomic459의 payload2519996B는 원본Noto.len과 같지만 accepted JSON3375713/rejected9161 및 capability/visual-wire 차감은 독립 wire-size receipt인지 layout간접고정인지 추가 판정해야 한다. 현재 JSON size literal을 전체SVG golden과 동일시하거나 자동 적합 처리하지 않았다. stageQ4-D2 전체 보고는 직접 읽었고 실제통합/원해결PR 전체대사는 아직 보정 전 별도로 필요하다.

## 이슈/완료 여부

#8232 OPEN, updated_at2026-10-25T23:33:01Z. phase26-final-issue-body.md와 재조회APIbody를 말미 개행까지 실제byte동일 확인했다. bodySHA77f56fa31d05405a1eb75b524774eea42faf41dd5eaaf8518032cb6c52e8c805이다. phase26-remote-verification.json/phase26-semantic-ledger-validation.json을 참고한다. 완료 체크/close/PR을 하지 않았다.

최종실제 프로세스에 Cargo/nextest/rustc/capture/driver가 없고 Git clean이다. block 없이 다음 미분류4156·pending489와 공용/동적/조건부 소비 경로를 계속 조사한다. 전체 조사 뒤 실제 부적합 검사 수정·issue_8232 개명·등록/호출 갱신·필수 검증을 완료해야 한다. 목표 ACTIVE 유지.

## 다음 직접 읽기 후보

현재 원장의 남은 integration 후보는 tests/issue_1058_textbox_list_header.rs389줄, tests/issue_554.rs521줄, tests/issue_1143.rs524줄, tests/cases/issue_2792_search_nested_tables.rs255줄 등이다. 이번에 전체 읽기/실행한 파일은 아니다. 현재 HEAD/dirty/process/log부터 확인하고 해당 파일의 functions/assertions/등록을 직접 연결한다. 앞선26차41개 검사/15font조건/GPOS 진단을 중복 시작하지 않는다.
