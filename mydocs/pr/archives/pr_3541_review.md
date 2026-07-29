# PR #3541 검토 기록 — HWPX 편집 산출 형식 보존

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#3541](https://github.com/edwardkim/rhwp/pull/3541) — edit 계열 HWPX 입력 형식 보존 (#3383) |
| 작성자·검토자 | `@kevin9327` · `@jangster77` |
| source head / 통합 commit | `6be8efb58117e077082dce8c9973eb06201963ff` / `5db6d24dfe1e0226fe8eb3a7015588963d7bf4a1` |
| conflict 처리 | #3478·#3482 이후 적용. `ambiguous`·`overflow`·`outputFormat` 계약을 모두 보존 |

`fill-fields`, `replace-text`, `set-cell`가 HWPX 입력을 무조건 HWP5로 저장하던 경로를 공통 format
판정으로 통합한다. 기본 출력명은 입력 형식을 따르고, 명시 `-o`는 경로를 존중하되 지원 불가 변환은
경고와 실제 형식을 JSON `outputFormat`으로 보고한다. edit 저장은 HWPX→HWP 변환 어댑터도 경유한다.

## 검증과 판정

- `edit_format_preserve_contract`: 7 passed — 세 명령의 HWPX 기본 산출, HWP 무회귀, 양방향 명시
  경로 경고, dry-run 무산출을 확인했다.
- 충돌 해소 시 fill-fields의 occurrence/ambiguous, set-cell의 overflow를 잃지 않고, MCP tool schema의
  `outputFields`에는 `overflow`와 `outputFormat`을 함께 노출하도록 메인터너 보정을 추가했다.
- `cli_json_contract`: 22 passed로 JSON/capabilities/MCP contract를 재확인했다.

HML은 종전 HWP5 산출이라는 범위 제한이 명시돼 있다. 사용자 데이터의 형식 강제 변환을 제거하므로
보정 포함 **기술적 수용 가능**이다.
