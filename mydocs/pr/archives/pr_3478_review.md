# PR #3478 검토 기록 — 반복 누름틀 순번 지정과 모호성 보고

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#3478](https://github.com/edwardkim/rhwp/pull/3478) — `fill-fields` 반복 항목 순번 지목·모호성 보고 (#3476) |
| 작성자·검토자 | `@kevin9327` · `@jangster77` |
| source head / 통합 commit | `7676ddd77d5f49596b05847e6c087b68d3a544c9` / `a09fc5971c0004e06475f333a08be5f9891bb4ae` |
| 적용 방식 | source의 devel merge commit은 제외하고 고유 기능 commit `098ae52437dc…`만 `-x` 체리픽 |

같은 이름의 누름틀이 여러 개인 경우 `name[N]`(0-based)로 특정 occurrence를 지정하고, 순번 없이 첫
항목만 채운 경우에는 `ambiguous`에 matched/total을 돌려준다. 숫자가 아닌 대괄호 표기는 기존 필드명의
일부로 남고, 범위 밖 번호는 `notFound`가 된다. 이는 조용히 미완성 서식을 성공으로 오인하는 문제를
막는 CLI 계약 변경이다.

## 검증과 판정

- `edit_field_occurrence_contract`: 4 passed — N번째 지정, 미지정 항목 보존, 모호성 수량, 범위 밖 및
  기존 첫 매치 행동을 고정했다.
- `cli_json_contract`: 22 passed — MCP/capabilities 계약 drift를 검증했다.
- #3541과 같은 `main.rs` 충돌은 `ambiguous`와 `outputFormat`을 함께 노출하도록 해소했다. 원 PR의
  occurrence·모호성 의미를 잃지 않았다.
- #3550 CI의 #2724 guard는 compatibility wrapper `set_field_value_by_name`의 위임 사실을 ledger에
  등록하라고 요구했다. `d0b42ae18`은 실제 무효화를 수행하는 `set_field_value_by_name_at`을
  검증되는 DelegatesTo 대상으로 명시하며, 중복 raw-stream 무효화를 넣지 않는다.

중복 필드 이름의 실제 제출 위험을 JSON 응답에 드러내며 기존 단일 매치 API를 보존한다. **기술적 수용 가능**이다.
