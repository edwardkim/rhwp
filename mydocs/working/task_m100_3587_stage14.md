# #3587 Stage 14 — C 종료 통합 검증

- 승인: 메인테이너 「다음 절차 진행을 승인합니다」, 2026-09-13.
- 시작 기준: `011248b01`, `task_m100_3587`. Stage 13 기록은 이미 커밋되어 있다.
- 범위: 전체 nextest, Native Skia 3종, Rust lint, Docker WASM 실제 실행 및 C 채우기 1/10/100회 계측.
- 상태: **C 종료 자동 검증 완료. 새 문단 블록 복제·채우기 산출물의 메인테이너 실물 판정 대기.** D/Gym·push·PR·댓글은 수행하지 않았다.

기존 C1/C2 실물 성공 판정과 B 계측은 재사용하되 현재 C 코드의 전체 검증을 대체하지 않는다.
검증 source를 고정하고 고정 review target의 Cargo 명령은 순차 실행한다.

준비 중 신규 CLI 테스트의 실행 파일 선택을 정정했다. nextest archive가 전달하는 런타임
`CARGO_BIN_EXE_rhwp`를 먼저 읽고 컴파일 시점 경로는 fallback으로만 사용한다.
제품 동작과 기대값은 바꾸지 않는다. 이 테스트 보정도 새 검증 SHA에 포함한다.

## 1차 전체 검사와 보완

`8351b73f7`에서 nextest 전체 **9,620 PASS / 8 FAIL / 47 skipped**였다.
실행 361.226초, 준비·컴파일 포함 612.778초. `output/3587/c-integrated/nextest.log`를 보존한다.
기존 조판/IR/overflow/off-canvas/text-overlap 검사는 통과했으나 전체 통과는 아니다.

| 실패 | 원인과 보완 |
| --- | --- |
| 스키마 object 정책 1건 | typed 템플릿 주소/상한의 엄격한 키 정책과 문자열 record map이 기존 정책 목록에 빠졌다. 닫힌 사유를 명시하고 record 값 문자열 제약을 검사한다. 제품을 느슨하게 만들지 않는다. |
| 계획 생성기 1건 | 기존 4-action 시퀀스만 생성했다. 새 3-action은 별도의 단독 step 생성기로 JSON·typed 요청·스키마 대조를 추가한다. 기존 시퀀스 검사는 유지한다. |
| 출처 fixture 대조 2건 | C3 MAP에 추가한 preview·operationResult·검증 이유가 소비자 fixture에서 누락됐다. 실제 MAP와 origin을 동기화한다. |
| 출처 실물 대조 1건 | 레시피에 run dry-run 호출이 없어 preview 필드를 관측하지 못했다. 면제 대신 실제 dry-run 레시피를 추가한다. |
| 원본 바이트 무효화 가드 1건 | 준비/공통 commit으로 옮긴 네 래퍼가 미분류였다. 실제 raw_stream 무효화 호출 경로를 확인하고 DelegatesTo 사유를 추가한다. 무효화 자체를 생략하지 않는다. |
| 스키마 중앙화 가드 1건 | 새 WASM JSON 경계에 봉투 버전 문자열을 직접 썼다. 동일 값의 ENVELOPE_SCHEMA_VERSION을 참조한다. |
| 지식지도 필드 사전 1건 | steps[].operationResult·steps[].workload 설명 누락. 두 필드의 출처·입력량/실측 구분을 추가한다. |

첫 실행의 실패 로그와 보완 후 결과는 분리한다. 위 항목은 C3 연동에서 함께 갱신해야 했던 누락이며,
내가 집중 검사만으로 발견하지 못했다. baseline 완화나 제외로 통과시키지 않는다.

## 보완 후 native 검증

제품·테스트 기준은 **`79be4d39d13724601024f2bf5cbf074c224a320a`**다.
review worktree는 `/home/edward/mygithub/rhwp-review-3587`, 고정 target은
`/home/edward/mygithub/rhwp/target/pr-review`이며 Cargo 명령을 순차 실행했다.
보완 후 로그와 명령·exit code는 `output/3587/c-integrated/r2/`의 `results.json` 및 각 `.log`다.

