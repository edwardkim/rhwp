# #7419 Stage4 — 저장 줄이 다른 TAC 형제의 프레임 소유

## 독립 입력과 수정 전 관측

Stage3의 `59d87766f`에서 새 적용 경계를 확인했다. 기존 수동 XML에 저장 줄을 임의로
덧붙이지 않고, `partial-nested-width-wrap.hwpx`를 한컴2024에서 HWP로 저장한 뒤 HWPX로
다시 저장했다. 실제 저장한 둘째 자식의 LineSeg는 유지하고 부모 host와 첫 자식의 줄만
제거한 혼재 입력을 만든다. 부모 host와 두 자식의 줄을 모두 제거한 대조군도 만든다.
LineSeg 제거 외의 표 폭·높이·문단/글자 속성은 두 변형에서 동일하다.

| 변형 | 독립 Print | 59d Native | 직접 관측 |
| --- | --- | --- | --- |
| mixed-child-storage | 2쪽 | 55.80081% / 68.31124% | 첫 자식이1쪽에서 빠지고2쪽에서 두 자식이 겹침 |
| no-child-storage | 2쪽 | 100% / 100% | 자식1은1쪽, 자식2와 뒤 본문은2쪽 |

두 Print의 페이지별 PNG가 바이트 동일하다. 저장 줄 혼재를 화면 차이의 정답으로
간주하지 않는다. 원본 저장 job `94075ab2-d378-4fd3-9eee-0050640e0f9b`, HWPX 저장 job
`7a141687-fbad-416c-8ae5-a5cee8d63e2d`, Print job은 각각
`d98accb7-7e02-4387-90c7-06ff11afa887` / `ba87fd38-33b3-4a3d-9c95-c447e1bb74da`다.
파일·SHA·제거한 문단/줄 수와 실제 한컴 버전/Print 경로는
`output/pr-review/pr7433-current-20261016/regression-fix/mixed-storage/{jobs,input-generation}.json`에 있다.

## 생산과 소비의 불일치

`composer/nested_table_frames.rs::recompose_cell_lines_with_nested_tables`는 현대 NO_LS
host를 재조판할 때 NO_LS 자식만 `FrameGeometry.inline_table_heights`에 넣었다.
실제 줄 구성은 저장 줄 자식을 포함한 전체 controls로 만든다. 그런데 셀 유닛의
`frame_owns_multiple_inline_tables` / `frame_owns_table_bands`와 partial paint의
`frame_owns_inline_table`은 일부 자식만 소유한다고 읽어 서로 다른 예약/배치를 한다.

`실측 형제 목록 → 재조판 줄/높이 → HeightMeasurer 상세 줄 → CellUnit 물리 줄 컷 →
table_partial의 같은 줄 소유·앵커`가 같은 전체 TAC 형제 목록을 소비해야 한다.
저장 줄이 있는 자식의 내부 줄을 다시 나누는 수정은 필요하지 않다.

## 보정과 비적용 계약

- 새 프레임의 시작 조건은 유지한다: 현대 비Native/비legacy NO_LS host에 NO_LS TAC 자식이
  하나 이상 있어야 한다. 모두 저장 줄인 일반 host를 새 경로로 넓히지 않는다.
- 그 시작 조건이 성립하면 같은 host의 **모든 TAC 표** 실측 높이를 기존 프레임에 전달한다.
  원본 자식의 LineSeg·controls 식별자는 보존하고 파생 프레임의 소유/본체 높이만 공유한다.
- Native HWP5·HWP3와 유효 저장 host의 기존 수용 조건은 유지한다. 문서 ID·수치 예외,
  clamp·겹침 숨김·허용치 변경을 추가하지 않는다.
- 새 입력은 먼저 Native/fresh WASM 전쪽90% 이상과 독립 Print 직접 판독을 확보한다.
  그 뒤 관계 검사와 자격을 충족한 완료 쪽의 독립±2px 보호를 보완한다.
  기존11문서와 Native 정상 대조군도 보정 source에서 다시 확인한다.

59d의 전체 검증은 새 실패 확인 후 SIGINT로 중단했다(pipeline exit130, test summary 없음).
이 실행을 제품 test FAIL이나 전체 PASS로 표시하지 않는다. 코드 보정 후 선행 경계/시각 판독을
마친 뒤 최종 전체·lint·Native Skia 묶음을 다시 실행한다.

## 보정 후 실제 검증

렌더링 source는 `dca310c06036c7f3fa7ca30cce74c1704f1169f2`, 기준 devel은
`4662c069e557e928ed7968d061121fde90b1f027`이다. 사용자 요청으로 remote fetch,
`git ls-remote`와 GitHub REST head를 다시 확인했지만 기준이 동일하여 재리베이스하지 않았다.
주 작업공간의 로컬 devel만1커밋 뒤처져 있으며 사용자 Studio JS 변경은 보존했다.

