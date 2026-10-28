# #3587 Stage 22 — D2 CLI/MCP 가져오기와 반환 경로 채우기

- 승인: Stage 21 보고 뒤 메인테이너의 「다음 절차를 진행하세요」.
- 시작 commit: `88d080e76`, 작업 브랜치 `task_m100_3587`.
- 범위: `import_paragraph_block` 단독 계획 action, source 단일 읽기/지문,
  기존 `hwp_run_plan`, schema/metadata/문서, import → 반환 경로 fill 레시피.
- renderer/serializer/native 자원 이식 규칙은 변경하지 않는다. 전체 회귀·필수 세 Clippy·
  Native Skia·비용 계측은 D3이며 이번 집중 검사를 그 완료로 해석하지 않는다.

## 계약과 구현

1. source는 `source:{path,sha256}`, request는 기존 `ImportParagraphBlockRequest`다.
   64자리 SHA-256을 필수로 받으며 한 번 읽은 바이트로 검사·파싱한다.
   준비된 source 문서를 선검증과 실제 반영에서 공유한다. 파일을 재파싱하지 않는다.
2. 원본 파일 입력은 64 MiB 이하 일반 파일로 제한한다. 압축 파일 transport 상한이며
   native의 64 MiB 디코딩 바이너리·32 MiB 자원 metadata 예산과 별개다.
3. source와 대상 input/output 별칭은 거부한다. 기존 파일 식별 helper를 재사용하므로
   Unix hardlink/symlink를 포함한다. 저장 전 output 별칭도 확인한다.
   비협조 외부 프로세스의 경로 교체와 쓰기를 OS 수준에서 봉쇄한 것으로 주장하지 않는다.
4. 원본 지문 불일치는 `sourceSha256` 판정 exit 3, 파일 읽기/파싱은 exit 1,
   요청/지원 범위/상한 오류는 exit 2다. 거짓 if는 source I/O를 생략한다.
5. 기존 대상 CAS, 단독 구조 action 제한, 검증 후 저장, 출력 형식 선택을 유지한다.
   기존 `fs::write` 저장을 이번에 atomic rename으로 바꿨다고 주장하지 않는다.
6. CLI 및 MCP는 같은 실행기와 native preview/import를 사용한다. 새 서버/세션 API가 없다.
   응답의 `source.sha256`은 실제 읽은 원본의 지문이고 `operationResult`는 native 결과다.
7. plan schema는 1.4로 minor 증가, `planVersion`과 envelope는 1.0 유지.
   schema의 단독 step·source·request, property 생성기, capabilities/MCP/help,
   지식지도/CLI/템플릿 매뉴얼을 함께 갱신한다. 기존 provenance의 `preview` 및
   `steps[].operationResult` 표지가 같은 위치의 새 결과도 덮는다. 경로/지문은 호출자 데이터다.
8. 실행 레시피는 labnote-001의 pi12 블록을 다른 파일에 가져오고, 응답 mapping으로
   C의 상대 경로를 구성해 다음 계획에서 채운다. 예제 입력의 셀 주소를 엔진에 넣지 않는다.

## 검증 기록

기존 `/home/edward/mygithub/rhwp-review-3587`에 이번 파일만 overlay하고 기존 WIP는 보존했다.
generated suite는 review에만 준비하며 source commit에는 포함하지 않는다.

- 첫 집중 빌드: 새 테스트의 `sha2 0.11` digest가 `LowerHex`를 구현하지 않아 컴파일 실패.
  테스트의 해시 출력만 바이트별 2자리 hex로 수정했다. 제품 실행 실패로 판정하지 않는다.
- 후속 집중 검사 및 결과 산출은 아래 최종 기록으로 확정한다.
- 다음 실행에서 기존 CAS 계약 검사의 plan schema 예상 문자열 `1.3` 누락이 검출됐다.
  이번 additive action에 따른 `1.4`를 등재했다. CAS 조건·실패 코드·원자성 단언은 바꾸지 않았다.
  최종 실행에서는 해당 검사를 제외하지 않는다.
- property 생성기의 간이 schema 검사기가 새 64자리 hex pattern을 아직 해석하지 못했다.
  pattern 검사를 면제하지 않고 길이/ASCII hex 판정을 추가했으며, 63자리·비hex·공백 suffix
  거부 사례도 추가했다. CLI의 가져오기/채우기 실행 검사는 이 시점에 양 형식 모두 통과했다.
- 실제 Node 레시피 초회는 mapping 객체를 JSON 문자열로 비교해 key 순서 차이를 주소 차이로
  오인했다. 제품이 반환한 경로는 정상이며, 예제를 객체 구조 비교로 수정했다.
  초회 부분 산출은 `output/3587/d2-cli`, 최종 재실행은 별도 `output/3587/d2-cli-final`이다.

## 최종 검증과 남은 순서

### 최종 집중 검증 — PASS

