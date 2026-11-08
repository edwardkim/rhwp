# #7419 Stage6 — 0953598 리베이스와 검증

## 기준과 승인 범위

- 기준 devel: `0953598d8e08af5e9149b3e4cdbbbf2f5f2bae64`(#7947).
- 검증 source: `f5a4774049fa10cc661f7220de0127d61fa081ba`.
- 16개 commit의 리베이스는 충돌 없이 끝났다. 기존 보정 source·검사·입력·Print는
  리베이스 전과 같고 오늘할일에는 upstream 기록을 보존했다.
- 사용자 답변 **Docker 미검증 상태로 유지**를 적용한다. Docker를 설치하지 않는다.
  host `--dev --no-opt` fresh WASM은 진단 결과이며 표준 Docker 성공으로 보고하지 않는다.
- 기본 작업공간의 사용자 Studio JS는 원문 SHA-256
  `fd499c9eb514018d2063c8eef1e58ec892c734d512cf5e88ab97bd9e5d47a0a3`로 보존했다.
  외부 백업과 stash를 유지하며 review worktree의 별도 WASM 출력만 갱신한다.

## 실제 변경과 검증 계획

새 upstream Rust 변경은 `HwpDocument.supportsPdfImport()` 조회 6줄이다. 조판·parser·
Rust 검사·Cargo 의존성은 바뀌지 않았다. Studio는 PDF 열기와 npm 의존성 보정을 포함하므로
새 WASM API가 실제 노출되고 기본 PDF-off에서 false를 반환하는지, 기존 HWPX 입력이
각 실제 backend에서 계속 열리는지 확인한다. PDF 기능을 새로 켜지는 않는다.

1. 공용 `target/pr-review`, Cargo 잡8개로 기준·후보 Native와 host fresh WASM을 순차 빌드한다.
   작업공간 전환 시 진입점 mtime을 갱신하고 실제 컴파일·생산자 해시·API를 확인한다.
2. 동일한 13문서19쪽의 전쪽 Native/host WASM TSV를 새 base와 비교한다. Print 쪽수·
   90/98%·0.01%p 하락·누락을 판정하고 대표 review와 standalone overlay를 직접 대조한다.
3. 집중7419·7699, 전체 nextest, lint6종, Native Skia3종을 현재 source에서 실행한다.
   Stage5의10,454개 PASS를 현재 source에서 재실행한 결과로 바꾸어 쓰지 않는다.
4. 새 의존성 계약과 실제 Studio HWPX 소비 경로를 확인한다. 결과는 실행 후에만 기록한다.

기존3문서의 base 쪽수 차이에 대한 전쪽 일대일 TSV 보류 조건은 유지한다. Docker 선택은
이 gate의 예외 승인이 아니다. PR/push는 별도 승인 전까지 수행하지 않는다.
원시 로그·TSV·JSON은 ignored
`/home/tsjang/rhwp/output/pr-review/pr7433-current-20261016/rebased-0953598/`에 보존한다.

## 프런트엔드 선행 검증

새 lockfile로 Studio·VSCode·frontend-metrics의 `npm ci`가 모두 exit0으로 끝났다.
작업공간의 의존성 링크만 재사용하며 tracked lockfile과 사용자 JS는 변경하지 않았다.

- 의존성 호환 계약: exit0. 실제 설치한 brace-expansion·Jake FileList·Ajv/fast-uri를 실행했다.
- Studio `tsc --noEmit`: exit0.
- Studio `npm test`: 2,023개 중2,021 PASS·실패0·2 skipped, 40.025초.

source는 위의f5a이며 명령·시간·exit는 `frontend-validation.json`, 각 `.log`에 기록한다.
이는 새 WASM 실제 호출과 브라우저 소비 검증을 대신하지 않는다.

## 새 기준 Native 실행

0953598의 Native Skia CLI는 실제 재컴파일245.992초·exit0이고 SHA-256은
`d845080eb0acfeda3bcc5bca495bd687fafd78789a6e1629adb65d054737c252`다.
첫 기준 배치에서 `page-height-28000`의 렌더2쪽/독립 Print3쪽 불일치를 재현했다.
이는 후보 실패가 아닌 기준 출력의 쪽수 차이이며, 전체 일대일 TSV gate의 보류로 남긴다.
앞서 산출된 기준 PNG12개는 cdef 기준 출력과 바이트가 같다. 이 비교는 배치가 중단된
범위까지만의 부분 증거이며 미실행 입력·후보·WASM의 결과로 확대하지 않는다.
명령·실제 source·실행 파일 해시는 `producer-builds.json`, 실패 로그는
`base-native-sweep.log`, 부분 PNG 비교는 `base-native-historical-png-partial.json`에 있다.

## 후보 Native 전쪽 산출 완료

f5a의 Native Skia CLI는 실제 재컴파일243.7초·exit0이다. 동일13입력의19쪽 모두
2px 관용 실루엣100%,90% 미만·측정 불가·Print 쪽수 차이0이다. 최신 PNG19개는
직전7fe 검증본과 바이트가 같다. 실제 manifest·PNG 해시·명령/종료는
`candidate-native-verification.json`, `candidate-native-sweep-result.json`,
`candidate-native-sweep/`에 기록한다. 실루엣 보조 산출의 `not_evaluated`를 승인으로
바꾸지 않으며, fresh WASM과 같은 영역의 직접 확대 판독은 아직 진행 전이다.

## 실제 저장본 정상 대조군

Stage4에서 한컴으로 실제 저장한 HWP/HWPX를 새 기준·후보 Native CLI에 각각 전달했다.
4명령 모두 exit0이며 각2쪽 render tree가 기준/후보 및 Stage5와 바이트 동일하다.
저장 줄이 있는 정상 경로의 출력 보존을 확인한 것이며, 별도의 Print 시각 일치 주장으로
확대하지 않는다. 입력 SHA·명령·생산 source·쪽별 출력 해시는
`saved-control-verification.json`에 기록한다.

## host fresh WASM과 후보 전쪽 산출

기준 WASM build282.0초·후보300.6초, 양쪽 모두 실제 재컴파일 후 exit0이다.
WASM SHA-256은 기준 `6a6099e620bd34afdc81ae459cb41c0a37a74d6d8ff1e9c929c59819d79efd21`,
후보 `3e295a5d1e44f645214c3c804a57e21c81d2a6dd006ca0da609693afecfd9900`다.
새 패키지 양쪽을 실제 초기화하여 `supportsPdfImport()`의 false 반환과 두 PDF 입력 진입점의
`PDF_IMPORT_DISABLED` 거부를 확인했다. 새 API가 없는 이전 패키지를 사용하지 않았다.
`wasm-capability.mjs`의 assertion 실행은 exit0이며 `wasm-capability.json`에 실제 WASM 해시를 남긴다.

후보 Native/host fresh WASM 각각13문서19쪽의2px 관용 실루엣 최저100%, Print 쪽수 일치다.
출력 PNG38개는7fe의 검증 PNG와 바이트 동일하다. 직접 판독과 표준 review/overlay 생성,
새 기준3문서의 최종 일대일 판정은 별도로 진행 중이며 이 수치로 승인 판정을 대신하지 않는다.

## 실제 Studio 소비 경로

현재 후보 WASM을 review worktree의 ignored `pkg`에 복사하고 실제 Vite·Chromium에서
원본2개·혼재1/2쪽·NO_LS 대조군2쪽의5조합을 실행했다. Canvas2D와 CanvasKit
compat/default software의15캡처 모두 요청/실제 backend·Print profile이 일치했고,
캡처 페이지의 초기 WASM15응답 SHA도 후보 `3e295a5d…`와 일치했다. fallback·replay hard
위반·text fallback·unsupported·실행 오류0이다. 캡처 후 전용 Vite는 종료했다.

15개 PNG는 Stage5와 바이트 동일하다. 5조합을 실제3 backend별로 나란히 열어 내용·줄바꿈·
쪽 소속·후속 본문을 확인했다. 원본 A4의 Studio1123px와 PDF1122px는 canvas 높이 반올림
차이이며, 판독 crop은 같은96dpi 원점을 유지하고 이미지를 확대/축소하지 않았다.
Studio의 모서리 안내선과 획 raster 차이는 남는다. 엄격 backend 픽셀10건은 report-only
미달이며 readiness gate의 선택 corpus는0건이므로 이를 parity/readiness gate PASS로 쓰지 않는다.

실제 명령·종료206.0초·생산자/응답 해시·직접 판독은 `studio-run.json`,
`studio-initial-responses.json`, `studio-verification.json`, `studio-contact-provenance.json`에 있다.

## 새 기준 비교와 전쪽 직접 판독

기준의 뒤쪽 입력까지 개별 실행해 미실행을 남기지 않았다. 원본 일반/중첩의 기준 Native는
각각88.2452%/72.8091%이며 후보는100%로 개선됐다. 전체 비교 가능한10문서·backend별12쪽에서
0.01%p 이상 하락0·조건 불일치0이다. 나머지3문서는 양쪽 backend 모두 기준 렌더 쪽수가
작은 용지2/Print3, partial 폭 부족1/Print2, wrapper1/Print2다. 후보는 독립 Print와 같은
3/2/2쪽이지만 전쪽 일대일 TSV gate는 사용자 답변대로 `re_review_required`를 유지한다.

실제 빌드4개와 모든52개 실행 manifest의 생산자 해시를 연결했다. manifest의 `git_head`는
실행 runner의f5a이며 기준 producer를 뜻하지 않는다. 기준095/후보f5a의 실제 source는
build 명령·실행 파일/WASM 해시에서 구분한다. `producer-sweep-link.json`과
`final-devel-comparison.json`, `visual-verification.json`에 원시 수치를 연결한다.

같은19쪽의 Native/독립 Print/host fresh WASM을 같은96dpi 영역으로 직접 열어 줄 구성·
여백·쪽 소속·자식 이월·wrapper와 뒤 본문을 확인했다. 새 내용 손실·중복·겹침·빈 쪽은
관측하지 않았다. 실제 저장 줄 혼재2쪽은 양 backend의 표준 review와 standalone overlay도
함께 열었다. 획 raster 차이는 남으며 실루엣100%를 엄격 픽셀 동일성으로 쓰지 않는다.
입력에 없는 수식·각주 조합은 검증했다고 주장하지 않는다. 판독 파일과 해시는
`direct-read.json`에 기록한다. 집중 회귀·전체 회귀·lint·Native Skia의 최종 종료는 아직 남았다.

## 집중 회귀 완료와 전체 실행

- 현재 suite 준비3.7초·exit0,7419는suite025·7699는suite006으로 실제 배정됐다.
- `issue_7419_no_ls_tac_row_growth`: 14/14 PASS, 테스트0.754초·빌드 포함396.7초·exit0.
- `issue_7699_stored_square_top_band`: 5/5 PASS, 테스트0.129초·명령29.5초·exit0.
- 같은f5a에서 release-test 전체 nextest를 시작했다. 전체→lint6종→Native Skia3종의
  순차 pipeline이며 이 기록 시점에는 전체 컴파일 중이다. 종료 전 PASS로 기록하지 않는다.
- 실제 실행 중인 전체 launcher의 `/proc`에서 명령과 허용한 환경 항목만 읽어,
  잡8개·공용 target·원본2개 `RHWP_SECURITY_SWEEP_SAMPLES_JSON` 전달을 확인했다.
  사후 계획 인용 대신 실행 증거를 `full-runtime-launcher-evidence.json`에 남겼다.

명령·source·exit·시간은 `focused-validation.json`, 실제14/5건은 각 focused 로그,
전체 실행 상태는 `pipeline.json`, `final-pipeline.json`, `full-nextest.log`에 있다.

## 실행 세션 중단과 남은 검증 재개

시각 full review도 Native/host WASM 각각13문서19쪽·최저100%·exit0으로 끝났다.
TSV를 산출한 PNG와 각 review의 입력 PNG도 바이트 동일하며, 대표26개와 Studio2개 자산을
현재f5a의 새 산출물과 직접 대조했다. 모두 기존 tracked PNG와 같아 불필요한 PNG 교체 없이
README의 생산 source·base·증거 경로만 현재 결과로 갱신했다.

장시간 실행 세션의 관측 handle이143으로 종료된 뒤 실제 pipeline·Cargo·Rust PID도
사라진 것을 확인했다. 전체 회귀는 컴파일 중이었고 test summary가 없었으므로 PASS나
회귀 실패로 기록하지 않는다. 완료된 시각/집중/Studio 결과는 보존하고,
원시 중단 로그와 상태는 `interrupted-attempt-1-*`, 관측 근거는 `observation-status.json`에 둔다.

남은 전체→lint→Native만 task 전용 session으로 재개했다. 공유 캐시를 유지하고 소스·
검사·baseline은 바꾸지 않았다. 실제 PID는 `resumed-launch.json`, 전체 재개 프로세스의
명령·허용 환경 검증은 `full-runtime-launcher-evidence.json`의 attempt2에 기록한다.
실제 프로세스와 완료 상태·test summary를 확인하며, 관측 handle의 종료만으로 재시작하지 않는다.

## 현재 source 전체 회귀 완료

재개한f5a의 전체 nextest는 **10,454개 실행·전부 PASS·실패0·13 slow·45 skipped·exit0**다.
실제 compile7분56초·테스트749.168초·명령 전체1227.969초였다. 중단됐던 이전 시도를
이 결과에 합산하지 않는다. 이번7419의14개와 Square5개도 실제 전체 로그에서 모두 PASS다.

이전 실패였던 `overflow_cell_lines_do_not_grow_partition_5`(13.403초),
`missing_stored_rows_do_not_assume_one_inline_row`(0.017초),
`off_canvas_does_not_grow_partition_4`(4.225초)도 현재 전체 실행에서 PASS다.
실제 원본2개 환경 전달을 확인한 신규 sample 보안 검사도 PASS다.
원시 summary와 정확한 PASS 행은 `full-nextest.log`, 실행·기준·대상14+5개와 기존 실패/보안
검사 추출 근거는 `full-summary.json`, 명령의 종료0은 `full-validation.json`에 있다.
이 시점에는 fmt·Native Clippy가 통과했고 나머지 lint·Native Skia를 이어서 실행 중이다.

## 원격 기준과 원본 commit 출처 재확인

`git ls-remote upstream refs/heads/devel`은0953598로 유지됐고 원 PR7433은
head `d895e51ac5fd1621abb99f0a20aa52034e01e083`·OPEN, 이슈7419도OPEN이다.
추가 upstream rebase나 원격 상태 변경은 수행하지 않았다.

출처 감사에서 현재 기능177650456·검사ae67b089c에 원본 출처 trailer가 없는 것을 확인해
review/오늘할일의 `-x` 표기를 정정했다. 원본52a51c0a9·4905a31d0과 각각 author/author date는
같고, 검사 stable patch-id도 같다. 기능은 당시 devel의 `nested_table_is_overlay` 제외와
`stored_rows.is_some()` 우선 분기를 유지하도록 port해 patch-id가 다르다. 이 둘의 실제
patch 차이와 원본/통합 SHA는 `source-feature-port.diff`, `source-commit-provenance.json`에
연결한다. 원 history를 다시 쓰거나 이미 검증한 source를 바꾸지 않고 출처 기록을 바로잡았다.

## lint6종과 Native library 완료

현재f5a의 lint6종은 모두 exit0이다. 실제 명령·시간·source는 `lint-validation.json`에 있다.

| 검사 | 명령 전체 초 |
| --- | ---: |
| fmt check | 41.264 |
| Native Clippy | 101.303 |
| WASM Clippy | 102.516 |
| workspace build | 122.511 |
| workspace all-target Clippy | 131.954 |
| suite manifest check(base095 고정) | 3.904 |

Native Skia `--lib`도 root3,861 + 기타15/165/2 = **4,043 PASS·실패0·12 ignored·exit0**다.
compile6분17초·root 실행38.26초·명령415.985초다. 그림 placeholder와 직접 PDF 검사는 이어서
실행 중이며 실제 종료 전 통과로 표시하지 않는다.

## 원 이슈의 입력과 비적용 대조군

현재 GitHub7419의 base64를 직접 풀어 frozen 원본8,457바이트와 완전 일치를 확인했다.
원 이슈가 제외한7416과 원 PR의 과거 source9c190efb OVR5 표를 현재 검증 범위/결과로
바꾸어 주장하지 않는다. 현재 보정의 독립 Print 검증 범위는 원본2개·관련 반례11개다.
`original-issue-scope-audit.json`에 입력 SHA/MD5와 제외 범위를 연결한다.

원 이슈의 정상 `treatAsChar=0` 경계는 같은 원본의 두 top-level 속성1→0만 바꾼 대조군으로
기준/후보 Native를 각각 실행했다. 두 render tree는 동일했다. 저장 줄이 있는 실제 HWP/HWPX
대조군과 함께 기존 정상 경로의 출력 보존 증거이며, 새로운 독립 Print 정확도나 정식 신규
렌더링 회귀로 표시하지 않는다. 변형 방법·SHA·명령·출력은 `non-tac-control/verification.json`에 있다.

미게시 본문 초안은 `pr-body-draft.md`에 보존했다. 작성 당시 Native 잔여 검증을 표시했고
완료 후 실제 결과로 갱신했다.3개 일대일 TSV 보류를 명시하며 원격에 쓰거나
게시 예정 raw URL의 접근 성공을 주장하지 않는다.

## 최종 로컬 검증과 판정

Native 그림 placeholder도2/2 PASS·명령427.347초·exit0, 직접 PDF4/4 PASS·명령20.295초·exit0으로
끝났다. 마지막 pipeline의 full/lint/Native3단계와 원래 pipeline의 재개 final 상태가 모두
complete·exit0이다. 실행 중인 Cargo/Rust/WASM 작업은 남지 않았다.

- 검증 sourcef5a에서 집중14+5, 전체10,454, lint6종, Native4,043+2+4를 실제 로그와 종료0으로 대조했다.
- Native/host fresh WASM13문서19쪽씩 최저100%·Print 쪽수 일치·직접 판독 완료다.
  대표26개와 Studio2개 자산은 현재 산출물과 같고 원본/Print26개도 검증 source에 실제 추적된다.
- 마지막 원격 devel 확인은0953598 그대로이며 검토 branch의 원격 ref는 없다.
  기본 작업공간의 사용자 JS는 원문 해시fd499c9e…로 보존했다. 기존 stash/외부 백업도 유지한다.
- 표준 Docker WASM은 사용자 선택대로 미검증이며 host 진단을 Docker 성공으로 표시하지 않는다.
- **최종 판정은 머지 보류**다. 작은 용지·partial 폭 부족·wrapper3개의 base 쪽수 오류로
  전쪽 일대일 TSV가 없고 사용자가 현행 기준 유지를 선택했다. 모든 후보 쪽의100%와 전체 회귀
  성공을 이 조건의 예외로 사용하지 않는다. PR/push·원 이슈/PR 종료·merge·post_merge는 수행하지 않았다.

원시 감사는 `final-verification.json`, 실행 상태/명령은 `pipeline.json`, `final-pipeline.json`,
각 validation JSON/log, 판정은 `final-devel-comparison.json`에 연결한다. 이후 문서-only 기록
commit은 검증 source와 구분하고 코드/검사/입력/asset 불변을 확인한다. 문서-only 변경으로
전체 Cargo 검증을 다시 실행하지 않는다.

## 사용자 보정 요청 후 3문서의 누락 진단 보완

원격 devel을 다시 읽어0953598 그대로임을 확인했다. 따라서 추가 리베이스는 하지 않았다.
기준/후보 source·입력·Print·글꼴·측정 방법은 바꾸지 않았다. 기존 PNG를 재측정한 결과이며
새 source에서 다시 렌더한 결과로 보고하지 않는다.

기존에는 기준 쪽수 차이 때문에 TSV 산출이 중단돼 공통 쪽의 수치도 없었다. 이제 같은
Visual Sweep의`write_silhouette_tsv`에 실제 기준 PNG와 PDF PNG 및 원래 전체 쪽수를
전달해 공통 쪽의 진단 TSV6개를 별도 산출했다. 각 manifest는 원래 쪽수 차이와
`re_review_required`를 유지하며 생산 source·원시 provenance·PNG 해시·누락 쪽을 기록한다.

- 공통8쪽(양 경로 합산)의 후보 점수는 모두100%,0.01%p 이상 하락0이다.
- 작은 용지 Native p1/p2는10.49571/21.71547%, WASM은9.29248/23.50084%다.
- partial 폭 부족 p1은 Native58.58906%, WASM56.69763%다.
- wrapper p1은 Native90.93783%, WASM91.23852%다.
- 기준에 없는 작은 용지 p3, partial 두 입력 p2는 양 경로에서 총6행이다.
  해당 점수·하락폭은`NA`이며 성공이나0점으로 꾸미지 않는다.

원시 위치는 기본 작업공간의 ignored
`output/pr-review/pr7433-current-20261016/rebased-0953598/` 아래다.

- 재현:`venv/bin/python <위 경로>/audit_missing_base_pages.py`(기본 작업공간에서 실행), exit0.
- 로그:`base-mismatch-audit.log`.
- 전체14행의 누락/측정 구분:`base-mismatch-audit/page-comparison.tsv`.
- 요약·생산 source·진단 TSV6개 경로:`base-mismatch-audit/summary.json`.

후보는 이미 독립 Print와 같은3/2/2쪽으로 보정됐으므로 후보의 쪽수를 다시2/1/1로 줄이는
수정은 정답 출력을 훼손한다. 고정된0953598의 누락을 현재 후보 코드로 바꿀 수도 없다.
이 자료는 원인·수치의 미검증을 줄였지만 사용자가 선택한 엄격한 일대일 보류를 해제하지 않는다.
기준 SHA를 변경하거나 검증 조건을 예외 처리하지 않았고 원격 조치도 하지 않았다.
코드·검사·렌더 출력 불변인 진단/문서 보완이므로 전체 Rust 회귀를 반복하지 않았다.
