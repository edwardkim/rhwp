---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase75-pdf-boundary-relations/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase75-pdf-boundary-relations/goal-checkpoint.md` · 원문 SHA-256: `11d04bd64fba863d439387e7e3843c5b4a753af82b493f3b84c80a5090369c01`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 75차 체크포인트

목표는 ACTIVE다. 전수 조사·독립 판정을 끝낸 뒤 부적합 검사와 필요한 구현을 보정하고, 실제 보정한 파일·함수를 issue_8232로 개명하며 helper/module/suite/manifest/생성/스크립트/E2E 참조를 함께 갱신한다. 유효한 목적·정상 검사는 보존하고 옛 이름 wrapper의 중복 등록은 하지 않는다. 제품·검사 수정 0, 개명 0이며 PR은 나중이다. 새 push·PR 승인은 없다. 다른 task·subagent·예약·메모리 쓰기는 하지 않는다.

작업장 /Users/tsjang/rhwp, 브랜치 audit/8232-px-regressions-20261025, HEAD 01ced3d42724285c8f9765a453393ab6af326617이다. 기존 dirty rhwp-studio/public/rhwp.js의 SHA c1c8d1820cc4375ab668427fa0a42c87d71c3d0d83955f4d300880559023ddc6를 그대로 보존한다. 원본·PDF·공유 target·다른 작업장·기존 Studio 서버를 보존한다. 요청 없이 fetch/rebase하지 않는다.

최신 원장은 phase75-pdf-boundary-relations/audit-ledger.json이다. 누적 12649그룹/2139파일, 직접 9251함수/Rust 6257검사/817파일이다. 의미 판정 8745·file-scope 8·근거 pending 506·미분류 3128+의미 메모 97+문맥 165=3390이다. pending은 판정한 함수에도 포함되므로 미분류와 중복 없이 구분한다. 사용자 제외인 삭제 Rust 3검사(4그룹)·신규 Rust 95검사/48파일·신규 JS callback 1개(schema 2건)와 관련 helper는 감사/PASS에 넣지 않았다. 일반 암호 E2E는 유지하고 새 sample16 caller는 제외한다.

74차: PDF 소유·면/점선 편집 6파일 전체 2216줄, 96함수/41검사/55helper를 읽고 54그룹을 초기 원장 밖에서 추가했다. 모두 신규 직접 판정이다. 이전 완료 producer의 현재 HEAD·바이너리 SHA를 확인한 기존 libtest 29개만 실행해 PASS/0 FAIL, 1.595초였다. periodic/raster 12root는 미실행이다. Cargo·prepare·추가 빌드는 0이다. canonical 소스/PDF를 수정하지 않고 output 안에 독립 생성한 periodic/fill 13PDF의 bytes가 같았다. 독립 한컴 Print 실제 path/CTM에서 pen 5/7/12/35 tick과 on/off 7/10·10/15·17/25·52/78 tick을 읽어 네 공개 CSS 패턴의 근거를 확인했다. 길이는 600/72 tick 반올림 후 96/600 CSS 환산이며 고정 페이지 x/y가 아니다. 독립 local source의 고유 ASCII label 37개도 확인했다. probe_independent_print.py와 independent-print-components.json에 근거가 있다.

75차: PDF 경계 원속성·편집 관계 6파일 전체 1657줄, 78함수/24검사/54helper를 읽었다. 신규 직접 판정 77함수와 이미 판정된 cell_paint_frame.verify helper 1개 재독을 구분했다. 초기 원장 밖 52그룹 추가다. 기존 libtest 10개 PASS/0 FAIL, 0.525초이며 ownership/replacement/external paint의 14root는 미실행이다. Cargo·prepare·추가 빌드는 0이다. 독립 generator의 solid/mixed/periodic/occluded PDF 4개 bytes를 확인했다. PDFium raw path/fill/CTM으로 실제 빨간 사각형 4개를 origin JSON과 비교했고 최대 오차는0.00005pt 미만이다. olefile/zlib/struct로 정상 한컴 HWP의 CTRL_HEADER(tag71) byte16/20을 읽어 부모 24000×18000HU·자식 9000×2800HU 및 PARA_LINE_SEG(tag69)의 vpos 945를 확인했다. 제품 모델/기대 배치 helper에서 좌표를 얻지 않았다. probe_source_properties.py와 independent-source-properties.json이 근거다.

두 묶음은 신규 173함수·실행 39 PASS/0 FAIL·미실행 26root다. 0.5는 실제 소유 셀 내부 포함의 관계 여유, 0.02는 HWP 1/100pt 끝점 양자화의 면적 오차,100/12000은 실패해야 하는 HU 이동/크기 입력,150×150은 resource 표본 속성,69/101 등은 byte 형식 길이다. 점선의1e-6은 독립 Print에서 온 성분 메트릭 전달 검사이며 실제 페이지 위치 잠금이 아니다. 셀 bbox 중간 공유축·높이와 source HU frame 대비 실제 frame은 관계로 검사한다. 이번 묶음에서 새 부적합 실물 페이지 핀은 확정하지 않았다. PASS는 내부 구조·편집/형식 계약의 실행이며 미실행 root나 현재 전쪽 Native/fresh WASM 수용으로 확대하지 않는다. Picture 수·색 기반 수집·Rectangle ink 판정의 실제 범위와 visibility/clip 미포함도 각 fact에 남겼다.

원 해결 PR은 #8120, merge 80968497c2c178eeecb3b2e1d04d086ad27a0dc7이며 현재 HEAD의 조상이다. 선택 12파일은 이 커밋의 source history를 보존했다. pr-8120.json은74차 폴더에 있다. B15의 보류 원본·B16의 후속 해결·이후 셀 격자 확장을 구분하고 과거 전체 PASS/Native/fresh WASM/승인 예외를 현재 검증으로 재인용하지 않는다. #8064 또는 과거 PR 자동 품질 래칫도 연결하지 않았다.

#8232 본문은 최신 수치와74–75차 정상 조사 요약으로 갱신했다.73차의 basic2007 p2 65.96489%·PrEP 현재 WASM/전쪽 자격 pending 및 이전의 실제 핀·검출 한계를 모두 보존했다. 현재 진행 상황 밖의 다른 본문과 #8266 및 완료 조건은 바이트 그대로다. 기존 status는 previous-status.md에 보존해 역복구를 확인했고 원격 exact readback도 확인했다. updated_at 2026-10-26T13:43:01Z, UTF16 길이 65536이다. 초과 로컬 초안은 원격에 게시하지 않았다. 다음 업데이트는 fresh GET·본문 한도와 다른 작업의 변경 보존을 먼저 확인한다. 정상 상태 요약만 간결하게 고칠 수 있으며 실제 발견 사항을 삭제하지 않는다.

사용자의 반복 빌드 지적 이후 정적 감사와 정확한 기존 바이너리 재사용을 우선한다. 이번74·75차는 빌드0이며 소스 수정도 없다. 미빌드 root를 실행하려는 이유만으로 매 묶음 Cargo를 시작하지 않는다. 필수 소스 변경 또는 필요한 미빌드 검증을 묶어 수행할 때만 빌드한다. 실행 공백이나 사라진 관찰만으로 이전 작업을 재시작하지 않고 실제 프로세스·로그를 확인한다.

전수 조사와 독립 판정·실제 보정·개명·참조 갱신은 미완료다. 남은 미분류3390그룹과 근거 pending506을 계속 조사한다. 다음 미판정 PDF 항목은 body paragraphs/indentation/soft wrap, inferred/shared/source-space tabs, text clip/placement/owned lists 및 edit baselines다. 현재 원장의 unresolved 목록에서 선택하고 사용자 신규 제외와 교집합이 없는지 확인한다. 이번 turn은173함수 판정·독립 PDF/원레코드 확인·39개 실행·원장/이슈 갱신의 실제 progress다. complete/paused/blocked로 바꾸지 않는다.
