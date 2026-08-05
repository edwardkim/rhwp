# schema-validator — rhwp ingest JSON 선검사기 (#4044 후속)

`rhwp build-from-ingest` 에 넘기기 전에 ingest JSON 을
[`tools/rhwp-ingest/schema/ingest_schema_v1.json`](../rhwp-ingest/schema/ingest_schema_v1.json)
(JSON Schema draft-07 부분집합) 기준으로 검증한다. **표준 라이브러리만** 사용하는
단일 파일 스크립트이며, 스키마 사본을 두지 않고 저장소 canonical 스키마를 직접
읽는다(사본 드리프트 방지).

#4044 리뷰에서 지적된 두 결함을 근본 수정한 재제출본이다:

1. **중첩 검증 실패의 반환값 전파** — 이전 구현은 `properties`/`items` 하위
   검증 결과를 버려서, `oneOf` 대안 판정이 전부 참으로 계산됐다. 지금은
   `_validate()` 반환 bool 이 "값과 그 모든 하위 값이 스키마에 부합"을 뜻하며
   모든 하위 실패가 상위로 전파된다.
2. **`oneOf` 의미론** — draft-07 그대로 **정확히 한 대안** 일치만 통과다.
   0개 일치(`ONEOF_FAILED`)와 2개 이상 일치(`ONEOF_AMBIGUOUS`)는 모두 ERROR.
   저장소 샘플(`sample_minimal.json`, `sample_structured.json`)은 오류 0건·경고
   0건으로 통과한다.

## 사용법

```bash
# 사람용 출력 (스키마는 기본 경로 자동 사용)
python tools/schema-validator/schema_validator.py my_ingest.json

# 기계 판독 JSON
python tools/schema-validator/schema_validator.py my_ingest.json --json

# 종료 코드만
python tools/schema-validator/schema_validator.py my_ingest.json --quiet

# 다른 스키마로 검증
python tools/schema-validator/schema_validator.py my.json --schema path/to/schema.json
```

### 종료 코드

| 코드 | 의미 |
|------|------|
| 0 | 유효 (ERROR 0건 — WARNING 은 무관) |
| 1 | 스키마 위반 (ERROR 1건 이상) 또는 JSON 구문 오류 |
| 2 | 사용법·환경 오류 (스키마 파일 없음/손상, 지원하지 않는 `$ref` 등) |

### `--json` 출력

```json
{
  "valid": true,
  "error_count": 0,
  "warning_count": 0,
  "errors": []
}
```

`valid` 는 종료 코드와 동일 기준(`error_count == 0`)이다. `errors[]` 각 항목은
`level`(ERROR/WARNING)·`path`(예: `questions[0].choices[1]`)·`message`·`code`·
`position` 을 갖는다. `position` 은 JSON 구문 오류에만 `Line L, Column C` 로
채워지고, 구조 오류는 JSON 경로(`path`)로 위치를 특정한다.

## 판정 규약: ERROR 와 WARNING

- **ERROR** — 스키마 위반. `valid=false`, 종료 코드 1.
- **WARNING** — 스키마상 허용이지만 주의 신호. 현재 유일한 경고는
  `UNKNOWN_FIELD`: 스키마 `properties` 에 정의되지 않은 필드다. JSON Schema
  기본값(`additionalProperties: true`)으로는 허용이지만, Rust 측
  `build-from-ingest` 는 `deny_unknown_fields` 라서 이런 필드(대개 오탈자)를
  거부한다. 오탈자를 빌드 전에 잡으라고 경고로 신고하되 `valid` 판정은
  스키마 의미론을 따른다.

역으로 이 검증기는 Rust 파서가 잡지 않는 것도 잡는다 — 예: `version` 의
`const "1"` 위반(Rust 쪽은 임의 문자열 허용).

## 오류 코드

| 코드 | 의미 |
|------|------|
| `INVALID_JSON` | JSON 구문 오류 (줄/칸 위치 포함) |
| `FILE_READ_ERROR` | 파일 읽기 실패 |
| `TYPE_MISMATCH` | 타입 불일치 |
| `CONST_MISMATCH` | `const` 불일치 (예: `version` ≠ `"1"`) |
| `ENUM_MISMATCH` | `enum` 밖의 값 (예: `placement`) |
| `MISSING_REQUIRED_FIELD` | 필수 필드 누락 |
| `UNKNOWN_FIELD` | 미정의 필드 (WARNING; `additionalProperties: false` 명시 시 ERROR) |
| `ONEOF_FAILED` | `oneOf` 어느 대안과도 불일치 (대안별 실패 사유 힌트 포함) |
| `ONEOF_AMBIGUOUS` | `oneOf` 2개 이상 대안과 일치 |
| `BELOW_MINIMUM` / `ABOVE_MAXIMUM` | 수치 범위 위반 |
| `STRING_TOO_SHORT` / `STRING_TOO_LONG` | 문자열 길이 위반 |

지원 키워드: `type`, `const`, `enum`, `required`, `properties`,
`additionalProperties`, `items`(단일 스키마), `oneOf`, `minimum`, `maximum`,
`minLength`, `maxLength`, `$ref`(`#/definitions/*` 한정). 그 외(`default`,
`description` 등)는 주석으로 간주해 무시한다.

## 파이프라인 통합 예

```bash
python tools/schema-validator/schema_validator.py exam.ingest.json || exit 1
rhwp build-from-ingest exam.ingest.json -o exam.hwpx --json
```

## 테스트

```bash
python tools/schema-validator/test_schema_validator.py
# 또는
python -m unittest discover -s tools/schema-validator -p "test_*.py"
```

회귀 고정 내용:

- 저장소 실물 샘플 2종이 **오류 0건·경고 0건** 통과 (리뷰 3번 허위 경고 회귀)
- 고의로 깨뜨린 사본(const/required/type/minimum 위반)이 `valid:false` + CLI 비 0 종료
- `oneOf` 정확성: 1개 일치 = 통과, 0개/2개 일치 = 실패
- 4단계 중첩(`questions[i].choices[j]`) 오류의 최상위 bool 전파
- JSON 구문 오류의 줄/칸 보고, 스키마 부재 시 종료 코드 2
