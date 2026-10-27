---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase73-page-selectors/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase73-page-selectors/goal-checkpoint.md` · 원문 SHA-256: `299316422e45a60a9dbc7582feb74484e560bdf4296aada069c17cbb509b3de6`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 73차 체크포인트

목표 ACTIVE: 전수 조사·독립 판정을 끝낸 뒤 #8232 갱신, 지침에 맞지 않는 검사와 필요한 구현 보정, 실제 보정한 파일·함수의 issue_8232 개명 및 helper/module/suite/manifest/생성/스크립트/E2E 참조 갱신을 완료한다. 기존 이름 wrapper의 중복 등록은 하지 않는다. 제품/검사 수정 0, 개명 0이다. PR은 나중이며 push/PR 승인은 없다. 새 agent·다른 task 메시지·예약·메모리 쓰기는 하지 않는다.

R=/Users/tsjang/rhwp, branch=audit/8232-px-regressions-20261025, head=01ced3d42724285c8f9765a453393ab6af326617 그대로다. 유일한 기존 dirty는 rhwp-studio/public/rhwp.js 97줄 추가이고 SHA c1c8d1820cc4375ab668427fa0a42c87d71c3d0d83955f4d300880559023ddc6를 보존했다. 원본/PDF/공유 target/다른 작업장과 Studio 서버를 보존한다. 요청 없이 fetch/rebase를 반복하지 않는다.

이번 Cargo/prepare/추가 빌드는 모두 0이다. 현재 head의 이전 완료 metadata와 바이너리 SHA가 같은 기존 libtest만 --exact --test-threads 8로 실행했다. 19 PASS/0 FAIL, 35.775초, 건너뜀 메시지0이다. Native CLI SHA 9d496496e8561ff04c7c64b7b7171581d29fa6babe1dcfcbe38467574250ecf5는 그대로다. 정적 감사와 기존 바이너리를 우선하며 소스 변경 또는 필수 미빌드 대상이 생길 때만 필요한 범위를 묶어 빌드한다.

최신 active-audit.json은 phase73-page-selectors/audit-ledger.json이다. 누적 12543그룹/2139파일, 직접9078함수/Rust6192검사/805파일이다. 의미 판정8572/file-scope8/근거pending506/미분류3195+의미메모97+문맥165=3457이다. 65함수(19검사/46helper)를 읽었고 신규64함수·기존 pending near1개 재검토다. 초기 원장 밖41그룹 추가, 새 pending6개, 기존 near는 해소하지 않았다. 사용자 제외 신규Rust95/삭제3(원장4그룹)/새JS callback1(schema2)·관련 helper는 그대로이며 선택과 교집합0이다.

읽은 source 전체1510줄: tests/issue_2007_nested_cell_pagination.rs867, tests/issue_505.rs289, tests/cases/verified_prep_completed_table_page.rs354다. 원 PR/통합/삭제를 추적했다. #2007 original2052(6→15쪽)→4069 original4122(17쪽)→4159 original4174의 clip 및6269 ink bbox 보정이다. dc3e31be54에서 제거한6검사를 복원하지 않았다. #505는7bcbe2c6b2의 직접 커밋/원 보고서이며 commit-associated merged PR 조회는빈배열이다. 이후fa07b3c818은 EqAlign 구조의 AST호환 변경이다. PrEP 완료 쪽494e2c491e는 실제 #8064에 포함됐고 원 producer는057a584e9eafc9f8ec12f7557326ca6a6c010b15다. #8064 강제 자동 래칫은 다시 연결하지 않았다. primary PR snapshot·source history·origin-contract-map을 보존했다.

원 helper13개는 현 root에서 도달하지 않는다. first_text_run_top/vertical_bounds, ClipRect.intersect/intersects_node/fully_contains, contains_painted_text/exact, collect_exact_text_line_clips, top/bottom frame/line 관련 일부 helper다. static-unreached-helpers.json은 같은 파일 calls를 root에서 닫은 결과이며 일반 동적 dispatch 완전성 증명은 아니다. 보존 helper에 visible/clip 처리가 있다는 이유로 현 활성 root가 이를 모두 검사한다고 말하지 않는다. Eq505의 chained root도 분수 최소2개만 세며 실제 좌결합 모양 전체를 검사하지 않는다. source 목적과 실제 assertion 범위를 구분한다.

현재 Native 선택4쪽만 canonical visual_sweep.py/96dpi/PNG Native Skia/Print/총capture8로 새 산출했다. driver는 run_phase73_native_visual.py이며 main guard가 있다. 실프로세스/handle을 확인해 완료까지 기다렸고 중복 실행하지 않았다. frozen CLI는 phase73/rhwp-current-native다. 원본/PDF SHA와 실제 명령/TSV SHA는 native-visual-producer.json에 있다. 전체 쪽수 metadata는 basic2007 양쪽17, PrEP 양쪽140이다. 이는 전쪽 TSV 완료가 아니며 선택4쪽 결과만 다음과 같다.

- basic2007 p2 raw65.96489%, p3 raw99.82268%. 실제 표 머리행·셀 본문이 p2에서 위로 당겨졌고 글꼴 문제로 단정하지 않았다. p2 9행 표의 Native bbox y613.2와 독립 PDF 긴 top선 중심622.039998은 정의가 다르므로 약9px 차이라고만 설명한다. p3은 조문 제2호 이후·마지막 조항·종료선이 양쪽에 보인다. capture/TSV 모두 exit1인 미달 gate를 그대로 보존했다.
- PrEP p69 raw99.98404%, p70 raw100.00000%. 69쪽 표27의 마지막 이상지질혈증 행, 70쪽 반복 머리행/종료4,761·표28/트리코모나스증·후속 본문을 review로 대조했다. p70은 standalone overlay도 열었다. raw100은 RGB 동일성이 아니며 glyph/쪽번호 raster 차이는 남는다. capture/TSV exit0이다.
- 대표 review/overlay는 native-visual/<basic2007|prep>/<review|overlay>/ 아래다. 직접 판독 기록은 direct-visual-reading.json이다. p2 미달을 p3이나p70 점수로 상쇄하지 않는다. 현재 fresh WASM/전17·140쪽 수용은 주장하지 않는다.

독립 PDF path/CTM을 PDFium으로 읽어 좌하단pt→96dpi 좌상단CSS px로 환산했다. PrEP70의 기대 y경계16개는 최대0.00004117px 차이였고 두 표의 x왼쪽75/right718.65332도 일치한다. 원본 SHA c68baed.../PDF732baa...는 stage1577 README와 일치한다. basic3의 y>820/폭>500 조건에 맞는 독립 가로선은1개, y827.894674이다. PDF basic canonical은 -2020.pdf SHA9b0390...이며 -hwp-2020.pdf SHA450aae...와 다르다. 원 #4174의 실제 기준9b0390을 골랐고 다른 파일을 덮어쓰거나 혼합하지 않았다. probe_pdf_borders.py, independent-pdf-borders.json/log, independent-basic-p2-lines.json이 재현 근거다.

남긴 pending7그룹: PrEP near/completed_row_borders/check_completed_page/root4개(이 중near기존1), basic terminal_bottom_lines_with_cell_clips/helper와root2개다. PrEP 독립 좌표는 확인됐으므로 잘못된 핀으로 단정하지 않는다. 현재Native70쪽100% 및 과거양쪽100%를 구분하며 현재freshWASM/전쪽자격이남는다. basic은 y>820·폭>500·높이≤2와 SVG x<80·폭>650 고정 영역으로 먼저 선택한 뒤 clip관계를 검사한다. source소유/visible 전체 연결이 부족하고현재p2가98%미만이므로 source소유선택보완/자격판정대상이다. 아직 구현·회귀를수정하지않았고 change_required는pending으로남겼다. 19검사PASS를전체시각완료로쓰지않는다.

#8232는73차최신수치/정상요약과새진단을갱신했다. actual failure findings·외부본문·8266·완료조건은보존했고 이전status역복구/EOF exact readback을확인했다. updated_at2026-10-26T13:21:27Z, UTF16길이65514다. 두번의로컬초안길이점검은65536초과로게시전에중단됐고 실제원격변경은검증한한번뿐이다. 정상최신요약의중복소스보존문장만줄였다. 다음게시도freshGET/길이확인을우선한다. issue-before/previous-status/publication-receipt에증거가있다.

이번 turn은64함수신규판정/19개실행/현재선택4쪽의실제미달과PrEP100%확인/독립PDF16경계및양끝재확인/원장·이슈갱신이라는progress다. 남은3457그룹과근거pending506의전수조사·독립판정을계속한다. 이전phase57/58/60/61/62/63/68의실제핀/검출한계/자격부족도보존한다. source보정·issue_8232개명·참조갱신과필요검증은전수조사이후에완료한다. 전체목표미완료이므로complete/paused/blocked로바꾸지않는다.
