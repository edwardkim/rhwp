# 20 — 종료 코드 (#2707)

편집·조회 공통 계약이다. 이 스킬이 코드를 재정의하지 않는다.

| 코드 | 뜻 | 이 스킬에서 |
| --- | --- | --- |
| 0 | 성공 | fieldCount 0, filledCount 0, dry-run 보고도 0 |
| 1 | 런타임 | 파일 없음, UTF-8 아님, 쓰기 실패. stdout 비움. 원본 불변, 출력 미생성 |
| 2 | 사용법 | 인자 없음, 깨진 JSON, 빈 데이터 파일, 쪽 범위, 그림 형식 |
| 3 | 검증 실패 | `--verify` IR 차이. 산출물은 남음 |
| 4 | 쪽 수 불일치 | convert/export-hwpx 전용. fill 축에 없음 |

## 0 인데 미완료

ambiguous, notFound(잔류), textSecurity, overflow 는 코드 0 이다.
게이트는 봉투를 읽는다.

## 1 과 2

1 은 "문서를 열거나 쓰지 못했다". 2 는 "호출이 틀렸다".
2 를 재시도하지 않는다. 인자를 고친다.

없는 파일 fill: exit 1, stdout 빈 (`edit_fill_fields_contract`).
깨진 `--data`: exit 2.

## 3

`--verify` 가 있을 때만. `identical: false` 와 코드가 모순되면
계약 위반이다 (`edit_verify_contract`). 에이전트는 코드를 고치지 않고
보고한다.

batch 는 한 행 실패 시 최종 1, verify 차이는 요약에 반영. 행별 레코드의
`verify` / `error` 를 본다.

## 파이프라인

```bash
set -e
rhwp edit fill-fields … --dry-run --json > dry.json
jq -e '(.notFound|length==0) and (.ambiguous|length==0)' dry.json
rhwp edit fill-fields … --verify --json > run.json
# exit 3 이면 set -e 가 멈춘다. 산출 경로는 run 전 -o 로 이미 안다.
```

PowerShell 은 `$LASTEXITCODE` 를 본다. `jq -e` 실패(1) 와 rhwp 3 을
구분한다.
