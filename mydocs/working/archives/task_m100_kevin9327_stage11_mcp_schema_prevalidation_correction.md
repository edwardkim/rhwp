# Stage 11 - MCP 입력 schema 사전 검증 보정

## 발견

13개 개별 PR review에서 두 MCP schema가 CLI보다 약한 입력을 허용했다.

- `hwp_add_bookmark`는 빈 문자열·공백 문자열 name을 schema에서 허용하지만 CLI는 `trim().is_empty()`로 거부했다.
- `hwp_insert_header_footer`는 `path`만 요구하지만 CLI는 `--header` 또는 `--footer` 중 정확히 하나를 요구했다.

이 불일치는 MCP 클라이언트가 schema상 유효한 요청을 만들고도 실행 시 usage error를 받게 한다.

## 보정

책갈피 이름 schema에는 비공백 문자를 요구하는 `.*\\S.*` pattern을 추가했다. 머리말·꼬리말 schema에는
각 Boolean 중 하나만 `true`인 두 `oneOf` branch를 추가해 누락·동시 선택을 사전에 거부한다.

## 계약

두 원본 integration contract가 capabilities MCP JSON에서 새 pattern과 `oneOf` 필수·상수 조건을 확인한다.
CLI 파서의 기존 입력 검증은 변경하지 않았다.

## 검증 계획

두 contract를 generated suite 준비 상태에서 실행하고, manifest 계약 검증으로 파생 산출물이 PR diff에
포함되지 않았음을 함께 확인한다.
