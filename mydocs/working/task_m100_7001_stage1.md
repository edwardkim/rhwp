# #7001 Stage 1 — 실행 도구·공개 모듈의 목록과 Gym 관계

- 이슈: https://github.com/edwardkim/rhwp/issues/7001
- 일자: 2026-09-11 (KST), 브랜치 `task_m100_7001`
- 제품 기준: `313bd4273dc0cc36a3f3f9b01425797636601b61`
- 승인 기록: `51239b878` — 수행계획 승인 후 Stage 1 시작.
- 상태: Stage 1 조사 완료. Stage 2 계약 대조 승인 요청.
- 성격: 메인테이너 분석. 제품 구현·리팩토링·Gym 채점 실행 없음.

## 1. 메인테이너 판단을 위한 결론

세 폴더는 하나의 agent/Gym 구현을 세 곳에 복제한 구조가 아니다.
**제품 CLI 어댑터, 별도 실험·조회 실행 파일, 공개 선택자 라이브러리**가 서로 다른 계약으로 존재한다.
따라서 Gym 분리라는 이유로 `src/bin` 또는 `src/agent`를 통째로 옮기거나 삭제하면 안 된다.

현재 분류와 우선 조사 지점은 다음과 같다.

| 대상 | 실제 역할 | Gym 관계 | 다음 판단 |
| --- | --- | --- | --- |
| `src/cli` + `src/main.rs` | 제품 CLI의 dispatch·입출력·편집·질의·작업 프로토콜 | Gym이 제품 명령을 소비 | Gym 사용 사실만으로 제품에서 제거하지 않음 |
| `src/bin/rhwp-agent` | 본 CLI 승격을 전제로 시작한 별도 운영 실험 CLI | Gym 직접 호출 참조 미검출 | 승격 후 잔존 구현과 계약 차이를 우선 비교 |
| `src/bin/rhwp-q-*` | 공개 DocumentCore 조회를 노출한 개발·진단 도구 23개 | Gym 직접 호출 참조 미검출 | 폰트 계측·계약 테스트 소비자를 보존하며 공통화 검토 |
| `src/agent` | 공개 DSEL 선택자 라이브러리 | Gym 직접 연결 및 모듈 밖 DSEL 소비 참조 미검출 | 커널 설명과 실구현 차이 확인. 외부 소비자는 미확정 |
| `src/tools/font_metric_gen.rs` | 폰트 메트릭 생성 개발 도구 | Gym 직접 호출 참조 미검출 | 제품 실행 파일과 빌드·설치 경계를 별도 분류 |

세 경로 안에서 **Gym 평가 전용이라고 확정한 실행 타깃/공개 모듈은 없다**.
이는 모든 함수의 전용성이나 외부 사용 부재를 증명한 결론이 아니다. 특히 `volume-probe`의
장기 필요성은 미확정이며, 이름·코드 생성 이력만으로 Gym 전용이라 판정하지 않았다.

## 2. 전수 목록과 재현 방법

[사람용 타깃·명령 목록](assets/issue7001/README.md),
[기계용 목록](assets/issue7001/inventory.json),
[재현 스크립트](assets/issue7001/inventory.mjs)를 함께 보존한다.

| 구역 | 추적 Rust 파일 | 물리적 줄 수 | 해석 |
| --- | ---: | ---: | --- |
| `src/bin` | 109 | 21,659 | 실행 타깃 24개, 공통 봉투·명령 모듈 포함 |
| `src/agent` | 12 | 5,019 | `mod.rs`와 DSEL 11개 파일 |
| `src/cli` | 107 | 44,323 | 본 CLI 어댑터·등록부·입출력·프로토콜 |
| 합계 | 228 | 71,001 | 주석·공백·source-side 테스트 포함. 복잡도/실행 비용 수치가 아님 |

