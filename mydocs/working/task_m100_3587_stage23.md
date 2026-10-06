# #3587 Stage 23 — D3 통합 검증과 가져오기 비용 계측

- 승인: Stage 22 다음 절차 진행 지시.
- 제품·테스트 기준: `9acd8d54f`, 브랜치 `task_m100_3587`.
- 범위: 필수 세 Clippy/전체 회귀/Native Skia/Docker WASM 및 1/10/100 가져오기 비용.
- 제품 기능 확장, 별도 Studio 결함, 원격 push·PR·댓글·이슈 종료는 포함하지 않는다.

## 검증 환경과 기준 고정

기존 `rhwp-review-3587`의 overlay를 보존한다. 해당 worktree HEAD 자체는 과거 D1이지만
현재 기준 commit의 `src`, `crates`, `tests`, `scripts`, `.config`, `.cargo`, Cargo/build 관련
추적 파일 **3,760개를 바이트 대조해 차이 0개**임을 확인했다. 따라서 이 기록은 worktree HEAD가
아닌 비교한 제품·테스트 내용에 대한 검증이다. 파생 suite와 기존 proptest 진단 seed는 제출하지 않는다.

- WSL2, 논리 CPU 16, 메모리 31 GiB 중 가용 28 GiB, 디스크 가용 345 GiB.
- 다른 Cargo/Rust 작업 없음. Docker 29.7.2 사용 가능.
- 고정 target: `/home/edward/mygithub/rhwp/target/pr-review` (시작 시 121 GiB).
- Cargo 명령은 순차 실행하며 전체 nextest 동시성은 host 기본값을 사용한다.
- 초기 비교 스크립트의 전체 파일 목록 조회가 Node 기본 버퍼를 초과했다. 범위를 검증 대상 경로로
  한정하고 버퍼 상한을 명시하여 재실행했다. 제품 오류가 아니다.

## Rust lint 및 전체 회귀

로그·측정 증거는 `output/3587/d3/` 아래 보존한다. `run-gates.mjs`는 실행 순서·명령·작업
디렉터리를 고정하고 명령별 로그와 종료 코드·시간 JSON을 남긴 로컬 실행기다.

필수 세 Clippy, workspace build, fmt, manifest 및 unit tier는 모두 통과했다.
`lint-results.json`에 명령별 결과가 있다. native Clippy 1.86초, WASM32 Clippy 44.70초,
workspace build 81.50초, all-target Clippy 71.62초이며 빌드 캐시를 사용하는 wall time이다.
전체 nextest 초회에서 `knowledge_map_field_dictionary_contract`의 누락 1건이 검출됐다.
`steps[].source` 설명은 최신 지식지도 본문에 있지만 전수 사전 행에는 없었다.
처음에는 review의 오래된 문서 때문이라고 보고했으나, 최신 문서 직접 대조로 그 설명을 정정했다.
Stage 22 문서 계약 누락을 사전 행 추가와 필드 수 336→337로 보완한다. 테스트 단언과 제품은 그대로다.
전체 실행은 끝까지 완료했다.

- 명령: `cargo nextest run --locked --cargo-profile release-test --target-dir
  /home/edward/mygithub/rhwp/target/pr-review --tests --no-fail-fast`.
- run `1509d858-865e-48db-932c-f7979cbe64e6`: **9704 PASS / 1 FAIL / 51 skipped**.
  컴파일 포함 512.97초, 실행 420.31초. 대용량 표 #2063도 174.53초에 통과했다.
- 사전 보완과 review의 최신 manual 3개 동기화 후 관련 필드 사전·agent surface·provenance
  문서 계약 재검증: run `e021c466-bcfc-4ae2-992e-079ed47c3cbf`, **23 PASS / 0 FAIL**.
  `dictionary.log`, `dictionary-results.json`으로 남겼다.
- 보완은 문서만 2행이며 제품·테스트 단언은 변경하지 않았다. **전체 실행과 수정 영향 검사의
  합산 판정으로 미해결 실패 0**이다. 단일 전체 실행이 처음부터 9705 PASS였다고 표현하지 않는다.
- nextest 권장 0.9.140/설치 0.9.137 및 미지원 JUnit 설정 키 경고는 이전 단계와 같다.
  필수 최소 버전은 충족하며, 이 작업에서 도구 업그레이드나 경고 우회는 하지 않았다.

## 비용 계측 — 완료

