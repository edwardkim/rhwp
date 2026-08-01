---
kind: canonical
status: active
canonical: mydocs/manual/python_binding_guide.md
last_verified: 2026-08-02
---

# 파이썬 바인딩 가이드 — `bindings/python`

M18(#3762)의 산출물인 파이썬 패키지 `rhwp` 의 설계·규약·유지보수 문서다.
설계 근거의 권위는 [`bindings_foundation.md`](../tech/bindings_foundation.md) 이고,
본 문서는 그 결정을 **실제로 구현한 방식**과 앞으로 지켜야 할 계약을 담는다.

## 1. 대원칙 — 바인딩은 새 표면이 아니다

> **바인딩은 기존 계약의 재포장이다.**

이 한 줄이 전부다. 파이썬 쪽에서 판정 로직을 새로 만들면 rhwp 본체와 바인딩이
서로 다른 답을 내는 순간이 온다. 그래서 바인딩이 하는 일은 셋뿐이다.

1. rhwp 프로세스를 띄우고 인자를 조립한다.
2. stdout 봉투를 파싱해 파이썬 객체로 감싼다.
3. 종료 코드를 파이썬 예외 체계로 옮긴다.

**하지 않는 일**: 문서 파싱, 좌표 계산, 유효성 판정, 재시도 정책.
전부 rhwp 본체가 이미 하고 있고, 두 번 하면 어긋난다.

## 2. 3층 구조 — 표면이 그대로 API 가 된다

| 층 | 파이썬 | 대응 rhwp 표면 |
|---|---|---|
| 1층 무상태 | `rhwp.info(path)` 등 | CLI `--json` 명령 |
| 2층 세션 | `with rhwp.open(path) as doc:` | `mcp-serve` 세션 도구 |
| 3층 계획 | `rhwp.Plan(...).check()/.run()` | `rhwp run` 계획 실행기 |

층 이름을 새로 짓지 않은 것도 의도적이다 — rhwp 문서를 읽은 사람이 파이썬 API 를
바로 이해하고, 반대도 성립한다.

## 3. 판정 vs 고장 — 이 바인딩의 핵심 규약

| 상황 | exit | 파이썬 |
|---|---|---|
| 성공 | 0 | 정상 반환 |
| 읽기·파싱·렌더·쓰기 실패 | 1 | `RhwpRuntimeError` |
| 인자가 틀림 (**호출자 버그**) | 2 | `UsageError` |
| 검증 단언 실패 | 3 | **반환값의 판정 필드** |
| 페이지 수 불일치 | 4 | **반환값의 판정 필드** |

exit 3/4 를 기본으로 예외로 만들지 **않는** 이유:

`--verify` 가 불일치를 보고하거나 `render-diff` 가 회귀를 검출한 것은 **도구가
정상 동작한 결과**다. 예외로 올리면 호출자가 `try/except` 로 "고장"처럼 다루게
되고, 정작 봉투에 담긴 판정 근거(`diff_count`·`status`·`pages`)를 읽지 않는다.

```python
result = rhwp.export_hwpx("원본.hwp", out="변환본.hwpx", verify=True)
if not result.verify.identical:
    print(f"차이 {result.verify.diff_count}건")   # 근거를 읽고 판단
```

예외가 필요하면 `raise_on_verdict=True` 로 **명시**한다. 기본값을 뒤집지 않는다.

## 4. 이름 규약 — 수기 개명 금지

봉투 키(camelCase)와 파이썬 속성(snake_case) 사이는 `_naming.py` 의 **기계 변환**이
잇는다. 사람이 이름을 다시 붙이기 시작하면 봉투에 필드가 하나 늘 때마다 바인딩이
뒤처지고, 어느 쪽이 맞는지 알 수 없게 된다.

```python
meta.page_count      # 속성 (변환)
meta["pageCount"]    # 원문 키
meta["page_count"]   # 변환 키
```

세 방식이 같은 값을 가리킨다. 원문 키를 계속 받는 이유는, 봉투 문서를 보고 코드를
쓰는 사람이 변환 규칙을 몰라도 되게 하기 위함이다.

## 5. "모름"과 "없음"을 섞지 않는다

```python
result.changed_pages    # None = 확정 불가 / [] = 바뀐 쪽 없음 / [0,2] = 그 쪽들
result.verify           # None = 검증 안 함 (실패가 아님)
```

부분 목록은 침묵보다 나쁘다 — 빠뜨린 항목이 있는 목록은 거짓 통과를 만든다.
rhwp 가 확정할 수 없을 때 `null` 을 내는 규약을 바인딩이 `None` 으로 그대로 전한다.
**둘을 falsy 로 뭉뚱그리지 말 것.**

## 6. 오타는 조용히 넘어가지 않는다

```python
meta.page_conut          # AttributeError — 있는 필드를 함께 알려준다
```

없는 필드가 `None` 이 되면, 이름을 잘못 쓴 코드가 "값이 없네"로 흘러가 가장 찾기
어려운 버그가 된다. `Envelope` 는 없는 키에 대해 실패한다.

## 7. 바이너리 탐색 — 순서가 계약이다

1. 환경변수 `RHWP_BIN`
2. 패키지 동봉 (`rhwp/_bin/`)
3. `PATH`

순서를 뒤집으면 개발자가 로컬 빌드를 가리켜도 동봉본이 실행돼 "왜 수정이 반영
안 되지"라는 진단 불가 상황이 생긴다.

**환경변수를 줬는데 못 쓰면 조용히 다음으로 넘어가지 않고 즉시 실패한다.**
사용자는 그 바이너리를 쓰고 있다고 믿는데 다른 게 실행되면 디버깅이 불가능하다.

## 8. IR 스키마 — 코드 생성의 단일 출처

`rhwp export-ir-schema` (#3762)가 공개 IR 의 JSON Schema 를 낸다.
바인딩은 IR 모양을 **하드코딩하지 않고** 이 스키마를 읽는다.

```bash
rhwp export-ir-schema --bare -o ir-schema.json   # JSON Schema 도구 입력용
python tools/gen_models.py -o src/rhwp/ir.py     # 모델 생성
python tools/gen_models.py -o src/rhwp/ir.py --check   # CI: 최신인지 검사
```

`--check` 는 생성 결과가 디스크와 다르면 exit 1 이다. IR 이 바뀌었는데 모델을
다시 만들지 않은 PR 을 CI 가 잡는다.

### 스키마가 공개 표면을 좁히는 이유

serde 파생에서 자동 추출하면 "직렬화 표현"이 새어 나온다 — 라운드트립 보존용
원본 바이트(`raw_stream`·`extra_streams`)나 내부 shim(`is_hwp3_variant`)처럼
**바인딩이 알 필요도 없고 알아서도 안 되는** 필드까지 공개 계약이 된다.
`src/ir_schema.rs` 에 명시적으로 쓴 목록이 곧 "우리가 외부에 약속하는 IR"이다.

## 9. 계약 패리티 가드 — 뒤처짐을 CI 가 잡는다

`tests/test_integration.py::test_binding_covers_every_agent_value_command` 가
`rhwp capabilities` 선언과 파이썬 API 를 대조한다.

rhwp 에 `--json` 명령이 늘었는데 바인딩이 따라가지 않으면 **CI 에서 실패**한다.
실제로 이 가드가 `export-doclang` 누락을 개발 중에 잡았다.

새 명령을 추가할 때:

1. `src/rhwp/commands.py` 에 래퍼 함수
2. `src/rhwp/__init__.py` 의 import 와 `__all__`
3. `tests/test_commands.py` 에 인자 조립 테스트
4. `tests/test_integration.py` 의 `exported` 집합

## 10. 테스트 전략

| 종류 | 바이너리 필요 | 무엇을 지키나 |
|---|---|---|
| 단위 | 없음 | 탐색·변환·예외 매핑·계획 직렬화 (순수 로직) |
| 프로세스 | 가짜 스크립트 | 종료 코드별 동작·봉투 계약 위반 감지 |
| 세션 | 가짜 JSON-RPC 서버 | 프로토콜 취급 (id 대조·알림 무시·정리 보장) |
| 통합 | **실물 rhwp** | 계약 재포장 정합·패리티 가드 |

단위 테스트가 바이너리 없이 도는 것이 중요하다 — CI 의 대부분을 Rust 빌드 없이
수 초 만에 돌릴 수 있고, 그래야 바인딩 기여의 문턱이 낮아진다.

### 가짜 픽스처의 인코딩 함정

실물 rhwp(Rust)는 콘솔 코드페이지와 무관하게 **항상 UTF-8** 을 주고받는다.
가짜 픽스처(파이썬 스크립트)는 플랫폼 기본 인코딩을 따르므로, 명시적으로
UTF-8 래퍼를 씌워야 한다. 안 그러면 윈도우에서만 깨져서 "바인딩 버그"로 오인된다.

```python
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", newline="\n")
sys.stdin = io.TextIOWrapper(sys.stdin.buffer, encoding="utf-8")
```

## 11. 자원 정리

세션은 자식 프로세스를 띄운다. **남으면 다음 작업이 파일을 못 연다.**

- `Document`·`Session` 모두 컨텍스트 매니저다. `with` 를 쓰면 예외로 빠져나가도 닫힌다.
- `close()` 는 멱등이다.
- stdin 을 닫아도 안 죽으면 5초 후 강제 종료한다.
- 정리 경로에서는 **새 예외를 만들지 않는다** — 원인 예외를 가리면 진단이 어려워진다.

## 12. 앞으로 (M19·M20)

`bindings_foundation.md` §4 의 착수 조건을 따른다.

- **M19 (Node/TS)**: M18 의 봉투 매핑 규약 재사용 판정 + napi vs WASM 비교표 갱신
- **M20 (C#/Swift)**: 공공 SI 수요 실증 1건

세 계열 모두 같은 원리(얇은 재포장)를 따르면, 언어가 늘어도 계약은 rhwp 본체
한 곳에만 있다. 이것이 배수 확장의 구조적 근거다.

## 관련 문서

- [`bindings_foundation.md`](../tech/bindings_foundation.md) — 설계 결정의 권위
- [`agent_surface_playbook.md`](agent_surface_playbook.md) — 표면 추가 절차
- [`cli_json_pipeline_guide.md`](cli_json_pipeline_guide.md) — 봉투 계약
- [`mcp_integration_guide.md`](mcp_integration_guide.md) — 세션 도구 계약
