---
kind: decision
status: active
canonical: mydocs/tech/bindings_foundation.md
last_verified: 2026-08-02
---

# 외부 바인딩 공통 기반 — IR 스키마 버저닝·표면 판단·파이썬 1호 명세

M18(파이썬)·M19(Node/TS)·M20(C#/Swift) — 로드맵 #3608 의 바인딩 3계열이 공유하는
선행 결정을 한 문서로 고정한다(#3142 RFC 의 실행편). 원칙: **바인딩은 새 표면이
아니라 기존 계약의 재포장이다** — CLI `--json` 봉투(#2707 종료 코드·schemaVersion)
와 mcp-serve 가 이미 증명한 "선언-실행 단일 출처" 위에만 선다.

## 1. IR 스키마 버저닝 전략

현행 계약: 모든 `--json` 봉투는 `schemaVersion:"1.0"` 을 갖고, **필드 추가는 허용·
변경/삭제는 계약 테스트가 잡는다**(추가-전용 진화). 이를 IR 수준으로 확장한다:

- `irSchemaVersion` — `Document` IR 의 공개 JSON 표현 버전. 봉투 schemaVersion 과
  분리한다(봉투는 명령별, IR 은 전역).
- 진화 규약: 필드 추가 = minor / 의미 변경·삭제 = major. major 는 분기 회고(M17)
  승인 없이는 금지.
- 산출 명령 초안: `export-ir-schema --json` — 현행 IR 의 JSON Schema 를 기계 산출.
  스키마 자체가 자기서술(capabilities 패턴 승계)이므로 바인딩 세대가 코드 생성의
  단일 출처로 쓴다. (구현은 M18 착수 시 — 본 문서는 계약 모양만 고정.)

## 2. 표면 판단 매트릭스

| 표면 | 성능 | 배포 | 유지비 | 판단 |
|---|---|---|---|---|
| CLI 서브프로세스 래퍼 | 호출당 프로세스 기동(수십 ms) + 재파싱 | 바이너리 동봉만 | **최저** — #2707·봉투 계약 재사용, 바인딩은 봉투 파서만 | **1차 권고** |
| 장수명 서버(mcp-serve/stdio) 클라이언트 | 세션 재파싱 회피 | 동일 | 낮음 — 세션 도구 계약 재사용 | 대형 문서 반복 시 1차와 병용 |
| C ABI(cdylib) | 최고(인프로세스) | 플랫폼별 네이티브 휠/패키지 | 높음 — ABI 안정성·메모리 규약 신설 필요 | 수요 실증 후 승격 |
| WASM 재포장 | 중간 | JS 생태계 한정 | 중간 — @rhwp/core 기존재 | M19 에서 napi 와 비교 판단 |

근거: mcp-serve(#3571)가 이미 "검증된 CLI 배선을 그대로 실행하는 얇은 껍데기" 로
서버·CLI 괴리 0 을 실증했다. 바인딩도 같은 원리를 따르면 언어 수가 늘어도 계약은
한 곳(rhwp 본체)에만 있다 — 이것이 M18~M20 을 배수 확장 가능하게 하는 구조다.

## 3. 파이썬 1호 골격 명세 (M18)

- 패키지: `rhwp` (PyPI). 동봉 바이너리 탐색 순서: 환경변수 `RHWP_BIN` → 패키지
  동봉 → PATH.
- API 1층(무상태): `rhwp.info(path) -> Info` 등 — 각 함수는 CLI 봉투를 dataclass 로
  매핑. 필드명은 봉투 키를 snake_case 로 기계 변환(수기 개명 금지 — 드리프트 방지).
- API 2층(세션): `with rhwp.open(path) as doc:` — mcp-serve stdio 클라이언트로
  hwp_doc_* 도구를 그대로 노출.
- 판정 규약 승계: exit 3/4 는 예외가 아니라 반환값의 판정 필드(`verify.identical`),
  exit 2 는 `UsageError`(호출 조립 버그), exit 1 은 `RuntimeError`.
- 이주 수요: 6년 무릴리스에도 월 29만 다운로드가 유지되는 기존 파이썬 생태계
  수요를 MIT 라이선스로 흡수한다(#2659 실측 승계).

## 4. 착수 조건·수용 기준 (마일스톤별)

- **M18 착수 조건**: 본 문서 머지 + `export-ir-schema` 계약 이슈 승인.
  수용: 봉투→dataclass 자동 매핑 테스트, 무상태 5도구+세션 왕복 예제, 휠 CI.
- **M19 착수 조건**: M18 의 봉투 매핑 규약 재사용 판정(napi vs WASM 비교표 갱신).
- **M20 착수 조건**: 공공 SI 수요 실증 1건(이슈 유입 또는 사용 보고).
- 공통: 플레이북(agent_surface_playbook)의 절차·증적 규약을 바인딩 저장소에도 적용.