| 검사 | 결과 |
| --- | --- |
| 실패 관련 계약군 집중 검사 | **47 PASS** |
| 전체 nextest, `release-test --tests --test-threads 8 --no-fail-fast` | **9,629 PASS / 0 FAIL / 47 skipped**, 실행 415.301초 |
| `cargo fmt --all -- --check` | PASS |
| native / WASM32 lib / workspace all-targets Clippy, 각각 `-D warnings` | **3종 PASS** |
| `cargo build --locked --workspace` | PASS |
| integration manifest `--check` / source unit tier `--check` | PASS / PASS |
| Native Skia `--lib` | root 3,930 PASS·13 ignored, workspace lib 포함 합계 **4,112 PASS** |
| Native Skia 누락 이미지 / 직접 PDF 내보내기 | **2 PASS / 4 PASS** |
| doc-test | **8 PASS / 3 ignored** |

새 단독 템플릿 계획 property 검사 1건 때문에 전체 실행 수가 9,628 → 9,629로 늘었다.
기존 47 skipped는 그대로이며 실패를 skipped로 전환하지 않았다. IR·왕복·overflow·off-canvas·
text-overlap baseline도 변경하지 않았다. 신규 샘플은 기존 승인 목록을
`RHWP_SECURITY_SWEEP_SAMPLES_JSON`으로 전달했다(`r2/sample-inputs.json`).
nextest 설치 0.9.137/권장 0.9.140 및 JUnit unknown key 경고는 기존 환경 경고이며 테스트 실패와 구분한다.

## C 채우기 비용 — 1/10/100개

**복사 수 N개를 한 번의 원자 API 호출로 처리**했다. N번 따로 호출한 측정이 아니다.
동일 C API에서 bindings가 없는 복제와, 복사본마다 한 대상에 값을 채운 경우를 비교한다.
B 시점의 기계·코드 결과와 직접 뺄셈하지 않고, C 코드 안에서 채우기 증가분을 분리했다.

- 입력: `samples/rnote/labnote-001.hwp`, SHA-256
  `8401e778edc386f0a87a0df4120a2cf0733aee36c84070ddc864499bde73a1af`.
- 문단 블록: section 0의 `[12,13)`을 경계 13에 N개 복제하고, 상대
  `paragraph(0)/control(1)/cell(5)/paragraph(0)`의 빈 범위를 채웠다.
- 행: 같은 본문 표의 `[5,6)` 행을 경계 6에 N개 복제하고, 투영 경로
  `paragraph(0)/control(0)/cell(0)/paragraph(0)`를 채웠다.
- 값: `실험 N: 템플릿😀\n연구 기록 N`. 빈 bindings 대조군의 records는 빈 객체 N개다.
- 환경: WSL2, i9-9900KF / 논리 CPU 16 / RAM 약 31 GiB, Rust 1.93.1,
  기본 feature의 `release-test`(최적화·LTO 없음). 전체 검사와 Docker 빌드 사이에 단독 계측했다.
- 각 조합 3회, 매번 새 프로세스·새 원본 로드. 대조/채우기 순서를 번갈아 실행하고 중앙값을 썼다.
  원본 로드와 상태 검사는 코어 타이머 밖이다. preview 후 실제 실행을 측정했다.

단위는 ms이며, 소수점 이하의 작은 차이에 통계적 유의성이나 성능 보장을 부여하지 않는다.