| 검사 | 실제 결과 |
| --- | --- |
| Native Skia 실행 파일 빌드 | exit0, 117.01초 |
| fresh WASM locked host dev 빌드 | exit0, 149.76초. Docker 미설치 호스트의 진단 경로이며 Docker CI 통과로 표시하지 않음 |
| mixed-child-storage Native/fresh WASM | 각각2쪽 모두100%, Print2쪽과 일치 |
| no-child-storage Native/fresh WASM | 각각2쪽 모두100%, Print2쪽과 일치 |
| 기존11문서15쪽 Native/fresh WASM | 양쪽 전쪽100%, 독립 Print 쪽수 일치, 직접 대조 완료 |
| 실제 저장 host 정상 대조군 | `hancom-saved.hwpx`의2쪽 render tree가59d와 dca에서 바이트 동일. 이 비교만으로 독립 Print 정확도를 주장하지 않음 |
| 갱신한 정식14검사 | `run-rust-test.mjs` dev profile, 14pass/실패0, exit0 |
| 같은14검사 실행 파일의 출력 생산자 전후 교체 | 59d:12pass/2fail → dca:14pass/0fail |

전후 실패2건은 혼재 자식의 쪽 소유 관계와 독립 Print 원점±2px다. 수정 전 첫 자식이
1쪽 대신2쪽에 있었고 둘째 표 제목 원점이69px 이상 밀렸다. 전부NO_LS 대조군은 전후 모두
통과한다. 테스트 실행 파일은 dca의 파서를 사용하고, 렌더링 생산자 CLI를 런타임에
59d/dca 고정 바이너리로 바꿨다. 두 source 사이 파서는 변경되지 않았다.
실행 파일·생산자 SHA와 실패 assertion은 `before-after.json`, `before-after-{before,after}.log`에 있다.

초기 집중 실행은 검사 소스 크기 변경 후 stale generated suite를 사용하여0건/exit4로 끝났다.
이 결과를 통과로 집계하지 않았다. `node scripts/rust-test-suite-manifest.mjs --prepare`로
파생 목록을 재생성한 뒤 같은 runner로14건을 실제 실행했다. 파생 harness/manifest는 커밋하지 않는다.

### 직접 판독과 정식 검사

Native·독립 Print·fresh WASM의 같은19쪽 영역을 직접 대조했다. 새 혼재2쪽과 대조군2쪽은
첫 자식의1쪽 표시, 둘째 자식의2쪽 이어받기, 후속 본문 순서·간격을 확인했다.
겹침·누락·중복·불필요한 빈 쪽은 관측하지 않았다. 100%는2px 관용 실루엣이며 글리프 윤곽의
엄격 픽셀 일치를 뜻하지 않는다. 이 입력에 없는 수식·각주 조합을 검증했다고 주장하지 않는다.
대표 Visual Sweep **review와 standalone overlay**를 함께 생성·판독하고 검토 자산에 보존했다.

새 입력2개와 독립 Print2개를 원래 검사와 같은 경로에 추가했다.
[생성·SHA 기록](../../tests/fixtures/issue7419-boundaries/README.md),
[정식 검사](../../tests/cases/issue_7419_no_ls_tac_row_growth.rs)를 따른다.
관계 검사는13개이며 제목·후속 본문의 독립±2px 검사1개가 완료18쪽의 핵심 위치를 보호한다.
작은 용지1쪽은 제목만 있으므로 별도 위치 핀을 추가하지 않았다.

모든 빌드는 기본 작업공간의 `target/pr-review`와 `CARGO_BUILD_JOBS=8`을 사용했다.
Native는 `cargo build --locked --features native-skia --target-dir <공용target> --bin rhwp`,
WASM은 `CARGO_TARGET_DIR=<공용target> CARGO_PROFILE_DEV_CODEGEN_UNITS=16
scripts/wasm-pack-locked.sh --target web --out-dir <증거>/candidate-wasm --dev --no-opt`다.
Visual Sweep은 `--rasterizer png --compat 2024 --dpi 96 --embed-fonts full --page-workers 2`로,
전쪽 점수는 `--silhouette-only`, 직접 판독용 review/overlay는 일반 모드의 별도 output에 산출했다.
집중 검사는 `node scripts/run-rust-test.mjs issue_7419_no_ls_tac_row_growth --
--cargo-profile dev --target-dir <공용target> --test-threads 12 --no-fail-fast`다.

