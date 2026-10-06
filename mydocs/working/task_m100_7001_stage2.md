# #7001 Stage 2 — 중복 후보의 계약 대조와 보호 경계

- 이슈: https://github.com/edwardkim/rhwp/issues/7001
- 일자: 2026-09-11 (KST), 브랜치 `task_m100_7001`
- 제품 기준 SHA: `313bd4273dc0cc36a3f3f9b01425797636601b61`
- 입력 증적: Stage 1 커밋 `9931ee622`, [전수 목록·관계 보고서](task_m100_7001_stage1.md)
- 승인: 메인테이너의 Stage 2 진행 승인 후 조사.
- 상태: 계약 대조 완료. Stage 3 방향·우선순위 보고 승인 대기.
- 범위: Stage 1의 우선 후보 6군 + volume-probe, DSEL·설치·검증 경계.
  동일 이름 15종 전체나 228개 파일 전체의 동등성 증명은 아니다.

## 1. 판단 요약

**이름이 같은 명령을 합치는 것보다, 입출력 계약을 유지하면서 중복된 내부 처리를 분리하는 것이 먼저다.**
Gym이라는 기준으로 이 후보를 삭제할 근거는 추가로 발견되지 않았다.

| 후보 | 판정 | 가능한 작업 경계 | 지금 하면 안 되는 변경 |
| --- | --- | --- | --- |
| q-more/q-pack 봉투 | 실제 코드 복제 | 도구명을 매개변수로 받는 개발 CLI 공통 helper | tool/command/출처 표지·exit 변경 |
| scan | 처리 중복 + 서로 다른 출력·인증 어댑터 | 수집·정렬·형식 분류부터 공통화 후보 | agent JSONL 제거, 모든 probe 실패를 exit 1로 변경 |
| explore | 공통 코어 + 중복된 입력 집계·정책 차이 | DocFacts 집계를 공통화하되 도구 목록을 명시적 입력으로 전달 | 고정 도구 4개와 현 MCP registry를 승인 없이 동일시 |
| structure | 이미 코어 공유, 별도 어댑터 | 공통 코어 유지. 별도 승격/호환 정책 검토 | 본 CLI 파일 출력 또는 agent 요약 출력을 없애기 |
| verify | 목적은 같지만 판정 계약이 분기 | 계약 이름·옵션 의미를 먼저 고정, 평가기 공통화는 그 이후 | 한쪽 handler로 단순 위임 |
| search 3종 | 범위·주소·상한이 다른 API | 독립 보존 우선. 로더/출력 helper만 별도 검토 | byte offset을 charOffset으로 이름만 바꿔 반환 |
| volume-probe | 외형은 유사하나 다른 계산 | 실소비 필요성을 먼저 결정. 유지 시 각각의 계산 보존 | 봉투와 함께 계산까지 무차별 통합 |

위 판정은 리팩토링 실행 승인이 아니다. 다음 단계에서 최소 구현 단위와 순서를 제안한다.

## 2. scan — 수집 코어와 출력 정책의 분리 후보

근거: [본 CLI scan](../../src/cli/queries/scan.rs),
[agent scan](../../src/bin/rhwp-agent/scan.rs),
[본 CLI 로더](../../src/cli/document_io.rs), [agent 로더·봉투](../../src/bin/rhwp-agent/envelope.rs).

| 축 | 본 CLI | rhwp-agent |
| --- | --- | --- |
| 입력 | 경로 여러 개, `--probe`, `--max-depth`, `--limit`, `--json` | 동일 기본 옵션 + `--jsonl`; json/jsonl 동시 지정 거부 |
| 수집 | 디렉터리 안 hwp/hwpx/hml, 명시 파일은 확장자에 관계없이 후보 | 동일 구조 |
| 순서/상한 | 경로 정렬·중복 제거 후 limit | 동일 구조 |
| 링크 | 디렉터리 순회 중 symlink 건너뜀 | file_type 검사로 순회 중 symlink 건너뜀 |
| probe 로더 | 전역 인증을 소비하는 `load_document` | 암호 입력 없는 `DocumentCore::from_bytes` |
| 결과 | roots/files/summary + 중앙 출처 표지 | 같은 주요 필드 + tool/command/version + 인라인 출처 표지 |
| JSONL | 지원 안 함 | file 레코드들 다음 summary 레코드, 각각 schemaVersion |
| 종료 | 수집·파일 읽기 실패 1, 사용법 2, probe 판정은 데이터로 0 | 같은 큰 분류, stdout 쓰기 실패는 명시적 1 |