| 연산 | 복사 수 | 복제만 코어 실행 | 복제·채우기 코어 실행 | 채우기 증가분 | CLI 실제 실행 전체: 복제만 → 채우기 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 문단 블록 | 1 | 0.400 | 0.412 | +0.012 | 9.91 → 9.61 |
| 문단 블록 | 10 | 1.681 | 1.990 | +0.309 | 34.74 → 30.53 |
| 문단 블록 | 100 | 14.250 | 18.531 | +4.281 | 243.50 → 261.29 |
| 행 | 1 | 0.368 | 0.382 | +0.014 | 9.87 → 10.19 |
| 행 | 10 | 0.510 | 1.030 | +0.520 | 9.67 → 10.75 |
| 행 | 100 | 1.865 | 5.332 | +3.467 | 17.16 → 19.98 |

CLI 전체는 프로세스 시작·파싱·선검증·실행·HWP 저장·JSON 반환을 포함한다.
작은 입력에서 대조군보다 빠른 측정이 있는 것은 변동이며 최적화 성과가 아니다.
100개 채우기의 CLI 전체는 문단 블록 약 **261 ms**, 행 약 **20 ms**였다.
해당 입력에서 실용적인 동작을 확인한 기초 자료이지 모든 템플릿의 지연 상한은 아니다.

CLI 실제 실행이 준비를 두 번 하는 비용도 숨기지 않는다. 채우기 입력의 native preview 중앙값은
문단 블록 1/10/100개가 **0.141 / 0.870 / 6.802 ms**, 행이 **0.159 / 0.234 / 0.907 ms**였다.
이는 추가 준비 한 번의 관측값이다. CLI 전체와 코어 실행의 차이에는 파싱·저장 등이 섞이므로
그 차이를 전부 중복 준비 비용이라고 해석하지 않는다. 이번 절편에서 최적화 변경은 하지 않았다.

100개 채우기 프로세스의 HWM은 문단 블록 최대 **19,816 KiB**, 행 최대 **10,844 KiB**였다.
원본 파싱·검증용 상태 사본·preview와 실행을 모두 포함한 프로세스 최고값으로, 순수 연산의 추가
할당량은 아니다. 조판 수는 각각 102쪽/7쪽이었으나 이를 한컴 시각 판정으로 사용하지 않는다.

재현용 입력·로컬 driver와 원자료는 `output/3587/c-integrated/`에 있다.
`r2/probe-artifacts.log`의 Cargo 산출 경로로 `native-probe.rs`를 링크했으며 임의의 오래된 rlib를
선택하지 않았다. `build-probe.mjs` → `cost.mjs`가 실행 경로이고,
`cost-results.json`은 36회 원자료, `cost-summary.json`은 중앙값,
`environment.json`은 환경과 입력 해시다. 생성된 `cost-*.hwp`는 계측 산출물이다.

## Docker WASM·CLI·MCP 실물 실행

`docker compose --env-file .env.docker run --rm wasm`은 **exit 0**, 외부 실행 시간
468.311초였다. named volume 빌드와 wasm-opt 최적화를 마친 `pkg`를 사용했다.
검증 WASM SHA-256은
`ea6771860f93406e70fbeae3b16c93b0b40ca6dc1bae3db69cd82becf3c314d3`다.

`wasm-probe.mjs`는 Node 24에서 **실제 생성된 JS/WASM 모듈**을 초기화했다.
native에서 WASM wrapper 이름만 호출한 검사와 구별한다. 브라우저 UI 편집을 자동으로
수행했다는 뜻도 아니다. 실행 중인 rhwp-studio(7700)가 HTTP 200으로 제공하는 WASM의
해시가 디스크와 같은 것도 확인했다(`wasm/http-asset.json`). 서버·사용자 탭은 재시작하지 않았다.

| 대조 | 결과 |
| --- | --- |
| 고정 양식·문단 블록·행 복제 × HWP/HWPX 입력 | **6조합 PASS** |
| WASM dry-run ↔ 실행 ↔ native CLI run ↔ 로컬 MCP stdio hwp_run_plan | operationResult 동치 |
| JSON 오류·boolean 타입 오류·잘못된 범위·마지막 record 오류 | JS 예외 전달, 해당 CLI 거부 이유 동치 |
| preview 및 오류 후 무변경 | HWP·HWPX 양쪽 export hash 유지, preview 문서 정보 유지 |
| 실제 채운 값·원형 셀 보존·HWP/HWPX 저장 재열기 | PASS, 두 포맷의 magic byte도 확인 |
| 입력 HWP 파일 | 원본 SHA-256 유지 |

