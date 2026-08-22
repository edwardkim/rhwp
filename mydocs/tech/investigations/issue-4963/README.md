---
kind: investigation
status: active
canonical: mydocs/plans/task_m100_4963.md
last_verified: 2026-08-22
---

# Issue #4963 W5 Oracle Profile·controlled ladder

이 디렉터리는 W4 조판 위험 상위 17개 face를 한컴 exact/missing 상태에서 비교하기 위한 기계 검증
계약을 보존한다. Stage W5-4B까지 profile 형식, deterministic fixture, SFNT/PDF 관측기, 17개 후보
준비도 원장, 기존 한컴 2022 evidence import, 한컴 2020 read-only exact-installed canary, disposable
snapshot 실행계약과 rank 1·7 acceptance ladder를 고정했다. 제품 font metric·fallback·paint 결과는
변경하지 않았다.

## 산출물

| 파일 | 역할 |
| --- | --- |
| `oracle_profile_contract.json` | W4 입력 hash·17개 queue·ladder·관계·환경·privacy 계약 |
| `oracle_profile.schema.json` | 개별 Oracle Profile JSON Schema Draft 2020-12 |
| `oracle_profile_public_fixtures.json` | Oracle 결과가 아닌 공개 synthetic 정상 fixture와 9개 negative mutation |
| `oracle_stage2_contract.json` | fixture matrix·source hash·PDF 자원 상한·privacy 계약 |
| `oracle_stage3_contract.json` | historical import hash와 현재 HWP 2020 feature/readback/canary 증적 계약 |
| `oracle_stage4_contract.json` | 3개 canary·5개 질문·snapshot restore·font delta 실행계약 |
| `oracle_stage4_current_host_preflight.json` | 현재 호스트가 disposable이 아님을 고정한 read-only 판정 |
| `oracle_stage4_public_fixtures.json` | 3개 물리 상태와 snapshot attestation 공개 contract fixture |
| `oracle_stage4_acceptance_attestation.json` | 경로·호스트명 없는 Hyper-V restore와 updated-base identity 증명 |
| `oracle_stage4_acceptance_projection.json` | rank 1·7의 8개 profile file hash와 조판 projection 공개 인덱스 |
| `oracle_stage4_rank{1,7}_acceptance_ladder.json` | exact/subst/none 물리 실행과 semantic question 재사용 계보 |
| `oracle_stage4_rank13_blocked_disposition.json` | managed font 0개에서도 exact가 남은 immutable/unmanaged 정지 근거 |
| `oracle_stage5_queue_projection.json` | 17개 face의 profile 재사용·terminal·후속 실행 action matrix |
| `fixtures/oracle_typesetting_fixture.hwpx` | rank 1 `문체부 바탕체`용 공개 synthetic HWPX canary |
| `fixtures/oracle_typesetting_fixture.manifest.json` | fixture semantic matrix·LineSeg lane·ZIP entry hash |
| `font_oracle_readiness.json` | 17개 face의 path-free source·ladder 준비도 원장 |
| `profiles/` | 한컴 2022 historical 2개, HWP 2020 read-only 1개, W5-4B acceptance 8개 profile |
| `scripts/oracle_profile_contract.mjs` | 계약·schema·W4 handoff·profile·negative fixture 실행 검사 |
| `scripts/tests/oracle_profile_contract.test.mjs` | 상태·advance·관계·권위·privacy 회귀 test |
| `scripts/generate_oracle_typesetting_fixture.py` | byte-exact HWPX fixture 생성기 |
| `scripts/font_oracle_inventory.py` | SFNT name/face/`hmtx`/outline/`fsType` 분석기 |
| `scripts/font_oracle_readiness.py` | local-only font와 기존 HFT evidence를 공개 원장으로 투영 |
| `scripts/pdf_oracle_observe.py` | 제한된 PDF font/subset/glyph/advance/line/page 관측기 |
| `scripts/oracle_stage3_historical_import.py` | hash가 맞는 기존 #2430 evidence를 profile v2로 투영 |
| `scripts/oracle_stage3_windows_canary.ps1` | font 상태 무변경 manifest·readback·HWPX open·PDF export runner |
| `scripts/oracle_stage3_profile.py` | local-only 실행 증적을 path-free acceptance profile로 투영 |
| `scripts/oracle_stage4_contract.py` | attestation·동일 입력·managed/unrelated font·restore validator |
| `scripts/oracle_stage4_profile.py` | local-only W5-4B 증거를 path-free profile·ladder·projection으로 변환 |
| `scripts/oracle_stage5_rank16_disposition.py` | rank 16 read-only 기능 탐지를 path-free blocked disposition으로 변환 |
| `scripts/oracle_stage5_queue_projection.py` | 기존 증거를 전건 disposition으로 결합하고 재계측 후보를 제한 |
| `scripts/tests/test_oracle_stage2.py` | 결정론·손상·상한·path escape·symlink 회귀 test |
| `scripts/tests/test_oracle_stage3.py` | historical 결정론·현재 canary·negative control·privacy 회귀 test |
| `scripts/tests/test_oracle_stage4.py` | snapshot·상태 membership·ambient drift·restore fail-closed test |
| `scripts/tests/test_oracle_stage4_profile.py` | acceptance hash 연결·결정론·validator·privacy 회귀 test |
| `scripts/tests/test_oracle_stage5_queue_projection.py` | 17개 queue·profile hash·terminal/actionable·privacy 회귀 test |