`--limit`은 **전수 수집 후** 적용한다. 방문 디렉터리 수/파일 크기/작업 시간을 제한하는 옵션이 아니다.
`--probe` 없이도 파일 전체를 읽는다. JSONL도 records를 모은 다음 출력하며, 즉시 스트리밍 순회가 아니다.
이를 성능 개선과 함께 바꾸면 이번 분석에서 확인한 내부 공통화보다 범위가 커진다.

링크 정책의 범위도 엄밀해야 한다. 양쪽 모두 명시된 root에서는 `Path::is_file/is_dir`를 먼저 쓰므로
**명시 root까지 모든 symlink를 차단한다는 주장은 하지 않는다**. 순회와 root 해석을 별도 테스트해야 한다.

출처 표지 차이: main은 `provenance::marked`로 실제 존재하는 `files[].probe.error`만 선언한다.
agent JSON은 probe 활성화만으로 그 경로를 선언한다. agent JSONL에는 같은 봉투가 적용되지 않는다.
출처 선언을 단순 도구명 치환으로 합칠 수 없다.

기존 보호 근거: [scan 계약](../../tests/scan_contract.rs),
[agent 계약](../../tests/agent_toolkit_contract.rs)의 `scan_classifies_mismatch_empty_and_parse_failure`,
`scan_jsonl_streams_records_then_summary`. 이번에는 이 테스트 소스를 대조했고 실행하지 않았다.

## 3. verify — 같은 이름이지만 판정 의미가 다르다

근거: [본 CLI verification](../../src/cli/queries/verification.rs),
[agent verify](../../src/bin/rhwp-agent/verify.rs).

| 축 | 본 CLI | rhwp-agent |
| --- | --- | --- |
| 기대값 저장 | 숫자/format은 Option, 텍스트/field는 Vec | 모든 기대값을 입력 순서대로 Vec |
| 숫자·format 반복 | 같은 옵션은 마지막 값이 덮어씀 | 반복된 조건 모두 평가 |
| 쪽수 숫자 범위 | u64 파싱 | u32 파싱 |
| 필드 | `--expect-field 이름=값` 필수 | `이름`만으로 존재 검사 가능, `이름=값`도 가능 |
| 빈 contains 인자 | 입력 단계에서 빈 문자열 거부 안 함 | 빈 문자열은 사용법 오류 |
| format만 검사 | 파일 읽기 후 **문서 파싱도 반드시 수행** | magic 판정만, 문서 파싱 생략 |
| contains | `doc.grep`의 문서 구조 검색, 매치 수 | 쪽별 텍스트를 개행으로 결합한 문자열의 contains, found bool |
| 계산 범위 | 필요한 기대 축을 개별 계산 | format 외 기대가 있으면 쪽 텍스트·표·필드 등을 함께 수집 |
| JSON | expectations[].kind/pass, passCount/failCount/verdict | assertions[].name/ok, failed/total/ok + tool/command/version |
| 순서 | 코드에 정한 축 순서 | 사용자가 준 기대값 순서 |
| 종료 | 만족 0, 불일치 3, 판정 불능 1, 사용법 2 | 동일 숫자이나 어떤 입력이 각 분류가 되는지 다름 |

