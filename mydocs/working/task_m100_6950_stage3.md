# #6950 Stage 3 — 영향 검증·PR 준비

- Issue: [#6950](https://github.com/edwardkim/rhwp/issues/6950)
- 시작: 2026-09-10 메인테이너 진행 승인.
- 선행: [Stage 2](task_m100_6950_stage2.md) §9의 시각 판정 통과·작은 용지 실험 범위 제외.
- Stage 2 확정 커밋: `390d81e74d77a1541ab838b204263a07a2d0a972`.
- 작업 브랜치: `task_m100_6950`.
- 상태: **전체 회귀 검증 미통과 — PR 준비 보류**. 메인테이너가 문서 양쪽 보존 병합을
  승인했고 Docker 연결도 복구됐지만, 전체 회귀 23개 검사 실패와 대표 기존 문서 4건의
  base 대비 악화를 확인했다(§5). 원격 변경 없음.

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