원본 로딩을 제외한 native preview/apply와 CLI 실행 전체를 구분한다.
연구노트 및 표·그림을 포함한 글상자 각각 HWP/파생 HWPX 입력, 1/10/100 복사본, 3회 조건이다.
각 측정은 새 프로세스이고 **N개를 한 번에 가져오는 호출**이다. N번 연속 호출의 비용이 아니다.
대상은 원본과 다른 파일·문서인 같은 서식 사본으로, 이번 비용 축은 자원 재사용 조건이다.
새 자원 이식·충돌 검증은 D1 계약 테스트와 실제 WASM 글상자 입력이 담당한다.

환경은 i9-9900KF, Rust 1.93.1, 기본 feature `release-test`이며 다른 빌드와 겹치지 않고
36회 측정했다. 아래는 3회 중앙값(ms)이다. 시간 제한·통계적 유의성·전체 템플릿 성능 보장은 아니다.

| 입력 | 개수 | native preview | native 실행 | CLI 실행·저장 전체 | 결과 파일 bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| 연구노트 HWP | 1 | 0.534 | 0.970 | 14.11 | 9728 |
| 연구노트 HWP | 10 | 1.465 | 2.276 | 36.16 | 11776 |
| 연구노트 HWP | 100 | 12.683 | 18.379 | 263.50 | 28672 |
| 연구노트 파생 HWPX | 1 | 0.442 | 0.656 | 13.70 | 9789 |
| 연구노트 파생 HWPX | 10 | 1.217 | 1.806 | 37.93 | 16477 |
| 연구노트 파생 HWPX | 100 | 8.423 | 15.057 | 273.05 | 80773 |
| 글상자 HWP | 1 | 8.502 | 8.610 | 52.65 | 159744 |
| 글상자 HWP | 10 | 9.925 | 9.302 | 76.60 | 163328 |
| 글상자 HWP | 100 | 21.932 | 19.770 | 363.30 | 195072 |
| 글상자 파생 HWPX | 1 | 6.550 | 7.113 | 43.23 | 84529 |
| 글상자 파생 HWPX | 10 | 7.336 | 7.467 | 70.05 | 107708 |
| 글상자 파생 HWPX | 100 | 16.910 | 16.181 | 378.99 | 339489 |

- CLI 전체는 프로세스 시작·양 문서 파싱·선검증·실행·직렬화·파일 저장·JSON 응답을 포함한다.
  native와 CLI의 차이를 전부 자원 이식 중복 비용으로 해석하지 않는다.
- native 실행은 preview 뒤에 측정해 lazy byte cache가 따뜻한 조건이다. 일부 실행이 preview보다
  짧은 결과는 이 조건과 측정 변동을 포함하며 더 빠른 알고리즘이라는 뜻이 아니다.
- 연구노트는 서식 29개, 글상자는 86개를 재사용했고 서식 추가는 0이다.
  복사 수와 무관하게 HWP 글상자는 바이너리 2개 추가, 파생 HWPX는 기존 2개 재사용이다.
  **HWP도 바이너리 추가 0이라고 주장하지 않는다.** importer는 바이트뿐 아니라 metadata 및
  ordinal/storage ID의 양 출력 경로 호환성을 요구한다(`import/resources/binary.rs`).
  이번 기록에서는 2개 추가의 세부 원인을 확정하지 않으며, 복사 수에 비례해 추가되지 않는 관측만 확정한다.
- 100개에서 반환 결과 JSON은 연구노트 약 2.17 MB, 글상자 약 2.71 MB다. 파일 크기만으로
  메모리 비용을 대표하지 않는다. native 프로세스 HWM 최대는 각각 HWP/HWPX 연구노트
  74,968/75,244 KiB, 글상자 98,280/96,644 KiB다. 입력 로딩·검증용 상태 문자열·preview까지
  포함한 최고값이지 코어의 추가 할당량이 아니다.
- 원본/대상 입력 파일 불변, preview 미변경, 실제 삽입 수, preview/apply 결과 일치는 36회 모두 확인했다.
  기존 C의 다른 연산·다른 시점 수치와 직접 뺄셈해 성능 회귀를 주장하지 않는다.