예를 들어 파싱 가능한 문서에 `--expect-pages 1 --expect-pages 2`를 주면 main은 2만 평가하지만
agent는 1과 2를 모두 평가한다. `--expect-format`만 준 손상 문서는 main에서 파싱 실패가 될 수
있지만 agent는 magic 기반 판정까지 갈 수 있다. 이는 소스에서 확인한 분기이며 이번에 실제 파일로
재현한 실행 결과가 아니다. 어느 동작이 더 맞는지는 이 분석에서 임의 결정하지 않는다.

**추가로 발견한 로더 불일치:** 본 CLI main은 전역 인증 옵션을 받아 제거하지만 verification은
`load_document` 대신 `HwpDocument::from_bytes`를 직접 호출한다. scan/search/structure/explore와
달리 이 경로는 전역 암호를 소비하지 않는다. 따라서 “본 CLI면 모두 암호 입력 지원”이라고
공통화의 전제로 삼을 수 없다. 암호 파일 실제 재현은 미실행이며, 구현 착수 시 확인할 정확성
항목으로 남긴다. 이 상태를 영구 보호 불변식으로 지정하거나 이번에 몰래 수정하지 않는다.

기존 테스트도 양쪽 JSON을 다르게 고정한다:
[main verify 계약](../../tests/verify_contract.rs)의 `all_pass_is_exit_zero_with_envelope`는
verdict/passCount와 축 순서를 검사하고,
[agent 계약](../../tests/agent_toolkit_contract.rs)의 `verify_pass_fail_and_usage_contracts`는
ok/failed/assertions를 검사한다. 후자는 일부 기본 판정 근거이지 위 반복·암호·format-only
경계를 전부 보장하는 증거가 아니다.

## 4. structure와 explore — 공통 코어가 있다고 출력 계약까지 같지는 않다

### structure

[본 CLI](../../src/cli/queries/structure.rs)와 [agent](../../src/bin/rhwp-agent/structurecmd.rs)는
모두 `build_structure(document, mode)`를 호출한다. 기본 auto, outline/clause 선택은 같은 코어다.

- main 기본 출력은 **무봉투 pretty JSON**, `-o/--out`은 결과 파일 저장이다.
- main `--json`은 한 줄 봉투를 stdout에 출력하고 일찍 반환하므로 `-o`보다 우선한다.
- agent 기본 출력은 `nodes=N` 요약이며 `-o` 옵션이 없다. JSON에는 tool/command/version을 넣는다.
- main 로더는 전역 암호·NeedPassword exit 2 경로, agent 로더 실패는 exit 1이다.
- core 계산은 입력 Document와 mode가 같으면 공유되지만, 문서 열기/파일 쓰기/출처 표지는 별도다.

출처 원천도 현재 서로 다르다. [중앙 provenance](../../crates/rhwp-contracts/src/provenance.rs)의
export-structure는 preamble/marker/children 등도 선언한다. agent 인라인 목록은 roots의
heading/body다. helper 통합 시 기존 선언의 단순 교집합으로 축소하지 않는다.
지금은 차이를 확인했으며 프롬프트 주입의 실제 악용/안전성 판정을 수행한 것은 아니다.

### explore

[본 CLI](../../src/cli/queries/explore.rs)와 [agent](../../src/bin/rhwp-agent/explorecmd.rs)는
DocFacts를 각각 만든 후 같은 `build_menu`를 호출한다.

- table/field/chart/structure/note/hidden-text의 집계 구성은 중복된다.
- injection 탐지의 `tool_names`는 main의 현재 MCP registry와 agent의 고정 4개
  (`fill-fields`, `replace-text`, `run`, `mcp-serve`)로 다르다.
- agent JSON에는 fieldCount/tableCount가 추가로 있고, main과 사람용 출력도 다르다.
- 입력 파일은 읽기 전용이고 기본적으로 제한된 건수만 출력하는 별도 예산 옵션은 없다.
- 중복 제거 후보는 **DocFacts 수집 함수**다. 도구 목록 정책은 호출자가 전달하게 하고
  정책을 통일할지는 별도 결정해야 한다. 고정 4개로 통일하면 main의 현재 목록을 잃는다.

