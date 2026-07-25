# PR #3280 검토 기록

| 항목 | 내용 |
|---|---|
| 원 PR | [#3280](https://github.com/edwardkim/rhwp/pull/3280) |
| 작성자 / base | `kevin9327` / `devel` |
| 검토자 | `@jangster77` (GitHub review request 확인) |
| 원 head | `11a2edddb764c6dd101be5cf1c0450ff79d06b66` (2026-07-25 조회 참고값) |
| 규모 | +629/-0, 6 files, 3 commits |
| 관련 이슈 | #3278 (제목·처리 보고서 기준; GitHub closing reference는 없음) |
| 통합 보정 | `42fe6b6b7` 다중 입력, `90c8e5a7c` table control·container path |
| 판단 | v2 통합 PR 수용 후보 |

## 범위와 검토

- `export-tables --json`은 표 격자와 중첩·글상자·머리말 등의 컨테이너 내 표를 조회한다. 원 feature
  `d764760ca`를 누적 적용했다.
- 원 출력의 `section`/`paragraph`만으로는 동일 컨테이너의 표, 중첩 표, 글상자/머리말의 표를 유일하게 다시
  찾을 수 없어 PR이 주장한 역참조 계약이 성립하지 않았다. 다중 입력도 마지막 입력을 묵시적으로 사용했다.

## 보정과 검증

- `42fe6b6b7`는 다중 입력을 `EXIT_USAGE`로 바꾼 v2 보정이다. 추가 보정 `90c8e5a7c`는 table-level control과
  `containerPath`(컨테이너 종류, control, 문단, 필요 시 cell)를 JSON에 **additive**로 기록한다.
- `table_extract_json_contract` 9 passed, 누적 full release-test 전체 성공을 확인했다. `treatise sample.hwp`의
  머리말 표 두 개와 본문 표는 서로 다른 `control`/`containerPath`로 출력됨을 직접 확인했다.
- renderer·layout·fixture·golden 변경은 없으므로 visual sweep과 baseline 등록은 불필요하다.

## 권고와 다음 조건

- **권고: v2 통합 PR로 수용.** 두 보정은 source head가 아니라 v2에만 포함한다. code 보정이 있으므로 full CI를
  기다린다.
- 원 PR의 `MERGEABLE`/`BEHIND` 및 CI는 참고값이다. merge는 최신 통합 head의 CI와 사용자 승인 뒤에만 한다.