- Cargo metadata의 루트 실행 타깃은 **26개**: 위 24개 + `rhwp` + `font-metric-gen`.
- 본 CLI 실제 최상위 dispatch **102개**, catalog **102개**, 이름 차집합 **0개**.
- rhwp-agent 등록부 **80개**, q-kit **50개**, q-more/q-pack 각각 **51개**.
  more/pack의 51개는 조회 50개와 `volume-probe` 1개다.
- 나머지 q 단일 실행 파일 **20개**의 명령 표지는 JSON `singleCommand`에 보존했다.
- 본 CLI와 rhwp-agent의 동일 이름 **15개**:
  `armor`, `bookmarks`, `capabilities`, `charts`, `digest`, `explain`, `explore`,
  `extract-data`, `fields`, `headers-footers`, `info`, `scan`, `search`, `threat-scan`, `verify`.
  `structure` ↔ `export-structure`처럼 이름이 다른 대응은 이 숫자에 포함되지 않는다.

추적 텍스트 8,273개를 정적으로 검색했다. mydocs의 역사 문서는 자동 소비자 검색에서 제외하고
관련 보고서를 별도로 읽었다. 바이너리·generated·submissions·2 MiB 초과 텍스트·미추적 파일은
자동 검색 대상이 아니다. JSON의 참조는 **문자 참조 후보**이며 주석/테스트도 포함한다.
동적 호출·별칭을 모두 해결한 호출 그래프가 아니며, 외부 사용자를 조사한 것도 아니다.
각 파일 목록의 역할은 소속 표면의 역할이지 그 파일 안 모든 함수의 상세 계약 판정이 아니다.

## 3. 소유 관계와 Gym 경계

```text
선택적 Gym (Python runner / task의 cmd·run)
    └─ 지정한 rhwp 실행 파일 → main → cli → DocumentCore / 제품 라이브러리
개발 계측 scripts / CLI 계약 tests
    └─ rhwp-agent 또는 rhwp-q-* → DocumentCore / 제품 라이브러리
제품 lib.rs
    └─ pub mod agent → dsel (선택자 파싱·선택)
       ※ rhwp-agent 실행 파일과 직접 연결된 라이브러리가 아님
```

### 3.1 Gym이 소비하는 것은 제품 명령이다

`gym/core/runner.py`의 `find_bin`은 `--bin` → `RHWP_BIN` → 로컬 `rhwp` 후보 → PATH의
`rhwp` 순서로 선택한다. runner의 `cli_argv`는 `[bin_path] + argv`를 만들고,
`gym/tools/build_baseline.py`도 `[bin_path] + resolved`를 실행한다.

추적 `gym/packs/**/*.json` **2,092개**의 `cmd`·`run` 배열 첫 항목을 집계하면
**54종**이고 전부 본 CLI dispatch에 존재한다. `info`, `fields`, `search`, `verify`,
`edit`, `replay`, `lineage`, `audit`, `gate`, `harness` 등이 포함된다.
이는 **구성상 소비 근거**다. 이번에 실행하지 않았으므로 성공률이나 실제 사용 빈도로 해석하지 않는다.
Python 동적 생성, MCP 및 자연어 문서까지 포함한 총 소비 기능 수를 54로 확정하지 않는다.

`src/cli/protocol`의 `run/replay/audit/lineage` 등은 본 제품의 작업 계획·증빙 계약이다.
Gym이 이들을 평가해도 Gym 채점 코드가 되는 것은 아니다. 실제 protocol 모듈은 제품의
`anchor_log`, `capsule_sign`, `policy_gate` 등을 사용하며 `src/agent/dsel`과 별개다.

### 3.2 Gym 계기 주석과 현재 의존은 다르다

`src/cli/queries/explore.rs`, `metadata/mcp/read.rs`, `metadata/capabilities/extended.rs`의
`[#gym]`, `outputs/raster.rs`와 `main.rs`의 `[gym_gpu_raster]`는 도입 계기의 단서다.
현재 역할은 문서 행동 메뉴·MCP 어댑터·GPU 이미지 출력이다. 해당 주석만으로 평가 전용이라고
판정하지 않는다. 위 JSON 54종에 없는 기능은 이 집계로 현재 Gym 실행을 주장하지 않는다.