[explore 메뉴 테스트](../../tests/cases/explore_menu_contract.rs)는 DocFacts를 직접 구성해
코어 메뉴를 확인한다. **두 CLI가 같은 DocFacts를 구성하는지 검증하는 테스트는 아니다.**
공통화 시 registry 입력·JSON 필드를 확인하는 어댑터 테스트가 추가로 필요하다.

## 5. search — 주소와 검색 범위가 달라 독립 보존 우선

근거: [main](../../src/cli/queries/search.rs), [agent](../../src/bin/rhwp-agent/searchcmd.rs),
[q-kit](../../src/bin/rhwp-q-kit/search_all.rs),
[grep 코어](../../src/document_core/queries/grep.rs),
[검색 API](../../src/document_core/queries/search_query.rs).

| 축 | main search | agent search | q-kit search-all |
| --- | --- | --- | --- |
| 검색어 | 위치 인자, `--` 지원 | `--q`, 빈 값 거부 | `--q`, 빈 값은 API에서 빈 결과 |
| 옵션 | ignore-case/-i, limit/max-matches, context | 해당 옵션 없음 | ignore-case, include-cells |
| 원천 | 문서 구조 grep; 본문·셀·글상자·중첩 등 | 조판 후 쪽별 텍스트 추출 | search_all_text_native의 SearchHit |
| 주소 | section/paragraph/charOffset, page와 컨테이너 주소 등 | page + 쪽 텍스트의 **byte offset** | sec/para/charOffset/length와 cellContext/equationControl 등 |
| 셀 선택 | 별도 include-cells 없이 grep 범위 | 쪽 텍스트 추출의 내용/순서에 의존 | 기본 false: cell_context 및 equation_control 있는 hit 제외 |
| 상한 | 전수 검색 후 출력만 절단, total/omitted/truncated 보고 | 500개에서 매치 수집 중단, 전체 쪽 텍스트는 먼저 추출 | CLI 자체 매치 상한 옵션 없음 |
| 0건 | exit 0 | exit 0 | exit 0 |

agent는 실제 전체 매치가 정확히 500개여도 `matches.len() >= 500`으로 truncated=true다.
main은 실제 총량과 반환량을 비교한다. 공통 출력 helper를 도입하더라도 이 의미 차이를 숨기지 않는다.
특히 한글의 byte offset과 문자 인덱스는 값이 달라 같은 JSON 필드로 합치면 소비자의 위치가 틀어진다.

기존 근거: [search JSON](../../tests/search_json_contract.rs),
[대시 검색어](../../tests/search_dash_query_contract.rs),
[q-kit 계약](../../tests/cases/agent_q_kit_contract.rs).
main의 `search_json_value`는 main.rs에 남아 있고 batch/MCP도 관련 계약을 소비한다.
세 폴더만 옮기고 이 연결을 놓치는 리팩토링은 피해야 한다.

## 6. q-more/q-pack — 봉투는 복제, volume-probe는 서로 다른 계산

[more envelope](../../src/bin/rhwp-q-more/envelope.rs)와
[pack envelope](../../src/bin/rhwp-q-pack/envelope.rs)는 도구명 치환 후 **파일 전체가 동일**하다.
JSON 생성, stdout 오류 처리, 파일 로더, 1파일 파서, slot 파서가 공통화 후보다.
현재 helper의 stdout/직렬화 실패는 exit 1을 반환한다. agent helper는 stdout 실패 시
process::exit(1)를 호출하므로 세 봉투를 한꺼번에 동일 함수로 바꾸는 범위까지 확정하지 않는다.

[more probe](../../src/bin/rhwp-q-more/probe.rs)와
[pack probe](../../src/bin/rhwp-q-pack/probe.rs)의 현재 차이:

