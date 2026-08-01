---
kind: report
status: active
canonical: mydocs/report/task_m100_3762/README.md
last_verified: 2026-08-02
---

# #3762 처리 기록 — M18 rhwp-python 1호 바인딩 + export-ir-schema

## 문제

`bindings_foundation.md` 가 M18 착수 조건(문서 머지 + `export-ir-schema` 계약)을
고정했고, 그 조건이 열렸다. 기존 파이썬 HWP 생태계는 6년 무릴리스에도 월 29만
다운로드가 유지된다(#2659 실측) — 그 수요를 MIT 라이선스로 흡수하는 것이 목적이다.

## 구현 1 — `export-ir-schema` (Rust, M18 착수 조건)

공개 IR 의 JSON Schema(2020-12)를 기계 산출한다. capabilities 가 **명령 표면**의
자기서술이라면, 이 스키마는 **문서 모델**의 자기서술이다.

- `src/ir_schema.rs` — 정의 **42개**, 루트 `Document`.
- 봉투: `{schemaVersion, irSchemaVersion, dialect, definitionCount, schema}`.
  `--bare` 는 봉투 없이 본문만 (JSON Schema 도구 입력용), `-o` 는 파일 산출.
- `irSchemaVersion` 은 봉투 `schemaVersion`(명령별)과 **분리**된 전역 버전이다.

**손으로 쓴 이유**: serde 파생에서 자동 추출하면 직렬화 표현이 새어 나온다 —
라운드트립 보존용 원본 바이트(`raw_stream`·`extra_streams`)나 내부 shim
(`is_hwp3_variant`)처럼 바인딩이 **알 필요도 없고 알아서도 안 되는** 필드까지
공개 계약이 된다. 명시 목록이 곧 "외부에 약속하는 IR"이다.

**추가-전용 진화**: 모든 객체가 `additionalProperties: true` 다. false 로 두면
rhwp 가 필드를 하나 더할 때마다 모든 바인딩이 동시에 깨진다. 계약 테스트가 이를 고정한다.

## 구현 2 — `bindings/python` (패키지 `rhwp`)

문서 §2 표면 판단 매트릭스의 **1차 권고(CLI 서브프로세스 래퍼)** 를 따랐다.
C ABI 는 수요 실증 후 승격 — 지금은 유지비 최저 경로다.

### 3층이 그대로 API 가 된다

| 층 | 파이썬 | 대응 |
|---|---|---|
| 1층 | `rhwp.info(path)` 등 21개 | CLI `--json` |
| 2층 | `with rhwp.open(path) as doc:` | `mcp-serve` 세션 |
| 3층 | `rhwp.Plan(...).check()/.run()` | `rhwp run` |

층 이름을 새로 짓지 않은 것도 의도적이다 — rhwp 문서를 읽은 사람이 파이썬 API 를
바로 이해하고, 반대도 성립한다.

### 판정 vs 고장 (이 바인딩의 핵심)

exit 3/4 는 **예외가 아니라 반환값의 판정 필드**다. `--verify` 불일치나 회귀 검출은
도구가 정상 동작한 결과이기 때문이다. 예외로 올리면 호출자가 `try/except` 로
"고장"처럼 다루고, 정작 봉투의 판정 근거를 읽지 않는다. 예외가 필요하면
`raise_on_verdict=True` 로 명시한다.

| exit | 파이썬 |
|---|---|
| 1 | `RhwpRuntimeError` |
| 2 | `UsageError` (+ `did-you-mean` 힌트 구조화) |
| 3/4 | 반환값 판정 필드 (옵션 시 `VerdictFailed`) |

### 이름은 기계 변환 — 수기 개명 금지

`_naming.py` 가 camelCase ↔ snake_case 를 규칙으로 잇는다. 사람이 이름을 다시
붙이면 봉투에 필드가 하나 늘 때마다 바인딩이 뒤처진다. 세 방식이 같은 값을
가리킨다: `meta.page_count` / `meta["pageCount"]` / `meta["page_count"]`.

### "모름"과 "없음"을 섞지 않는다

`changed_pages` 의 `None`(확정 불가)과 `[]`(바뀐 쪽 없음), `verify` 의
`None`(검증 안 함)과 실패는 각각 다른 결론이다. 부분 목록은 침묵보다 나쁘다.

### 오타가 조용한 `None` 이 되지 않는다

`Envelope` 는 없는 필드에 `AttributeError` 를 내고 **있는 필드를 함께 알려준다**.
없는 필드가 `None` 이면 이름을 잘못 쓴 코드가 "값이 없네"로 흘러가 가장 찾기
어려운 버그가 된다.

### 모델 생성기

`tools/gen_models.py` 가 `export-ir-schema` 를 읽어 dataclass 를 만든다.
`--check` 는 생성 결과가 디스크와 다르면 exit 1 — IR 이 바뀌었는데 모델을 다시
만들지 않은 PR 을 CI 가 잡는다.

## 개발 중 가드가 잡은 실제 결함 2건

계약 패리티 가드와 세션 계약 테스트가 **설계대로** 결함을 잡았다.

1. **`export-doclang` 누락** — `test_binding_covers_every_agent_value_command` 가
   capabilities 선언과 파이썬 API 를 대조해 잡았다. PR #3708 로 늘어난 명령을
   바인딩이 따라가지 않은 상태였다.
2. **`hwp_doc_render_page` 의 `output` 필수 인자 누락** — 도구 스키마상 필수인데
   바인딩이 빠뜨려, 세션 왕복 테스트가 서버 거부로 실패했다.

수기 목록을 뒀다면 둘 다 조용히 통과했을 것이다.

## 실측·검증

- 파이썬 **184건 green + 1 skip**(계획 `--dry-run` 은 #3759 머지 전이라 자기서술로
  확인 후 건너뜀 — 버전 불일치를 실패가 아니라 skip 으로 다루는 것이 정직하다)
  - 단위 165건은 **rhwp 빌드 없이** 돈다 (탐색·변환·예외 매핑·계획 직렬화는 순수 로직)
  - 통합 19건은 실물 바이너리·실물 문서로 왕복
- Rust: `ir_schema_contract` (스키마 건전성 — 끊어진 참조·고아 정의·닫힌 객체 전부 실패),
  `cli_json_contract` 무회귀, clippy 0, fmt clean
- 생성기 왕복: 31개 정의 → `src/rhwp/ir.py` 454줄, `--check` 멱등 확인

## 인코딩 함정 (윈도우)

가짜 픽스처(파이썬 스크립트)가 stdout/stdin 을 플랫폼 기본 인코딩으로 쓰면
윈도우(cp949)에서만 깨져 "바인딩 버그"로 오인된다. 실물 rhwp(Rust)는 콘솔
코드페이지와 무관하게 **항상 UTF-8** 이므로, 픽스처도 명시적으로 UTF-8 래퍼를
씌워야 계약을 제대로 검증한다. 이 함정으로 테스트가 두 번 실패했고, 원인은
바인딩이 아니라 픽스처였다.

## 남은 것

- 휠에 바이너리 동봉 (플랫폼별 CI 매트릭스) — 지금은 `RHWP_BIN`·`PATH` 경로만 검증
- PyPI 배포 (이름 선점 확인 필요)
- **M19(Node/TS)**: 문서 §4 대로 M18 의 봉투 매핑 규약 재사용 판정 + napi vs WASM 비교
- 계획 `--dry-run` 통합 테스트는 #3759 머지 후 skip 이 풀린다