세 경로 밖 `src/mcp_serve.rs`의 `rhwp://docs/gym`은 현재
`include_str!("../mydocs/manual/gym_optional_tool.md")`를 읽는다. Gym 디렉터리 의존이 아니다.
[#6916 결과보고서](../report/task_m100_6916_report.md)의 Gym 없는 제품 빌드·설치 검증을
재사용했다. 과거 검증이며 이번 Stage 1에서 빌드를 다시 한 것은 아니다.

채점기·과제·기준 풀이·리더보드는 `gym/`의 선택적 평가 책임이다. 이번 타깃 목록에 포함된
제품 명령을 그쪽으로 이동시키는 결론은 내리지 않았다.

### 3.3 src/agent의 설명과 실구현

`src/lib.rs`는 `pub mod agent`를 공개한다. `src/agent/mod.rs`의 실제 모듈 선언은
`pub mod dsel` 하나이며 `Selector`, `SelectorError`, 커널 버전 상수를 공개한다.
문서의 8층 커널(DSEL/Op/Anchor/Verify/Txn/Policy/Provenance/Manifest)은 전부 구현된
모듈 목록이 아니다. 본 CLI 102개 dispatch에 `agent` 명령도 없다.

`dsel`, `agent::dsel`, `agent::Selector`, `AGENT_KERNEL_VERSION` 검색에서 해당 모듈 밖
추적 소스 소비 후보가 검출되지 않았다. **라이브러리 공개 API라는 사실은 그대로 남으므로
dead code 또는 안전한 삭제 대상으로 확정할 수 없다.** 문서의 미래 설계와 현재 계약을
먼저 분리해서 읽어야 한다.

## 4. 왜 지금 구조가 되었는가

아래 날짜는 `git show -s --format='%h %cs %s'`의 커밋 날짜다. PR 생성/병합 시각과 혼동하지 않는다.

| 날짜·커밋 | 사실 | 현재 구조에 남은 영향 |
| --- | --- | --- |
| 08-04 `db94bde055` | #3918 rhwp-agent 9종 실험 표면 도입 | main/Cargo 수정 경합을 피하려 별도 자동 탐색 bin 선택 |
| 08-08 `7912beb1f1`, `71084a5301` | verify·scan 본 CLI 승격 | 현재 실험 CLI에도 같은 명령·별도 handler가 남음 |
| 08-16 `47006bf6a1` | #4875 DSEL 커널 1층 | 공개 선택자 모듈. 실행 파일 rhwp-agent와 별개 계보 |
| 08-20 `2d897ca04d`, `f9616a95fd` | 조회 CLI 누적 통합 및 q-pack 묶음 | 기존 공개 조회 API를 별도 명령/봉투로 노출 |
| 08-20 `b9a56a11d2`, `8a8bdcf180` | q-more/q-pack volume-probe 공통 수집기로 축소 | 과거 대량 생성 순회 코드에 대한 CodeQL 비용 보정 이력 |
| 08-25 `9983ae66d3` | #4967 동일 스냅샷 폰트 계측 도구 | 후속 폰트 분석 scripts의 실소비자로 이어짐 |

[#3918 보고서](../report/archives/task_3918_agent_toolkit_report.md),
[scan 승격 보고서](../report/archives/task_m100_3918_promotion3_scan.md),
[#5511 결과보고서](../report/archives/task_m100_5511_report.md)를 현재 소스와 대조했다.
#5511은 main.rs의 기능군을 어댑터로 옮겼지만 service layer/DIP 전환은 미착수로 종료했다.
지금 별도 실행 파일과 본 CLI가 공통 서비스 대신 각자 입력·집계를 갖는 배경과 관련된다.
이것을 #5511이 중복을 새로 만들었다는 주장으로 확대하지 않는다.

[실험 CLI 매뉴얼](../manual/agent_toolkit_cli.md)의 승격 절차에는 **실험 표면에서 그 명령 제거**가
있지만 현재 scan/verify는 잔존한다. 원인 계보의 핵심은 경합 회피를 위한 독립 표면 추가와
승격 후 두 계약의 공존이다. 누가 왜 잔존을 최종 결정했는지까지는 미확정이다.

## 5. 중복 후보와 실제 소비자

| 후보 | 정적으로 확인한 관계 | Stage 2에서 보호·대조할 것 |
| --- | --- | --- |
| main scan ↔ agent scan | 각각 디렉터리 순회·형식 판정·probe를 갖는 승격 계열 | agent JSONL, 본 CLI loader, 제한·출력 순서·실패 계약 |
| main verify ↔ agent verify | 독립 handler와 기대값 검사 구성 | 지원 expectation·암호·실패 exit·오류 봉투 |
| main export-structure ↔ agent structure | 둘 다 `queries::structure::build_structure` 사용 | `-o`, mode, JSON 봉투 등 어댑터 차이. 코어 중복으로 세지 않음 |
| main explore ↔ agent explore | `DocFacts` 집계가 양쪽에 있고 `build_menu` 공유 | main은 현재 MCP registry, agent는 도구 이름 4개 고정. 동일 문서에서 차이 가능한 입력 경로 |
| main search ↔ agent search ↔ q-kit search-all | 각각 `grep_with_context`, 쪽 텍스트의 문자열 검색, `search_all_text_native` | 주소 단위·셀 포함·대소문자·상한·입력 문법. 이름만 보고 alias 금지 |
| q-more envelope ↔ q-pack envelope | 도구명 치환 후 파일 전체 동일 | 공통 봉투로 옮겨도 tool/command/exit·stdout/stderr 보존 |
| q-more/pack volume-probe | 실제 dispatch·slot 계약 테스트가 존재 | 실소비 필요성은 미확정. 코드 축소 이력이 있어 임의 재생성/삭제 금지 |

`agent search`는 페이지별 문자열의 byte offset과 최대 500건으로 구현되어 있다.
본 CLI search는 section/paragraph/char offset 및 context 등을 제공한다. 이 차이는
단순 코드 정리로 없애면 안 되는 외부 계약이다. 이번에는 런타임 동등성 판정을 하지 않았다.

소비자 교차 확인 예:

- `scripts/font_metric_coverage_full_manifest.mjs`: `rhwp-agent` 지문 계측 호출 경로.
- `scripts/kerning_q1_baseline.mjs`, `kerning_q2_fixture_baseline.mjs`: q-kit 경로.
- `scripts/font_rank8_trace_baseline.mjs`: q-font-trace/q-text-layout 경로.
- `scripts/font_rank8_private_qualification.py`: q-font-layout-evidence 경로.
  이번 조사는 script 호출부만 읽었으며 비공개 입력·출력을 실행하거나 복제하지 않았다.
- `tests/cases/agent_q_kit_contract.rs`, `agent_q_more_contract.rs`, `agent_q_pack_contract.rs` 및
  단일 q 계약: `CARGO_BIN_EXE_*`와 JSON의 tool/command·도움말·exit 등을 고정한다.
- `tests/agent_toolkit_contract.rs`, `tests/agent_context_cost_contract.rs`:
  rhwp-agent 계약 소비자. 일부 테스트의 위치가 다르므로 `tests/cases`만 검색하면 놓친다.

전체 타깃별 참조 파일·첫 행은 inventory에 있다. 계약 테스트만 발견된 타깃의 외부 사용 부재는
증명되지 않았다. 공개 native 조회 API를 감싼 도구이므로 **wrapper 이동과 core 삭제는 별개**다.

## 6. 빌드·설치·배포·검증을 구분한 판정

| 경계 | 확인 사실 | 하지 않은 주장 |
| --- | --- | --- |
| Cargo 빌드 타깃 | 자동 bin 24개 + 명시 bin 2개, 모두 required-features 없음 | 26개가 제품 release에 들어간다는 뜻 아님 |
| 로컬/CI 검증 | ci.yml의 workspace build와 all-targets Clippy, bin 계약 테스트가 보조 도구를 소비 | 실행 시간·비용을 이번에 측정하지 않아 절감률 제시 안 함 |
| CLI 릴리즈 구성 | release-binary.yml은 `--bin rhwp`; binary와 LICENSE/README/README_EN만 포장 | 폴더 이름이 bin이라 배포된다는 뜻 아님 |
| 배포 실물 사전 확인 | v0.8.6 Linux x86_64 archive 목록에 위 네 파일만 존재 | 모든 플랫폼·배포 채널·버전의 전수검사로 확대 안 함 |
| 개발자 설치 | 설치 명령·선택 타깃에 따라 경계가 달라질 수 있음 | cargo install 기본 설치 결과는 실측하지 않음. Stage 2 확인 대상 |

따라서 지금 문제는 **제품 패키지에서 24개를 제거하는 작업이 아니라**, 개발 빌드와 검증·
설치 경계를 명확히 하고 중복된 어댑터를 정리할지 결정하는 작업이다. `src/bin`은 Cargo의
자동 실행 타깃 소스 위치이며 컴파일된 바이너리 보관 폴더가 아니다. 이름을 바꾼다면
Cargo 타깃 등록과 기존 실행 이름·테스트 소비 경로를 함께 설계해야 한다.

## 7. 검증 정산과 다음 승인 단위

- offline Cargo metadata와 실제 파일·명령 등록부를 대조했다.
- 재현 JSON의 26개 타깃·228개 파일·명령 수 및 동일 이름/차집합을 검증했다.
- 재현 스크립트 2회 출력 동일, JSON 파싱·Node syntax·문서 상대 링크·diff 공백 검사 통과.
- 목록 검증 중 단일 q 2개의 command 표지가 상수가 아니라 `envelope` 호출의 리터럴임을
  발견했다. 추출기를 보완해 20개 전부 확인했다. 제품 결함이 아닌 조사 도구의 누락 정정이다.
- 기준 SHA 대비 제품 소스·Cargo·Gym·tests·scripts·workflow 변경 없음.
- Gym 참가자 과제 수행·전수 평가, 코드 빌드·WASM·전체 회귀·원격 CI 실행 없음.
- 원격 push, PR 생성, 이슈 종료 없음. #7001 전체 분석은 아직 완료하지 않았다.

**다음은 Stage 2**: 위 여섯 중복 후보군과 volume-probe 필요성을 계약·소비자 기준으로 대조하고,
공통화 가능/호환 어댑터 유지/독립 보존/추가 확인으로 나눈다. 15개 동일 이름을 곧바로 15개
통합 대상으로 바꾸지 않는다. 이 단계에도 제품 리팩토링은 하지 않는다.

## 8. 용어

- **DSEL (Document Selector Language)**: 문서 안의 대상을 조건으로 찾아 선택하는 언어.
- **dispatch**: 사용자가 입력한 명령 이름을 실행 함수에 연결하는 분기.
- **catalog**: 명령 이름·분류 등 자기서술용 등록부. handler 소유 여부는 별개다.
- **어댑터**: 인자/파일/JSON/종료 코드를 코어 API 호출과 연결하는 계층.
- **봉투(envelope)**: 실제 결과와 함께 tool·command·schemaVersion 등을 담는 JSON 외피.
- **소비자**: 해당 명령/API의 이름·입출력 계약에 기대는 script·테스트·외부 프로그램.
- **DIP (Dependency Inversion Principle)**: 상위 정책이 구체 구현 대신 추상 계약에 의존하도록 하는 원칙.