| 계산 | more | pack |
| --- | --- | --- |
| slot seed factor | 10,009 | 10,007 |
| 본문 문단 text 길이 | `chars().count()` | `len()` — UTF-8 bytes |
| 표 내부 common 합계 | 셀 수·셀 문단 수·그 문단 문자 수/컨트롤 수 추가 | 같은 추가 순회 없음 |
| 공통 외형 | 0..49 slot, slot당 280 probe, wrapping u64 합산 | 같은 외형이나 같은 결과라는 뜻 아님 |

제품/운영 소스에서 두 probe 구현·계약 테스트·문서를 제외한 `volume-probe` 직접 참조는
`git grep`으로 검출되지 않았다. 이 결과는 동적 호출/외부 사용자를 배제하지 않는다.
[more 계약](../../tests/cases/agent_q_more_contract.rs)은 과거 slot별 golden 값을 보존하지만,
[pack 계약](../../tests/cases/agent_q_pack_contract.rs)의 slot 검사는 JSON과 숫자 타입을 확인한다.
**pack의 계산값 동등성을 이 테스트만으로 주장할 수 없다.**

필요성이 미확정인 probe를 위해 새 공통 프레임워크를 먼저 만들 필요는 없다. 유지/축소/폐기 중
정책 결정은 Stage 3 제안과 메인테이너 판단으로 남긴다. 이번에 삭제하거나 golden을 수정하지 않았다.

## 7. src/agent 및 설치·검증 경계

DSEL은 `parse/select/count/select_with`, Selector AST, Node 주소와 EvalLimits 등 공개 API를 가진다.
CLI의 파일 입력·JSON/exit와 다른 계약이다. 이름이 agent라는 이유로 rhwp-agent의 봉투 helper를
여기에 넣으면 선택자 라이브러리가 프로세스 입출력 책임까지 갖게 된다. 공통화 장소로 확정하지 않는다.
공개 API 소비자 미검출을 삭제 승인으로 해석하지 않는 Stage 1 판정을 유지한다.