## 핵심 경계

- `observed`는 실제 값이 있어야 한다.
- `unavailable`, `not-applicable`, `blocked`는 값이 `null`이고 이유가 있어야 한다.
- PDF에서 관찰한 CID/glyph advance와 SFNT `hmtx` advance는 서로 다른 envelope다.
- exact, alias, official successor, document substitution, metric surrogate, Hancom missing-font를
  하나의 fallback 관계로 합치지 않는다.
- `official-successor`를 포함한 모든 확정 relation은 직접 관찰한 anchor가 필요하다.
- synthetic fixture, historical import, acceptance Oracle run을 동일한 증거 등급으로 다루지 않는다.
- 한컴 빌드 번호가 아니라 실제 HWPX open과 font readback을 feature detection 근거로 사용한다.
- 현재 HWP 2020 `11.0.0.9136`은 HWPX open과 보안 모듈 등록을 실제 통과해 acceptance-primary다.
- mutable font state는 disposable snapshot과 메인테이너 승인 없이는 실행하지 않는다.

## 검증

다음 검사는 private corpus, font bytes, 한컴 실행과 원격 GitHub 상태를 변경하지 않는다.

```bash
node --test scripts/tests/oracle_profile_contract.test.mjs
node scripts/oracle_profile_contract.mjs check
python3 -m unittest -v scripts.tests.test_oracle_stage2
python3 -m unittest -v scripts.tests.test_oracle_stage3
python3 -m unittest -v scripts.tests.test_oracle_stage4
python3 -m unittest -v scripts.tests.test_oracle_stage4_profile
python3 -m unittest -v scripts.tests.test_oracle_stage5_queue_projection
python3 scripts/oracle_stage4_contract.py check
```

개별 profile은 다음처럼 같은 계약으로 검사한다.

```bash
node scripts/oracle_profile_contract.mjs check --profile <oracle-profile.json>
```

JSON Schema 자체는 Draft 2020-12 validator로 compile한 뒤 public `validProfile`을 검증한다. status별
조건, W4 queue 결합, relation direct anchor와 `hmtx`/PDF advance 분리는 실행 validator가 추가로
검사한다.

Stage W5-2의 공개 fixture와 readiness 원장은 다음 명령으로 재생성한다. font root는 저장소 밖의 승인된
local-only 보관소를 실행 시점에만 전달하며, 경로는 산출물에 기록하지 않는다.

```bash
python3 scripts/generate_oracle_typesetting_fixture.py \
  --output-root mydocs/tech/investigations/issue-4963/fixtures
python3 scripts/font_oracle_readiness.py \
  --font-root <local-font-root> \
  --output-root mydocs/tech/investigations/issue-4963 \
  --output font_oracle_readiness.json
```

## Stage W5-3 재생성과 관찰 경계

기존 한컴 2022 evidence는 다음처럼 hash를 다시 검사하면서 생성한다.