HWPX 입력은 원본 HWP를 변환한 **파생 형식 probe**다. 별도의 한컴 정답지라고 주장하지 않는다.
MCP는 PDF 변환 서비스가 아니라 현재 CLI의 로컬 `hwp_run_plan` 호출이다.
원자료는 `output/3587/c-integrated/wasm/wasm-results.json`, 실행 driver는
`output/3587/c-integrated/wasm-probe.mjs`다. 최초 로컬 driver 실행은 import alias 문법 오류로
시작 전에 실패했다. 문법을 고치고 `node --check` 후 실행한 위 6조합은 모두 통과했다.

## 시각 증거 재사용과 확인 대상

고정 양식과 행 복제는 C1/C2에서 메인테이너가 승인한 **동일 값**으로 실행했다.
기존 승인 HWP/HWPX 4개와 새 출력 4개를 현재 WASM으로 각각 재열어 렌더링했을 때
**총 10쪽의 SVG hash가 모두 동일**했다. 이는 같은 엔진에서 공개 호출 경로의 결과가
기존 승인 산출물과 같다는 증거이며 새로운 한컴 판정을 대신했다고 표현하지 않는다.
기존 성공 판정을 유지하고 이 네 파일의 동일한 시각 확인을 다시 요구하지 않는다.

새로 확인할 것은 문단 블록을 **2회 복제하고 복사본마다 다른 값을 넣은 4쪽 출력**이다.
원형은 2쪽에 유지되고, 3·4쪽 두 번째 표의 대상 셀에는 `실험 1`, `실험 2` 기록을 넣었다.
파일 열기·채워진 내용·원형/앞뒤 구조 보존을 한컴과 Studio에서 확인 요청한다.

- HWP: `output/3587/c-integrated/wasm/wasm-block-from-hwp.hwp`
- HWPX: `output/3587/c-integrated/wasm/wasm-block-from-hwp.hwpx`
- SVG: 같은 폴더의 `wasm-block-from-hwp-page1.svg` ~ `page4.svg`

#7065 Studio 재편집 페이지네이션과 #7084 이모티콘 폭은 계속 별도 이슈다.
새 출력이 4쪽으로 재열렸다는 자동 검사만으로 메인테이너 시각 통과를 선언하지 않는다.
이 산출물 판정 후 C 종료 결과를 승인받고, 다음으로 D(다른 문서 자원·참조 가져오기)의
상세 계획을 검토한다. D·선택적 Gym 시나리오·#3587 전체 완료·원격 제출은 아직 남아 있다.

마지막 문서 정리에서는 `capabilities` 실측 고유 recordFields **328개**를 확인해 사전 본문의
수를 정정했다(사전 전체 336개). 새 내부 링크 3개와 `git diff --check`를 통과했고,
제품·테스트는 위 검증 SHA에서 바뀌지 않았다. review worktree는 추적 변경이 없다.
CLI·WASM 해시는 `output/3587/c-integrated/final-artifacts.json`에 남겼다.

## 용어

- **preview / dry-run**: 복사·채우기의 준비를 실제로 수행하지만 문서·파일에 반영하지 않는 실행.
- **코어 실행**: 공통 native 연산의 준비·반영 시간. 외부 CLI 프로세스와 파일 파싱·저장은 제외한다.
- **중앙값**: 3회 측정값을 정렬한 가운데 값. 신뢰구간이나 최악 실행 시간 보장은 아니다.
- **HWM (High-Water Mark)**: 프로세스가 관측한 최고 상주 메모리. 순수 연산의 추가 할당량과 다르다.
- **operationResult / workload**: 각각 적용 구조·경로 결과 / 입력 작업량이다. workload는 비용 실측값이 아니다.
