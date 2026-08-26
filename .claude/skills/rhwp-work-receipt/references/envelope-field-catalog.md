# 봉투 필드 카탈로그

지식 지도 §작업 영수증·노동 감사·작업 계보를 스킬 소비 관점으로 재배치한다.
타입은 `--json` 봉투 기준이다. `null` 의 뜻을 빠뜨리면 판정을 오독한다.

## 1. `replay` 영수증

| 필드 | 타입 | null 의 뜻 | 비고 |
|------|------|------------|------|
| `schemaVersion` | string | 없음 | `"1.0"` |
| `mode` | string | 없음 | `attest` 또는 `verify` |
| `input` | string | 없음 | 계획서 input 에코 |
| `inputSha256` | string | 없음 | 입력 파일 바이트 SHA-256 |
| `planSha256` | string | 없음 | 계획 **원문** 바이트 SHA-256 |
| `outputSha256` | string | 없음 | 임시 재실행 산출 바이트 SHA-256 |
| `expectedOutputSha256` | string\|null | attest | verify 에서 호출자 주장 |
| `reproduced` | bool\|null | attest (검증 안 함) | verify 에서 false 면 exit 3 |
| `toolVersion` | string | 없음 | 재현 조건. 선대조 |
| `steps` | number | 없음 | 실행된 step **수**. 배열 아님 |
| `untrustedContent` | bool | 없음 | 엔진 값. 보통 false |
| `untrustedFields` | array | 없음 | 보통 `[]` |

동명 함정: `run` 저널의 `steps` 는 배열, `replay` 는 숫자.
`run` 의 `outputSha256` 은 디스크에 남은 파일, `replay` 는 임시 재실행.

## 2. `workCapsule` 파일

| 필드 | 타입 | null 의 뜻 | 비고 |
|------|------|------------|------|
| `kind` | string | 없음 | 반드시 `workCapsule` |
| `parent` | object\|null | 뿌리 | `{capsule, sha256}` |
| `parent.capsule` | string | — | **캡슐 파일 기준** 상대 경로 |
| `parent.sha256` | string | — | 부모 **파일 바이트** SHA-256 |
| `plan` | object | 없음 | `planText` 파싱 결과와 같아야 함 |
| `planText` | string | 없음 | `receipt.planSha256` 의 대상 |
| `receipt` | object | 없음 | §1 영수증 |

`parent` 키 자체가 없으면 뿌리가 아니라 **깨진 캡슐**이다.

## 3. `audit` 회계

| 필드 | 타입 | null 의 뜻 | 비고 |
|------|------|------------|------|
| `root` | string | 없음 | 폴더 에코 |
| `total` | number | 없음 | 직속 `*.capsule.json` 수. 0 이면 봉투 없음 |
| `reproduced` | number | 없음 | 성공 건수. bool 아님 |
| `failed` | array | 없음 | 비면 전건 성공 |
| `failed[].capsule` | string | 없음 | 파일명 |
| `failed[].kind` | string | 사유형 | `inputSha256` / `steps` / (산출) |
| `failed[].expected` | string\|number | 사유형 | 해시 또는 step 수 |
| `failed[].actual` | string\|number | 사유형 | 실측 |
| `failed[].error` | string | 해시형 | 읽기·파싱·plan 불변식 |
| `reproducedRate` | number | 없음 | `reproduced / total` |

## 4. `lineage` 연대기

| 필드 | 타입 | null 의 뜻 | 비고 |
|------|------|------------|------|
| `head` | string | 없음 | 머리 경로 에코 |
| `depth` | number | 없음 | 걸은 링크 수 |
| `valid` | bool | 없음 | false 면 exit 3 |
| `brokenAt` | string\|null | 체인 유효 | 처음 깨진 캡슐 |
| `links` | array | 없음 | 머리 → 뿌리 |
| `links[].capsule` | string | 없음 | 경로 |
| `links[].inputSha256` | string | 오류 링크 | |
| `links[].outputSha256` | string | 오류 링크 | |
| `links[].parentOk` | bool\|null | 머리 | 부모 파일 무결 |
| `links[].lineageOk` | bool\|null | 머리 | 부모 산출 == 자식 입력 |
| `links[].reproduced` | bool\|null | `--deep` 없음 | 링크 재실행 |
| `links[].error` | string | 정상 링크 | 파싱·필드 결함 |
| `links[].signerOk` | bool\|null | `--keyring` 없음 | 1부 기본 경로에 없음 |
| `links[].anchoredOk` | bool | `--anchor-log` 없음 | 1부 기본 경로에 없음 |

## 5. 종료 코드와 봉투 존재

| exit | 봉투 | 읽는 키 |
|-----:|------|---------|
| 0 | 있음 | 단별 성공 키 |
| 1 | 없음 | stderr |
| 2 | 없음 | stderr |
| 3 | 있음 | `reproduced` / `failed` / `valid` / `brokenAt` |

## 6. 픽스처 교차

각 키의 표본 파일은 `fixtures/envelopes/` 와 `fixtures/catalog.json` 이
목록이다. 카탈로그에 없는 봉투 JSON 을 추가하면 시험이 실패한다.
