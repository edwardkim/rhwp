---
kind: report
status: active
canonical: mydocs/report/task_m100_3884_g4/README.md
last_verified: 2026-08-06
---

# 처리 결과 — edit·inspect 하위 명령 자기서술 등재 (#3884 G4, 로드맵 R7)

## 분석

`capabilities` 의 commands[] 에는 `edit`·`inspect` 부모만 있고, 실제 하위 9종
(edit 6·inspect 3)은 summary 산문에만 있었다. 결과: `capabilities --search redact`
가 아무것도 못 찾는다 — R31(키워드 발견)이 하위 명령 위에서 절반만 동작한다.
R6 전수 대조에서 나온 28건 중 15건이 같은 뿌리(하위 명령 사각)였다는 것이 정량
근거다. 실패 자체는 규약을 지키므로(exit 2 + stdout 0 B) 깨짐이 아니라 **발견
가능성의 공백**이다.

## 설계 판단

- **1차 범위(이번 PR)**: `subcommands: [{name, summary}]` 배열 + 검색 편입.
  `batch.subcommands`(문자열 배열) 선례를 commands[] 항목으로 옮기되, 검색이
  요약까지 매치해야 하므로 이름+요약 객체로 한 단계 확장했다.
- **하위별 recordFields 분화(2차)는 하지 않는다** — R7 문서가 별도 판단으로
  분리해 둔 지점. 현재 부모 합집합 선언은 R6 가드가 그 기준으로 잡고 있어
  동작에 문제가 없다.
- **삽입 지점을 vec 밖으로** — 항목 정의 자리(cmd_json 호출)는 거의 모든 표면
  PR 이 지나는 자리라, 후처리 `attach_subcommands()` 로 빼서 병렬 PR 충돌면을
  줄였다(무충돌 방법론의 삽입 지점 분산).

## 변경

- `src/main.rs`
  - `EDIT_SUBCOMMANDS`(6)·`INSPECT_SUBCOMMANDS`(3) 상수 + `attach_subcommands()`.
  - `capabilities_command_entries()` 끝에서 후처리 부착.
  - `show_capabilities_search()` 의 haystack 에 하위 명령 이름·요약 편입.
- `tests/capabilities_subcommands_contract.rs` 신설 — 계약 4본:
  1. 선언 ↔ **USAGE 실물** 대조(디스패치 코드 옆의 `<a|b|c>` 목록이 오라클).
  2. 선언된 하위 전수가 실제로 디스패치된다(exit 2 usage, "알 수 없는" 부재).
  3. 미선언 하위는 거부된다("알 수 없는" + exit 2).
  4. R7 DoD: `--search redact` → edit, `--search hidden-text` → inspect.

## 실측

```
$ rhwp capabilities --search redact
'redact' 검색 결과 (1건):
  edit                     문서 편집 — fill-fields: 누름틀 채우기 / … / redact: 개인정보 마스킹 / sanitize: 메타데이터 제거
```

계약 테스트: `cargo test --release --test capabilities_subcommands_contract`
→ **4 passed; 0 failed** (1.64s).

증적 이미지(재현 명령 포함):

- `search_subcommands.png` — `--search redact`·`hidden-text`·`누름틀` 3검색이
  부모를 찾는 터미널 화면.
- `subcommands_envelope.png` — `--search redact --json` 봉투의 edit 항목
  `subcommands` 발췌.

재현: `target/release/rhwp capabilities --search <키워드> [--json]`.

## 무회귀 근거

`subcommands` 는 **필드 추가**다 — `cli_json_contract` 머리말의 계약("필드 추가는
허용, 변경·삭제는 실패로 잡는다") 안에서 움직였고, 기존 항목·필드는 건드리지
않았다. 실렌더 증적은 해당 없음(문서를 여는 명령이 아니다).