```bash
cargo nextest run --locked --cargo-profile release-test \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review --no-fail-fast \
  -E 'test(issue_3587) | test(foreign_paste) | test(issue_4275) | test(issue_5819) | test(passthrough_invalidation) | test(issue_1058) | test(prop_edit_plan) | test(plan_schema) | test(schema_registry) | test(schema_version_registry) | test(run_plan_cas_contract) | test(provenance_contract)'
cargo clippy --locked -p rhwp --lib --bin rhwp \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings
node scripts/rust-test-suite-manifest.mjs --check
cargo fmt --all -- --check
node scripts/rust-unit-test-tiers.mjs --check
```

- nextest `6bc48b1f-922c-45a3-987e-01eb64f29509`: **265 PASS / 0 FAIL / 9491 skipped**.
  compile 5m07s, tests 6.576s. 기존 CAS 버전 검사와 새 property 검사를 포함하며 제외하지 않았다.
- native lib+CLI Clippy: **PASS**, 31.49s. WASM32·workspace/all-target 세 lint 전체는 D3에 남았다.
- fmt 및 unit tier PASS(4205 tests / 298 modules). manifest: 1294 sources,
  5573 static attrs, 28 suites + 20 exceptions. nextest 0.9.137과 repo 권장 0.9.140
  차이 및 새 설정 키 경고는 있었고 실행 실패는 아니다. 도구 업그레이드는 하지 않았다.
- source 파일 신규 5개 테스트는 양 형식 native 결과와 CLI preview/apply 일치,
  반환 경로 fill·저장 재열기, 원본/대상 input 불변, source SHA·주소·파일 오류,
  단독 step, skip/no-op/CAS 및 크기/별칭을 검사한다. MCP는 양 형식 preview와 실제 저장을 실행했다.
- 파서·renderer·serializer·native import 규칙 변경 0. 이번 검사로 Studio 시각 검증을
  새로 통과했다고 주장하지 않는다. Stage 20 한컴 정상 열림과 이번 기계 계약 검증을 구분한다.

### 실제 파일·재현 레시피

```bash
node mydocs/tech/investigations/issue-3587/probes/import-plan-recipe.mjs \
  target/pr-review/release-test/rhwp output/3587/d2-cli-final
```

새 output 폴더에서 실행할 명령이다. 이미 존재하는 폴더에서는 의도적으로 중단한다.
두 형식 모두 **dry-run → 가져오기 → 반환 경로 채우기 → 저장 → verify 재열기 PASS**.

- 최종 [filled.hwp](../../output/3587/d2-cli-final/filled.hwp),
  [filled.hwpx](../../output/3587/d2-cli-final/filled.hwpx).
- [result.json](../../output/3587/d2-cli-final/result.json): preview/apply/fill/verify 원문,
  source와 binary 지문. 같은 폴더의 import/fill 계획 4개로 재현할 수 있다.
- 원본의 pi12 블록을 대상 pi13 앞에 가져와 삽입 범위 `[13,14)`를 받았다.
  대상은 원본과 별도 파일인 같은 서식의 연구노트 사본이므로 양쪽 29개 자원을 재사용하고
  새 자원 추가는 0이다. 다른 서식 충돌/그림 이식 검사는 포함된 D1 집중 테스트의 책임이다.
- 실행 바이너리 SHA-256: `deaadef19f9ac2fed2f84f2212aa0c276307e88e3f491c0aec98441b53b2e548`.
- HWP SHA-256: `3df55f4f10db78c7fa47e2179d0b6fd6f30f78ae26d3fa482f9afe39a226e2d4`.
- HWPX SHA-256: `3753c8f66e1b692b9d6b25af07e154c2d58adc7b85838fe595a1d3113453b480`.

핵심 source SHA-256(review overlay와 일치):

| 파일 | SHA-256 |
| --- | --- |
| `cli/protocol/plan/import.rs` | `a0a4e9ee426b8e8ea7d2b4b2ba58a1de79c8a83aa0fbd0ff17530b4f297cf337` |
| `cli/protocol/plan/execution.rs` | `442a1693053be5b2a97a52c8259e7329c69c7a4b270bf8a04b3abaa6366bc141` |
| `plan_schema.rs` | `8ce5e4fe4bf37abe71cdf9ef71a8ee1da0ffb92065362901b08adee85c177c89` |
| `plan_schema/template.rs` | `e200f88c1970133b41886edad162592bb4bd7ab2c990a817305563d9b68cea76` |
| `tests/cases/issue_3587_import_plan.rs` | `1bdb7ee538be585df9b02e9d056a164bec39cf0bfd0c88e61a287553a4112350` |

### 남은 순서

D2 공개 연결/레시피 결과를 확인한 뒤 D3의 동일 최종 SHA 통합 검증·비용 계측으로 진행한다.
선택적 Gym은 제품 API 완결 뒤다. 원격 push·PR·댓글·merge 및 #3587 종료는 수행하지 않는다.
