# Task M100 #4329 Stage 1 — R67×R83: 스키마 버전 레지스트리 + 버전 정책 canonical

- 이슈: [#4329](https://github.com/edwardkim/rhwp/issues/4329)
- 기준 브랜치: `upstream/devel` (`e48fe8694`)
- 작업 브랜치: `task_m100_4329`
- 작성일: 2026-08-09 KST
- 상태: 구현·검증 완료

## 착수 게이트 통과 기록

- R67 게이트("트랙 I R83 과 공동 설계 진입")·R83 게이트("트랙 G R67 과 함께")는
  본 작업이 두 단계를 한 PR 로 공동 설계·구현함으로써 충족.
- 실수요 근거는 #4327 실조사 §3: 외부 소비자 2곳이 v0.7.x 스냅샷 표면 인식에
  고정된 스테일 마찰 실측 — "상류가 소비자에게 버전 추종 경로를 제공하지 않은
  구조"의 실물 피해. #4327 §7 U2(외부 소비자 대사 채널)를 DoD 에 편입했다.

## 구현 내용

1. **`src/schema_registry.rs` 신설 (lib)** — 4축 버전 상수(`ENVELOPE 1.0` ·
   `IR 1.0` · `CAPABILITIES 1.3` · `PLAN 1.0`) + `crate_version()` +
   `registry_value()`(기계 소비용 자기서술). 판올림 이력을 상수 doc comment 로
   일원화. 단위 테스트 2본(축 집합·정책 경로 실물성).
2. **산개 회수** — 봉투 리터럴 `"schemaVersion": "1.0"` 67사이트(8파일: main.rs
   53 · mcp_serve.rs 8 · ontology/provenance/plan_schema/capabilities_schema/
   ir_schema/rhwp-agent caps 각 1)와 rhwp-agent envelope.rs 의 insert 형 1곳을
   `ENVELOPE_SCHEMA_VERSION` 참조로 치환. `IR/CAPABILITIES/PLAN_SCHEMA_VERSION`
   상수 3본은 레지스트리 **재수출**로 전환(호출부 무변경). JSON Schema `$id` 의
   버전 조각 4곳(capabilities·capabilities-mcp·ir·plan)도 `format!` 파생으로 전환.
3. **capabilities 봉투 `schemaRegistry` 노출** — `capabilities_value()` 에 필드
   추가, `capabilities` 명령 recordFields 등재, capabilities 스키마에
   `$defs.SchemaRegistry` 신설 + `Capabilities.required` 등재.
   `capabilitiesSchemaVersion` 1.2→**1.3**(minor, 필드 추가).
4. **드리프트 가드** — `tests/schema_registry_contract.rs` 4본: ① 소스 스캔(src/
   전체에서 봉투 리터럴·`*_SCHEMA_VERSION: &str =` 정의·`$id` 조각 금지) ② 실행
   봉투 `schemaRegistry` ↔ 상수 일치·축 집합 고정·정책 경로 실물성 ③ 각
   `export-*-schema` 봉투·`$id` 파생 일치 ④ capabilities 스키마의 새 표면 등재.
5. **버전 정책 canonical** — `mydocs/tech/agent_runtime/version_policy.md`:
   축 정의(+비축 인접 상수 구분), minor/major 규약 단일 서술, semver 연동
   (0.x/1.0 이후 — "제안" 표지), 바인딩 추종 규칙, 소비자 대사 절차(U2),
   릴리스 태그·서명 연결(R38 게이트 ① 의 버전 정책 몫), 가드 지도, 미결정
   사항(메인테이너 판단 4건).
6. **Node 타입 재생성**(#4140 규약) — `npm run gen:types` 로 `envelopes.ts` 재생성
   (CapabilitiesEnvelope 에 `schemaRegistry?` +1줄). `ir.ts` 는 내용 무변경.

## red 실증 (실제 발생)

가드 첫 실행이 **작성자도 놓친 실제 산개 1곳을 red 로 잡았다**:

```
no_version_literals_outside_registry ... FAILED
src\diagnostics\render_geom_diff.rs:486: const SCHEMA_VERSION: &str = "1.0";
```

`render-diff --json` 봉투가 같은 봉투 축(동일 규약 문구·동일 값)을 로컬 상수로
따로 들고 있었다 — 계획된 67사이트 밖의 미계획 발견. 레지스트리 별칭
(`use … ENVELOPE_SCHEMA_VERSION as SCHEMA_VERSION`)으로 이주 후 green. 가드가
회귀 시나리오를 실제로 검출함을 구현 과정 자체가 증명한 사례다.

## 검증 (전부 로컬 실행, debug 프로필)

| 검증 | 결과 |
|---|---|
| `cargo build --bin rhwp --bin rhwp-agent` | 성공 |
| `cargo test --test schema_registry_contract` | 4/4 (포맷 적용 후 재실행 포함) |
| `cargo test --lib schema_registry` (단위) | 2/2 |
| `cargo test --test capabilities_schema_contract` | 17/17 |
| `cargo test --test cli_json_contract` | 31/31 |
| `cargo test --test provenance_contract` (R6 전수 대조 포함) | 10/10 |
| `cargo test --test agent_toolkit_contract` (rhwp-agent) | 13/13 |
| `cargo test --test mcp_server_contract` | 24/24 |
| `cargo clippy --lib --bins -- -D warnings` | 경고 0 |
| `rustfmt` (변경 파일 전체, 쓰기 모드) | 적용 완료 |
| Node: `npm run gen:types` → `gen:check` → `build` → `vitest run` | 최신 확인 · 빌드 성공 · **463/463** |
| Python: `pytest` (RHWP_BIN=debug 바이너리) | **294/294** |

전체 release-test 스위트·WASM check 는 로컬 미실행(시간 비용) — CI 게이트가
동일 검사를 수행하며, 변경 성격(값 동일·참조화 + 추가-전용 필드)상 위 표적
스위트가 위험 표면을 덮는다.

## 판단 기록

- `REQUIRED_PLAN_VERSION`(계획 파일 문법 게이트)은 레지스트리에 **편입하지
  않음** — 모듈 doc 이 "계획서 planVersion 과 독립 진화"를 명시한 별개 축.
  version_policy.md §1.1 에 혼동 방지 서술을 남김.
- 테스트 파일의 기대값 리터럴(`assert_eq!(…, "1.0")`)은 스캔 대상에서 제외 —
  이중 장부로서 유익(가드 doc comment 에 사유 명시).
- capabilities 스키마 `$defs` 추가·required 등재는 코드 생성기 시야 확보 목적
  (스키마에 없으면 생성기가 축을 통째로 모른다).

## 다음 (별도 이슈)

- R63(PyPI/npm 휠 파이프라인) — 이 정책·레지스트리를 재사용(§6 서명 뿌리 공유).
- R38 잔여(README 도달 1줄·채널 등재) — 게이트 ① 의 버전 정책 몫이 본 작업으로
  해소, 서명 도구 선택(§8-2)은 메인테이너 판단 대기.