```bash
python3 scripts/oracle_stage3_historical_import.py \
  --output-root mydocs/tech/investigations/issue-4963/profiles
```

HWP 2020 canary의 원본 HWPX·PDF·914개 ambient font manifest·원시 관측 JSON은 저장소 밖 owner-only
보관소에만 있다. 공개 profile에는 그 digest와 path-free 관찰만 남는다. Windows runner는 설치된 font
bytes와 입력 hash가 정확히 맞지 않으면 export 전에 중단하며, font 설치·제거 또는 다른 HWP process
종료를 하지 않는다. 실제 이번 관찰은 rank 9 `맑은 고딕`의 exact readback, CID TrueType subset,
30개 visual line과 1쪽을 기록했다. rank 1 `문체부 바탕체`와 KoPubWorld 2종은 local font source가
있어도 현재 Windows에는 exact-installed가 아니므로 `함초롬바탕` readback을 성공으로 승격하지 않았다.

Stage W5-4A는 다음 실행계약을 완료했다.

- 대상: rank 1 `문체부 바탕체`, rank 13 `휴먼명조`, rank 7 `KoPubWorld돋움체 Light`
- 같은 target에서는 모든 상태가 같은 HWPX bytes를 사용하고, 공개 fixture가 선언한
  `KoPubWorld바탕체 Light`만 document substitution 관계로 인정
- 5개 질문을 `exact-only`, `subst-only`, `none-related` 3개 고유 실행으로 매핑
- 직접 official-successor anchor가 없어 successor-only는 `not-provided`로 유지
- `exact-removed`와 `all-related-fonts-missing`의 managed set이 같을 때만 같은 실행을 명시적으로 재사용
- 외부 control plane snapshot, 실행 전후 restore, baseline manifest 복구와 unrelated font projection
  동일성을 모두 요구
- `휴먼명조`처럼 managed TTF 제거 뒤에도 같은 face가 readback되면 한컴 bundled HFT 가능성을
  `blocked-immutable-or-unmanaged-font`로 기록하고 missing 성공을 주장하지 않음

W5-4B 관찰 해설은
[`task_m100_4963_w5_stage4b.md`](../../../working/task_m100_4963_w5_stage4b.md)에 기록한다. rank 1과
rank 7은 updated-base에서 세 물리 상태를 완료했고 8개 semantic profile로 정규화했다. rank 13은 관리
TTF가 없는 기준선에서도 exact face가 남아 `blocked-immutable-or-unmanaged-font`로 판정했다.
`문체부 바탕체` exact bytes(`MBatang`)와 fixture-declared `KoPubWorld바탕체 Light`는 서로 다른 역할이며,
subst-only가 실제 KoPub subset을 사용했다는 주장은 하지 않는다. 17개 queue 확대는 이 acceptance
projection에 대한 메인테이너 판정 뒤 별도 게이트로 진행한다.

Stage W5-5A는 이 판정을 17개 전건 action matrix로 확장했다. rank 1·7은 완료 profile을 재사용하고,
source가 없는 10개와 보호된 ambient/HFT provider를 가진 rank 9·10·13은 증거 있는 terminal
disposition으로 닫는다. 추가 Oracle 실행 후보는 rank 8과 rank 16뿐이며, 안전한 순서는 rank 16
read-only exact profile 뒤 rank 8 controlled ladder다. 세부 근거와 rank 8의 distinct substitution 계약은
[`task_m100_4963_w5_stage5a.md`](../../../working/task_m100_4963_w5_stage5a.md)에 기록한다.

Stage W5-5B는 rank 16을 복원된 기준선에서 font resource 추가 없이 실행했다. 영문 SFNT alias는 exact로
선택됐지만 문서의 한글 face는 `함초롬바탕`으로 readback됐고 PDF도 `HCRBatang-Bold`만 사용했다.
따라서 Stage W5-3의 단발 selection 성공만으로 exact-installed profile을 발행하지 않고
`blocked-document-face-name-resolution` disposition으로 닫았다. 현재 actionable rank는 rank 8 하나이며,
세부 판정은 [`task_m100_4963_w5_stage5b.md`](../../../working/task_m100_4963_w5_stage5b.md)에 기록한다.
