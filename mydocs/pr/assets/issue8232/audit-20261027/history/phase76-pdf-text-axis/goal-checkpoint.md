---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase76-pdf-text-axis/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase76-pdf-text-axis/goal-checkpoint.md` · 원문 SHA-256: `b35a88fc685ad6b592c4da6231a2842966843a2ff8eac5aa835bf79e62254065`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 76차 체크포인트

목표 ACTIVE다. 전수 조사와 독립 판정 후 부적합 검사·필요한 구현을 보정하고, 실제 보정한 파일·함수를 issue_8232로 개명하며 helper/module/suite/manifest/생성/스크립트/E2E 참조를 갱신한다. 정상 목적·검사는 보존하고 옛 이름 wrapper를 중복 등록하지 않는다. 제품·검사 수정0, 개명0이다. PR은 나중이며 push/PR 승인은 없다. 다른 task·subagent·예약·메모리 쓰기는 하지 않는다.

R=/Users/tsjang/rhwp, branch=audit/8232-px-regressions-20261025, HEAD=01ced3d42724285c8f9765a453393ab6af326617이다. 기존 dirty rhwp-studio/public/rhwp.js의 SHA c1c8d1820cc4375ab668427fa0a42c87d71c3d0d83955f4d300880559023ddc6를 보존한다. 원본·PDF·공유 target·다른 작업장·기존 Studio 서버를 보존한다. 요청 없이 fetch/rebase하지 않는다. 이전 전체 조건과 실제 발견 사항은 phase75-pdf-boundary-relations/goal-checkpoint.md 및 그 대장에 연결된다.

최신 대장은 phase76-pdf-text-axis/audit-ledger.json이다.12732그룹/2139파일, 직접9355함수/Rust6286검사/826파일이다. 의미 판정8844·file-scope8·근거 pending511·미분류3109+의미메모95+문맥165=3369다. pending은 직접 판정에 포함된다. 사용자 제외 삭제Rust3검사(4그룹)·신규Rust95검사/48파일·신규JS callback1개(schema2건)와 helper는 감사/PASS에 넣지 않았다. 일반 암호 E2E 유지·새 sample16 caller 제외는 같다.

이번 범위는 PDF 본문/들여쓰기/soft-wrap·inferred/shared/source-space tabs·text clip/owned lists/placement·edit baselines의10개 파일 전체2811줄이다.115함수/30검사/85helper를 읽고 신규104함수/29검사·초기 원장 밖83그룹을 추가했다. 기존 pending4개를 재검토하고 정상7개는 재독만 했다. 독립 PDF의 H10pt·2의6pt·상향4pt에서0.6em/-0.4em을 확인해 script pending1개를 해소했다. 두 exercise helper의 고정 source 목적지를 호출하는 신규root6개에도 같은 자격 pending을 연결했다. pending506→511은 새 runtime FAIL이 아니라 의존성 추적과1건 해소의 결과다.

현재 HEAD·기존 등록 metadata·동결 바이너리 SHA를 확인한 libtest만24개 실행해 PASS/0 FAIL,10.766초다. source-space-tabs6root는 미실행이다. Cargo·prepare·추가 빌드0이다. RHWP_GAP_FIXTURE/RHWP_MULTISTOP_INPUTS/RHWP_SOURCE_SPACE_INPUTS를 자식 환경에서 제거해 정식 원본만 사용했다. producer.json과 rhwp-regression_suite_*.log의 실제 요약·전후 source/input/binary hash가 근거다. 사용자의 반복 빌드 지적대로 정적 감사와 기존 정확한 바이너리를 우선한다. 매 묶음 미빌드 root 때문에 Cargo를 시작하지 않는다.

독립 근거: generate_body_sources.py를 output 자기 폴더로 복사해14PDF를 동일 bytes로 재생성했다. 원본 폴더를 덮어쓰지 않았다. PDFium GetUnicode/GetCharOrigin/GetFontSize로 보고서7쪽 네 글자 원점, shared/source-space6PDF의 마지막 해당 단어 목적지, 첨자0.6/-0.4, 들여쓰기12pt/-15pt를 확인했다. superscript.pdf의 외부 Base14 font SECOND advance 합은42.77999973297119pt로 기대42.78pt와 맞는다. 모든 시스템 Helvetica의 윤곽 동일성은 주장하지 않는다. 제품 helper나 RenderTree로 기대값을 생성하지 않았다. independent-text-properties.json 및 independent-generator-check.json에 모든 원본 hash·관측이 있다.

실물 보고서 pdf/pr7704/6778-square-2020.pdf는 SHA d4c1fe7a8a3c025b2be3648ba6e2ad1123eff3dde008983edadcef7e8f7b6656, 전체12쪽이다.7쪽 네 행 적응훈련·사료조절·개체관리·관련사진은 x71.961pt, y373.03233/452.74520/532.57797/697.15810pt와0.00004CSSpx 이내다. 공백 제거 PDF 검색에서 적응훈련은 본문에도 두 번째 일치가 있어 모든 일치·원char index를 보존했다. Rust owner는 모델의 실제 원문 needle이 유일한 Paragraph 경로여야 한다.

수치 의미: 탭8은 HWP 스트림 유닛, baseline*75 후±1은1HU다.0.25/0.5/0.1은 같은 줄 분류·편집 inverse 복귀·소유 셀 포함/진행의 관계 여유다. clip1e-8은 source-local HU와 실제 content bbox의 공통 이동 관계다. shared/source-space의 독립 source 목적지±2px 부분은 일반 편집 관계와 구분한다. helper2개·호출root6개·실물원점root1개 총9개는 현재 양 backend/저장 경로 자격을 pending으로 유지한다. 아직 잘못된 핀 확정이나 삭제/완화 판정이 아니다.

원 PR은 #8120, merge80968497c2c178eeecb3b2e1d04d086ad27a0dc7이며 현재HEAD의 조상이다. 선택 source history를 보존했다. snapshot은74차/pr-8120.json이다. stage34의 보고서7쪽 과거N99.86930/W99.64065, stage35의 독립source4쪽 양쪽100%와 저장Print96%의 구분, stage36의169쪽·scope종료/등록 기록을 읽었다. 과거 점수·승인·원래 Docker 빌드를 현재head의freshWASM/전쪽 수용으로 재사용하지 않는다. 과거 PR 자동 품질 래칫도 연결하지 않았다.

현재 Native는 기존 동결 CLI phase73-page-selectors/rhwp-current-native SHA9d496496e8561ff04c7c64b7b7171581d29fa6babe1dcfcbe38467574250ecf5를 재사용했다. canonical visual_sweep.py/96dpi/compat2022/PNG Native Skia/PDFium/capture총8로 보고서7쪽만 산출했다. capture/TSV exit0, raw99.99786%,11.257초다. run_phase76_report_visual.py는 main guard가 있고 handle42175를 완료까지 관찰했다. review와 standalone overlay를 직접 열어 네 행·표 경계·사진4개·번호7을 대조했다. 글자 굵기·가는 선 농도 차이는 남고 raw100이 RGB 동일성은 아니다. review의100.0%는 반올림이며 엄격 내용 픽셀30.70%는 별도 값이다. 현재freshWASM·전12쪽·다른 저장본 완료가 아니다. native-visual-producer.json/direct-visual-reading.json/Native 로그·TSV·review/overlay가 근거다.

TextClip 검사는 숨은 원문을 모델/paint command에 보존하는 typed sidecar 계약이다. rhwp 저장 왕복과 한컴 native 비사각clip 호환 완료는 구분한다. note/master 검사는 metadata이며 실제 note paint 전체 시각 검사는 아니다. 빈PNG나 점수만으로 숨은 원문 보존을 판단하지 않았다.

#8232는76차 최신 수치·첨자 근거 해소·현재 보고서7쪽 Native·WASM/탭 root자격 보류로 갱신했다.73차 basic2007 p2 65.96489%·PrEP현재WASM/전쪽 pending 및 앞선 실제 핀/검출 한계를 모두 보존했다. 현재 진행 상황 밖 본문·#8266·완료조건은 그대로다. 역복구와 exact readback을 확인했고 updated_at2026-10-26T13:57:35Z/UTF16길이65536이다. 초과 로컬 초안은 게시하지 않았으며 실제원격update는1회다. publication-receipt.json이 근거다. 첫 로컬 체크포인트 작성은 absolute/relative Path 비교 assertion에서 멈췄고, 동일 절대 경로로 고쳐 완료했다. 제품/검사 실패나 원격본문 손상은 아니다. 다음 갱신도 freshGET·본문한도·다른 작업 보존을 먼저 확인한다.

다음77차 후보는 이미 읽었으나 아직 대장 반영/실행하지 않았다. tests/cases/issue_7998_tac_prefix_numbering.rs164줄과 pdf/issue7998/local-regions/generate_controls.py34줄 전체다. prefix AST함수9개 중 pages·owned는 미분류, a_later_number_restart...는 기존pending, 나머지는 초기검색 밖이다. generator는 top-level file-scope1개가 미분류다. stage38은26쪽 독립Gulim기준선798pt와 과거양쪽100%를 기록한다. phase68의7PASS는 모두issue1133이며 이prefix검사 실행증거가 아니다. 신규선택과 사용자제외 교집합을 확인하고 source이력·독립자료로 이어 판정한다. 기존 #7998의 나머지 미분류는 이3그룹뿐이다. 신규 함수의 루트/소유 검사까지 포함하고 검색에 걸린3그룹만으로 범위를 축소하지 않는다.

이번 turn은104함수 신규판정·24검사 실행·독립근거 확인·pending1해소/호출root6연결·현재Native7쪽 및 직접판독·대장/이슈 갱신이라는progress다. 전체 목표는 미완료이므로 complete/paused/blocked로 바꾸지 않는다. 남은 미분류3369그룹과 근거pending511의 전수조사·독립판정을 이어가고 이후 실제보정·개명·참조갱신을 완료한다.
