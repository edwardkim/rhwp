---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase94-header-footer-selection/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase94-header-footer-selection/goal-checkpoint.md` · 원문 SHA-256: `55a5aa3fd15103bde94b28b15098a3e2ec3d2dc5bc63358e7bcfdfd643c94285`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 94차 체크포인트

전체 목표는 ACTIVE이며 미완료다. 전수 조사와 독립 판정을 마친 뒤 #8232를 최종 갱신하고, 지침과 다른 기존 검사를 실제 출력 보정·독립 기대값·필수 시각 조건에 맞게 수정한다. 실제 보정한 파일·함수는 issue_8232로 명명하고 helper/module/suite/manifest/생성/스크립트/E2E 참조를 함께 갱신한다. 정상 검출 목적과 원본 자료를 보존하며 이전 이름의 중복 wrapper는 만들지 않는다. 이번에도 제품·검사 수정과 개명은 0건이다. PR은 나중이며 push/PR 승인은 없다. 다른 task/subagent/예약/메모리 쓰기는 하지 않는다.

작업장은 /Users/tsjang/rhwp, 브랜치는 audit/8232-px-regressions-20261025, HEAD는 01ced3d42724285c8f9765a453393ab6af326617다. 유일한 dirty rhwp-studio/public/rhwp.js는 감사 전부터 존재했고 97줄 추가와 SHA c1c8d1820cc4375ab668427fa0a42c87d71c3d0d83955f4d300880559023ddc6를 보존했다. 임의로 revert/stash/commit하지 않았다. 사용자 원본·PDF·다른 작업장·공유 target·기존 Studio 서버를 보존했다. 요청 없는 fetch/rebase, Cargo/prepare/추가 빌드는 모두 0회다. 마지막 실제 프로세스 조회에서 Cargo/rustc/nextest/wasm-pack 및 93~94차 감사 작업은 없었다. 실행 session 78032/38238과 원장 기록 session 63337/14470은 실제 exit 0으로 종료했다. 커밋-PR 연관 조회 session 84983/83521도 종료를 확인했다.

정본은 active-audit.json → phase94-header-footer-selection/audit-ledger.json 및 scope-summary.json이다. 현재 21,439그룹 / 2,501파일, 직접 판정 10,166함수, 그중 Rust 검사 6,506개 / 839파일이다. 미분류 11,004그룹 + 의미 메모 95그룹 + 파일 문맥 165그룹 = 11,264그룹은 추가 판정이 필요하다. 직접 판정 안의 근거 pending은 512개다. 보정 필요로 명시된 87개는 유지했다. 새 200px 상한 1개는 근거 부족으로 pending에만 추가했고, 실제 제품 회귀 또는 교체 확정으로 단정하지 않았다. 함수/원장 수를 실행 testcase 수로 부르지 않는다.

80차의 누락 후보 8,517개 연결과 사용자 제외는 유지한다. Rust 2,887개와 scripts/tools/bindings 5,630개를 포함하며 workspace 명시 제외 vendor 6,496개는 별도다. 사용자 지시로 삭제된 Rust 3검사(원장 4그룹), 신규 Rust 95검사/48파일, 신규 JS callback과 관련 helper는 감사/PASS에 넣지 않았다. is_test=false callback·매크로·동적 순회·생성 경로·feature별 실제 등록 대조도 여전히 미완료다. 이전 #8266·실물 미달·잘못된 핀과 검출 한계는 삭제하지 않는다. 선행 근거와 전체 범위는 92차 goal-checkpoint.md 및 그 연결을 따른다.

93차는 src/renderer/image_resolver.rs의 두 cfg(test) 블록 34함수와 tests/cases/issue_536_direct_pdf_linear_gradient.rs의 12함수, 총46함수를 읽었다. 기존 직접 판정 10개는 이전 원장을 보존한 재검토이고 36개만 신규 판정했다. 신규는 Rust roots 20개와 helper 16개다. 모든 지역 helper와 source bytes/형식 변환/mime/캐시/신원 키 및 gradient 축/stop/Native shader/PDF preflight의 생산·소비 경로를 확인했다. 다른 하위 모듈 전체와 실제 문서 corpus 전수를 이 파일 조사로 대신하지 않는다.

이미지 검사의 RGB 값·합성 BMP(64×64)의 중앙 띠·PCX 헤더/행 패딩·TIFF IFD 오프셋·WMF/EMF 레코드 좌표·EPS preview·캐시 건수/byte budget·epoch/bin ID는 원본 문서의 관측 px 핀이 아니다. source_image_key의 variant와 inline bytes가 같은 변환을 선택하는 계약을 현재 production과 연결했다. 작성 watermark를 일반 PictureTone으로 잘못 해석하지 않도록 picture_tone_parameters의 watermark 제외도 읽었다. source key/inline 동등성이 그 결과의 한컴 정답을 보장하지 않으며, covered 표본은 확장된 모든 variant의 전수 검사가 아니다. PNG/SVG magic만으로 모든 그림 내용·ink를 인증하지 않는 한계도 기록했다.

image_resolver의 현재 source library 검사 실행은 0이다. 보유한 동일 HEAD의 성공 registered-tests metadata에서 해당 library test를 찾지 못했고, 이를 실제 CI 미등록이나 실패로 주장하지 않았다. 단순 감사를 위해 새 빌드를 하지 않았다. 기존 10개 판정을 현재 실행 통과로 다시 세지 않았다. image-unit-registration-search.json과 previous-direct-functions-reread.json이 근거다.

Native gradient 8개는 phase66 producer와 같은 regression_suite_002 바이너리 SHA c2ff43869cd998b7d0a470749476898f24641021834612c6916142ec40959c12를 실행 전후 확인해 재사용했다. 실제 --list와 기존 metadata의8개 이름을 대조하고 --exact/threads8로 실행해 8 PASS / 0 FAIL / 0 ignored, 0.19초, exit 0이다. 새 빌드나 전체 suite/nextest는 실행하지 않았다. 이 검사들은 실제 합성 Skia raster/PDF를 메모리에서 만들지만 실물 HWP/HWPX·독립 한컴·fresh WASM 페이지 수용은 아니다.

gradient에서 abs_diff<=2는 RGB/A 8-bit 채널 강도 차이이며 ±2px 위치 허용 오차가 아니다. >100도 채널 변화량이다. 샘플 좌표는 작성 96×64 페이지의 (8,8,80,24) 입력 내부다. 독립 산술로 pixel center12.5/27.5의 중심y20 대칭·가운데 빨강과 양끝 파랑 관계,45도에서(+8,+8)의x-y 불변,96CSSpx→72PDFpt·64→48 MediaBox, binary32의1e10+1 끝점 붕괴를 확인했다. 원본 문서의 잠정 배치를 복사한 px 기대값과 구분했다. source color4096/expanded8192 및 최소1CSSpx 축 길이는 제한된 direct export admission 계약이며 위치 오차를 완화한 값이 아니다.

14개 함수 도입 커밋의 조상 관계와 GitHub commit-PR 연관을 읽기 전용으로 확인했다. 이미지 변환/메인터너 보강은 #1704, cache·전체 bytes hash는 #3320, #3315 키 계약은 #3680, PCX/TIFF/BMP/EPS·WMF/EMF 연결은 #4083에 연결된다. gradient 원 PR #7893과 체리픽 통합 #7969, source-center 후속 보정과 #7661의 독립 Print 대조군 provenance를 읽었다. 이 과거 수치/시각 결과를 현재 PASS로 재사용하지 않았다. 원 PR #7893은 직접 merge가 아니라 체리픽 통합이며 누적 이력 #8196 연관도 원 도입으로 오인하지 않는다. 각 원문 snapshot·commit-pr-associations.json·origin-contract-map.json·producer-consumer-scope.json에 보존했다.

94차는 tests/cases/issue_4121_header_footer_text_selection.rs 전체10함수(7roots·3helper)를 신규 조사했다. 명시한 field-01.hwp를 API로 편집한 문서와 blank2010 생성본의 입력 경로를 구분했다. 다문단 선택·active/대표 preview target·field 표시와 모델 offset·범위 치환·접미사·clipboard·부분 서식·저장/재열기는 논리 내용/소유 관계다. 반환한 rect 안의+1 또는폭75% probe를 실물 페이지의 절대 위치 핀으로 오분류하지 않았다. 본문4000회 반복과2쪽 이상은 홀짝 target을 시험할 생성 전제이며 특정 문서의 과거 쪽수 잠금이 아니다.

현재 HEAD의 phase59 producer와 같은 regression_suite_004 바이너리 SHA는94차 producer.json에 고정했다. 실제 --list와 metadata 이름7개를 대조하고 --exact/threads8로 실행해7 PASS / 0 FAIL / 0 ignored,0.62초,exit0이다. 현재PASS는200px 기대값의 독립 근거가 아니다. 브라우저/WASM 또는 실제 한컴 시각 검증은 실행하지 않았다.

원 PR #6394는 새 HF의 본문 기본 양쪽 정렬과 짧은 마지막 줄의 자연 폭을 보존하라는 목적을 명시한다. 함수 선언의0697b2af...와200px assertion 도입cfa0620c...를 구분해 추적했다. 도입patch는 blank2010 생성·스타일 캐시 조회·문자열 삽입·width<200.0을 추가했지만200 자체의 독립 출처를 설명하지 않는다. 모든 관련 함수 도입·assertion commit이 #6394 merge와 현재HEAD의 조상임을 확인했다. width-ceiling-introduction.diff·width-ceiling-origin.json·pr-6394.json을 보존했다.

saved/blank2010.hwp는 제품 parser를 거치지 않고 CFB DocInfo를 직접 풀어 첫 PARA_SHAPE attr의bit2..4=0을 확인했다. 공식 설명본의0=양쪽 정렬과 대응하므로 Justify 기대값의 출처는 확인됐다. 이것은 삽입한 텍스트의200CSSpx 상한이나 실제 자연 폭을 입증하지 않는다. 실제 선택 geometry는 render tree의 HF TextRun·replay_positions·model/display offset 매핑으로 양끝 x를 얻고 width=(x2-x1).max(1.0)를 JSON 소수둘째자리로 노출한다. 생산 규칙에200은 없다. selection-geometry-consumer.json·independent-template-para-shape.json에 근거와 한계를 분리했다.

newly_created_header_and_footer_use_body_default_justification은 현재 실행PASS와 별개로 function_evidence_pending에 두었다. change_required_for_issue8232_px_policy는 null이며, 실제 제품 회귀/지침 위반 확정/PASS 판정으로 승격하지 않는다. 다음 조사는 원래 자연 폭·스타일 갱신 결함을 계속 검출하도록 독립 기대값 또는 원본 글자/문단 속성과의 관계 근거를 확보하는 것이다.200을 먼저 완화/삭제하거나 모델 Justify·다른6개의 정상 편집 관계를 제거하지 않는다. 실제 수정·개명 전에는 적용되는 선행 시각/독립 판정 요건을 충족한다.

이번 목표 turn의 신규 직접 조사는46함수 = Rust roots27개 + helper19개다. 기존10개 재검토는 중복하지 않았다. 실제 실행은 Native gradient8 + HF편집7 =15 PASS / 0 FAIL / 0 skip다. 이미지의 신규12roots는 정적 조사이며 실행 통과로 세지 않았다. 독립 수치 근거 pending1개가 늘었고 기존 보정 필요87개는 유지했다. 추가 빌드·제품/검사 수정·개명·원격 게시 모두0건이다. 전체 조사·독립 판정·실제 보정·issue_8232 개명/참조 갱신·필수 검증은 미완료이므로 목표를 완료로 표시하지 않는다.

#8232 본문은80차 게시 당시 exact readback(updated_at2026-10-26T14:23:00Z,UTF16 65,534)이 마지막 확인이다. 현재 live 본문이 이후 바뀌지 않았다는 주장이 아니다.81~94차는 로컬 기록만 추가했다. 전수 완료 후 최종 이슈 갱신 지시를 유지하며, 게시 시 fresh GET·UTF16 한도·다른 작업/실제 발견 근거 보존·exact readback을 따른다.90차 wrapper 집계 오류와 재실행 없는 복구를 이번 exit0으로 덮어쓰지 않는다.

다음은200px 상한의 자연 폭/독립 기대값 근거와 나머지 source/tests/scripts/tools/bindings의 미분류·helper/생성/동적 경로를 계속 확인한다. 같은 검사·빌드를 반복해서 현재 근거 부족을 해결한 것으로 처리하지 않는다. 완료 요건을 충족하기 전 goal은 complete/paused/blocked로 바꾸지 않는다.
