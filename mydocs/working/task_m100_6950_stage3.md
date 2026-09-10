# #6950 Stage 3 — 영향 검증·PR 준비

- Issue: [#6950](https://github.com/edwardkim/rhwp/issues/6950)
- 시작: 2026-09-10 메인테이너 진행 승인.
- 선행: [Stage 2](task_m100_6950_stage2.md) §9의 시각 판정 통과·작은 용지 실험 범위 제외.
- Stage 2 확정 커밋: `390d81e74d77a1541ab838b204263a07a2d0a972`.
- 작업 브랜치: `task_m100_6950`.
- 상태: **시각 승인·전체 nextest 9,414/9,414·로컬 Render Diff 통과 후 PR 제출 준비**.
  §28~30에서 잔여 회귀를 해소했다. 메인테이너가 커밋·push·Open PR 및 최신 devel 충돌
  해결을 승인했다. §31의 통합 후보 재검증·최종 lint·Native Skia를 완료하기 전 원격 제출하지 않는다.

## 1. 최신 base와 병합 사전 검사

`git fetch upstream devel`로 원격 기준을
`13c92feb67d2bf0ae62349f41c5f5cd83845a4a5`에서
`0d36da4096fab2fef0e0a654e466fa449330d7a6`으로 갱신했다.
당시 `HEAD...upstream/devel`은 작업 측 24 / 원격 측 44커밋 차이였다
(이후 Stage 2 문서 확정 커밋 1개 추가). 로컬 `devel`은 변경하지 않았다.

`git merge-tree --write-tree HEAD upstream/devel`은 작업트리를 바꾸지 않는 사전 검사다.
Stage 2 문서 확정 전 HEAD에서 다음 결과를 확인했다.

- 소스 `src/document_core/queries/rendering.rs`,
  `src/renderer/layout/paragraph_layout.rs`는 자동 병합. 의미상 회귀 없음의 증명은 아니다.
- 유일한 충돌: `mydocs/orders/20260910.md`의 add/add.
  로컬은 #6950 기록, 원격은 PR #6962 기록이다.
- 제안: 하나의 날짜 제목 아래 양쪽 이슈별 기록을 모두 보존한다. 원격의 과거 체크 상태를
  추정으로 변경하지 않는다. 충돌 해결 방침 승인 후 최신 base를 작업 브랜치에 병합하고 검증한다.
- `.github/workflows/ci.yml`은 현재 작업 HEAD와 원격 devel 사이에 차이가 없다.

사전 점검 당시 실제 merge는 시작하지 않았다. 이후 승인에 따른 실행은 §4에 기록한다.

## 2. 실행 환경

- Linux/WSL: 16 logical CPUs, 메모리 31GiB 중 available 약29GiB, 디스크 가용373GiB.
- `cargo-nextest 0.9.137` 확인. 실행 중 Cargo/Rust 작업은 점검 시 없었다.
- 기존 review worktree: `/home/edward/mygithub/rhwp-6950-review` (tracked clean).
- 고정 공유 target: `/home/edward/mygithub/rhwp-shared-review-target`. 새 target이나 worktree는
  만들지 않았고 기존 캐시·Studio 산출물은 삭제하지 않았다.
- `docker info`는 WSL 통합 CLI를 사용할 수 없다고 반환했다. `/var/run/docker.sock`과
  Docker Desktop의 WSL CLI 경로가 없다.
- 호스트 `docker.exe info`도 `dockerDesktopLinuxEngine` named pipe가 없어서 실패했다.
  PATH 문제만이 아니라 Linux engine 접속도 현재 불가능하다. Docker Desktop 기동 및
  해당 Ubuntu WSL 통합 확인이 필요하다. 대체 native WASM 빌드는 실행하지 않았다.

## 3. 선행 조건 충족 후 검증 순서

1. 승인된 문서 병합 방침으로 최신 devel 반영 → code head 고정 → 기존 review worktree 정합.
2. review 전용 파생 suite 준비 → fmt → native Clippy → WASM Clippy → workspace build →
   all-targets Clippy → manifest 및 source unit-tier 검사. Cargo는 공유 target에서 순차 실행한다.
3. 해당 code head의 focused 검사와 release-test 전체 nextest, Native Skia 3종.
   현재 자원 기준 Cargo build 2 jobs, nextest 8 threads로 시작하고 실행 시간을 기록한다.
4. 새 fixture `samples/hwpx/20260909-para-table.hwpx`에 대해 필수 코퍼스 래칫·명시적
   보안 검사 입력을 확인한다. PDF 쪽수 원장 등록 여부도 확인한다. 실패를 숨기기 위한
   baseline 갱신은 하지 않는다.
5. Docker 표준 WASM 빌드 및 필요한 실제 WASM 확인. 원본 시각 판정 범위를 유지하며
   작은 용지의 추가 구현·대체 실험은 하지 않는다.
6. 최종 보고서·PR 본문 초안과 정확한 결과/미검증 범위 제시. push·PR 생성은 별도 승인.

현재까지 새로 실행한 것은 Git/환경 사전 점검과 Stage 2 문서 확정이다.
최신 base와 합쳐질 소스가 달라지므로 이전 code head에서 긴 전체 검증을 먼저 반복하지 않는다.

## 4. 승인 후 통합과 검증 실행

- 시스템 시각 `2026-09-10T09:04:15+09:00` 확인. 메인테이너는 다음날이라고 안내했으며,
  시스템에서 확인된 날짜와 별도 업무 날짜의 차이를 알리고 현재 날짜 문서를 유지했다.
- `docker info`가 서버 `29.7.2`를 반환했다. 앞선 Docker 접속 장애는 해소됐다.
- fetch 결과 원격 devel은 여전히 `0d36da4096`이었다.
  Stage 3 사전 기록을 `33bb9659a`로 보존한 뒤 승인된 방침으로 병합했다.
- merge `3e29223c9`: 오늘할일은 하나의 제목 아래 #6950·PR #6962 기록을 모두 보존했다.
  원격 PR 기록의 체크 상태를 임의 갱신하지 않았다. 소스 충돌은 없었다.
- review harness 준비 후 `cargo fmt --all`이 신규 테스트 마지막 두 함수의 포맷을 변경했다.
  동일 포맷을 주 브랜치에 `44a9040a5`로 반영했다. 테스트 조건·단언 변경은 없다.
- 첫 native Clippy에서 `src/renderer/layout.rs`의 `let_and_return` 1건을 발견했다.
  반환용 중간 변수만 제거한 `e2436aa2d`로 정정했다. 검사를 약화하거나 lint allow를 추가하지 않았다.
- 작업 중 최초 로그 경로가 없어 tee 저장에 실패했고, review 전환이 포맷 변경 때문에 막혔는데
  이전 head의 Clippy를 한 차례 더 실행했다. 해당 실행은 같은 오류로 실패했고 통과로 세지 않는다.
  review의 작업 소유 포맷 변경은 주 브랜치에 보존된 것을 확인한 뒤 정리하고, 후보로 전환했다.
- 최종 검증 소스: `e2436aa2dfc55994740c4f9fe4d4cc3db67cd657`.
  review worktree의 tracked diff 0, fmt check 성공 후 검증 묶음을 시작했다.
  파생 manifest: 1,243 sources / 5,279 static attrs / 28 suites + 20 exceptions.
- native Clippy 재실행 통과(로그 `clippy-native-final.log`, Cargo 표시53.71초).
  이후 gate는 `output/6950/stage3/validate.sh`에서 순차 실행하며 실패하면 중단한다.
  원시 로그는 같은 output 폴더에만 보존하고 PR에 포함하지 않는다.
- WASM Clippy(51.51초), workspace build(1분37초), all-targets Clippy(1분41초),
  manifest·unit-tier 검사도 통과했다. 이는 전체 회귀·Docker WASM 빌드를 대신하지 않는다.
- 전체 nextest 최초 명령은 `-j 2`가 `--test-threads` 별칭이라 8 threads 설정과 중복되어
  테스트 시작 전 종료했다. `CARGO_BUILD_JOBS=2`로 빌드 동시성을 분리해 재실행했다.
  security 입력은 신규 fixture 1개를 JSON 배열로 명시했다.
- 병합 후 CLI SHA-256은 `21414fbfe138fcd959062150af29aa25b10341795bb59ac76b99a3dc55700384`.
  `stage3-merged` 원장 및 기존 flow 검사에서 본문 하단580.1px / 표 상단603.8px로 통과했다.
  원본 3쪽 SVG를 `output/6950/stage3-merged-svg/`로 내보내고 이전 판정본과 바이트 동일,
  `overflowCellLines=0`을 확인했다. 별도 작은 용지 실험은 하지 않았다.
- 새 fixture의 `info --json`: HWPX / Hancom Office 2024(13.0.0.3622) / pageCount3 /
  printMethod4 / printMethodImpliesNup=true. 기존 PDF는 Hancom 생성본 3쪽, A4다.
  저장된 모아찍기 설정 때문에 `local_validation.md` §4.3.1 규칙상 쪽수 원장 등록 대상에서
  제외한다. MCP 출력은 앞서 기록한 one-up 설정이며, 현재 양쪽 3쪽 실측을 원장 통과와 혼동하지 않는다.

## 5. 전체 회귀 결과와 대표 base 대조

### 5.1 실행 결과

- 후보: `e2436aa2dfc55994740c4f9fe4d4cc3db67cd657`, review tracked diff 0.
- 실행: `CARGO_BUILD_JOBS=2 RHWP_SECURITY_SWEEP_SAMPLES_JSON='["samples/hwpx/20260909-para-table.hwpx"]'`
  환경에서 `cargo nextest run --locked --cargo-profile release-test
  --target-dir /home/edward/mygithub/rhwp-shared-review-target --tests --test-threads 8 --no-fail-fast`.
- 컴파일 Cargo 표시12분48초. 테스트 실행361.527초. time wall1092.84초.
- **9,401개 실행: 9,378 passed / 23 failed / 46 skipped**, 종료코드100.
  46개는 실행 수에 포함하지 않는다. 2개 slow 알림은 자체로 실패 판정이 아니다.
- #6950 신규 검사15개와 명시적으로 전달한 신규 샘플의 3종 보안 검사는 통과했다.
  그 통과를 전체 회귀 통과로 확대하지 않는다.
- nextest0.9.137은 필수 최소버전은 충족하나 권장0.9.140보다 낮다. JUnit의 `report-skipped`
  키 무시 경고가 있었다. 이 경고와 아래 실제 단언 실패는 구분한다.

| 실패 축 | 검사 수 | 대상 |
| --- | ---: | --- |
| 기존 직접 회귀 검사 | 12 | #2439, #3738, #6854, #6267, #6797 두 검사, #5941, #1789, #2097, #6025, #6764, synam001 |
| 기존 코퍼스 겹침 원장 | 7 | text_overlap partitions 1/10/12/14/7/11/3 |
| 기존 코퍼스 셀 넘침 | 1 | overflow_cell partition3: issue6764 문서 5줄 신규 |
| 기존 코퍼스 쪽 밖 배치 | 1 | off_canvas partition12 |
| 기존 한컴 쪽수 원장 | 1 | oracle partition11: 재난 별표 2→3쪽 |
| 신규 fixture IR 왕복 원장 | 1 | 신규 para-table HWPX의 raw_header_extra 3경로: 254/95/38 |

총23은 실패한 **검사 수**이지 서로 다른 결함 또는 문서 수가 아니다.
신규 fixture IR 왕복 차이는 별도 원인 대조가 필요하며 현재까지 허용된 정규화인지 확정하지 않았다.
baseline을 추가하거나 기대값을 완화하지 않았다.

### 5.2 최신 devel과 동일 환경 CLI 대조

전체 실행이 끝난 후 기존 review worktree만 `upstream/devel`로 전환해 CLI를 직접 빌드했다.
대조군 SHA: `0d36da4096fab2fef0e0a654e466fa449330d7a6`, 빌드1분10초.
주 작업 브랜치·사용자 문서·샘플을 바꾸거나 별도 작업 브랜치/worktree를 만들지 않았다.

동일 샘플에 `rhwp info --json`과 `rhwp layout-anomaly --json`을 실행했다.
명령·SHA·실행 파일 해시·원문 결과는 로컬 `output/6950/stage3/compare-existing.mjs`와
`existing-base.json`, `existing-candidate.json`에 남겼다.

| 실제 저장소 문서 | devel 쪽수→후보 | devel 글자 겹침→후보 | 대조 판정 |
| --- | --- | --- | --- |
| `samples/issue2439/issue2439_repeat_table_overlap.hwp` | 10→11 | 0→18 | 쪽수·겹침 악화 |
| `samples/task2097/21298295_byeolpyo5_disaster.hwp` | 2→3 | 0→0 | 쪽수 악화 |
| `samples/issue6854/70833-electrical-safety-rule-regulatory-analysis.hwp` | 18→19 | 15→10 | 겹침 건수는 감소하지만 쪽수 악화 |
| `samples/hwp_table_test.hwp` | 3→3 | 0→2 | 겹침 악화 |

이 네 건은 용지나 IR을 변조하지 않은 기존 파일의 A/B 결과로, 이번 후보의 영향이 확인됐다.
나머지 실패 모두가 동일 원인이라는 뜻은 아니며 23개 전부를 base에서 재실행한 것도 아니다.
원본 #6950의 시각 판정 통과를 취소하는 것이 아니라, 그 구현의 기존 문서 영향 게이트가 실패한 것이다.

### 5.3 진행 경계와 다음 권고

1. 기존 점유 영역·다음 문단 흐름·표 첫 조각/캡션 계약과 이번 확정 배치 전달 사이에서
   책임이 빠지거나 중복된 지점을 대표 회귀로 좁혀 조사한다. 실패마다 샘플 예외를 추가하지 않는다.
2. 원본 #6950의 승인된 조판은 유지하면서 기존 정상 동작을 복구하는 수정안을 확정한다.
   신규 샘플 IR 왕복 원장은 renderer 회귀와 분리해 기존 base에서도 발생하는지 확인한다.
3. 정정 후보의 focused·전체 회귀가 통과한 뒤 Native Skia3종·Docker WASM·실제 WASM 확인을
   이어간다. 변경된 code head의 Rust lint도 다시 확인한다.

이번에 제외한 **작은 용지 높이 합성 실험은 재개하지 않는다**. 기존 문서 회귀를 그 범위 제외
결정으로 무시하거나 신규 자식 이슈로 자동 분리하지 않는다. 이번 실행에서는 lint의 반환 변수와
포맷만 정정했으며 위 회귀의 구현 수정은 하지 않았다.

Native Skia3종, Docker WASM, 실제 WASM smoke는 **미실행**이다. Docker는 연결되지만 선행 전체
회귀가 실패해 다음 gate로 진행하지 않았다. 따라서 최종 완료보고·push·PR 생성·병합은 보류한다.

대조 후 review worktree를 후보 `e2436aa2d`로 복귀하고 CLI도 다시 빌드했다(1분04초).
tracked diff0·manifest 일치를 확인했다. 복귀 빌드의 CLI SHA-256은
`ed255224e0eb41965aae745b67aaac39d739dfba4f04634eefaafdf3f5dc2b80`이며 최초 후보 바이너리와
바이트 동일하다고 주장하지 않는다. 대조 증적의 바이너리 해시는 각각의 JSON에 고정했다.
주 작업 브랜치, review worktree, 공유 CLI를 대조군 상태로 남기지 않았다. Studio pkg는 교체하지 않았다.

## 6. 개별 회귀 정밀 조사 — 첫 번째 묶음

메인테이너 지시: 실패를 하나씩 확인하고 변경된 규칙이 기존 조판에 미치는 영향을 추적한다.
이번 절은 **진단**이며 수정 구현·기대값 변경·완료 판정이 아니다. 기존 23개 실패 중
#2439, #6267, #6797의 두 검사, synam001 검사와 연결되는 네 가지 계약을 우선 조사했다.
나머지 실패를 같은 원인으로 묶거나 해결됐다고 보지 않는다.

### 6.1 대조 방법과 근거의 수준

- 대조군: `0d36da4096fab2fef0e0a654e466fa449330d7a6`.
- 후보: `e2436aa2dfc55994740c4f9fe4d4cc3db67cd657`.
- 기존 review worktree에서 순차 빌드하고 같은 파일에 `dump`, `dump-pages --json`,
  `dump-extents`, `layout-anomaly --json`을 실행했다. 입력의 용지·표 속성·IR은 변경하지 않았다.
- 후보의 배치 결정 직전에는 임시 stderr 출력만 넣어 `current_height`, 문단 시작 원점,
  `resolved_host_placement`, 활성 배제 영역을 기록했다. 제품 분기·계산은 바꾸지 않았다.
  계측 패치는 `output/6950/stage3/diagnostic-only.patch`로 보존한 뒤 review 소스에서 제거했다.
- 원시 증적: `output/6950/stage3/trace-{base,instrumented}/`. 각 폴더의 `metadata.json`에
  소스·바이너리·입력 SHA-256이 있다. 재실행 도구는 `trace-regression.mjs`다.
  `instrumented`의 `DIAG6950 page`는 `st.pages.len()` 원값이며 페이지 인덱스로 해석하지 않는다.
- 아래 표의 좌표는 `dump-extents`의 **종이 기준, 96 DPI, px**다. 계측 로그의
  placement는 **단 기준 px**이므로 본문/단 원점을 더한 뒤 비교한다.
- 이번에 한컴 PDF를 새로 생성하거나 새로운 인간 시각 판정을 받은 것은 아니다.
  기존 보호 근거와 현재 devel의 정상 결과, 후보의 변화, 실행 분기를 교차 확인했다.

### 6.2 R1 — #2439: 앞 표의 점유가 새 확정 배치에서 누락됨

입력은 기존 회귀 fixture `samples/issue2439_zero_offset_coanchored_float_exclusion.hwp`다.
원래부터 축소 재현 fixture이며 이번에 만든 문서가 아니다. A4, 문단 0에 글과 두 표가 있다.
두 표는 non-TAC / Para / Top / TopAndBottom / RowBreak이며 첫 표 offset은 0,
둘째 표는 3,000 HU(40px), 각각 outer-top은 283 HU(3.773px)다.

| 대상 | devel | 후보 | 변화 |
| --- | --- | --- | --- |
| 첫 표 A, ci=2 | 136.0..210.7 | 136.0..210.7 | 불변 |
| 다음 표 B, ci=3 | 214.5..294.5 | 176.0..256.0 | A와 실제 교집합 34.7px |
| host 글줄, pi=0 | 298.3..316.9 | 259.8..278.5 | 잘못 짧아진 스택 뒤에서 재개 |

원인 경로는 다음과 같이 확인됐다.

1. A는 offset=0이어서 새 placement 대상이 아니다. 기존 경로가 A를 배치하고
   `current_height=78.44`까지 진행하지만 `visible_float_exclusions`에는 넣지 않는다.
2. B에는 `from_stored_host`가 적용돼 단 기준 `table_top=43.7733`,
   `occupied_bottom=127.5467`을 만든다. 이때 `zones=[]`여서 `clear_occupied_bands`는
   아무 것도 바꾸지 못한다.
3. 기존 `place_table_with_text`는 같은 문단의 앞 표가 있으면 자연 상단과
   `current_height + outer_top` 중 큰 값을 썼다. 이 경우 단 기준 하한은
   `78.44 + 3.7733 = 82.2133`이며 종이 기준 214.5px다.
4. 새 `Some(placement)` 분기는 그 하한 계산보다 먼저 확정 상단을 채택한다.
   layout도 같은 확정 값이 있으면 기존 배제·호스트 보정을 건너뛴다.
   따라서 예약과 출력은 서로 일치하지만 **둘 다 앞 표의 점유를 놓친 값**을 쓴다.

계보: `261af62767`의 기존 회귀 검사, `8e80671b63`의 co-anchored flow 보존과
[#2439 완료 보고](../report/archives/task_m100_2439_report.md)를 확인했다.
#6950의 `8bea53b5f` hunk에서 기존 stacking 하한보다 `Some(placement)`가 우선하도록
변경된 것을 확인했다. 이는 책임이 이동한 코드 지점이며, 모든 중간 커밋을 실행해 최초 실패
커밋을 이분 탐색한 결과라는 뜻은 아니다. 이후 `fa28cdbba`의 밴드 회피 추가도
목록에 없는 A를 복구하지 못한다.

실제 문서 `samples/issue2439/issue2439_repeat_table_overlap.hwp`에서도 같은 경로를 확인했다.
pi=12의 첫 표는 양쪽 181.1..378.4px인데 둘째 표는 385.9..583.1에서
277.6..474.8px로 올라와 실제 100.8px 겹친다. 계측에서 앞 표 뒤 현재 높이는344.3467,
둘째 확정 상단은239.7867, 배제 목록은 비어 있다. §5의 10→11쪽·겹침0→18 전체를
이 한 위치만으로 모두 설명했다고 주장하지는 않는다.

**보호 불변식**: 같은 흐름에 참여하는 선행 표의 점유 하단과 여백은 저장 offset이 0인지와
무관하게 후행 표의 최소 위치에 반영돼야 한다. 예약 결과를 하나로 전달하는 것만으로는
충분하지 않고, 전달 전에 기존 흐름의 제약을 빠짐없이 합쳐야 한다.

### 6.3 R2 — #6797: 선행 표의 작은 이동이 후속 회피를 무효화함

입력: `samples/issue6797/156160455-social-pig-farm-income.hwp`, 7쪽 pi=70/71.

| 대상 | devel | 후보 |
| --- | --- | --- |
| pi=70 선행 표 | 181.5..294.9 | 188.1..301.6 |
| pi=70 본문 두 줄 | 122.7..140.1 / 148.7..166.1 | 불변 |
| pi=71 후속 표 | 296.8..462.5 | 174.8..340.5 |

새 배치의 pi=70 `para_start=36.6933`, `anchor_y=43.36`, `table_top=108.76`이다.
본문 글줄은 움직이지 않았으나 표와 그 밴드는6.6667px 내려갔다. 선행 표의 정확한 원점을
어떤 단계가 소유해야 하는지는 추가 설계 검토 대상이며, 이 차이를 임의 상수로 빼지 않는다.

후속 영향은 소스 조건으로 확인된다. 기존 #6797/#6798의 빈 host 표 회피는 유효한 저장
앵커가 앞 표의 밴드 하단 이상일 때만 그 앵커를 사용한다
(`stored_top + 0.5 >= zone.bottom`). 기존 밴드 하단은 약296.8px로 저장 앵커와 맞지만,
후보에서는 약303.4px로 내려가 조건이 거짓이 된다. 뒤 표가 원래 미보정 위치174.8px에 남는다.

현재 순서 부족분은 `301.6 - 174.8 = 126.8px`이며 실제 두 표의 교집합은113.4px다.
두 수치를 혼동하지 않는다. 테스트를 통과시키려고 저장 앵커 허용 오차0.5를 늘리면,
선행 표 원점의 불일치를 가릴 뿐이다.

**보호 불변식**: 선행 개체의 실제 출력·예약 밴드·후속 저장 앵커는 같은 좌표계를 써야 한다.
앞 표를 이동하는 규칙은 뒤 표의 분기 조건까지 영향 검증해야 한다.

### 6.4 R3 — synam001: 앞 밴드는 피했지만 실제 호스트 줄을 피하지 못함

입력: `samples/synam-001.hwp`, 30쪽 pi=228/229. #6797의 반대 방향 검사와
`issue_synam001_visible_float_host_line_overlap`이 같은 부근에서 실패한다.

| 대상 | devel | 후보 |
| --- | --- | --- |
| pi=228 선행 표 | 767.6..926.4 | 758.7..917.5 |
| pi=229 host 글줄 | 930.2..942.2 | 921.3..933.3 |
| pi=229 표 | 945.9..998.9 | 925.1..978.1 |

후보의 pi=229는 활성 밴드를 보고 `clear_occupied_bands`를 실제로 실행했다.
하지만 그 결과는 앞 표의 하단+여백일 뿐, **앞 밴드에 밀린 자기 host 글줄의 실제 하단**이 아니다.
표 상단925.1이 글줄 하단933.3보다8.23px 높다. 표 첫 글줄과 host 글자의 간격 검사도
4.52px로 최소8px를 만족하지 못했다.

기존 layout의 `title_flow_y + host_line_px + visible_outer_top_px` 하한은 앞 밴드를
반영한 위치에서 자기 host 한 줄을 확보한다. 새 확정 값이 있으면 그 경로가 생략된다.
저장 줄이 표보다 앞선다는 판별과, 현재 페이지에서 실제 출력할 host 줄이 표 위에 들어간다는
판별은 같지 않다. 현재 helper 입력의 문단 시작 높이만으로 후자의 조건이 보장되지 않는다.

**보호 불변식**: 선행 개체 회피 뒤에도 표 위에 놓일 실제 host 글줄·여백을 다시 만족해야 한다.
표와 텍스트가 각자 다른 시점의 원점을 채택하면 같은 placement 전달만으로 겹침을 막을 수 없다.

### 6.5 R4 — #6267: 겹침 단언이 아니라 통째 표의 분할 때문에 실패

입력: `samples/issue6267/kdt_result_para_float_table.hwpx`, 원본 그대로의 문단8이다.
작은 용지 높이로 변조했던 제외 실험과 다른 **기존 저장소 회귀 문서**다.

- devel: 표 한 개 954.0..1064.0px. 기존 검사의 한컴 실측 상단952.9px 허용 범위에 들어간다.
- 후보: 첫 조각946.4..1014.8px, 다음 쪽 조각75.6..117.1px. 문서 쪽수도1→2다.
- host 네 줄은 양쪽 동일하며 마지막 줄 하단945.1px다. 첫 조각은 그보다 아래다.
- 실제 실패는 `tables.len() == 1`에서 2가 나왔기 때문이다. ‘본문 겹침이 재발했다’고
  검사 이름만으로 보고하면 잘못이다.

base 실행의 기존 진단은 `plain=true, cur_h=724.3, total=143.8, avail=971.3`을 기록한다.
즉 기존 통째 배치 판단은 참이다. 새 계산은 `occupied_bottom=984.5333 > 971.3`이어서
`resolved_host_placement.map_or(legacy_whole_fits, ...)`가 그 판정을 대체하고 분할로 보낸다.
원인 경로는 확인했지만, 기존 문서의 본문 경계·종이 경계·쪽 나눔 정책을 보존할 조건은 아직
설계하지 않았다. 단순히 두 fit 결과를 OR로 합치거나 표가 두 개여도 통과하도록 바꾸지 않는다.

**보호 불변식**: 앵커 수정과 표 분할 정책을 혼동하지 않는다. 새 상단/하단 계산이 기존의
정상적인 쪽 귀속을 바꾼다면, 먼저 원점·여백·본문/종이 경계와 속성의 의미를 검증해야 한다.

### 6.6 이번 진단으로 확정한 설계 위험과 남은 순서

새 배치 결과를 단일 값으로 전달하는 방향 자체와, 그 결과가 충분히 확정됐다는 보장은
별개다. 현재는 **기존 제약의 일부만 수집한 값을 최종 값으로 취급하고 기존 처리를 생략**한다.
그 결과 점유 누락(R1), 선행 위치와 저장 앵커 불일치(R2), 이동된 본문 하한 누락(R3),
통째/분할 정책 대체(R4)가 드러났다. 샘플 이름이나 이슈 번호별 예외를 추가하는 해법은 내지 않았다.

다음 조사 순서는 다음과 같다. 각 항목에서 기존 테스트의 실패 지점부터 확인하고, 원본 속성,
정답 근거, 동일 base/후보 출력, 원인 분기, 보호 불변식을 연결한다.

1. R1~R4에서 남은 원점·통째 배치 정책의 세부 책임을 확정한다.
2. #1789, #5941: 저장 줄 vpos·이동된 본문 원점. test 이름이 암시하는 과거 원인을 재사용하지 않는다.
3. #2097, #6854, #6025: 쪽수·행의 쪽 귀속. 이미 확인한 쪽수 증가를 실제 이동 행에 연결한다.
4. #3738, #6764: 첫 조각·캡션·각주·셀 예산. 기존 정식 문서만 사용한다.
5. text-overlap/overflow/off-canvas/쪽수 원장의 개별 문서를 위 원인과 교차 연결한다.
   중복 검출과 독립 결함을 구분하고 연결되지 않는 건 별도 미분석으로 남긴다.
6. 신규 fixture IR 왕복 필드 실패는 renderer와 분리해 base에서도 발생하는지 판별한다.

이 전체 조사 후 보호 조건을 명시한 수정안을 확정하고 구현한다. 이번에 신규 테스트 추가,
제품 수정, baseline 갱신, 원격 게시, WASM 교체는 하지 않았다.

### 6.7 진단 종료 시 작업 상태

review worktree와 공유 CLI를 후보 `e2436aa2d`로 복귀했다. 계측 코드는 남아 있지 않다.
계측 제거 후 재빌드(1분04초)한 `trace-restored/`에서 6입력 각각의 extents·pages·anomaly
출력, 총18개를 계측 빌드 출력과 비교해 **바이트 동일**임을 확인했다. 계측 때문에 발생한
회귀가 아니다. 쪽수 대조는 zero1→1, repeat10→11, target3→3, host1→2,
band11→11, synam35→35다. 신규 #6950의 출력도 계측 제거 전후 동일하다.

review tracked diff0, 파생 manifest `--check` 통과, 주 브랜치 `git diff --check` 통과.
주 브랜치 변경은 이 Stage 3 조사 기록뿐이다. 기존 전체 nextest 결과를 지우지 않았으며,
이번 CLI 대조를 23개 테스트의 재실행 또는 통과로 계산하지 않는다.

## 7. 메인테이너의 30쪽 확인용 Docker WASM 빌드

메인테이너가 현재 후보의 `samples/synam-001.hwp` 30쪽을 Studio에서 직접 확인하도록
WASM 빌드를 지시했다. 전체 회귀 미통과 상태는 유지하며, 이 빌드는 **회귀 확인용**이다.

- 브랜치 `task_m100_6950`, HEAD `acfa1c4a8fe2711de6f9711409cf2bb1635ebd42`.
  `src`, `Cargo.toml`, `Cargo.lock`은 앞선 후보 `e2436aa2d`와 동일함을 확인했다.
- `docker compose --env-file .env.docker run --rm wasm` 성공.
  release 컴파일3분51초, wasm-opt 포함 wasm-pack 표시6분43초.
  로그: `output/6950/stage3/wasm-synam-build.log`.
- WASM SHA-256: `4107170506947e478245f8e7f26cf97968423b46dc17bad2a54cbf70b465d663`.
- 7700 포트에 기존 서버가 없음을 확인하고 이 저장소의 `rhwp-studio`에서
  `npm run dev -- --host 0.0.0.0 --port 7700 --strictPort` 실행.
  Studio HTTP200, `/samples/synam-001.hwp` 응답 해시와 원본 일치,
  Vite가 제공하는 WASM과 위 빌드 파일 해시 일치를 확인했다.
- WASM API로 입력을 열어35쪽, 30쪽 pi=228/229의 존재를 확인했다.
  pi=229 host 하단933.3px, 표 상단925.1px, 간격−8.2px로 CLI에서 조사한 배치가 재현된다.
  JSON 좌표는 소수 첫째 자리 정밀도다. 이는 브라우저 인간 시각 판정의 대체가 아니다.
- 확인 스크립트의 최초 상대 import 경로가 한 단계 상위로 지정돼 실행 전 실패했다.
  출력 폴더 기준 경로를 정정한 뒤 실제 WASM 검사를 성공시켰으며 제품 코드는 변경하지 않았다.
- 증적: `output/6950/stage3/wasm-synam-page30.json`, `wasm-synam-page30.svg`.
- 접속: `http://localhost:7700/?url=/samples/synam-001.hwp`.
  기존 탭은 강력 새로고침 후 문서를 다시 열고 하단 쪽 번호에서30쪽으로 이동한다.

회귀 수정·테스트 기대값 변경·전체 검증 통과 선언·원격 작업은 하지 않았다.

## 8. 메인테이너 시각 판정 — 명시적 개행의 기존 배치 보호

메인테이너가 현재 WASM으로 `samples/synam-001.hwp` 30쪽을 직접 확인한 뒤 다음과 같이
판정했다: **Enter로 명확하게 개행된 부분은 이전 구현의 배치가 맞고 이번 구현이 틀리다.**
해당 관찰을 기존 배치를 보호해야 하는 근거로 추가한다. 앞선 #6950 원본의 시각 판정 통과와
이 회귀 판정은 각각 유지한다.

수정안에서 구분해야 할 검증 축은 다음과 같다.

1. 작성자가 Enter로 확정한 문단 경계/개행에 따른 배치.
2. 같은 흐름에서 텍스트 뒤 남은 너비가 부족하여 자동으로 다음 줄에 배치되는 경우.

새 규칙으로 두 경우를 동일하게 재해석해서는 안 된다. 명시적 개행으로 확정된 흐름의
원점·글줄·간격은 보존하면서 이번 목표인 자동 줄바꿈 사례를 해결해야 한다.

단, 이 판정을 곧바로 `Paragraph.text`에 `\n`이 있으면 기존 분기를 쓰는 조건으로 번역하지
않는다. 현재 pi=229 CLI 덤프는 `"7. [필수] "`, 저장 줄1개, 표 컨트롤1개를 보여주며,
문단 내부의 `\n` 자체가 확인된 상태는 아니다. Enter에 의한 문단 경계와 문단 내부 강제
줄바꿈, 자동 줄바꿈을 원본 레코드·IR·배치 입력에서 각각 어떻게 표현하는지 연결해야 한다.
이번 시각 판정만으로 23개 실패 모두의 원인이 명시적 개행이라고 확대하지 않는다.

제품 코드·테스트 기대값·WASM은 변경하지 않았다. 현재 Studio의 확인용 후보를 그대로 유지한다.

## 9. 메인테이너 추가 관찰 — 빈 문단을 사이에 둔 후속 vpos

메인테이너는 **문단 끝에 표가 있고, 그 뒤 빈 문단 하나(Enter만 입력)를 거쳐 다음 본문
문단이 시작하는 경우 후속 본문의 vpos도 교정해야 한다**고 관찰했다.

검증할 흐름은 `텍스트 + 끝의 표 → 빈 문단 → 다음 본문 문단`이다. 이는 기존 구현계획의
‘표만 맞고 후속 문단이 겹치면 실패’ 조건을 구체화하는 것이며 별도 타스크로 분리하지 않는다.

확인할 보호 조건:

1. 표의 배치가 변하면, 그 표가 실제로 점유하는 영역을 후속 흐름에서도 일관되게 참조한다.
2. 빈 문단은 글자가 없다는 이유로 높이0으로 취급하거나 제거하지 않는다.
   해당 문단의 서식·유효한 줄 정보에 따른 줄높이, 줄간격, 문단 앞뒤 간격을 확인한다.
3. 빈 문단을 통과한 다음 본문의 vpos가 실제 표 점유와 빈 문단의 진행량에 맞는지 확인한다.
   표 위치만 옮기고 후속 본문은 이전 저장 좌표에 남기는 불일치를 검사한다.
4. 저장 vpos에 이미 반영된 표 높이나 간격을 다시 더하지 않는다. 반대로 배제 영역을
   소비하면서 빈 문단의 의도된 간격까지 없애지도 않는다. 실제 페이지/단 원점에서
   예약·출력·후속 vpos가 같은 결과를 사용하는지 추적한다.

‘빈 문단이면 일정 px 가산’ 같은 보정식은 정하지 않았다. 이번 메시지에는 대상 문단 번호와
기대 좌표가 명시되지 않았으므로 특정 pi나 앞선23개 실패와의 대응은 아직 확정하지 않는다.
재현 위치를 고정한 뒤 표·빈 문단·다음 본문의 원본 구조와 변경 전후 좌표를 함께 계측한다.

이번에는 관찰과 검증 조건만 기록했으며 제품 코드·테스트 기대값·WASM은 변경하지 않았다.

## 10. 동일 이슈 내 해결 결정과 수정계획

메인테이너가 관측된 문제들을 #6950 안에서 해결하도록 지시했다.
[구현계획 §5](../plans/task_m100_6950_impl.md#5-stage-3-회귀-정정-수정계획--승인-요청)에
명시적 개행 보호·기존 배치 제약 전달·빈 문단 이후 흐름·쪽 귀속 보호를 반영해 승인 요청한다.

추가 read-only 확인으로 원본 `samples/hwpx/20260909-para-table.hwpx`에도 해당 빈 문단 구조가
있음을 확인했다. pi=2는 text_len0/controls0, 저장 vpos55291/줄높이1200/줄간격672이며
pi=3의 시작 vpos57163은55291+1200+672다. 현재 후보 extents에서는 pi=2가590.5px로
표603.8..782.4px의 위쪽에 남아 있고 pi=3만786.2px로 내려가 있다.
이 구조는 이번 수정에서 사용할 재현 근거다. 메인테이너가 지목한 모든 위치를 이 한 문단으로
한정하거나, 저장 좌표 산술만으로 한컴 PDF의 절대 좌표를 새로 확정한 것은 아니다.

`tests/cases/issue_6950_paragraph_end_topbottom_anchor.rs`의 기존 후속 본문 비겹침 검사는
이 빈 문단의 위치와 진행량까지 보장하지 못했다. 수정계획에서 그 누락을 보완한다.
이번 작업은 문서 갱신과 원본·기존 출력 확인까지이며 제품·테스트·WASM은 변경하지 않았다.

## 11. 정정 A — 적용 경계와 선행 점유 복구

### 11.1 승인·소스와 구조 근거

메인테이너가 수정계획 A~C를 승인했다. 승인 기록을 `e04dfff80`으로 보존한 뒤
회귀 보호 테스트 `bf39eb882`, 제품 정정 `b737f09b911d0df97b66eca2ee352861a77c9e0f`를
로컬 커밋했다. 원격 push·PR 생성은 수행하지 않았다.

기존 WASM의 `getParagraphLength`·`getControlTextPositions`로 실제 IR의 연결 위치를
확인하고 신규 native 테스트에서도 고정했다. 위치는 `Paragraph.text`의 scalar 문자 축이다.

| 원본 문단 | 텍스트 길이 | 컨트롤 위치 | 이번 판별 |
| --- | ---: | --- | --- |
| synam001 pi229 | 8 | 0 | 문단 시작 — 텍스트 끝 앵커 규칙에서 제외 |
| #6797 pi70 | 50 | 0 | 문단 시작 — 동일 |
| #6267 pi8 | 143 | 0 | 문단 시작 — 동일 |
| #6950 pi1 | 174 | 174 | 텍스트 끝 — 새 규칙 유지 |
| #2439 zero-offset pi0 | 23 | 0,0,23,23 | 끝의 표이지만 선행 표의 점유 하한도 필요 |

`synam001`의 관찰을 단순히 문단 문자열에 `\n`이 있다는 뜻으로 해석하지 않는다.
HWP 파서에서 0x000A는 문단 내부 줄바꿈이며 0x000D는 문단 종료다. 문단 시작에
붙은 표가 저장 offset 때문에 글줄 아래에 보인다는 사실만으로 텍스트 끝 표라고
판정했던 적용 범위를 정정했다.

- `float_placement.rs`: 저장·재조판 양쪽에서 실제 끝 연결과 마지막 논리 줄의 텍스트를
  확인한다. 문자 매핑이 없거나, 명시적 줄바꿈 직후의 빈 줄에 놓인 컨트롤이면 이 배치
  계약이 소유하지 않는다. 앞쪽에 줄바꿈이 있더라도 마지막 줄에 텍스트가 있으면 일괄 제외하지 않는다.
- `typeset.rs`: 같은 문단의 선행 표가 이미 소비한 흐름과 outer-top을 확정 상자의 하한에
  포함한다. offset=0인 표가 exclusion 목록에 없다는 이유로 점유를 잃지 않게 했다.
- 문서명·쪽번호 분기, 기존 테스트 기대값 완화, 기준 원장 갱신은 없다.

### 11.2 검증

기존 review worktree를 명시적 커밋으로 전환하고 공유 target을 재사용했다.
generated suite·manifest는 review 검증용으로만 준비했으며 소스 커밋에 포함하지 않았다.

1. `bf39eb882`(제품은 수정 전): 신규 2개 테스트 **2 failed**. 문단 시작 오인과
   명시적 줄바꿈 직후 컨트롤 오인에서 각각 실패했다. RED 로그:
   `output/6950/stage3/correction-a-red-focused.log`.
2. `b737f09b9`: #6950 전체17개 **17 passed**.
   `correction-a-green-contract.log`, 컴파일4분22초/검사0.235초.
3. 기존 #2439, #6797, #6267, synam001, #6718, #6879, #6860 집중 검사 **27 passed**.
   `correction-a-green-regressions.log`, 추가 suite 컴파일1분18초/검사0.185초.
4. `cargo fmt --all -- --check`, `git diff --check` 통과.

총44개 집중 통과다. 이전 전체23개 실패 중 #2439·#6267·#6797 두 개·synam001의
**5개 실패 검사를 재실행해 통과**했으며, 나머지18개는 아직 이 정정본에서 재실행하지 않았다.
전체 회귀나 이번 소스의 필수 Clippy 묶음·Native Skia·Docker WASM 통과로 확대하지 않는다.
권장 nextest 버전 및 `report-skipped` 키 경고는 앞선 환경과 같으며 실제 검사 실패와 구분한다.

첫 RED 명령은 suite를 지정하지 않아 불필요한 전체 target 빌드가 시작됐다. 해당 작업 소유
프로세스만 중단하고 `--test regression_suite_026`으로 재실행했다. 중단 실행은 검사 결과에서
제외했다. 이후 focused 명령은 suite를 명시했다.

### 11.3 실제 문서 대조와 남은 차이

수정본의 `release-test/rhwp`를 사용했다. CLI SHA-256:
`aeab794501ab774ca22fc9f7919c2d8a33590f6f9fcd3ccd1c97306fc9f41a9c`.
`output/6950/stage3/trace-regression.mjs correction-a b737f09b911d0df97b66eca2ee352861a77c9e0f
/home/edward/mygithub/rhwp-shared-review-target/release-test/rhwp`로 기존6건만 다시 대조했다.
증적은 `trace-correction-a/`; metadata에 코드·바이너리·입력 해시를 보존했다.

| 대상 | 정정 전 → 정정 A | 판정 |
| --- | --- | --- |
| #2439 zero-offset | 뒤 표176.0..256.0 →214.5..294.5px | 선행 표136.0..210.7px 아래로 복구. extents는 base와 동일 |
| #2439 반복 서식 | 11쪽 →10쪽 | extents·anomaly는 base와 동일 |
| #6797 7쪽 | 두 번째 표174.8..340.5 →296.8..462.5px | 첫 표181.5..294.9px 뒤로 복구. extents·pages·anomaly base 동일 |
| #6267 | 2쪽 분할 →기존1쪽 통째 | extents·pages·anomaly base 동일 |
| synam001 30쪽 pi229 | 표925.1..978.1 →945.9..998.9px | 제목930.2..942.2px와 함께 해당 위치 base 복구 |
| 원본 #6950 | 표603.8..782.4px·3쪽 유지 | extents·pages·anomaly가 승인된 수정 전 후보와 동일 |

**잔여를 숨기지 않는다.** synam001 전체 extents가 base와 동일한 것은 아니다.
pi224 표 상단은 base633.2px, 정정 A629.5px로 차이가 남는다. 30쪽 pi229 복구와
구분하여 정정 C에서 연결 위치·실제 host 하한을 재검토한다. #2439의 출력 extents는
같지만 typeset `usedHeight`는 zero-offset에서+7.5467px, 반복 서식의 두 단에서
각+3.7733px 차이가 남는다. 바깥 여백 소비의 중복 여부를 B/C에서 확인하며 정상 차이라고
확정하지 않는다. anomaly 동일만으로 이를 무시하지 않는다.

원본 #6950의 빈 문단 pi2 상단590.5px와 후속 pi3 상단786.2px도 아직 그대로다.
정정 B의 `표 → 빈 문단 → 다음 본문` 흐름 교정은 미구현이며 이 이슈 안에서 계속 처리한다.

synam001 30쪽 SVG는 `output/6950/stage3/correction-a-svg/synam-001_030.svg`에
canonical layer backend·`--font-style`로 내보냈다(35쪽 중1쪽, overflowCellLines0).
Studio의 기존 WASM은 이번 정정본으로 교체하지 않았다. 이 SVG 생성은 메인테이너의
새 시각 판정을 대신하지 않는다. 다음 실행 순서는 승인된 정정 B, 남은 차이와 전체 게이트 C다.

### 11.4 pi224 메인테이너 추가 시각 판정 — 세로 기준점

메인테이너는 pi224의 표가 마지막 글자 뒤에 연결되고, 너비 부족으로 다음 줄에 배치되는
처리는 개선되었다고 확인했다. 다만 본문과의 배치 속성인 **세로 문단 위 기준3.70mm**를
적용한 간격이 한컴보다 좁다. 이 관찰을 단순한 base 복원 문제와 구분한다. base633.2px가
곧 한컴 정답 좌표라는 뜻도 아니다.

동일 정정 A CLI의 원본 dump 확인:

- pi224: 텍스트20자, 표1개, 저장 줄1개(vpos40690/높이900/줄간격360HU).
- 표: 비TAC·자리차지·세로문단/Top, vertical_offset1050HU(약3.70mm).
- outer-top/outer-bottom 각각283HU(약1mm). 문단 spacing-before/after는0.
- 96dpi 환산 시 offset1050HU는14px다. 현재 실제 출력은 문단 줄상단618.1px,
  표상단629.5px로 그 차이가 약11.4px다. 이는 글자 아래의 빈 간격과는 다른 측정값이다.

원본 속성값은1050HU로 읽힌다. 따라서 현재 근거로 mm 파싱 오류라고 단정하지 않는다.
`from_stored_host`는 전달받은 원점에 앵커 줄 상대값·offset·outer-top을 더한다.
우선 추적할 부분은 typeset이 전달한 문단 원점과 실제 출력된 host 줄 원점의 대응 및
바깥 여백 적용 책임이다. 필요한 차이를 고정 상수로 더하거나 다음 줄 배치 개선을
되돌리지 않는다. 이번 기록에서는 코드·WASM을 변경하지 않았다.

## 12. pi224 앵커 연결 보완 — 시각 판정 실패·폐기

**이 절의 후보는 메인테이너가 시각 판정에서 기각했다.** 아래45개 검사 통과와 bbox 수치는
정상 렌더링의 증명이 아니었다. 코드·해당 기대값 테스트는 철회했다(§13). 기록은 실패 계보로만 보존한다.

메인테이너가 원점 불일치 분석에 동의하고 수정·재검토를 승인했다.
테스트 커밋 `0e7f59675`, 제품 커밋 `a29f82b9241b0fef1acc3ab37e96801a3a99de5f`다.

### 12.1 원인 추적의 구체화

3.70mm는1050HU/14px로 읽힌다. 불일치는 두 단계에서 누적됐다.

1. typeset의 문단 원점은 약611.7px였고, layout의 문단 순차 원점은615.5px였다.
2. `relocate_float_anchor_lines_below_band`가 이웃 문단의 확정 y와 저장 vpos 차이로
   제목 줄만618.1067px로 옮겼다. 표는 typeset 원점으로 계산한629.4533px에 남았다.

따라서 이전 설명의 ‘이후 저장 좌표 보정’은 일반 문단 진입 스냅이 아니라 **열 구성 뒤
앵커 줄 재배치 후처리**였다. 진단 로그 `pi224-before-trace.log`, `pi224-typeset-origin.log`,
`pi224-layout-origin.log`로 누적 커서·출력 커서·최종 노드를 구분했다.

### 12.2 이번 보완과 검증 경계

기존 후처리가 제목 줄을 이동할 때, 확정된 텍스트 끝 앵커 배치를 갖는 통째 표도
`최종 제목 원점 - 기존 계획의 앵커 원점`만큼 함께 이동한다. 제목 노드가 이동한 차이만
더하면 앞 단계의 원점 차이가 남으므로 계획에 기록한 앵커를 기준으로 연결했다.
표 하위 셀·텍스트·테두리는 같은 변환을 받는다. 이미 연결된 표에는 기존 HWPX 후처리를
중복 적용하지 않는다. continuation 조각은 별도 page-local 계약이므로 이 통째 표 처리에서 제외한다.

문단번호·문서명 조건이나 고정6.4px/3.70mm 가산은 제품 코드에 없다.
이번 보완은 **기존 최종 재배치에서 앵커 종속 관계를 보존하는 수정**이다. 후처리를
페이지 배정 이전으로 이전한 구현은 아니다. 수정계획 §5.2의 사전 예약·최종 출력 일치까지
완료했다고 주장하지 않으며, 일반적인 쪽 경계와 후속 흐름 검증은 B/C의 잔여 게이트다.

### 12.3 결과

- 신규 회귀 테스트는 수정 전 정확한 상대 좌표 단언에서 실패했다(`pi224-red.log`).
- 수정 후 #6950 18개 + 기존 집중27개 = **45 passed**, 실패0.
  `pi224-green.log`: 컴파일5분51초, 검사0.294초.
- 포맷·diff check, native Clippy(`-D warnings`) 통과. Clippy29.68초.
  전체 Clippy 묶음·전체 회귀·Native Skia·Docker WASM 재실행을 대신하지 않는다.
- CLI SHA-256: `d2b3b7c21743160311f5594e6e77a526ac8c54506287dbcf8e50d01b5a5efa9a`.
  같은6건을 `trace-pi-anchor/`에 재측정했다. 원본 #6950, #2439 두 건, #6267, #6797의
  extents/pages/anomaly는 정정 A와 바이트 동일하다.
- synam001은35쪽·anomaly·pages가 정정 A와 동일하며 extents의 변경17줄은 pi224의
  표·셀·텍스트·테두리다. pi228/229 및 제목 pi224는 그대로다.

| 항목 | 정정 A | 이번 후보 |
| --- | ---: | ---: |
| pi224 제목 상단 | 618.1067px | 618.1067px |
| 표 상단 | 629.4533px | 635.8800px |
| 표 하단 | 662.6533px | 669.0800px |
| 후속 pi225 줄상단 | 672.8533px | 672.8533px |

후보의 표-제목 상단 차이는17.7733px = offset1050HU + outer-top283HU다.
표 하단에서 후속 줄까지 약3.77px로 바깥 아래 여백도 남는다. 이는 현재 fixture의
내부 좌표 정합 증적이며, 메인테이너의 한컴 시각 판정을 대신하지 않는다.

검토 파일:

- 디버깅: `output/6950/stage3/pi224-anchor-debug-svg/synam-001_030.svg`
- 일반: `output/6950/stage3/pi224-anchor-svg/synam-001_030.svg`
- 이전 디버깅본: `output/6950/stage3/correction-a-debug-svg/synam-001_030.svg` (보존).

canonical layer·`--font-style`, 디버깅본에는`--debug-overlay`를 추가했다.
export manifest의 renderedCount1/overflowCellLines0 확인. WASM 및 Studio는 갱신하지 않았고
원격 push·PR 작업도 하지 않았다. pi224의 세로 간격을 메인테이너에게 재검토 요청한다.

## 13. pi224 후처리 수정 철회와 재접근

메인테이너는 표 안의 내용이 바깥으로 보이는 악화를 확인했고, 이번 수정 폐기와 재접근을
지시했다. `a29f82b92`의 제품 변경 및 `0e7f59675`의 테스트만 되돌렸다.
정정 A `b737f09b9`의 소스·기존 테스트는 유지했다. `git diff b737f09b9 -- src tests`가
비어 있음을 확인했다. 문서·기존 작업·원본 샘플·과거 커밋은 삭제하지 않았다.

### 13.1 확인된 실패 원인과 검증 오류

`translate_subtree_y`는 각 노드의 `bbox.y`만 변경한다. 하지만 `LineNode`는 bbox 외에
실제 선 좌표 `x1/y1/x2/y2`를 갖고, SVG 출력은 이 끝점으로 선을 그린다.
완성된 표 트리를 이 함수로 이동시켜도 실제 테두리는 이동하지 않는다.

보존된 canonical SVG 원문을 직접 대조했다. 이전 `correction-a-svg/synam-001_030.svg`와
실패본 `pi224-anchor-svg/synam-001_030.svg`의 pi224 테두리 네 선은 **동일 좌표**다:

- 위 테두리 y=629.4533333333333, 아래 테두리 y=662.6533333333333.
- 좌우 테두리도 위 두 y 끝점을 그대로 사용한다.
- 실패본의 Table bbox는635.88..669.08로 바뀌었으나 이는 실제 테두리 위치가 아니다.

따라서 ‘표·셀·텍스트·테두리가 같은 변환을 받는다’는 §12.2의 주장은 잘못이었다.
새 테스트는 Table bbox와 TextLine bbox의 관계만 검사했고, 실제 선 좌표·출력 장면을
검사하지 않았다. 그 테스트를 그대로 유지해 다음 구현의 정답 조건으로 쓰지 않는다.
기존45개 통과·anomaly 동일만으로 메인테이너의 시각 실패를 반박하거나 경미하다고 축소하지 않는다.

### 13.2 재접근 원칙과 현재 상태

1. bbox 후처리 이동은 폐기한다. `translate_subtree_y`에 선·경로·clip 예외를 계속 덧붙이는
   방식으로 이번 수정안을 연명하지 않는다.
2. 문단 흐름 원점, 최종 host 원점, 표 offset·바깥 여백, 다음 문단으로 전달되는 예약을
   다시 연결한다. 표·셀·테두리·clip을 **생성하기 전** 사용할 배치 원점을 확정하는 쪽을 조사한다.
3. 후처리된 제목 y를 그대로 한컴의 문단 기준점으로 간주하거나, 이번635.88px를 새 정답으로
   고정하지 않는다. 소스 속성과 한컴의 실제 배치를 분리해 재확인한다.
4. 다음 검증은 bbox뿐 아니라 실제 SVG 선/경로 좌표와 셀 내용·clip의 일치도 포함한다.
   재접근 설계를 정리한 뒤 구현·시각 재검토를 진행한다.

이번 턴에는 실패 구현 철회와 원인 확인까지 수행했다. 대체 구현은 추가하지 않았다.
실패 SVG·진단 로그는 실패 증적으로만 보존하며 재검토용 정상본으로 안내하지 않는다.
정정 A 디버깅본은 `output/6950/stage3/correction-a-debug-svg/synam-001_030.svg`다.
공유 release-test CLI는 실패 후보의 바이너리이므로 다음 실행 전에 복구된 소스로 재빌드해야 한다.
WASM은 이번 실패 수정으로 빌드한 적이 없다. 원격 변경도 없다.

## 14. pi224 재구현 — 생성 전 저장 원점과 점유 구간 확정

메인테이너의 구현 지시에 따라 정정 A에서 재시작했다. 이번에는 완성된 노드를 이동하지 않는다.

### 14.1 사전 결정의 근거와 적용 경계

pi224의 다음 문단까지 저장 간격은 `44796−40690=4106HU`이며,
`offset1050 + outer-top283 + measured-height2490 + outer-bottom283`과 일치한다.
현재 단은 pi222의 저장 첫 줄 vpos0에서 시작한다. pi223은 vpos1260이고 pi224까지 저장 줄이
단 안에서 단조 증가한다. 이 두 증거로 현재 단 원점과 표를 포함한 저장 간격을 확인할 수 있다.
후처리된 제목 좌표나 다음 문단의 렌더 노드를 읽어 원점을 역산하지 않는다.

`ParagraphFloatPlacement`는 다음 조건을 만족할 때만 저장 원점을 확정한다.

- 기존 텍스트 말미/문단 상대/자리차지 앵커 자격을 이미 만족한 단일 줄·단일 컨트롤이다.
- 현재 단은 완전한 문단에서 시작하며, 출처가 유효한 저장 줄이 현재 호스트까지 연속·단조 증가한다.
  편집·합성 줄·줄 누락·분할 표·Shape·다단은 이 원점 복구의 근거로 쓰지 않는다.
- 다음 문단의 저장 간격이 현재 측정된 점유 높이와 1HU 이내로 일치한다.
  일치하지 않는 입력은 기존 흐름 배치를 유지한다. 이미 소비한 흐름을 뒤로 되돌리지 않는다.

확정 원점으로 앵커·표 상단·점유 하단을 함께 계산한 **후** fit를 판단한다. layout은 이 원점에서
표/셀/테두리/clip과 호스트 텍스트를 처음부터 생성한다. 이 원점을 사용한 호스트는 기존
`relocate_float_anchor_lines_below_band`의 후처리 대상이 아니다. 후속 흐름의 호스트 소비에도
같은 원점을 사용한다. 파일명·문단 번호·3.70mm 또는635.88px 상수 분기는 없다.

### 14.2 검증 방법

계약 테스트에 저장 간격 불일치·편집·합성·원점 이동을 추가했다. 실제 synam fixture 검사에는
표 bbox뿐 아니라 `LineNode.y1/y2`의 네 테두리와 셀 두 문단 및 후속 문단을 포함한다.
첫 debug probe에서 위/아래 실제 SVG 선과 cell clip이635.88..669.08로 함께 생성됨을 확인했다.
이는 생성 좌표의 내부 정합 확인이며 한컴 정답 좌표 또는 시각 판정 통과를 주장하는 값이 아니다.
최종 소스로 집중 회귀·재내보내기 결과를 확인한 뒤 아래에 기록한다.

### 14.3 실행 결과와 시각 재검토 요청

- 제품 커밋: `d1138fa23c20167244a812250bc8501397a234b5`.
- #6950의19개와 기존 보호27개, 총 **46 passed / 0 failed**.
  `prepaint-focused.log` 42개(컴파일5분46초, 실행0.313초)와 `prepaint-2439.log` 4개다.
  테스트 원본 포맷 보정 뒤 #6950의19개를 재실행해 모두 통과했다
  (`prepaint-formatted-focused.log`).
- native Clippy `-D warnings` 통과(32.49초). 테스트 원본을 포맷한 뒤 review worktree의
  파생 suite를 다시 준비하고 fmt·manifest check를 확인했다. 파생물은 커밋하지 않는다.
- CLI SHA-256: `83f46b1c056f4c514ea824dd4e8402d2e53d71ee68b8983d5a2984d1e0a3ea8d`.
  이 CLI의6건 재측정과 입력/바이너리 hash는 `output/6950/stage3/trace-prepaint/metadata.json`에 있다.
- 정정 A 대비 원본 #6950, #2439 두 건, #6267, #6797의 extents/pages/anomaly는 바이트 동일하다.
  synam001도35쪽·pages·anomaly는 동일하며 extents는 pi224 표와 자손17개 노드만 변경된다.
  제목 pi224, 다음 pi225, 기존 보호 pi228/229는 변하지 않았다.

| 실제 SVG 기하 | 정정 A | 폐기된 후처리 수정 | 생성 전 확정 후보 |
| --- | ---: | ---: | ---: |
| 위/아래 테두리 선 y | 629.4533 / 662.6533 | 629.4533 / 662.6533 | 635.88 / 669.08 |
| 셀 clip 상단/하단 y | 629.4533 / 662.6533 | 635.88 / 669.08 | 635.88 / 669.08 |
| 테두리와 clip 일치 | 일치 | **불일치** | 일치 |

현재 후보는 셀 두 문단과 실제 네 테두리가 함께 배치되며, 후속 문단까지 아래 여백을 보존한다.
독립 한컴 시각 판정 없이 이 수치를 최종 정답으로 승격하지 않는다. SVG 원문 검사에 더해
librsvg 래스터 이미지에서도 대상 두 문단이 테두리 안에 있는 것을 확인했다. 이 래스터는
로컬 폰트 환경의 보조 확인이며 Studio/한컴 폰트 동등성 증적이 아니다.

최종 재검토 파일(30쪽, CLI `--page 29`):

- 일반: `output/6950/stage3/pi224-prepaint-svg/synam-001_030.svg`
- 디버깅: `output/6950/stage3/pi224-prepaint-debug-svg/synam-001_030.svg`

두 파일 모두 canonical layer·`--font-style`로 내보냈고 renderedCount1/overflowCellLines0이다.
최종 일반 SVG는 먼저 이미지로 확인한 `prepaint-probe-svg` 파일과 바이트 동일하다.
이번 단계에서는 **WASM·Studio를 갱신하지 않았으며, 전체 회귀 및 전체 Clippy 묶음·Native Skia는
미실행**이다. 기존 anomaly가 없어졌다는 뜻도 아니다. B의 빈 문단 후속 흐름과 C의 전체 영향
검증은 계속 남아 있다. 원격 push·PR은 수행하지 않았다. pi224의 간격에 대한 메인테이너
시각 재검토를 요청한다.

### 14.4 메인테이너 시각 판정

2026-09-10 메인테이너가 이번 수정의 **시각 판정 통과**를 확정하고 WASM 빌드를 지시했다.
이는 pi224 생성 전 원점 교정의 승인이다. 빈 문단 후속 흐름(B)과 전체 영향 검증(C)의
완료로 확대하지 않는다.

## 15. 시각 승인 코드의 Docker WASM 빌드

- 빌드 기준: `task_m100_6950`, `a23668daa41c800f0f7b2ee276721ae1d4f05d6f`.
- 제품 소스는 시각 승인 대상 `d1138fa23`과 같다. 빌드 전 worktree는 clean이었다.
- 표준 명령: `docker compose --env-file .env.docker run --rm wasm`.
- 로그: `output/6950/stage3/pi224-prepaint-wasm-build.log`.
- 기존 Docker named-volume cache와 `.env.docker`를 재사용한다. 실행 중인 Studio7700 서버는 유지한다.
- 상태: **빌드 성공**(exit0). wasm-pack 보고6분30초, Rust release 컴파일3분42초.
- `pkg/rhwp.js`, `rhwp_bg.wasm`, `rhwp.d.ts` 갱신. 소유자는 모두 `edward:edward`다.
- WASM 크기:10,451,918bytes. SHA-256:
  `530b191b249d246c86d3732bdc5119b4a6a2aefeed706be62ab58ecacdfeb287`.
- Node `WebAssembly.compile` 성공(479 exports). 이는 모듈 컴파일 검증이며 브라우저 전체 조판
  시나리오 실행을 대신하지 않는다.
- `http://127.0.0.1:7700/`과 JS/WASM HTTP200 확인. Vite가 변환한 JS의 WASM URL은 현재
  저장소의 `pkg/rhwp_bg.wasm`을 가리킨다. HTTP로 받은 WASM은 디스크 산출물과 바이트·hash 동일하다.
- Studio 서버를 재시작하거나 다른 checkout으로 전환하지 않았다. 기존 탭은 강력 새로고침 후
  `samples/synam-001.hwp`를 다시 열어30쪽(pi224)을 확인한다. 브라우저가 이미 로드한 과거 WASM은
  HTTP 검증만으로 교체되지 않는다.
- 제품 코드 변경·원격 push·PR은 없다. B/C 잔여 작업은 그대로 유지한다.

## 16. WASM 시각 승인 후 전체 회귀 재실행

2026-09-10 메인테이너가30쪽 문제가 WASM에서도 해결됐음을 확인하고 전체 회귀 재실행을 지시했다.

- 대상: `9c0fdec0a` (`d1138fa23`과 제품 소스 동일), 기존 `rhwp-6950-review` worktree.
- 시작 전 main/review tracked diff0, 실행 중 Cargo/Rust 작업 없음. CPU16개, RAM31GiB 중
  available28GiB. 기존 공유 target을 유지하고 Cargo2 jobs / nextest8 threads로 실행한다.
- review worktree에서 suite prepare·manifest check·fmt check 통과.
- 이전 전체 실행과 같은 명령·신규 샘플 보안 검사 입력을 사용한다.
  `CARGO_BUILD_JOBS=2 RHWP_SECURITY_SWEEP_SAMPLES_JSON='["samples/hwpx/20260909-para-table.hwpx"]'`
  `cargo nextest run --locked --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp-shared-review-target --tests --test-threads 8 --no-fail-fast`.
- 로그: `output/6950/stage3/nextest-full-rerun.log`.
- 시간/자원: `output/6950/stage3/nextest-full-rerun-time.txt`.
- 상태: **실행 완료, exit100**. 이전 `nextest-all.log`의23개 실패와 검사 이름 기준으로 대조했다.
  판정 기준·baseline·제품 코드는 이번 실행을 위해 변경하지 않았다.

### 16.1 전체 결과

| 구분 | 이전 §5 | 이번 |
| --- | ---: | ---: |
| 실행 | 9,401 | 9,405 |
| 통과 | 9,378 | 9,403 |
| 실패 | 23 | 2 |
| 건너뜀(실행 수 외) | 46 | 46 |

이번 추가4개는 정정 A와 생성 전 원점 교정에서 추가한 검사다. #6950의19개 모두 통과했다.
이전 실패23개를 이름으로 대조한 결과21개는 실제 PASS, 2개는 다시 FAIL이다.
미실행으로 사라진 이전 실패나 신규 실패는 없다. suite 재배정으로 binary 번호가 달라졌으므로
`regression_suite_NNN` 번호가 아니라 검사 이름으로 비교했다.

- 회복된 직접 검사11개: #2439, #3738, #6854, #6267, #6797 두 개, #5941, #1789,
  #2097, #6764, synam001.
- 회복된 코퍼스 검사10개: text_overlap partitions1/10/12/14/7/11/3,
  overflow_cell partition3, off_canvas partition12, oracle_page_count partition11.
- 빌드8분32초, 검사363.577초, 전체 wall14분33.57초. 최대 RSS4,330,900KiB, swap0.
- slow2개는 통과 검사이며 timeout 실패가 아니다. nextest 권장 버전·`report-skipped` 경고는
  이전 환경과 같고 실제 실패2개와 구분한다.
- 실행 후 manifest check·diff check 통과, review worktree tracked diff0.

### 16.2 남은 실패 — #6025의 1쪽 말미 배치

- 검사: `issue_6025_cell_fragment_budget_pin::issue_6025_la_line_stays_on_first_page`.
- 입력: `samples/issue6025/3232693_employment_support_criteria.hwpx`.
- 총4쪽 단언은 통과했지만, 1쪽 y=1050..1070px에서 기대하는 ‘라. 국민행복기금…’ 문구가
  검출되지 않았다. 그 구간에서 읽힌 문구는 ‘있는자로서서민금융진흥원장으로부터확인서를발급받은경우’다.
- 함께 출력된 진단: page0/pi1 PartialTable 하단1093.5px, 본문 하단1084.7px, 초과8.8px.
- 이전 실행과 실패 단언·진단 값이 같다. **이번 pi224 보완으로 새로 발생한 실패는 아니지만**,
  #6950 이전 devel에도 존재하던 결함으로 확정한 것은 아니다. 정확한 이동 원인·회귀 경계와
  실제 한컴 배치는 별도 확인이 필요하다. 이 검사만으로 문구가2쪽으로 이동했다고 단정하지 않는다.

### 16.3 남은 실패 — 신규 HWPX fixture의 IR 왕복 차이

- 검사: `ir_field_sweep_baseline::ir_field_sweep_does_not_regress`.
- 입력: `samples/hwpx/20260909-para-table.hwpx`.
- 증가한3개 경로는 이전 실행과 동일하다. 공통 접두사 `sections[].paragraphs[]` 아래에서:
  - `controls[].cells[].paragraphs[].controls[].cells[].paragraphs[].raw_header_extra[]`: 0→254.
  - `controls[].cells[].paragraphs[].raw_header_extra[]`: 0→95.
  - `raw_header_extra[]`: 0→38.
- 렌더 위치 단언이 아니라 IR 필드 왕복 비교 실패다. 실제 정보 손실인지 허용 가능한 정규화인지
  아직 확정하지 않았으며, baseline 등록·갱신이나 검사 완화는 하지 않았다.

### 16.4 다음 판단에 필요한 작업

이번 전체 실행 요청은 완료했다. 전체 회귀는 아직 통과하지 않았으므로 PR 준비 보류를 유지한다.
남은 두 실패의 원인 확인을 먼저 하고, B의 빈 문단 후속 흐름을 마무리한 뒤 변경 범위에 맞춰
검증해야 한다. Native Skia3종과 전체 Rust lint 묶음은 이번 nextest 실행으로 대체되지 않는다.
현재 Studio/WASM은 메인테이너가 승인한 소스를 유지하며 제품 코드·기대값·원격 상태를 바꾸지 않았다.

## 17. #6025 시각 확인과 표 속성 조회 결함 — 2026-09-10

### 17.1 좌표 핀 실패와 실제 쪽 귀속의 구분

메인테이너는 한컴에디터와 rhwp의 조판이 동일하다고 확인했다. 현재 코드의 1쪽 SVG에는
‘라. 국민행복기금…’이 실제로 존재하며 text y=1084.32px다. 기존 검사는 1050..1070px의
글자만 수집하므로 이 문구를 놓친다. 그 구간은 y=1062.9867px의 ‘있는자로서…경우’다.
총4쪽 단언도 통과했다. 이 실패를 곧바로 문구의 2쪽 이월 또는 조판 회귀라고 해석하지 않는다.
기존 검사 기대값은 변경하지 않았으며 핀의 적정성 판정은 별도로 남는다.

- 디버깅 SVG: `output/6950/stage3/issue6025-debug-svg/3232693_employment_support_criteria_001.svg`.
- 일반 SVG: `output/6950/stage3/issue6025-native-svg/3232693_employment_support_criteria_001.svg`.

### 17.2 메인테이너가 발견한 위치 속성 0 표시

대상은 같은 문서 section0/para1/control0의 28행×1열 표다.

| 경로 | 가로/세로 오프셋(HU) | 폭/높이(HU) |
| --- | --- | --- |
| HWPX `hp:pos`, `hp:sz` | 709 / 4129 | 47199 / 69352 |
| 수정 전 CLI dump의 공통 IR | 709 / 4129 | 47199 / 69352 |
| 수정 전 실제 `pkg`의 `getTableProperties(0,1,0)` | 0 / 0 | 0 / 0 |

위치는 약2.50mm / 14.57mm다. 파서가 원본값을 버린 것이 아니라 속성 조회 API가
`raw_ctrl_data`만 읽고 데이터가 없으면 0을 반환한다. Studio는 이 JSON 값을 표시한다.
raw가 없는 HWPX·일부 HWP 표에서도 공통 IR에는 원본 기하가 있으므로 조회의 원천이 잘못됐다.

발생 계보: #6950 시작 기준 `13c92feb67`과 수정 전 `48bc5fc23` 사이의
`src/document_core/commands/table_ops.rs`, `src/parser/` diff는 없다.
시작 기준 getter의 위치 raw 읽기는 blame상 `bacb7484f23`(2026-05-26), 0 fallback은
`ea564999e1e`(2026-05-18)에서 온다. 따라서 이번 수정이 도입한 회귀로 판정하지 않고
**이번 검증에서 발견한 기존 조회 결함**으로 기록한다. 메인테이너는 #6950 안의 수정을 승인했다.
오래된 기준선을 별도로 빌드한 결과라고 주장하지 않는다.

### 17.3 수정과 검증 범위

- 위치·크기·바깥 여백·앵커 유지 조회를 `table.common`에서 읽는다. 위치는 `as i32`로
  기존 JSON의 음수 오프셋 계약을 유지한다. 조회에서 raw를 합성하거나 IR을 변경하지 않는다.
- 파서·조판·serializer·표 setter는 이번 절편에서 변경하지 않는다.
- 회귀 source: `tests/cases/issue_6950_table_properties_ir.rs`.
  공개 WASM wrapper를 native integration에서 호출해 실제 fixture의 HWP/HWPX,
  raw 유무, 음수/0 편집 후 조회, 조회 전후 컨트롤 IR 불변을 검사한다.
- 수정 전 제품 코드에서 신규3개 모두 실패했다(원본709 대신0, 편집-709 대신0,
  raw가 없는 HWP의 공통 기하 불일치). 로그: `output/6950/stage3/table-properties-before.log`.
- 수정 후 focused 검사와 Docker WASM 재빌드를 진행한다. 이번 절편으로 기존 전체 회귀2개나
  B/C 잔여를 완료 처리하지 않으며, 검사 기대값·원본 fixture·원격 상태는 변경하지 않는다.

### 17.4 수정 후 실행 결과

- review worktree의 동일 source로 신규3개 모두 PASS(수정 전3 FAIL → 수정 후3 PASS).
  native 빌드5분31초, 검사0.118초. 로그: `output/6950/stage3/table-properties-after.log`.
- review worktree 전체 fmt check와 suite manifest check 통과. 파생 suite는 제출하지 않는다.
- native root Clippy `--locked -- -D warnings` 통과(29.43초).
  로그: `output/6950/stage3/table-properties-clippy.log`.
- Docker `docker compose --env-file .env.docker run --rm wasm` 성공(6분51초).
  실제 새 `pkg`에서 `getTableProperties(0,1,0)`의 위치709/4129, 크기47199/69352,
  바깥 여백141을 단언했다. Studio의 한 자리 표시 기준으로 위치2.5mm/14.6mm다.
- 같은 Node/WASM 조건에서 수정 전후 #6025의4쪽 및 #6950 원본의3쪽 SVG SHA-256이
  **7/7 모두 동일**하다. 속성 조회 수정으로 실제 조판이 이동하지 않았음을 확인했다.
  이는 한컴 시각 재판정을 대신하지 않는다.
- 새 WASM SHA-256: `1d2367a67e1cb02fd132bbbeea9d61f7c68b993b19fa139b3052d262a6f6ea1d`.
  기존7700 Vite의 `/@fs/home/edward/mygithub/rhwp/pkg/rhwp_bg.wasm` HTTP 응답 해시도 일치한다.
  개발 서버는 재시작하지 않았다. 브라우저 속성창 자체의 최종 확인은 메인테이너에게 요청한다.
- 증적: `output/6950/stage3/table-properties-wasm-build.log`,
  `output/6950/stage3/table-properties-wasm-verify.json`(API 응답·쪽별 SVG 해시).
- 전체 nextest·Native Skia·WASM/workspace Clippy는 이번 focused 검사와 빌드로 대체하지 않는다.

## 18. 문단 끝 표 시작점 과대 배치 — 배포 0.8.6 직접 대조

### 18.1 메인테이너 관측과 조사 범위

메인테이너는 §17의 속성 수치 바인딩 통과를 확인했지만, 같은 문서의 제목 뒤 다음 줄에서
시작하는 표가 너무 아래에 놓인다고 지적했다. 배포 0.8.6은 속성창 수치가 잘못되어도
표의 실제 위치는 한컴과 유사하다는 관측이다. **속성 조회의 기존 결함과 #6950 배치 회귀는
별개다.** 기존 시각 피드백을 근거로 좌표 핀 실패를 무해한 검사 문제로 종결해서는 안 된다.
이번 절편은 조사이며 제품 코드·기존 기대값·원격 상태를 변경하지 않는다.

### 18.2 비교 대상과 재현성

태그를 재빌드하지 않고 실제 배포 `@rhwp/core@0.8.6`의 WASM을 사용했다.
`https://registry.npmjs.org/@rhwp/core/0.8.6`의 tarball을 내려받고 registry가 제시한
SHA-512 integrity와 내려받은 파일이 일치함을 단언했다. 프로젝트 의존성 설치·교체는 하지 않았다.

- 배포 WASM SHA-256: `8000e4ce320b7994dca6bcd58be0c862144c7b805576504e523a420439da658b`.
- 현재 WASM: §17.4의 `1d2367a6…`, 소스 `8b9c93dad`.
- 태그 소스 비교: `v0.8.6` → `f1f9c6ae58344ee9368996d3543f76b9345cf227`.
- 두 모듈에서 같은 입력 bytes로 `HwpDocument` 생성 → `renderPageSvg(0)` → 같은 text/line
  속성 추출을 수행했다. 별도 폰트 주입 없이 동일 Node 환경에서 비교했다.
- 입력: `samples/issue6025/3232693_employment_support_criteria.hwpx`.
- 증적 폴더: `output/6950/stage3/release-0.8.6-comparison/`.
  `metrics.json`은 API 응답·쪽수·text y·WASM 해시, `page-001.svg`는 배포판의 실제 SVG다.
  tarball과 추출 package는 로컬 증적이며 제출하지 않는다.

### 18.3 실제 측정: 표와 내부 글자가 함께 21.88px 아래로 이동

아래 값은 종이 위 기준 96dpi px다. text y는 SVG의 글자 기준선이며 표 괘선 y와 구분한다.

| 항목 | 배포 0.8.6 | 현재 | 차이 |
| --- | ---: | ---: | ---: |
| 제목 text y | 138.1867 | 138.1867 | 0 |
| 표 첫 가로 괘선 y | 156.24 | 178.12 | +21.88 |
| 첫 행 ‘1. 기초연금…’ text y | 170.7867 | 192.6667 | +21.88 |
| ‘라. 국민행복기금…’ text y | 1062.44 | 1084.32 | +21.88 |
| 쪽수 | 4 | 4 | 0 |

이는 글자만 괘선 밖으로 움직인 현상이 아니라 **표 상단과 내용의 동반 이동**이다.
21.88px는 약5.7891mm다. 배포판의 ‘라.’는 기존 검사 구간1050..1070px 안에 있고 현재는 밖에
있다. 따라서 기존 검사는 실제 배치 변화를 검출했다. §17.1의 ‘같은 쪽에 있다’는 사실만으로
조판 회귀를 부정하거나 검사 기대 좌표를 완화할 근거가 되지 않는다.
한컴과의 유사성은 메인테이너 관측이며, 이번 도구 계측은 두 rhwp 버전의 차이를 검증한 것이다.

### 18.4 시작점 계산의 차이와 코드 계보

이 표는 TAC가 아니라 문단 기준 TopAndBottom(자리차지), Top 정렬이며, 제목 1줄의 끝에
컨트롤이 있다. 계산에 사용되는 값은 다음과 같다.

- 본문 상단: 75.5867px.
- 앞 문단 `[별표 1]`의 흐름 높이: 25.6px → 제목 문단의 앞 간격 적용 전 원점101.1867px.
- 제목 문단의 위 간격: raw3000을 style resolver의2배 저장 단위 규칙으로 해석하여20px.
  따라서 실제 제목 줄 상단121.1867px. 저장 vpos3420HU=45.6px도 이 줄 위치와 일치한다.
- 표 세로 지정값: 4129HU=55.0533px, 표 위 바깥 여백141HU=1.88px.
- 제목은 한 줄이므로 저장 앵커 줄의 첫 줄 대비 추가 오프셋은0이다.

배포판의 측정값은 `101.1867 + 55.0533 = 156.24px`와 일치한다.
v0.8.6의 `layout_partial_table_item`은 호스트 글자를 그리기 **전**의 y를 `para_start_y`에
보존하며, 부분 문단 출력은 글자에 앞 간격을 적용한다. `table_partial`은 그 문단 기준점에
세로 지정값을 적용한다. 해당 문서의 첫 표에는 앞선 표 하단 보정이 없다.

현재는 `typeset.rs`의 `resolved_host_placement`가 `placement_para_start_height`에
`fmt.spacing_before`를 더한 **text_origin**을 넘기고,
`ParagraphFloatPlacement::from_stored_host`가 여기에 세로 지정값과 위 바깥 여백까지 더한다.
따라서 `101.1867 + 20 + 55.0533 + 1.88 = 178.12px`다.
첫 fragment에 확정한 `table_top`을 `layout_partial_table_item`이 전달하고
`table_partial`은 `resolved_table_top`을 직접 채택하므로 기존 원점 해석 대신 새 합계가 쓰인다.

추가분 `20 + 1.88 = 21.88px`가 실제 이동량과 정확히 같다. 문제는 offset4129의 파싱이나
속성창 표시가 아니라 **표 위치 기준점을 텍스트 시작점으로 바꾸고 바깥 여백을 실제 괘선
위치에 더한 배치 계약**이다. 다음 줄로 진행해야 한다는 판단과 그 줄에서 표를 어디에 놓는지는
별개의 계산인데, 뒤쪽 원점 해석이 바뀌었다.

- `10f5b3643a`(#6950): `from_stored_host`의 앵커+지정값+바깥 여백 계산 도입.
- `4f02ee904a`(#6950): 현재의 `text_origin = placement_para_start_height + spacing_before`.
- `ea5bfb8f015`(#6950): 부분 표에 `resolved_table_top`을 전달해 우선 적용.
- 시작 기준 `13c92feb67`에는 이 확정 상단 우선 경로가 없다. **배치 회귀는 #6950 안에서
  도입된 원점 계약 변경으로 추적된다.** 속성 getter의 오래된 결함과 발생 계보를 섞지 않는다.

### 18.5 수정 시 보호해야 할 경계

21.88px를 하드코딩해서 빼거나 모든 표에서 앞 간격/바깥 여백을 제거하는 방식은 채택하지 않는다.
문단 기준점·글자 줄 원점·표 괘선 상단·흐름 점유 영역을 구분해, 이번 표에서 잘못 더해진
앞 간격과 바깥 여백의 책임을 정정해야 한다. 표 내부 글자만 이동시키는 후처리도 금지한다.
확정 배치 결과를 예약과 출력이 공유하는 구조는 유지하되 입력 원점과 상자 의미를 바로잡는다.
원본 #6950의 다음 줄 배치, 승인된 synam001 30쪽 pi224의3.70mm 위치, 명시적 개행,
빈 문단 후속 흐름과 기존 #6025 핀을 함께 보호해야 한다. 구현은 조사 결과 검토 후 진행한다.

## 19. 첫 표 조각의 문단 기준점 정정 — 시각 재판정 요청

### 19.1 승인과 구현 경계

메인테이너가 §18 조사 근거에 따른 조판 코드 변경과 판정 요청을 승인했다.
`ParagraphFloatPlacement::for_first_fragment`에서 첫 조각의 기준점을 앞 간격 적용 전으로
돌리고, 원본 세로 지정값을 적용한다. 통째 표에 사용하던 위 바깥 여백을 첫 조각의 실제
괘선 위치에 다시 가산하지 않는다. 문서명이나21.88px 보정 상수는 제품 조건에 넣지 않았다.

`typeset.rs`는 이 변환을 선행 개체의 점유 배제 **전**에 수행한다. 변환 후 실제 마지막
호스트 글줄 하단과 선행 개체 점유를 만족시킨 위치를 행 분할 예산과 출력에 함께 전달한다.
텍스트 앵커는 유지하며, 렌더 트리를 만든 후 bbox나 글자만 이동시키지 않는다.
통째 표 경로와 다음 쪽으로 이월된 조각의 기존 frame 처리는 변경하지 않았다.

### 19.2 실행 결과

- 기존 review worktree와 공유 target을 재사용했다. native release-test 재빌드4분27초.
- 집중 nextest **25/25 PASS**(0.186초): 기존 #6950의19개, 속성 조회3개, 기존 #6025 핀1개,
  이번 추가2개. 필터 밖330개는 실행하지 않았다. 기존 #6025의1050..1070px 기대 구간은
  수정하지 않았으며, 직전 전체 검사에서 실패하던 검사가 이번에 통과했다.
- 신규 검사는 앞 간격/바깥 여백 조합과 선행 점유 처리 순서, 실제 #6025의 문단 기준점을
  확인한다. 메모리상의 속성 변형은 알고리즘 검사이며 한컴 오라클 샘플이라고 주장하지 않는다.
- review의 fmt check·suite manifest check, native root Clippy `--locked -- -D warnings`
  통과(Clippy29.09초). nextest0.9.137이 권장0.9.140보다 낮다는 기존 경고는 남아 있다.
- 로그: `output/6950/stage3/fragment-origin-tests.log`, `fragment-origin-clippy.log`,
  `fragment-origin-prepare.log`. 정량 비교: `fragment-origin-metrics.json`.

| 종이 위 기준96dpi px | 수정 전 | 수정 후 | 배포0.8.6 (§18) |
| --- | ---: | ---: | ---: |
| 제목 글자 기준선 | 138.1867 | 138.1867 | 138.1867 |
| 표 첫 가로 괘선 | 178.12 | 156.24 | 156.24 |
| 첫 행 글자 기준선 | 192.6667 | 170.7867 | 170.7867 |
| ‘라. 국민행복기금…’ 기준선 | 1084.32 | 1062.44 | 1062.44 |

표 괘선과 내용이 함께21.88px 위로 복구되고 제목은 그대로다. #6025는4쪽,
#6950 원본은3쪽을 유지하며 출력한 페이지의 overflowCellLines는 모두0이다.
synam00130쪽의 새 SVG는 앞서 승인된 `pi224-prepaint-svg/synam-001_030.svg`와
**파일 바이트까지 동일**하다(SHA-256 `8ebe1b94ec57fff703f6c463c53633ddc213cf856052a1f18c4427c37fdad3cb`).

### 19.3 메인테이너 판정용 산출물과 잔여

- 일반 SVG: `output/6950/stage3/fragment-origin-svg/3232693_employment_support_criteria_001.svg`.
- 디버그 SVG: `output/6950/stage3/fragment-origin-debug-svg/3232693_employment_support_criteria_001.svg`.
- 보호 확인 출력: 같은 일반 SVG 폴더의 `synam-001_030.svg`, `20260909-para-table_001.svg`.
- 재현 명령: 공유 target의 `release-test/rhwp export-svg <sample> --page <zero-based> --font-style -o <output-dir> --json`.
  디버그 출력에는 `--debug-overlay`를 추가했다. #6025와 원본은page0, synam001은page29를 사용했다.
- 생성 CLI SHA-256: `e62475e3da966418bed29e4f35832535d8fe9d4b634b6b1f6fd47f709c1bfb8d`.

판정 대상은1쪽 제목과 첫 표 사이 간격 및 표 내부 내용의 정합성이다. **한컴 시각 재판정은
대기**이며 도구 검사 통과로 대신하지 않는다. 이번 변경의 WASM 재빌드는 아직 하지 않았으므로
기존 Studio 화면이 아니라 위 SVG로 확인을 요청한다. 전체 nextest 재실행, 신규 fixture IR
baseline 축, 빈 문단 후속 흐름과 최종 Rust/WASM/workspace·Native Skia 게이트는 남아 있다.
원격 push·PR·이슈 상태는 변경하지 않았다.

### 19.4 SVG 승인과 WASM 확인 준비

메인테이너가 §19.3의 SVG 시각 판정 통과를 확인하고 같은 수정본의 WASM 빌드를 요청했다.
소스 기준은 `9d928ff27`이며, 기존 `.env.docker`와 named volume을 보존한 채
`docker compose --env-file .env.docker run --rm wasm`을 실행한다.
기존7700 Vite 서버는 재시작하지 않는다. 새 WASM API의 표 위치·속성 및 실제 HTTP 제공
파일의 SHA-256 일치를 확인한 뒤 Studio 시각 판정을 요청한다.

- Docker 최적화 빌드 성공: wasm-pack6분23초(Rust 컴파일3분40초 포함).
- 새 WASM SHA-256: `fb1d46401f0bd3a2295a21c7246961705311ea14c2b6359c633616e9ac536445`.
- 새 WASM API에서 #6025의1쪽 표 상단156.24px, 가로709/세로4129HU, 총4쪽을 단언했다.
  synam001은35쪽과30쪽 표 괘선635.88/669.08px를, #6950 원본은3쪽과1쪽 SVG 생성 성공을 확인했다.
- 기존7700 Vite의 실제 `/@fs/home/edward/mygithub/rhwp/pkg/rhwp_bg.wasm` 응답은HTTP200이며
  위 로컬 파일 해시와 일치한다. 샘플의 `/samples/issue6025/3232693_employment_support_criteria.hwpx`
  응답도HTTP200이다. 서버·제품 코드·의존성은 추가 변경하지 않았다.
- 로그·재현 검사: `output/6950/stage3/fragment-origin-wasm-build.log`,
  `fragment-origin-wasm-verify.mjs`, `fragment-origin-wasm-verify.json`.
- 메인테이너는 기존 Studio 탭에서 강력 새로고침 후 샘플을 다시 열어1쪽을 확인한다.
  SVG 시각 판정은 통과, **Studio/WASM 시각 판정은 대기**다. 전체 회귀 게이트 완료를 뜻하지 않는다.

### 19.5 메인테이너 WASM 조판 판정 통과

메인테이너가 §19.4의 WASM 확인 후 **“wasm 조판 판정 통과”**를 통보했다.
소스 `9d928ff27`, WASM `fb1d46401f0bd3a2295a21c7246961705311ea14c2b6359c633616e9ac536445`의
#6025 첫 표 위치 보정은 SVG·Studio/WASM 양쪽 시각 판정이 통과했다.
이 기록으로 §19.3~19.4의 당시 판정 대기 상태를 갱신한다.
이번 보정의 시각 검증 완료이며, #6950 전체 회귀·잔여 검증이나 PR 제출 완료를 의미하지 않는다.

## 20. 두 번째 실패 분류 정정 — 신규 샘플의 기존 문단 ID 정규화

### 20.1 확인 범위와 비교 방법

메인테이너가 신규 추가 샘플이라는 점을 지적했고, 이 샘플만 기존/현재 코드에서 비교하는
조사를 승인했다. 전체 코퍼스 검사·제품 수정·baseline 갱신은 실행하지 않았다.
입력은 `samples/hwpx/20260909-para-table.hwpx`, SHA-256은
`cbf2ee7235861e93011d80834bfc49525349f6776c929b98ebd4f06003581d07`다.

기존 §18에서 integrity를 확인한 실제 배포 `@rhwp/core@0.8.6`과 §19의 현재 WASM에서
같은 bytes를 `HwpDocument`로 열고 `exportHwpx()`로 각각 별도 저장했다.
두 저장본을 같은 현행 native `ir-sweep`으로 원본과 비교했다. 따라서 비교기는 고정하고
저장 구현을 바꾼 실험이며, 구버전 검사기를 실행했다고 주장하지 않는다.
현행 소스는 `af17af2b6`(제품 수정 `9d928ff27`과 동일)이다.

### 20.2 결과 — 이번 조판 변경의 회귀가 아니다

| 왕복 IR 차이 경로 (공통 접두사 sections[].paragraphs[] 생략) | 배포0.8.6 | 현재 |
| --- | ---: | ---: |
| raw_header_extra[] | 38 | 38 |
| controls[].cells[].paragraphs[].raw_header_extra[] | 95 | 95 |
| controls[].cells[].paragraphs[].controls[].cells[].paragraphs[].raw_header_extra[] | 254 | 254 |
| 합계 | 387 | 387 |

두 버전의 재저장 HWPX는 **파일 바이트까지 동일**하며 SHA-256은
`55c28bc42119f764fe5a702acc439c143ec85d223f31f55d002bf190d6b7ddb1`이다.
전체387개 상세 차이를 잘림 없이 확인했다. 바뀐 byte index는6(209건),7(1건),8(1건),9(176건)
뿐이다. 387은 손상 문단 수나 소실 글자 수가 아니라 **IR 바이트 값의 차이 건수**다.

원본 XML의 문단은210개이고 ID 값은3종(예:3121190098,2147483648,0)이 반복된다.
저장본도210개 문단이며210개의 서로 다른 ID를 갖는다. 첫 본문 ID는3121190098→14다.
원본·재저장본을 현행 WASM으로 렌더하면 둘 다3쪽이고 **3/3쪽 SVG SHA-256이 동일**하다.
이 결과는 현행 렌더의 동일성 증거이며, 재저장본의 한컴 편집기 재판정을 수행했다는 뜻은 아니다.

### 20.3 필드 의미와 발생 계보

- `src/parser/hwpx/section.rs`는 `<hp:p id>`를 `raw_header_extra[6..10]`에 UINT32 LE로
  보존한다. 앞6바이트는 별도 count 필드 자리이며 이번 차이에는 포함되지 않는다.
  이 매핑은 `ec6007e7294`(2026-05-22, #1058)에서 도입됐다.
- `src/serializer/hwpx/context.rs::next_para_id`는 문서 전역 카운터로 ID를 재부여한다.
  `section.rs`의 본문/셀 문단 저장 경로가 이를 사용한다. 도입 커밋 `45eb14d09f8`
  (2026-06-01, #1134)의 목적도 **HWPX 문단 ID 충돌 수정**으로 명시되어 있다.
- 앞선 원격 병합 비교 기준 `0d36da4096`과 현재 사이에는 `src/parser`, `src/serializer`,
  `src/diagnostics`, `src/model` 변경이 없다. 이번 동작은 #6950의 조판 변경으로 생긴 것이 아니다.
- `tests/fixtures/ir_field_sweep_baseline.tsv`에는 이 신규 샘플 행이 없다. 검사기는 미등록 키를
  `unwrap_or(0)`으로 비교한다. 그러므로0→38/95/254는 **과거 이 문서가 무손실이었다는 측정이
  아니라 신규 샘플의 기준선 부재**다. §16.3의 미확정 상태를 이 근거로 갱신한다.

이번 샘플의 차이는 기존 저장기의 의도된 ID 재부여로 분류한다. `raw_header_extra` 전체를
무시하거나 모든 문서의 ID 차이를 무조건 무해하다고 일반화하지 않는다.

### 20.4 증적과 권고 후속

- 재현 스크립트: `output/6950/stage3/ir-roundtrip-compare.mjs`.
- 증적 폴더: `output/6950/stage3/ir-roundtrip/`의 `summary.json`,
  `release-0.8.6-sweep.json`, `current-sweep.json`, `render-check.json`.
- 직접 확인용 재저장본: 같은 폴더의 `current.hwpx`. 원본과 기존 PDF는 덮어쓰지 않았다.
- 권고: 이 샘플의 실측3행만 사전순으로 baseline에 등록한다. 검사기·제품 코드·다른 샘플
  기준선은 바꾸지 않는다. #6852의 `393401e5a`도 같은 문단 ID 정규화 계측 등록의 선행 사례다.
  상세 필드 의미와 실측을 확인한 경우에만 등록하는 `local_validation.md`의 IR sweep 절차에 따른다.
- **이번 턴에는 baseline을 변경하지 않았다.** 등록 승인 후 해당 관문과 전체 회귀를 확인한다.
  기존 전체 검사 실패를 자동으로 PASS 처리하지 않는다.

## 21. 신규 샘플 기준선 등록과 전체 회귀 재검증

메인테이너가 §20의 실측3행 등록과 회귀 검증 진행을 승인했다.
`tests/fixtures/ir_field_sweep_baseline.tsv`에 `hwpx / 20260909-para-table.hwpx`의
중첩 셀254·셀95·본문38건만 사전순으로 추가했다. 상세 값·샘플 해시·정규화의 원인 계보는
§20에 기록했다. 다른 샘플·검사기·제품 코드·기존 기대값은 변경하지 않았다.

기존 review worktree와 공유 target을 재사용한다. 미커밋 제품3파일은 현재 task의 커밋본과
바이트 동일함을 먼저 확인했으며, 이를 보존하는 최신 task 커밋으로 review 기준을 맞춘다.
CPU16개, 메모리 available28GiB, 동시 Cargo 작업 없음 확인 후 이전 전체 실행과 같은
Cargo2 jobs / nextest8 threads를 사용한다. 새 샘플 보안 검사 입력도 동일 문서1건으로 전달한다.
전체 검증 결과는 실행 완료 후 별도로 기록하며, 기준선 등록만으로 PASS 처리하지 않는다.

### 21.1 실행과 최종 결과

- 검증 기준: `b841ee549`(제품은 SVG·WASM 승인본 `9d928ff27`과 동일).
  main과 review는 이 커밋에 정렬했고, review tracked diff0에서 실행했다.
- review의 suite prepare·manifest check·fmt check를 통과한 뒤 아래 명령을 실행했다.

```bash
CARGO_BUILD_JOBS=2 \
RHWP_SECURITY_SWEEP_SAMPLES_JSON='["samples/hwpx/20260909-para-table.hwpx"]' \
RHWP_IR_SWEEP_DETAIL='20260909-para-table.hwpx' \
cargo nextest run --locked --cargo-profile release-test \
  --target-dir /home/edward/mygithub/rhwp-shared-review-target \
  --tests --test-threads 8 --no-fail-fast
```

| 항목 | 직전 전체 검사 (§16) | 이번 |
| --- | ---: | ---: |
| 실행 | 9,405 | 9,410 |
| 통과 | 9,403 | 9,410 |
| 실패 | 2 | **0** |
| 건너뜀 (실행 수 외) | 46 | 46 |

전체 exit0. 이전 실패인 #6025 좌표 핀과 IR field sweep 모두 실제 PASS로 전환됐다.
검사 이름을 suite 번호와 분리해 대조했고, 이전9,405개 중 사라진 검사는0개, 신규 실패0개다.
증가한5개는 첫 조각 기준점2개와 표 속성 IR 조회3개이며 모두 통과했다.
이전23개 실패를 함께 기준으로 보면 §16에서 회복한21개도 유지되고 남은2개도 해소됐다.

- 신규 문서1건을 명시적으로 전달한 보안3종 탐지 검사도 PASS(0.032초).
- IR 왕복 전수 검사 PASS(107.742초). 기준선에는 승인된3행 외 변경이 없다.
- slow2개 모두 PASS: IR sweep과 #2063 대형 표 분할(154.643초). timeout 실패가 아니다.
- 빌드12분06초, 검사314.062초(약5분14초), 전체 wall17분17.56초.
  최대 RSS4,246,248KiB, swap0. 현재 nextest0.9.137/권장0.9.140 및 `report-skipped`
  설정 경고는 기존 환경 경고로 남았으며 검증 실패와 구분한다.
- 실행 뒤 review의 manifest check·fmt check·diff check 통과, tracked diff0.
- 증적: `output/6950/stage3/nextest-baseline-registered.log`,
  `nextest-baseline-registered-time.txt`, `nextest-baseline-registered-delta.json`.

### 21.2 완료 경계

이번 승인 범위인 **신규 fixture 기준선 등록과 전체 nextest 회귀 재검증을 완료**했다.
이를 Native Skia 별도3종, Rust lint 전체 묶음 또는 B의 빈 문단 후속 흐름에 관한 최종
증적 정리까지 완료한 것으로 확장하지 않는다. 최종 제출 전에 남은 계획 항목을 대조한다.
제품 코드·WASM·개발 서버는 이번 턴에 변경하지 않았고, 원격 push·PR 생성도 하지 않았다.

## 22. 다음 절차 승인 후 계획·증적 대조

### 22.1 전체 테스트 통과와 계획 완수의 구분

메인테이너가 다음 절차 진행을 승인했다. `5ae42b6f4`의 계획·현재 코드·현재 native 출력을
대조했다. 전체9,410개 통과는 유지되지만, 구현계획 §5.3의 **정정 B는 미완료**다.
최종 보고서나 PR 준비를 완료로 선언하지 않는다.

| 계획 항목 | 현황 | 근거 |
| --- | --- | --- |
| 정정 A 및 #6025 기준점·속성 보정 | 완료·시각 승인 | §11~19, 원본·synam30쪽·#6025 SVG/WASM 판정 |
| 신규 fixture IR 기준선 | 완료 | §20~21, 기존 ID 재부여 및 실측3행 등록 |
| 전체 nextest | 현재 승인본9,410개 통과 | §21, 누락·신규 실패0 |
| 정정 B: 표 뒤 빈 문단과 후속 흐름 | **미완료** | 현재 좌표와 검사 소스 대조, 아래 §22.2 |
| 최종 Rust lint 묶음·Native Skia3종 | 미실행 | B 정정 후 최종 후보에서 순차 실행 |
| 최종 결과보고서·PR 제출 | 대기 | B 및 최종 필수 게이트 완료 후 결과 승인 절차 |

### 22.2 현재 B 재확인 — 과거 수치를 재인용한 것이 아니다

현재 공유 target의 release-test CLI로 원본 `samples/hwpx/20260909-para-table.hwpx`의
`dump-extents`와1쪽 `export-svg --font-style --debug-overlay`를 다시 실행했다.
출력은3쪽, 해당1쪽 overflowCellLines0이다.

| 본문 항목 (0 기준) | 현재 종이 위 y (96dpi px) |
| --- | ---: |
| pi1의 끝 표 | 603.8..782.4 |
| pi2 빈 문단 | **590.5..606.5** |
| pi3 다음 본문 첫 줄 | 786.2..804.8 |

빈 문단은 표 아래가 아니라 위쪽에 남아 있다. 기존
`paragraph_text_table_and_following_text_do_not_overlap` 검사는 pi1 본문·표와 pi3의 비겹침만
확인하고 **pi2 위치·진행량을 단언하지 않는다**. 전체 검사 통과만으로 B 완료를 추론할 수 없다.

코드 경로: `layout.rs`의 `item_para_inkless`는 저장 레이아웃에서 비가시·무컨트롤 문단을
`visible_float_exclusions` 충돌 회피에서 제외한다. #4613/#4599의 다른 문서에서는 빈 줄의
강제 이동이 후속 흐름을 악화시킨 근거가 주석에 있다. 따라서 이 조건의 일괄 삭제나 모든
빈 문단에 일정 높이를 추가하는 방식은 채택하지 않는다.

이번 원본의 저장 줄은 pi2→pi3 간격1872HU=줄높이1200+줄간격672를 가지며,
pi1 표 뒤의 연속 흐름이라는 구조를 보존해야 한다. 실제 빈 문단과 다음 문단을 함께
검사하고 typeset의 예약·layout의 소비 경계를 정정해야 한다. 이 저장 산술만으로 다른
문서의 한컴 조판이나 PDF 절대좌표까지 단정하지 않는다.

- 원본은 변경하지 않았다. 현재 증적: `output/6950/stage3/remaining-b-extents.txt`.
- 디버그 SVG: `output/6950/stage3/remaining-b-debug-svg/20260909-para-table_001.svg`.
- 제품·검사·WASM은 이번 대조에서 수정하지 않았다. 전체 nextest도 반복하지 않았다.

### 22.3 실행 순서 정리

기존 승인된 수정계획의 순서대로 **B 집중 검사 보강·원인 정정 → 원본/보호 사례와 시각 판정
→ 최종 Rust lint·Native Skia 및 변경 후보 회귀 → 최종 보고서**로 진행한다.
최종 필수 게이트를 지금 먼저 실행했다가 B 코드 변경 후 무효화하는 중복 검증은 하지 않았다.
새 이슈나 추가 타스크로 분리하지 않으며, 원격 push·PR·merge는 계속 별도 승인 범위다.

## 23. pi=0의 연속 표 뒤 문단 종료 진행량

### 23.1 메인테이너 구조 확정과 수정 전 증거

pi=0은 한 줄에 표 컨트롤 3개(oi=2,3,4)가 오고 Enter로 끝나는 문단이다. pi=1은 다음
문단이며 자체 끝에 표를 포함한다. 이 경계에 별도 빈 문단이 있다고 설명했던 표현을 정정한다.
pi=0의 마지막 개체 배치가 끝난 뒤 현재 문단의 종료 진행량을 소비해야 한다.

- 원본 host LineSeg: 시작30164HU, 줄높이1100HU, 줄간격616HU.
- 다음 문단 시작31880HU: 차이1716HU는 현재 문단의 줄높이+줄간격과 같다.
- 수정 전 마지막 표 하단과 다음 문단 첫 줄은 모두474.013333px: 종료 진행량 누락.
- 마지막 표의 바깥 아래 여백283HU는 줄 진행량과 별도다. 기존 #6147 계약을 적용한
  이 사례의 총 간격 후보는(1716+283)/75=26.653333px다. PDF 절대좌표의 정답 핀이 아니다.
- 새 실제-fixture 회귀 검사에서 수정 전 **0/1 PASS, 1 FAIL**. 로그:
  `output/6950/stage3/paragraph-end-red.log`. 예상 간격26.653333px에 실제0px로 실패했다.

### 23.2 승인된 개선

`stored_empty_anchor_band_host_line_advance_hu`의 다음 문단 시작점 조회에서
`controls.is_empty()` 의존을 제거했다. 현재 문단의 마지막 자리차지 개체에서만 진행량을
계산하는 공통 결정과 저장 사다리 등식은 유지한다. 이 결과는 기존 typeset 예약과 layout의
lane 종료 소비자가 함께 사용하므로, 그린 노드만 아래로 옮기는 처리가 아니다.

다음 문단의 텍스트/컨트롤 배치는 그 문단의 책임이다. 다음이 빈 앵커인 표-표 스택의 기존
간격 보존 경로, 글이 있는 host의 기존 #6312 경로는 변경하지 않았다. 표 수3·문서명·좌표를
제품 분기의 키로 사용하지 않았다. 실제 fixture 검사에는 표 사이에 문단 종료량을 추가하지
않고 마지막 표 뒤에서만 한 번 소비하는 단언과 3쪽 보존 단언을 넣었다.

원본/PDF를 수정하지 않았다. 이 절의 검증은 pi=1 표→pi=2 빈 문단→pi=3 본문 잔여 문제나
최종 전체 회귀의 완료를 뜻하지 않는다. 현재 수정 후 집중 검증·SVG 생성 진행 중이다.

### 23.3 집중 검증 중 확인한 무효 저장 좌표 경계

첫 수정의 #6950 검사22개 중21개 통과,1개 실패였다. 대상 문단 종료 검사는 통과했으나
`reflowed_host_does_not_use_stale_stored_line_coordinates`가 실패했다. pi=1을 편집으로
무효화했는데도 그 원본 시작점을 pi=0 종료량의 증거로 재사용한 것이 원인이다.
현재/다음 문단의 저장 텍스트 구획이 무효이면 이 저장 사다리 계약을 적용하지 않도록
보완했다. 기존 재조판 검사와 기대값은 변경하지 않았다.

첫 실행 로그는 `output/6950/stage3/paragraph-end-green.log`이며 파일명과 무관하게 결과는
**실패**다. 추가 보호 검사 #6147/#6312/#6025/#6797/#6860/#6879/#6718을 포함한 최종 집중
실행은 `paragraph-end-validated.log`에 별도로 기록한다.

### 23.4 집중 검증 결과와 시각 확인 요청

- 최종 집중 nextest **47/47 PASS**,0실패(검사0.289초, 빌드3분26초).
  위 무효화 보호 검사도 기존 기대값으로 통과했다. nextest 버전/설정 경고는 기존 환경 경고다.
- 일반 SVG 전체3쪽, `overflowCellLines=0`. 이것만으로 전체 회귀나 시각 합격을 선언하지 않는다.
- 앞선 세 표 위치는 그대로다. 마지막 표 하단474.013333px → 다음 문단 상단500.666667px로
  문단 종료량22.88px + 표 바깥 아래 여백3.773333px를 한 번 소비한다.
- pi=1의 뒤쪽 표는603.8→626.6px로 이동했다. pi=1 첫 줄 이동26.653333px와 동일한
  강체 이동은 아니므로, 후속 표까지 포함한 전체 조판의 최종 시각 판정은 메인테이너에게 요청한다.
- pi=2 빈 문단은 현재617.1px, pi=1 표603.8..782.4px였던 영역은 현재626.6..805.3px다.
  pi=2/3 잔여 문제를 이번 간격 보정으로 해결했다고 처리하지 않는다.

확인 경로:

- 일반 SVG: `output/6950/stage3/paragraph-end-svg/20260909-para-table_001.svg`
- 디버그 SVG: `output/6950/stage3/paragraph-end-debug-svg/20260909-para-table_001.svg`
- 좌표 덤프: `output/6950/stage3/paragraph-end-extents.txt`
- 표준 compare/overlay/review:
  `output/6950/stage3/paragraph-end-visual/para-table/{compare,overlay,review}/`
  각 `compare_001.png`, `overlay_001.png`, `review_001.png`.
- 표준 비교는 webfont/Chrome146,96dpi이며 기존 한컴2024 PDF를 재사용했다.
  페이지1 `pixel_match_percent=80.48532`, `visual_accuracy_proxy_percent=6.11933`.
  페이지 전체의 글꼴·위치 차이를 포함한 보조 수치이며 이번 문단 종료 처리의 합격 점수가 아니다.

이번에는 SVG 판정을 요청한다. WASM은 이전 승인 빌드를 유지한다. 전체 회귀·최종 lint 묶음·
Native Skia 검증과 원격 push/PR은 수행하지 않았다.

추가 확인: CLI 단독 `cargo build --locked -p rhwp --bin rhwp --profile release-test`도 성공했다
(1분50초). nextest가 준비한 CLI와 별도로 빌드한 CLI의 바이너리 해시는 다르지만,
같은 원본의 3쪽 SVG를 다시 내보내 `cmp`로 전쪽 바이트 일치를 확인했다.
단독 CLI SHA-256은 `a1ff125c63df403427280fad6e6406a59eee9b12ff79d2b60c899ee6bd7ecf38`.
비교 자료의 run manifest는 실제 생성에 사용한 nextest CLI 해시를 그대로 보존한다.
review worktree의 fmt 검사 통과. 검사 추가 후 manifest drift는 `--prepare` 재실행으로
동기화하고 `--check` 통과를 확인했다. 파생 파일은 source 제출 대상에 포함하지 않는다.

## 24. 문단 끝 표의 종료 위치를 후속 문단에 전달

### 24.1 메인테이너 재확정과 착수

pi=1 시작 위치는 한컴과 동일하다는 시각 판정을 받았다. 다만 §23은 텍스트 없는 표 문단의
기존 조건을 확대한 것이었고, 일반 문단 종료 계약을 구현했다는 설명에는 미치지 못했다.
메인테이너는 pi=1의 줄 끝 표가 너비 때문에 다음 줄에서 조판되어도 같은 문단의 마지막
개체이며, 표 조판 완료 위치에서 문단을 종료한 뒤 다음 문단의 줄간격을 처리하라고 확정했다.
수정 진행을 승인받았다. 승인 기준 코드는 `e1c8660b9`다.

현재 페이지 항목은 표와 텍스트의 논리 순서와 다른 출력 순서를 가질 수 있다. 표의 점유 하단이
이미 `ParagraphFloatPlacement`에 있어도 후행 host 텍스트 처리 뒤의 cursor는 글줄 끝으로
돌아가 있었다. pi=2를 충돌 회피의 예외에서 빼는 것만으로 이 종료 책임을 대신하지 않는다.

- 실제 fixture 신규 검사 수정 전 실패: 표 하단805.266667px, pi=2 시작617.146667px.
- 증적: `output/6950/stage3/paragraph-closure-red.log`(1실행/1실패).
- 별도 빈 문단을 만들어 입력하지 않았다. 기존 pi=2의 줄높이1200+줄간격672HU를 사용한다.

### 24.2 구현 책임

- `ParagraphFloatPlacement::paragraph_end`: 텍스트 흐름 끝과 표 점유 하단+문단 아래 간격의
  합집합 끝을 반환한다. 점유 하단은 바깥 여백을 이미 포함하며 반복 호출로 재가산하지 않는다.
- typeset: 문단의 텍스트·컨트롤 처리가 완료되는 경계에서 공통 종료 계산을 소비한다.
  완료된 문단 끝 표 이후의 문단은 저장 위치 보정으로 순차 cursor보다 위로 역행하지 않는다.
- layout: 단 내 마지막 소유 항목 인덱스를 사전 계산하고, 그 항목 처리 후 같은 종료 계산을
  소비한다. 문단 종료 floor는 다음 문단의 잉크 유무와 무관하게 순차 진행량을 보호한다.
- 기존 빈 문단 충돌 회피 조건은 그대로다. 파일명·pi번호·텍스트 없음 여부를 새 종료 분기의
  키로 쓰지 않는다. 문단 끝 표로 이미 확정된 배치 결과의 소유 관계로 완료 시점을 판단한다.
- 실제 문서 검사에 pi=1 표→pi=2→pi=3 연결을 추가하고, 종료 계산의 멱등성·원점 이동·
  문단 아래 간격은 합성 IR이 아닌 순수 계산 계약으로 별도 검사한다. 한컴 정답지로 주장하지 않는다.

### 24.3 검증 결과와 시각 판정 요청

- 집중 nextest **49/49 PASS**, 0실패. #6950 및 #6147/#6312/#6025/#6797/#6860/
  #6879/#6718 보호 검사를 포함한다. 로그: `output/6950/stage3/paragraph-closure-focused.log`.
- 일반 SVG **3쪽**, `overflowCellLines=0`. 원본 HWPX와 한컴2024 PDF는 수정하지 않았다.
- 기존 승인 위치 보존: pi=1 첫 줄500.666667px, 표626.64..805.266667px.
- pi=2 빈 문단 시작617.146667→809.04px. 표 하단805.266667px에 바깥 아래 여백
  283HU/75=3.773333px를 소비한 위치다.
- pi=3 시작809.04→834.00px. pi=2의 줄높이1200HU와 줄간격672HU를 합한
  24.96px가 반영된다. 문단 번호나 이 좌표를 제품 코드의 조건으로 사용하지 않는다.
- review worktree의 fmt 검사 통과. fmt 이후 파생 harness drift가 검출되어 같은 원본으로
  `--prepare`를 다시 실행한 뒤 manifest `--check` 통과를 확인했다. 파생물은 제출하지 않는다.

시각 확인 자료:

- 일반 SVG: `output/6950/stage3/paragraph-closure-svg/20260909-para-table_001.svg`
- 디버그 SVG: `output/6950/stage3/paragraph-closure-debug-svg/20260909-para-table_001.svg`
- 좌표 덤프: `output/6950/stage3/paragraph-closure-extents.txt`
- 표준 비교: `output/6950/stage3/paragraph-closure-visual/para-table/` 아래
  `compare/compare_001.png`, `overlay/overlay_001.png`, `review/review_001.png`.
- 기존 PDF와 webfont/Chrome146,96dpi 비교. 페이지1 `pixel_match_percent=80.62248`,
  `visual_accuracy_proxy_percent=6.16762`. 페이지 전체의 잉크 위치·형태 차이를 포함하는
  보조 수치이며 이번 문단 간격의 합격 점수가 아니다. 비교 이미지에서도 페이지 전체 차이는 남는다.

메인테이너에게 pi=1 표 아래의 pi=2 빈 문단과 pi=3 본문 간격 판정을 요청한다.
WASM은 이전 빌드를 유지한다. 전체 회귀·최종 lint 묶음·Native Skia 검증과 원격 push/PR은
아직 수행하지 않았으며 이번 집중 통과로 대체하지 않는다.

## 25. 문단 종료 보정 시각 승인 후 전체 회귀

메인테이너가 §24 SVG의 시각 판정을 통과시키고 전체 회귀 실행을 지시했다.
현재 task HEAD `e1c8660b9`에 §24의 미커밋 제품3파일·테스트1파일을 포함한 후보가 대상이다.
review worktree와 이4파일의 SHA-256 일치를 확인했다. 기존 기준선·기대값은 바꾸지 않는다.
같은 Stage 3 검증이며 새 단계나 원격 작업을 시작하지 않는다.

현재 CPU16개, 가용 메모리28GiB, 동시 Cargo/Rust 작업 없음, 공유 target 여유372GiB를
확인해 Cargo4 jobs / nextest8 threads로 실행한다. 기존 review worktree와 고정 공유 target을
재사용하며 manifest 검사와 diff 검사를 통과했다. 신규 샘플 보안 검사 입력도 전달한다.

```bash
CARGO_BUILD_JOBS=4 \
RHWP_SECURITY_SWEEP_SAMPLES_JSON='["samples/hwpx/20260909-para-table.hwpx"]' \
RHWP_IR_SWEEP_DETAIL='20260909-para-table.hwpx' \
cargo nextest run --locked --cargo-profile release-test \
  --target-dir /home/edward/mygithub/rhwp-shared-review-target \
  --tests --test-threads 8 --no-fail-fast
```

로그: `output/6950/stage3/paragraph-closure-full.log`.
시간·자원: `output/6950/stage3/paragraph-closure-full-time.txt`.
### 25.1 최종 결과 — 전체 회귀 미통과

| 항목 | 직전 전체 (§21) | 이번 |
| --- | ---: | ---: |
| 실행 | 9,410 | 9,413 |
| 통과 | 9,410 | 9,409 |
| 실패 | 0 | **4** |
| 건너뜀 (실행 수 외) | 46 | 46 |

- nextest 종료100. 빌드5분34초, 검사401.450초, 전체11분58.71초.
  최대 RSS4,529,972KiB, swap0. nextest 버전·설정 경고는 이전 실행과 같은 환경 경고다.
- 이전9,410개와 suite 번호를 제외한 crate+test 이름으로 대조했다. 누락0개, 추가3개이며
  추가한 문단 종료 검사3개는 모두 PASS다. 대조 증적:
  `output/6950/stage3/paragraph-closure-full-delta.json`.
- #6950 배치·속성 검사, #6025 기존 핀, 신규 문서1건을 전달한 보안3종 검사 모두 PASS.
  IR 왕복 전수83.776초 PASS. slow3개는 모두 완료했으며 timeout 실패가 아니다.
- 실행 후 review manifest·diff 검사 PASS. 기준선·기존 기대값과 제품 코드는 이번 실행 중
  변경하지 않았다. main/review의 제품3파일·테스트1파일은 동일 후보다.

### 25.2 실패4검사와 실제 대상2문서

| 대상 | 실패 검사 | 검출 내용 |
| --- | --- | --- |
| `samples/issue1510_coanchored_float_tables.hwp` | `oracle_page_count_baseline::page_counts_do_not_drift_from_hancom_oracle_partition_14`, `issue_1510::issue_1510_coanchored_visible_para_float_tables_stay_on_one_page` | 같은 쪽수 변화를 중복 검출. 한컴 PDF·기준1쪽 → 현재2쪽 |
| 같은 HWP | `issue_1510::issue_1510_visible_para_float_tables_apply_offsets_without_text_overlap` | 양수 세로 offset 표 위에 남아야 하는 `filler paragraph 07`의 하단582.4px가 표 상단362.7px보다 아래로 내려감 |
| `samples/issue1510_coanchored_float_tables.hwpx` | `issue_1510::issue_1510_hwpx_unsigned_negative_offset_and_visible_flow_match_two_page_baseline` | 2쪽 단언은 통과했으나 1쪽에 있어야 하는 `filler paragraph 29`가 그 쪽에서 발견되지 않음 |

이4검사는 §21 기준 `b841ee549`에서 모두 통과했으므로 이번 누적 문단 종료 변경 이후의
신규 검출이다. 4개의 독립 결함이나 새 fixture 기준선 미등록으로 해석하지 않는다.
현재 로그와 `tests/issue_1510.rs`에서 같은 문단에 연결된 복수의 문단 기준 floating 표와
후속 본문의 흐름이 영향 대상임을 확인했다. 다만 §23/§24 중 최초 발생 변경과 정확한
원인 분기는 아직 격리하지 않았으므로 특정 helper 하나의 원인으로 단정하지 않는다.

다음은 두 실제 샘플의 원본 컨트롤 앵커·세로 offset·문단 종료/후속 흐름을 확인하고,
직전 승인 변경들을 구분해 회귀 발생 지점을 좁히는 절차다. #6950 원본의 승인 배치를
보존해야 하며 기준선 증가나 `#1510` 파일명 조건으로 통과시키지 않는다.

이번 전체 회귀 실행 요청은 완료했지만 결과는 **미통과**다. 시각 승인 자체는 보존하되
PR 준비 완료로 처리하지 않는다. 이번 턴에는 제품 추가 정정·WASM 갱신·원격 작업을 하지 않았다.

## 26. #1510 메인테이너 비교용 WASM 빌드

메인테이너가 두 실패 샘플을 한컴편집기와 직접 비교하기 위해 WASM 빌드를 지시했다.
§25와 같은 `e1c8660b9`+미커밋 후보를 사용한다. 제품·샘플·기준선은 추가 수정하지 않는다.
이번 빌드 성공을 #1510 회귀 해소나 전체 검증 통과로 해석하지 않는다.

- 표준 `docker compose --env-file .env.docker run --rm wasm` 실행.
- 기존 `.env.docker`와 named volume을 재사용하고 7700 Vite 서버를 유지한다.
- 로그: `output/6950/stage3/paragraph-closure-wasm-build.log`.
- 빌드 후 확인: `output/6950/stage3/paragraph-closure-wasm-verify.mjs`로 두 #1510
  샘플과 #6950 원본을 실제 WASM에서 로드·전쪽 SVG 생성한다. HTTP WASM과 디스크 산출물,
  HTTP sample과 원본의 해시 일치를 확인한다. 브라우저 시각 판정은 메인테이너가 수행한다.
- 빌드 exit0, wasm-pack6분54초(Rust3분59초 포함). `pkg` 소유자 `edward:edward` 유지.
- 실제 WASM에서 #1510 HWP2쪽·HWPX2쪽, #6950 원본3쪽을 로드하고 전쪽 SVG 생성 성공.
  HWP의2쪽은 회귀 재현 결과이지 정답지 일치 판정이 아니다.
- WASM SHA-256:
  `ad4cd6f055235f1493315cc0ff2b351217bd44a3d87d587ff27de0d247e28d67`.
  Vite가 변환한 JS는 현재 `pkg/rhwp_bg.wasm`을 참조하며, 실제 HTTP200 응답 해시도 동일하다.
  세 sample의 HTTP200 응답과 원본 파일 해시도 각각 일치한다.
- 검증 결과: `output/6950/stage3/paragraph-closure-wasm-verify.json`.
- 비교 URL: `http://localhost:7700/?url=/samples/issue1510_coanchored_float_tables.hwp`,
  `http://localhost:7700/?url=/samples/issue1510_coanchored_float_tables.hwpx`.
  기존 탭은 강력 새로고침 후 다시 연다. 브라우저 내 시각 판정은 아직 수행하지 않았다.

## 27. #1510 메인테이너 관찰과 흐름 소비 설계 정정

메인테이너가 첫 표의 문단 위 기준59.96mm 위치는 정상이고, 후속 본문이 표 위 공간을
사용하다가 표 구간에서만 아래로 이어지는 기존 동작을 복구해야 한다고 확인했다.

원본 HWP의 pi=0 첫 표(ci=2)는 문단 기준 세로16996HU(59.958mm), 자리차지,
글자처럼 취급=false다. 같은 문단의 후행 표 ci=3/4는 각각 음수/0 세로 offset을 갖는다.
pi=1의 저장 첫 줄 vpos는0이다. 이 원본값은 자동 줄바꿈 여부의 단독 증거로 쓰지 않는다.

코드 확인 결과:

- 기존 typeset `place_table_with_text`의 양수-offset visible float 경로는 표의
  `top..bottom`을 배제 구간으로 등록하고 텍스트 흐름은 따로 진행했다.
- 기존 layout의 `visible_float_exclusions`도 현재 텍스트 위치/높이가 표 구간과
  충돌할 때만 표 아래로 진행한다. 표 위 공간 사용 자체를 금지하지 않는다.
- §24의 추가 문단 종료 루프는 확정 표 배치가 있다는 이유만으로
  `current_height`·`min_flow_floor`를 표 하단으로 상승시켰다. 이 때문에 정상적인
  표 위의 후속 텍스트 진행을 앞단에서 차단한다. 배치 좌표와 흐름 소비를 혼동한 설계다.
- `ParagraphFloatPlacement`의 기존 생성 조건은 텍스트 뒤 컨트롤의 위치/세로 기하를
  확인하지만, 그 표가 너비 부족으로 줄을 차지했다는 사실 전체를 증명하지 않는다.
  따라서 이 타입 존재 여부를 문단 종료량 소비 조건으로 삼은 것이 부적절했다.

표 위치를 유지하면서 두 흐름 계약을 분리하는 정정 설계를 구현계획 §5.9에 보완했다.
현재 제품/WASM은 메인테이너가 확인한 실패 후보 그대로 보존한다. 정정 설계 승인 뒤
구현·집중 검증·SVG 재판정을 진행하며, 여기서 기존 코드가 복구됐다고 보고하지 않는다.

## 28. 승인된 §5.9 구현 — floating 배제 구간과 문단 종료량 분리

메인테이너의 후속 승인을 받아 구현했다. 이번 절편의 목적은 #1510의 정상 표 좌표를
유지하면서 표 위 본문 진행을 복구하고, 승인된 #6950의 줄 끝 표·후속 문단 간격을 보존하는 것이다.

### 28.1 구현과 적용 경계

- `ParagraphFloatPlacement.flow`에 `Exclusion`(본문이 도달할 때 회피할 영역)과
  `NextLine`(호스트 줄의 잔여 폭이 부족한 후행 표)을 분리했다. 세로 배치 생성만으로는
  `Exclusion`이며, 배치 정보의 존재를 문단 종료량 소비 증거로 사용하지 않는다.
- 현재 frame에서 사용하는 composed/recomposed 마지막 줄의 폰트·장평·자간별 측정 폭과
  인라인 개체 폭을 문단 가용 폭에서 뺀다. 표 너비와 좌우 바깥 여백이 잔여 폭을 초과할 때만
  `NextLine`으로 전달한다. 파일명, 형식 이름, 표 개수,59.96mm 임계값으로 분기하지 않는다.
- 이 판단은 기존의 텍스트 끝 컨트롤·양수 문단 상대 offset·호스트 줄 아래 배치 검사를
  통과한 표에 연결된다. 임의의 floating 표 전체를 인라인 표로 재분류하지 않는다.
- 명시적 줄바꿈 또는 탭 정지점의 확정 폭을 이 경로에서 얻지 못한 경우 새 흐름 소비를
  추정하지 않는다. 탭 배치를 재구현하거나 저장 줄/원본 속성을 수정하지 않는다.
- typeset의 문단 종료·후속 vpos 역행 방지와 layout의 종료 floor는 `NextLine`만 소비한다.
  `Exclusion`은 기존 `visible_float_exclusions`가 실제 충돌 시점에 회피한다.
- 표·셀의 페인트 좌표 계산, 원점 복구, 선행 배제 구간 회피는 변경하지 않았다.
  첫 조각·이월 조각에도 흐름 분류를 전달한다. 빈 문단 예외를 전역 삭제하지 않았다.

### 28.2 집중 검사

최종 실행 `output/6950/stage3/float-flow-focused-complete.log`:

- **53실행/53통과/0실패**, 선택되지 않은1825검사는 미실행이다. 전체 회귀 통과가 아니다.
- #1510 기존7검사, 정답지 쪽수 partition14, #6950 조판·속성 바인딩, #6025·#6312·
  #2439·#6797·#6718 보호 검사를 실제 실행 목록에서 확인했다.
- 새 알고리즘 검사는 폭이 정확히 맞음/부족/확정 불가, 표 폭·좌우 여백·세로 위치 변화,
  문단 종료의 멱등성과 조각/배제 구간 처리 시 분류 보존을 확인한다.
  이것은 계산 계약 검사이며 새 한컴 정답 문서를 생성한 것이 아니다.
- 기존 fixture와 정답지 기대값은 변경하지 않았다.
- `cargo fmt --all -- --check`, review의 manifest `--check`, source unit tier `--check`,
  주 브랜치 `git diff --check` 통과. 파생 suite/manifest는 review 전용이다.
- 첫 컴파일에서 함수 모듈 경로 오기가 발생해 올바른 기존 helper 경로로 정정했다.
  `float-flow-compile-error.log`에 보존했다. 이후 엔진/선택 target 빌드4분00초와44검사를
  실행했으나, `--prepare`의 suite 재배치 때문에 일부 보호 검사가 선택되지 않았음을 발견했다.
  현재 배치를 다시 조회해 최종53검사를 실행했다(추가 target 빌드26.33초, 검사0.490초).
  44와53을 합산하지 않는다.

### 28.3 수정 전후 기하

좌표는 `dump-extents`의 화면 px, 소수 첫째 자리 표시다.

| 확인 항목 | 수정 전 | 수정 후 |
| --- | --- | --- |
| #1510 HWP 전체 쪽수 | 2 | 1 |
| 첫 A 표 상단..하단 | 362.7..437.3 | 362.7..437.3 (유지) |
| filler01 상단 | 441.1 | 219.8 (표 위 흐름 복구) |
| filler07 하단 | 582.4 | 361.1 (A 표 앞) |
| filler08 상단 | 590.4 | 441.1 (A 표 뒤) |
| #1510 HWPX 전체 쪽수 | 2 | 2 |
| HWPX filler29 귀속 | 2쪽 | 1쪽 (기존 계약 복구) |

두 형식 모두 세 표의 상단·하단·x·폭 기록은 수정 전후 동일하다.
HWP의 음수/0 offset 표 사이에 출력되던 `LAYOUT_TABLE_OVERLAP` 경고도 수정 전후 동일하다.
이번 표 위 본문 흐름 회귀와 이 기존 경고를 혼동하지 않는다.

#6950 원본은 **3쪽 SVG 전체와 전체 extents가 기존 시각 승인본 `paragraph-closure-*`와
바이트 동일**이다. pi=1 시작500.666667px, 후행 표626.64..805.266667px,
pi=2 빈 줄809.04px, pi=3 시작834.00px를 보존했다.
전후 extents 공통 SHA-256은 `7978f74cf7f045a8e5fd503bd9bec44e5df3b3011749970dc3ee82a76738bc93`다.

### 28.4 시각 재판정 자료

원본 HWP/HWPX와 Hancom2024 PDF는 기존 파일을 그대로 사용했다. 새 PDF를 생성하지 않았다.
아래 모든 경로의 기준은 `/home/edward/mygithub/rhwp/`다.

- HWP SVG: `output/6950/stage3/float-flow-after-hwp/issue1510_coanchored_float_tables.svg`
- HWPX 1쪽 SVG: `output/6950/stage3/float-flow-after-hwpx/issue1510_coanchored_float_tables_001.svg`
  (같은 폴더에 `_002.svg`도 보존)
- 디버깅 SVG: `output/6950/stage3/float-flow-debug-hwp/`, `float-flow-debug-hwpx/`
- #6950 보존 SVG: `output/6950/stage3/float-flow-after-target/`
- 전후 좌표: `output/6950/stage3/float-flow-{before,after}-{hwp,hwpx}-extents.txt`

프로젝트 표준 visual sweep을 두 형식의 실제1쪽에 실행했다. HWP 단일 SVG 파일명의
`issue1510` 숫자를 도구가 추출해 비교 파일은 `_1510.png`다. **물리1510쪽이 아니라1쪽**이며,
HWPX는 `_001.png`다. 한 페이지 선택 실행의 `svg_pages=1`을 HWPX 전체 쪽수로 보고하지 않는다.

| 대상 | compare | overlay | review | pixel match | 내용 픽셀 보조 일치율 |
| --- | --- | --- | --- | --- | --- |
| HWP 1쪽 | `output/6950/stage3/float-flow-visual/1510-hwp/compare/compare_1510.png` | `output/6950/stage3/float-flow-visual/1510-hwp/overlay/overlay_1510.png` | `output/6950/stage3/float-flow-visual/1510-hwp/review/review_1510.png` | 96.31587% | 8.43461% |
| HWPX 1쪽 | `output/6950/stage3/float-flow-visual/1510-hwpx/compare/compare_001.png` | `output/6950/stage3/float-flow-visual/1510-hwpx/overlay/overlay_001.png` | `output/6950/stage3/float-flow-visual/1510-hwpx/review/review_001.png` | 96.47994% | 8.02344% |

각 review PNG를 열어 본문이 표 전후로 이어지는 위치를 확인했다. 자동 후보0/1이어도
글꼴·잉크 위치 차이가 남는다. 낮은 보조값 자체를 시각 합격으로 해석하지 않는다.
후속 메인테이너 직접 판정은 아래 §28.5에 기록한다.

source/CLI SHA-256은 `output/6950/stage3/float-flow-source-build.sha256`에 보존했다.
주 브랜치와 review의3개 Rust source·기존 test source는 동일하다.
WASM은 아직 이전 후보이므로 현재 Studio에서 이 수정 결과를 확인할 수 없다.
SVG 재판정 후 Docker WASM 확인·전체 회귀 순서로 진행한다. 이번 절편에서는 커밋·원격
push·PR을 수행하지 않았으며 PR 제출용 전체 Clippy 묶음도 아직 재실행 전이다.

### 28.5 메인테이너 시각 판정 통과와 형식별 쪽수 기준

메인테이너가 이번 수정 후 HWP와 HWPX 모두 한컴편집기와 동일하게 조판됨을 확인했다.
또한 한컴편집기에서도 이 샘플의 **HWP는1쪽, HWPX는2쪽**임을 직접 확인했다.
따라서 두 형식 모두 이번 표 전후 본문 흐름의 시각 판정 통과로 기록한다.

이 샘플의 보호 기준은 HWP↔HWPX의 쪽수 일치가 아니라 **각 입력과 그 입력에 대한 한컴
출력의 일치**다. 형식 간 쪽수를 같게 만들 목적으로 정상 출력을 보정하지 않는다.
이 관측만으로 HWP/HWPX 형식 전체의 일반적 차이나 한컴 내부 구현 원인을 단정하지 않는다.
쪽수 차이의 상세 원인은 별도 조사 전까지 미확정이다.

이번 피드백 반영은 문서 기록만이며 제품 코드·WASM·기존 테스트 기대값은 변경하지 않았다.
남은 순서는 Docker WASM 빌드·확인 후 전체 회귀 재실행이다.

## 29. 형식별 기준 유지 확인 후 전체 회귀 재실행

### 29.1 기존 회귀·래칫 기준 확인

메인테이너가 한컴의 HWP1쪽/HWPX2쪽 차이가 회귀·래칫에도 반영돼야 한다고 지시했다.
확인 결과 기존 `tests/issue_1510.rs`가 각각1쪽/2쪽과 본문 쪽 귀속을 단언한다.
`tests/fixtures/oracle_page_count_baseline.tsv`도 확장자를 포함한 경로별로
HWP는 정답1/기준1, HWPX는 정답2/기준2를 보존한다. 두 쪽수를 하나의 허용 집합으로
합친 것이 아니다. 따라서 테스트·기준선·정답지 선택 코드를 변경하지 않았다.

현재 분배에서 HWP는 page-count partition14, HWPX는 partition6이다.
기존7검사와 이 두 partition을 재실행해9/9통과했다.
로그: `output/6950/stage3/float-flow-format-ratchet-check.log`.

### 29.2 전체 실행 대상과 조건

메인테이너가 전체 회귀를 먼저 다시 실행하도록 지시했다. WASM을 교체하지 않고
시각 승인된 §28의 현재 Rust 후보에 대해 전체 nextest를 실행한다.

- 주 브랜치 `task_m100_6950`, HEAD `e1c8660b9`와 미커밋 제품3파일·테스트1파일.
- 기존 `/home/edward/mygithub/rhwp-6950-review`를 재사용한다. 위4파일은 주 브랜치와
  바이트 동일하고, base 이후의 다른 제품 변경 누락이 없음을 확인했다.
- 준비된 review manifest `--check`, `cargo fmt --all -- --check`, diff 검사 통과.
- CPU16개, 가용 메모리28GiB, 동시 Cargo 작업 없음, target 여유372GiB를 확인해
  Cargo4 jobs/nextest8 threads를 유지했다. 신규 샘플1건의 보안 검사 입력도 유지했다.

```bash
CARGO_BUILD_JOBS=4 \
RHWP_SECURITY_SWEEP_SAMPLES_JSON='["samples/hwpx/20260909-para-table.hwpx"]' \
RHWP_IR_SWEEP_DETAIL='20260909-para-table.hwpx' \
cargo nextest run --locked --cargo-profile release-test \
  --target-dir /home/edward/mygithub/rhwp-shared-review-target \
  --tests --test-threads 8 --no-fail-fast
```

로그: `output/6950/stage3/float-flow-full.log`.
시간·자원: `output/6950/stage3/float-flow-full-time.txt`.
직전 전체 결과와의 대조는 suite 재배치를 제외한 crate+test 식별자로 수행한다.

### 29.3 최종 결과 — 전체 회귀 통과

| 항목 | 직전 전체 (§25) | 이번 |
| --- | ---: | ---: |
| 실행 | 9,413 | 9,414 |
| 통과 | 9,409 | **9,414** |
| 실패 | 4 | **0** |
| 건너뜀 (실행 수 외) | 46 | 46 |

- nextest 종료0. 전체 경과9분46.27초(`/usr/bin/time`), 테스트 실행357.981초.
  최대 RSS4,539,968KiB, swap0. 기존 nextest 버전·설정 경고는 남지만 검사 실패가 아니다.
- 직전9,413개 중 누락0개. 추가1개는
  `floating_band_consumes_flow_only_when_the_tail_line_has_insufficient_space`이며 PASS다.
- 직전 실패한 #1510 HWP 쪽수·표 전후 본문·HWPX 본문 귀속3검사와 쪽수 partition14가
  모두 FAIL→PASS로 전환됐다. 다른 기존 검사에 신규 실패는 없다.
- 쪽수 래칫16개 partition 모두 PASS. HWP1쪽/HWPX2쪽의 기존 기준은 변경하지 않았다.
- 신규 #6950 샘플1건을 입력한 보안3종 검사 PASS. IR 필드 전수 왕복87.868초 PASS.
  대형 CellBreak 표 분할 성능 검사172.914초 PASS. 장시간 표시3개는 모두 완료했으며
  건너뛰거나 타임아웃 처리하지 않았다.

대조 증적은 `output/6950/stage3/float-flow-full-delta.json`이다.
`compare-nextest-runs.mjs`는 실제 실행 결과의 crate+test 식별자 수를 시작 건수와 대조하고,
nextest 마지막 실패 요약의 중복 출력을 제거한다. suite 번호 변화 때문에 누락/추가를
잘못 집계하지 않았다.

실행 후 source3파일·test1파일·CLI SHA-256을 §28 증적과 대조해 모두 일치했다.
review manifest `--check`와 diff 검사도 통과했다. 종료 점검 중 주 checkout에서
manifest `--check`를 잘못 호출해 미준비 파생 suite의 drift가 출력된 적이 있다.
읽기 전용 검사였으며 파일을 재생성하지 않았다. 실제 전체 검증을 실행한 review 경로에서
다시 확인해 통과했고, 이 경로 혼동을 제품 회귀 또는 전체 테스트 실패로 집계하지 않는다.

이번 실행에서는 제품 코드·기준선·기존 테스트 기대값·WASM을 변경하지 않았다.
시각 판정과 전체 native 회귀는 통과했으며, 현재 수정 후보의 Docker WASM 빌드·확인과
PR 제출 전 나머지 검증 게이트는 아직 별도 절차로 남아 있다. 커밋·push·PR은 수행하지 않았다.

## 30. CI Render Diff 사전 검증 (2026-09-10)

### 30.1 대상과 실행 조건

메인테이너 요청에 따라 현재 후보가 CI Render Diff를 통과할 수 있는지 확인했다.
`gh api`로 원격 `devel`의 `.github/workflows/render-diff.yml`을 조회했고,
로컬 파일과 Git blob `c65103e3368b1ece08cafee146e65f1c54e652c5`가 일치했다.
검증 대상은 §29의 HEAD와 미커밋 제품3파일·테스트1파일이며, source 해시는
전체 회귀 실행 당시와 동일하다. 리뷰 워크트리와 주 checkout의 해당 파일도 바이트 동일하다.

- 기존 `/home/edward/mygithub/rhwp-6950-review`에서 검증했다. 새로운 브랜치·워크트리는 만들지 않았다.
- WASM은 Docker `wasm` 서비스에서 `scripts/wasm-pack-locked.sh --target web --dev`로
  빌드했다. CI와 같은 dev profile이며 완료2분00초, pkg 소유권은 edward로 유지했다.
  주 checkout의 `pkg/`를 갱신하고 리뷰 워크트리에 동일 산출물을 복사했다.
- native CLI는 리뷰 워크트리에서 `cargo build --locked --features native-skia --bin rhwp
  --target-dir /home/edward/mygithub/rhwp-shared-review-target`로 빌드했다. 완료1분59초.
- Node22.18.0, Chromium build1660786 (`152.0.7946.0`)을 사용했다.
  리뷰 워크트리에서 `npm ci`를 실행했다. 주 Studio의 오래된 의존성
  (`@types/chrome`0.2.7, `puppeteer-core`25.9.0)은 건드리지 않았다.
  검증 설치본은 lockfile에 맞는 각각0.2.8, 25.10.0이다.
- 기본 fixture, max-pages1, full-suite0, Direct PDF gate 활성화, 허용차이0.02,
  direct fallback raster DPI144를 CI와 동일하게 유지했다. Canvas 허용차이는0.0005다.
- 검증 서버는7701을 사용했다. 메인테이너의7700 서버 PID60726은 유지했고,
  검증 종료 후7701 서버는 harness가 종료했다.

WASM SHA-256: `9458322ea31ae05d54f743328ccef71a3ab7abc2e6833b7151b6f94d9bec3a78`.
native-Skia CLI SHA-256: `3a77f7ee54051add23ea8ada8c943528d4fc53a4cbd9e57fb50ec483eb60bb27`.
증적: `output/6950/stage3/render-diff-binaries.sha256`.

리뷰 워크트리의 `rhwp-studio`에서 실행한 본 검사:

```bash
env PATH="/home/edward/.nvm/versions/node/v22.18.0/bin:$PATH" \
  CHROME_PATH=/home/edward/.cache/puppeteer/chromium/linux-1660786/chrome-linux/chrome \
  VITE_PORT=7701 RHWP_RENDER_DIFF_FILES='' RHWP_RENDER_DIFF_MAX_PAGES=1 \
  RHWP_RENDER_DIFF_ALL=0 RHWP_RENDER_DIFF_WRITE_IMAGES=0 \
  RHWP_RENDER_DIFF_PDF=1 RHWP_RENDER_DIFF_PDF_WRITE_IMAGES=1 \
  RHWP_RENDER_DIFF_DIRECT_PDF=1 RHWP_RENDER_DIFF_DIRECT_PDF_GATE=1 \
  RHWP_RENDER_DIFF_DIRECT_PDF_MAX_RATIO=0.02 \
  RHWP_RENDER_DIFF_DIRECT_PDF_RASTER_DPI=144 \
  RHWP_RENDER_DIFF_RHWP_BIN=/home/edward/mygithub/rhwp-shared-review-target/debug/rhwp \
  npm run e2e:render-diff:ci
```

같은 Node·Chromium 환경에서 리뷰 워크트리 루트의 다음 검사도 실행했다.

```bash
python3 scripts/renderer_baseline.py --profiles screen --browser-mode headless \
  --readiness-only \
  --output /home/edward/mygithub/rhwp/output/6950/stage3/render-diff-readiness
```

### 30.2 결과 — 로컬 CI 게이트 모두 통과

| 검사 | 결과 |
| --- | --- |
| CI 지정 JS8파일 syntax·Python compile | PASS |
| native/CanvasKit provenance self-test | PASS |
| renderer contract·CanvasKit font coverage | PASS |
| Canvas legacy/layer 비교 | 3/3 PASS |
| Direct PDF / SVG compatibility PDF 비교 | 3/3 PASS |
| CanvasKit readiness (시각·backend·성능·이미지) | 8/8 PASS, 누락0 |

Canvas 차이: KTX0.01761%, biz_plan0%, tac-case-001 0% (허용0.05%).
Direct PDF 차이: biz_plan1.15895%, tac-case-001 0.39037%, kps-ai0.67672%
(허용2%). 두 실행 모두 종료0이며 기준선·기대값·임계치를 수정하지 않았다.

보고용 Browser Canvas / compatibility PDF 비교는4건 warn, error0이다.
이 경로는 기존 CI 설계상 report-only이며 Direct PDF gate와 다르다.
실제 보고서에 browser96dpi 크기(794×1123 등)와 PDF72dpi 크기(596×842 등)가
다르게 기록돼 있다. 이 결과를 한컴 조판 회귀 또는 무경고 통과로 해석하지 않는다.

readiness 초기 로딩에서 `바탕체` 준비 오류·2d context 오류가 로그에 출력됐다.
최종 gate에서는8건 모두 실제 backend가 CanvasKit이고 `renderError=null`,
blockers 없음, visual parity·성능 검사가 통과했다. 초기 로그를 숨기거나 오류가
전혀 없었다고 기록하지 않는다. 발생 원인의 확정 조사는 이번 검증에서 하지 않았다.
`npm ci`의 의존성 보안 알림5건(낮음1·중간1·높음3)도 남겼으며 자동 수정하지 않았다.

증적:

- `output/6950/stage3/render-diff-ci-local.log`
- `output/6950/stage3/render-diff-ci-artifacts/summary.md`
- `output/6950/stage3/render-diff-ci-artifacts/pdf-summary.md` 및 원본 JSON·PDF·PNG
- `output/6950/stage3/render-diff-readiness.log`
- `output/6950/stage3/render-diff-readiness/baseline-report.md`
- `output/6950/stage3/render-diff-readiness/browser/browser-baseline-report.json`
- `output/6950/stage3/render-diff-{wasm-build,native-build,npm-ci}.log`

**판정:** 현재 후보는 CI Render Diff job의 기본 검증 조건을 로컬에서 통과했다.
GitHub runner에서 실제 성공한 것으로 보고하지 않는다. 로컬 WSL2·Rust1.93.1·
설치 폰트와 GitHub ubuntu-latest 환경의 차이, 추후 PR merge base 변경은 원격 실행에서
최종 확인해야 한다. 원격 workflow dispatch·push·PR은 수행하지 않았다.
제품 코드·테스트·baseline은 이번 검증으로 변경하지 않았다.

## 31. 최신 devel 통합 및 제출 전 검증

- 시각 승인 후보를 `716624893ef23453db724aa18c921416dcc37627`에 커밋해 보존했다.
- 최초 simulation의 base는 `4e0ce92830`이며, 충돌 해결 방침 승인 뒤 다시 fetch한
  최신 base는 `2a780e0d298` 접두의 PR #6990 병합 결과다(정확한 SHA는 merge parent로 보존).
- 양쪽 source·오늘할일 기록을 보존한다는 메인테이너 승인에 따라 실제 merge를 수행했다.
- `layout.rs`: #6950 확정 배치의 좌표·공간 예약을 유지하며 legacy 배치 경로의
  #6985 고정 글상자 전체 높이 교차 검사와 `fixed_textbox` 식별을 보존했다.
- `table_layout.rs`: #6950 `resolved_table_top`과 #6643 `wrapper_margin_already_applied`는
  서로 다른 책임이므로 모두 전달한다. 재귀 unwrap에는 확정 원점 없음(`None`)과
  이미 반영한 여백(`true`)을 함께 전달한다.
- 오늘할일은 날짜 제목 하나 아래 양쪽 이슈별 기록을 보존했다.
- 새 base에는 renderer·Studio·fixture 변경이 있으므로 §29~30 결과를 통합 후보의
  검증으로 재사용하지 않는다. 기존 리뷰 워크트리·공유 target에서 순차 재검증한다.
  baseline 상향이나 회귀 은폐는 허용하지 않는다.
