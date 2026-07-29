# PR #3482 검토 기록 — 표 셀 넘침 경고

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#3482](https://github.com/edwardkim/rhwp/pull/3482) — `edit set-cell` 넘침 검사·보고 (#3480) |
| 작성자·검토자 | `@kevin9327` · `@jangster77` |
| source head / 통합 commit | `5f7efa90c9c6cf779cee08f145a72e57001438cb` / `ad7c866da32c01fe0875aa5fe3b8aa937cfd821e` |
| 메인터너 보정 | `7854a90f3493…`: MCP `hwp_set_cell.outputFields`에도 `overflow`를 명시하고 계약 test 추가 |

`set-cell`은 편집을 차단하지 않되, 셀 안여백을 뺀 폭과 첫 문단 글자폭 근사로 줄 수·폭·대상을 담은
`overflow` 신호를 JSON에 반환한다. `--dry-run`에도 같은 검사를 실행해 파일 생성 전 제출 불가 산출물을
구분할 수 있다. 주소처럼 여러 줄이 정상인 칸도 있어 경고는 판단 재료이지 강제 실패가 아니다.

## 검증과 판정

- `edit_fit_check_contract`: 4 passed — 초과만 보고, 경계값 무경고, dry-run 무산출, 비차단 편집을 확인했다.
- `cli_json_contract`: 22 passed. contributor는 일반 capabilities에 `overflow`를 등록했지만 MCP 자동
  등록 표의 outputFields에는 빠져 있어, 메인터너 보정으로 두 인터페이스를 일치시켰다.
- #3541과의 충돌 뒤에도 `overflow`와 `outputFormat`이 함께 노출된다.

근사 검사는 정밀 조판 판정이 아니라 조기 위험 신호라는 한계가 문서화돼 있다. 보정 포함 **기술적 수용 가능**이다.
