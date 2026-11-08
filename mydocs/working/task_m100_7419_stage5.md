# #7419 Stage5 — cdef 리베이스 후 통합 검증

## 기준과 범위

- 기준 devel: `cdef0073bb24512daf0622dec5051ac62cb8d57e`(#7948).
- 통합 source: `7fe3b2ce45b996c550397ab588f4e3903cbd9508`. Stage4의13개 commit을 replay했고
  이어 output 전체 제거 지침을 문서 commit으로 반영했다. 기존 수정 source·검사·입력·Print는
  리베이스 전과 바이트가 같으며 upstream의 Square 표 배치 변경을 함께 소비한다.
- 기본 작업공간 devel은cdef, 사용자 Studio JS만 dirty다. 해당 JS는 Native/WASM 렌더러의
  컴파일 입력이 아니며 원문 해시를 보존한다. 검토 worktree에는 단계 기록만 추가한다.
- 원본2개와 반례11개, 총13문서19쪽의 같은 입력·독립 Print·글꼴·96dpi·Print profile·
  compat2024를 사용한다. 기존3문서의 base 쪽수/TSV 보류를 새 base에서도 그대로 판정한다.

## 실행 순서

1. 기본 작업공간 cdef에서 기준 Native Skia CLI와 fresh WASM을 만들고 해시 고정한다.
2. 검토 worktree에서 통합 Native/fresh WASM을 만들고 같은13입력의 전쪽 TSV를 산출한다.
3. 독립 Print 쪽수·90/98%·devel 대비0.01%p 하락·누락을 확인한다. 영향 경계의 Visual Sweep
   review와 standalone overlay를 함께 생성하고 같은 영역을 직접 대조한다.
4. 파생 suite를 준비하고 이번14개와 새 Square 경계의 집중 회귀로 통합을 확인한다.
   해당 선행 검증을 통과한 source에서 전체 nextest·lint6종·Native Skia3종을 순차 실행한다.
5. 같은 검토 기록에 실제 source·명령·종료·시간·증거를 연결하고 PR 승인 준비 여부를 판정한다.
   사용자 PR/push 승인은 별도이며 현재 승인·원격 작업을 수행하지 않는다.

공용 target은 `/home/tsjang/rhwp/target/pr-review` 한 곳, Cargo 잡8개를 유지한다.
WASM은 Docker 미설치 호스트의 locked wrapper `--dev --no-opt`, codegen-units16 경로다.
이를 Docker CI 통과로 표시하지 않는다. 이전10,449건 성공·이전 시각 수치를 새 source의 결과로
재인용하지 않는다. 새 원시 증거는 ignored
`output/pr-review/pr7433-current-20261016/rebased-cdef/`에 보존한다.


## upstream와 이번 보정의 실제 경계 대조

새 `stored_square_top_band`는 TAC 표를 제외하고 Page/Paper 기준의 상단 Square 표와
실제 저장 첫 줄을 요구한다. `typeset::compute_body_wide_top_reserve_for_para`와
`layout/shape_layout`이 같은 예약 높이를 소비한다. 새 `wrap_flow`의 빈 안내 줄 경로도
절대 표 프레임과 앞뒤 실제 저장 줄의 닫힌 사다리를 요구한다. 이번 셀 NO_LS 재조판의
실측 TAC 높이/줄 소유 생산자와 코드 충돌은 없었지만 상위 본문 예약을 함께 소비하므로
정적 비적용 추정만으로 전체 통합 통과를 선언하지 않는다. 같은13입력의 실행·직접 판독과
새 Square 정식 검사, 전체 회귀로 확인한다.


## producer 정합성 검출과 복구

기준 Native를 만든 직후 최초 후보 build가2.1초에 Fresh로 끝났지만, 두 실행 파일의
SHA-256이 `4dda0970e8128d0fa6ffaff9b39b998bcd1dee1b7ca142b993a87e73fa66687f`로 같았다.
후보라는 이름의 실행 파일이 기준의 결함 출력을 그대로 생산한 것을 확인했다. 이 실행과
측정은 `candidate-native-cache-collision-*`로 분리하고 producer 기록을 `invalidated`로
표시했다. 이를 현재 코드의 회귀나 시각 검증 증거로 사용하지 않는다.

공용 target의 작업공간 전환 Fresh 판정과 실제 최종 실행 파일이 일치하지 않은 사례다.
캐시를 지우거나 별도 target을 만들지 않고 양 checkout의 `src/lib.rs`, `src/main.rs`
mtime만 갱신해 현재 checkout의 실제 컴파일을 유도했다. 파일 내용·Git diff·빌드 옵션은
유지했다. 후보 재빌드는203.7초/exit0이고 실행 파일 SHA-256은
`5ada3f4644c56be06ca98e9d0b274fb3c582f3db6ca298a633ab1837e1884cdd`다.
이 생산자로 다시 측정한 Native13문서19쪽은 모두100%, 독립 Print 쪽수 일치였다.
원본/기준 PDF/글꼴/명령은 `rebased-cdef/input-provenance.json`과
`producer-builds.json`, `candidate-native-sweep/`에 고정한다. fresh WASM과 직접 판독,
집중/전체 검증은 별도로 실제 종료 후 기록한다.


## 새 source의 시각 결과

- fresh WASM 실제 compile은209.1초/exit0, WASM SHA-256은
  `7a1a65966c57b3db13d1923b4e4f155c3b0f4e3ac133022c664bc2adb332f4a6`다. 기준 WASM과 다름도 확인했다.
- Native/fresh WASM 각각13문서19쪽의 최저100%, 미달/누락0, 독립 Print 쪽수 일치다.
  같은19쪽의 Native/Print/WASM 내용 영역을 모두 직접 열어 줄바꿈·내용 소유·자식 이월·
  wrapper·후속 본문·여백을 대조했다. 추가 빈 쪽·겹침·내용 손실은 관측하지 않았다.
  엄격 픽셀 차이는 남으며 수식/각주가 없는 입력에서 그 조합을 검증했다고 주장하지 않는다.
- 리베이스 전후 Native/WASM PNG38개가 바이트 동일하다. 비교 가능한 기준10문서의
  backend별12쪽에서0.01%p 이상 하락0, 조건 불일치0이다. base의3개 쪽수 미달은
  2→3,1→2,1→2쪽으로 독립 Print와 일치하도록 복구됐지만, 사용자의 현행 기준 유지에
  따라 전체 일대일 TSV gate는 `re_review_required`다.
- 기준 배치가 첫 쪽수 불일치에서 중단돼 뒤의4입력은 별도 실행으로 모두 확인했다.
  문서 미실행과 쪽수 결함을 구분하며 기존3개 외의 base 누락을 남기지 않았다.
- 직접 판독은 `direct-read.json`, 새 base 비교는 `final-devel-comparison.json`, 실제 PNG
  동일성은 `rebase-png-comparison.json`에 기록한다. raw TSV/log/JSON은 Git에 넣지 않는다.


## 집중 검증과 전체 실행 시작

- 파생 suite 준비exit0. `run-rust-test.mjs`가 현재7419를suite025,7699를suite006에
  배정했다. 생성 harness/manifest는 stage하지 않는다.
- `issue_7419_no_ls_tac_row_growth`: 실제14/14 PASS, nextest0.693초, 빌드 포함319.2초.
- `issue_7699_stored_square_top_band`: 실제5/5 PASS, 명령 전체18.3초.
- 14개·5개의 실제 Summary와 종료0을 확인한 후 같은 source의 전체 nextest를 시작했다.
  신규 sample2개는 `RHWP_SECURITY_SWEEP_SAMPLES_JSON`에 명시 전달한다. 환경변수 없는
  보안 검사의 조기 반환을 새 입력 검증으로 보고하지 않는다.
- 이어서 lint6종과 Native Skia3종을 순차 실행한다. 이 시점에 전체 성공을 선언하지 않는다.


## 현재 Studio 소비 경로 검증

이번 source의 공통 줄/표 소유 결과를 실제 PageRenderer Canvas2D 및 CanvasKit compat/default
software surface가 소비하는5입력/쪽 조합15캡처를 새로 실행했다. 원본2개1쪽, 혼재1/2쪽,
NO_LS 대조군2쪽을 직접 열어 줄바꿈·자식 소유·후속 본문 간격을 대조했다. 캡처 오류0,
선택 backend/profile 불일치0, 초기화 WASM 응답15개 모두 현재 package SHA 일치,
replay hard violation·text fallback·unsupported·runtime render error0이다.

캡처 harness는 제품 PageRenderer를 그대로 호출하며, ignored wrapper에서 입력 root와 정확한
`/samples/<name>` 응답만 frozen input으로 공급했다. root corpus를 변경하지 않았고,
네트워크 관측은 실제 캡처 탭의 초기화 요청 전에 설치했다. source/harness/network/명령과
별도 Vite 종료는 `studio-package-provenance.json`, `studio-initial-responses.json`,
`studio-run.json`, `studio-verification.json`에 기록한다. 기본 작업공간 JS는 건드리지 않았다.

backend 간 엄격 픽셀 동일성10건은 report-only 미달이고 최대 차이율은
0.02082074다. 이를 parity gate 성공이나 독립 Print 일치율로 바꾸어
표시하지 않는다. CanvasKit readiness 선택 corpus의 formal gate도 실행한 것으로 주장하지 않는다.
실제 화면의 줄바꿈·내용·쪽 소속 보존과 픽셀 raster 차이를 구분했다.


## 고정한 cdef 기준 전쪽 비교

| 입력 | devel Native/WASM 최저(%) 또는 쪽수 | 현재 Native/WASM 최저(%) | 판정 |
| --- | --- | --- | --- |
| common-height-4838 | 88.24520 / 87.71732 | 100 / 100 | 하락 없음 |
| mixed-child-storage | 55.80081 / 56.63809 | 100 / 100 | 하락 없음 |
| nested-explicit-break | 47.86180 / 47.04528 | 100 / 100 | 하락 없음 |
| nested-outer-margins | 54.89478 / 55.00565 | 100 / 100 | 하락 없음 |
| nested-same-line | 52.18348 / 52.09133 | 100 / 100 | 하락 없음 |
| nested-surrounding-text | 44.02413 / 45.53546 | 100 / 100 | 하락 없음 |
| nested-width-wrap | 48.91545 / 47.96036 | 100 / 100 | 하락 없음 |
| no-child-storage | 55.80081 / 56.63809 | 100 / 100 | 하락 없음 |
| page-height-28000 | 2쪽/Print3쪽 / 2쪽/Print3쪽 | 100 / 100 | 일대일 대응 보류 |
| partial-nested-width-wrap | 1쪽/Print2쪽 / 1쪽/Print2쪽 | 100 / 100 | 일대일 대응 보류 |
| partial-nested-wrapper | 1쪽/Print2쪽 / 1쪽/Print2쪽 | 100 / 100 | 일대일 대응 보류 |
| tac_nested_two_line_cell_no_lineseg | 72.80910 / 71.48351 | 100 / 100 | 하락 없음 |
| tac_two_line_cell_no_lineseg | 88.24520 / 87.71732 | 100 / 100 | 하락 없음 |

현재 후보는 모든 입력에서 독립 Print 쪽수와 일치한다. 위3개 base의 쪽수 결함을
정상 후보의 회귀로 분류하지 않으며, 사용자의 엄격 일대일 대응 보류는 별도로 유지한다.


## sweep runner와 출력 생산자의 신원 구분

비교 script는 검토 worktree에서 동일하게 실행했다. 따라서 원시 manifest의 `git_head`는
양쪽 모두 runner checkout7fe를 기록한다. 이 값을 기준 렌더러의 source라고 읽지 않는다.
실제 출력 생산자는 `producer-builds.json`의 기본 devel cdef/후보7fe와 frozen 실행 파일·
WASM hash로 연결한다.4종 생산자의 현재 파일 hash와 각13입력 manifest의 실제 producer
hash를 모두 대조했고 일치했다(`producer-sweep-link.json`). 첫 잘못된 후보 Fresh 실행은
무효 기록이며 이 대조에 포함하지 않았다. 원시 manifest 자체를 고쳐 source를 꾸미지 않는다.


## 현재 source 전체 회귀 완료

공용 target·jobs8·test threads12의 release-test nextest가10454 PASS/실패0/45 skipped,
31 slow, exit0으로 종료됐다. 컴파일 로그7분50초, 테스트1230.674초, 명령 전체1703.685초다.
현재14개, 새 Square5개, 명시 전달한 신규 sample2개의 보안 검사도 전체 실행에서 PASS다.
별도 집중 성공을 전체 결과에 합산하지 않았고 실제 Summary를 확인했다.
`full-validation.json`, `full-summary.json`, `full-nextest.log`가 증거다.
이후 lint6종과 Native Skia3종을 계속한다.


## lint6종 완료

| 명령 | 시간(초) | 결과 |
| --- | ---: | --- |
| fmt | 56.8 | exit0 |
| clippy-native | 122.0 | exit0 |
| clippy-wasm | 95.9 | exit0 |
| workspace-build | 152.0 | exit0 |
| clippy-workspace | 143.4 | exit0 |
| suite-check | 2.7 | exit0 |

같은 source7fe에서 실행했고 원시 명령/종료는 `lint-validation.json`에 보존했다.
이후 Native Skia3종을 시작했다.


## Native Skia3종과 최종 판정

| 명령 | 실제 결과 | 전체 시간(초) |
| --- | --- | ---: |
| native-lib | 4개 library 합계4043 PASS/실패0/12 ignored | 488.5 |
| native-picture | 2/2 PASS | 323.2 |
| native-pdf | 4/4 PASS | 21.8 |

세 명령 모두exit0이다. Native lib의 root 결과는3861 PASS/12 ignored이며 나머지
15+165+2 PASS를 합쳐4043이다. 생성 suite/manifest·raw TSV/log/JSON은 커밋하지 않는다.

전체 Rust10454 PASS, 집중14+5 PASS, lint6종, Native Skia3종과 host fresh WASM의
전쪽19페이지 및 Studio15캡처 확인은 완료했다. 검증 중 코드·기대값·baseline을 추가 변경하지
않았다. 기본 작업공간 JS 원문을 보존했으며 source7fe의 렌더 코드 결과에 이후 문서 기록만 더한다.

**판정은 머지 보류**다. 비교 가능한10문서·backend별12쪽의0.01%p 이상 하락0과
후보13문서19쪽100%/Print 쪽수 일치는 확인했지만, base의3문서는 쪽수 결함으로
전쪽 TSV가 없고 사용자 지시 및 현행 governance의 일대일 gate를 충족하지 못한다.
입력 제외·쪽수 기대값 변경·선택 쪽 비교로 이 보류를 해소하지 않는다.
Docker 미설치 호스트의 WASM은 `--dev --no-opt` 진단 결과다. 표준 Docker 최적화 빌드와
최신 GitHub CI는 미검증이며 해당 성공으로 표시하지 않는다. PR/push는 별도 승인 전까지
수행하지 않는다. `final-verification.json`에 완료한 명령과 이 남은 조건을 구분한다.

## 완료 감사의 정상 대조군 확인

원 이슈#7419의 재현 파일은8457바이트·MD5
`0b1ccdfe0631e69d609d0431b73a13af`다. 현재 검증한 원본 일반 입력과 일치한다.
이슈가 명시적으로 제외한 자기 모순 저장 LineSeg 문제#7416까지 해결했다고 주장하지 않는다.

같은 한컴 저장 job에서 보존한 `hancom-saved.hwp`와 `.hwpx`를 cdef와7fe의 frozen
Native CLI로 각각 `export-render-tree --compat 2024` 실행했다.4명령 모두exit0,
각 입력2쪽 모두 base/candidate render tree가 바이트 동일하다. HWPX는 리베이스 전
dca 결과와도 같다. 이는 실제 저장 host의 출력 보존 검사이며, 이 대조군의 독립 Print
전쪽 일치율을 새로 검증했다는 뜻은 아니다.

이전 실패3개도 현재 전체 로그에서 실제 PASS를 확인했다:
`overflow_cell_lines_do_not_grow_partition_5`,
`maintainer_nested_table_lines::missing_stored_rows_do_not_assume_one_inline_row`,
`off_canvas_does_not_grow_partition_4`.
입력 hash·명령·producer·쪽별 hash와 실제 PASS 행은
`rebased-cdef/completion-control-audit.json`에 보존한다. 코드·검사·baseline 변경은 없다.