원시 결과는 ignored `output/pr-review/pr7433-current-20261016/mixed-storage-fix/`에 있다.
`native-build.json`, `wasm-build.json`, `mixed-visual-evidence.json`, `final-direct-read.json`,
`focused.json`, `before-after.json`, `all-saved-comparison.json`에서 source·명령·해시를 확인한다.
이전 source의 증거는 `regression-fix/`에서 그대로 보존한다. 최종 release-test 전체·lint·Native
Skia3종은 이14건 집중 결과와 구분해 실제 종료 뒤 갱신한다. 현재 단계에서 전체 통과로 표시하지 않는다.

기준 devel의 기존3문서 쪽수 차이로 생기는 전쪽 일대일 TSV 누락 보류는 유지한다.
독립 Print와 일치하는 페이지 복구와 현행 gate의 충족 여부를 구분하며, 높은 점수로 면제하지 않는다.
PR/push는 사용자 별도 승인 전까지 수행하지 않는다.


## 최종 전체 회귀 완료

회귀 추가 head `9f464aa2c1e19bb7f7b852db0b65e8515effad48`에서 release-test 전체가
**10,449pass/실패0/45skip**으로 끝났다. 최종 summary는713.108초, 빌드11분18초,
명령 전체1398.2초다. 이전 Native 소유·셀 넘침·off-canvas 실패3개와 현재14개도 모두 통과했다.
그 뒤 문서-only `11579b9b6`은 Rust·검사·Cargo 변경이 없다. 새 전체 실행을 추가하지 않고
기록한 source의 결과를 보존한다. 이어서 필수 lint와 Native Skia3종을 같은 공용 target에서
순차 실행한다. 원시 로그 `full-nextest.log`, exit/head 기록 `full-validation.json`,
검사별 확인 `full-summary.json`은 위 `mixed-storage-fix/`에 있다.


## 추가 upstream 변경과 리베이스 전 고정

사용자 재확인 시점에 원격 devel이 `cdef0073bb24512daf0622dec5051ac62cb8d57e`
(#7948 Square 표 보정)로 바뀌었다. fetch 뒤 이 head를 기준으로 리베이스한다.
완료된10,449건 전체 성공과 fmt/Native Clippy/WASM Clippy 성공은 **리베이스 전** 결과다.
당시 진행하던 workspace build에서 소유 process group3298153에 SIGINT를 보내 pipeline
exit130으로 정리했다. 해당 Cargo 명령의 개별 exit는 기록되지 않았으며 통과/제품 실패로 세지 않는다.
남은 workspace Clippy·suite 확인·Native Skia3종은 미실행이다. 새 base와 통합 head에서
영향 경계·Native/fresh WASM·필수 게이트를 다시 판정하며 과거 성공을 소급 적용하지 않는다.
증거는 `stop-for-rebase.json`, `full-summary.json`, `{lint,full}-validation.json`에 보존한다.


## cdef 리베이스 완료와 후속 정리 지침

13commit을 `cdef0073bb24512daf0622dec5051ac62cb8d57e` 위로 replay하여 통합 head
`02dfe3a748258be5206cad2c7de96da6b822be4e`를 만들었다. 충돌은 오늘할일 한 파일뿐이며
upstream #7699 기록과 이번 #7433 기록을 함께 보존했다. replay한 기존 변경 경로 중 오늘할일만
바뀌었고, 기존 수정 source·검사·입력·Print·시각 자산의 바이트는 그대로다. 새 upstream의
Square 배치 변경을 소비하는 영향 검증은 별도로 다시 수행한다.

기본 작업공간 devel도375aff294→cdef로 fast-forward했다. upstream도 변경한 사용자 dirty
`rhwp-studio/public/rhwp.js`는 stash와 `~/.cache/rhwp-pr7433-user-preservation/20261016T092233Z/`에
임시 보존한 뒤 같은 바이트로 복원했다. SHA-256
`fd499c9eb514018d2063c8eef1e58ec892c734d512cf5e88ab97bd9e5d47a0a3`가 일치하며
사용자 보존 stash는 유지한다. 기본 devel은 upstream과 같고 검토 branch는0behind/13ahead다.
`rebase-latest.log/json`, `rebase-range-diff.log`, `main-devel-sync.json`이 실제 실행 증거다.

사용자의 리베이스 후 지시로 `post_merge.md`7번/7.7과 모 문서의 요약을 수정했다.
필수 원본·Print·TSV·review/overlay·재현 증거를 output 밖에 보존한 뒤 저장소 루트의
**output 디렉터리 자체를 통째로 제거**하고 부재를 확인해야 완료다. 다른 활성 작업 등으로
전체 삭제가 안 되면 정리 미완료로 보고한다. 현재 진행 중인 output은 삭제하지 않았다.
요청한 `/Users/tsjang/rhwp-7445-completed-20261009/` 경로는 이 Ubuntu 호스트에 없으므로
현재 검토 worktree의 대응 canonical 문서에 반영했다.