- 재현: `run-gates.mjs cost`가 Cargo `--message-format=json`으로 정확한 rlib를 기록하고
  `build-cost.mjs`로 local Rust probe를 링크한 뒤 `cost.mjs`를 실행했다. 새 output 경로를 사용한다.
  `cost/results.json`은 36회 원자료, `cost/summary.json`은 중앙값,
  `cost/environment.json`은 환경·코드/바이너리 지문이다. 한컴 시각 검증과 구분한다.
- CLI SHA-256: `840574460a91aaa5c32d69e66b6a38df3db687aa2b8a9fdcad9194b2540ccdde`.

## Native Skia 및 Docker WASM

Native Skia 3종 PASS (`skia-results.json`):

1. `cargo test --locked --profile release-test --target-dir <고정 target> --features native-skia --lib`:
   root 3930 PASS / 13 ignored, 내부 crate 182 PASS / 0 FAIL. 컴파일 포함 181.30초.
2. `node scripts/run-rust-test.mjs issue_2225_missing_picture_placeholder -- --cargo-profile release-test
   --target-dir <고정 target> --features native-skia`: 2 PASS / 0 FAIL, 145.73초.
3. 같은 runner의 `render_p37_direct_pdf_export`: 4 PASS / 0 FAIL, 6.33초.

`docker compose --env-file .env.docker run --rm wasm` **PASS**, wall time 469.32초.
Rust release compile 4m23s, wasm-pack 전체 7m24s이며 표준 최적화 `pkg/`를 갱신했다.

```bash
node mydocs/tech/investigations/issue-3587/probes/import-wasm-contract.mjs \
  output/3587/d3/wasm-contract
```

실제 생성 WASM에서 HWP/파생 HWPX **2개 입력 형식 PASS**(1.60초):
dry-run/apply 일치, 양쪽 상태 보존, 잘못된 JSON·한도·경계 거부, count=0,
앞뒤 문단 보존, HWP/HWPX export와 재열기를 실행했다. 동일 핸들 거부는 기존 계약대로
**JS 호출자 사전 검사**이며, wasm-bindgen의 동일 핸들 중복 borrow로부터 복구된다고 주장하지 않는다.
실행은 Node의 실제 WASM 로딩이며 브라우저 UI/CDP를 새로 시각 판정한 것은 아니다.

- 증적: `wasm-contract/result.json`, `wasm-contract-results.json`, `wasm-results.json`.
- WASM SHA-256: `6e23aa0232d780c1520567ed22ea1c66e0c0544b52a6a32b3db851fa047ca976`.
- JS SHA-256: `a6b6e1564d302881e7db02fd1a1b4864e65acc4b0ea737517240ac60e2fa708a`.
- d.ts SHA-256: `13eeeb30796fc2e8e9c68009b4aae38e7d438a449550693c4d1bea95067c2f2e`.

## 단계 판정과 다음 절차

**D3 자동 통합 검증·비용 계측 완료. 확인된 미해결 테스트 실패 0.**
전체 초회 실패와 문서 보완 후 재검증을 구분해 보존한다. 새로운 제품 코드 변경은 없다.
최종 대조는 main/review 절대 경로를 명시하여 tools/bindings까지 확장한 **7251개 파일 차이 0**,
관련 manual 3개 일치, `git diff --check`와 review manifest 검사 PASS다.
최종 사전 SHA-256은 `704940e1cb530d88ea9dfa33343c66480b3aaf5ef9896ae810891df41985420f`다.

한컴 근거는 기존 Stage 18/20 메인테이너 판정을 재사용한다. 이번 자동 실행이나 파생 HWPX를
새 독립 한컴 정답지 또는 모든 결과의 시각 일치 판정으로 승격하지 않는다.
D2의 최종 CLI 레시피 파일은 `output/3587/d2-cli-final/filled.hwp`와 `filled.hwpx`이며
그 파일에 대한 별도의 신규 한컴 시각 판정은 이번 실행에 포함되지 않았다.
미해석 raw 참조·외부 자원·ordinal 충돌 등 D1의 명시적 지원 제한도 유지한다.

다음은 **선택적 Gym 연구노트 시뮬레이션 → #3587 최종 보고·PR 준비**다.
Gym은 제품 API를 호출하는 평가 소비자이며 제품 의존성으로 되돌리지 않는다.
별도 Studio 이슈 #7065/#7084/#7090은 건드리지 않았다. 원격 push·PR·댓글·merge·이슈 close는 하지 않았다.
기존 review WIP와 생성 증적을 보존하고 파생 suite·manifest·output·pkg는 커밋하지 않는다.