Cargo 1.93.1의 로컬 `cargo install --help`에서 bin 선택 옵션을 확인하고,
[Cargo 공식 설치 문서](https://doc.rust-lang.org/cargo/commands/cargo-install.html)의
Target Selection에서 기본값이 모든 binaries임을 확인했다(2026-09-11 조회).

- **릴리즈 archive**: 현재 workflow는 `--bin rhwp`만 빌드·포장한다.
- **소스 설치**: `cargo install --path .`의 기본 선택은 현재 루트 패키지 bin 26개다.
  required-features가 없으므로 선택 게이트로 제외되는 보조 bin도 없다.
  이는 metadata와 Cargo 규칙을 결합한 선택 대상 판정이며 26개 설치 성공 실측이 아니다.
- **제품만 소스 설치**: `--bin rhwp`로 선택 범위를 명시할 수 있다. 설치는 이번에 실행하지 않았다.
- **CI/개발 빌드**: workspace build/all-target Clippy와 CARGO_BIN_EXE 계약 테스트가 보조 타깃에
  의존한다. 폴더를 이동하거나 feature gate를 추가하면 이 경계의 검증 준비도 같이 설계해야 한다.
- **WASM**: 개발 CLI helper를 제품 lib의 필수 의존으로 넣으면 원래 없는 프로세스 입출력 의존이
  WASM에 번질 수 있다. 보조 도구 공유와 제품 API 공유를 분리해서 검증해야 한다.

`default-run`으로 실행 대상을 정하는 것과 설치 대상 제한은 다른 일이다.
폴더명 변경만으로 배포/설치/CI 비용 문제가 해결된다고 결론내리지 않는다.

## 8. 후속 구현의 보호 불변식과 최소 검증

보호 불변식은 “현재 발견한 모든 잘못된 동작을 영구 보존한다”는 뜻이 아니다.
기존 계약 보존 리팩토링과 판정/지원 범위 오류 보정을 구분하고, 후자는 명시적 변경 승인을 받는다.

| 보호할 경계 | 필요한 검증 입력/판정 |
| --- | --- |
| 이름·출력 | 기존 실행 이름, flags, JSON 키/타입, command/tool/schemaVersion, 사람용 출력 |
| 판정과 실패 | 성공 0 / 실행 불능 1 / 사용법 2 / verify 불일치 3; 파이프 실패 별도 |
| 데이터 출처 | 인라인/중앙 표지를 무조건 교집합으로 줄이지 않음. 문서 파생 값과 사용자 입력 구분 |
| 검색 주소 | 한글·ASCII 혼합, 빈 검색어, 하이픈, 셀/글상자/수식, 정확히 500·501건 |
| verify 의미 | 같은 옵션 반복, 순서, field 존재/빈 값, format-only/손상 파일, 전역 암호 경로 |
| scan | 명시 파일·디렉터리·중복 root·symlink, limit/depth, 모든 probe 성공과 일부 실패, JSONL |
| structure/explore | 동일 Document/mode, `--json`과 `-o` 동시 입력, MCP registry 입력, adapter별 추가 필드 |
| probe | slot 0/49/50, non-ASCII 문단, 셀 내부 문단, wrapping 값, 각각의 기존 golden |
| 설치/빌드 | target 이름·자동 탐색·선택 feature·CARGO_BIN_EXE, 제품 --bin rhwp, WASM 의존 경계 |

이는 향후 검증 설계이며 이번 Stage 2에서 실행한 회귀 테스트 목록이 아니다.
새 테스트를 작성할 때는 CONTRIBUTING의 integration source 규칙과 변경 범위별 검증 게이트를 따른다.

## 9. 수행·미수행 및 다음 단계

- 기존 Stage 1 자료를 재사용하고 후보의 파서/handler/loader/봉투/코어·소비 테스트를 정적으로 대조했다.
- 제품 기준 SHA 대비 변경 없음, 봉투 정규화 동일성, probe 차이, 문서 링크·공백을 확인했다.
- Node 정적 확인 5항목(봉투 동일성, probe seed/길이 단위, verify 로더/조건부 파싱,
  verify 숫자 표현, Stage 1 inventory 재현) 통과. 변경 문서의 상대 링크 42개 존재 확인.
  이 숫자를 CLI 회귀 테스트 통과 수로 세지 않는다.
- 로컬 Cargo help와 공식 설치 문서를 확인했다. 설치·빌드·Gym 평가·전체 코퍼스 재계측은 하지 않았다.
- CLI 실행을 생략한 이유: 이 단계의 단순 위임 불가와 공유 후보는 코드 분기/계약 필드로 판정 가능하다.
  입력별 런타임 동등성·성공률은 주장하지 않는다. 출처가 확인되지 않은 기존 바이너리도 사용하지 않았다.
- 제품 소스·Cargo·test·CI 수정, remote push·PR 생성·새 이슈 생성·close 없음.
- 미확정: 나머지 동일 이름 명령의 상세 동등성, 외부 소비자, probe 장기 필요성, 암호 파일 실제 재현,
  환경별 설치 성공과 성능/시간 절감량. 구현 전 필요한 항목은 선택한 변경 범위에 따라 검증한다.

다음 **Stage 3**에서는 이 결과로 역할 배치 대안과 최소 구현 순서를 제안한다.
계약이 다른 명령의 삭제나 전체 서비스 계층 재작성부터 시작하지 않도록 범위를 나눈다.

## 10. 용어

- **계약(contract)**: 사용자가 기대할 수 있는 인자·출력·주소·부작용·실패 방식의 약속.
- **정적 대조**: 코드를 읽고 비교하는 것. 실제 문서를 실행한 검증과 구분한다.
- **golden**: 변경 전 독립적으로 확정·보존한 기대값. 새 코드 결과로 덮어쓰지 않는다.
- **wrapping**: 정수 범위를 넘으면 정해진 비트 폭에서 순환하는 계산.
- **AST (Abstract Syntax Tree)**: 선택자 같은 문법을 파싱한 구조 트리.
- **adapter/코어**: CLI 입출력 연결부 / 그 안에서 재사용하는 문서 처리 규칙.
