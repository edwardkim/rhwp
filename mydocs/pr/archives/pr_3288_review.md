# PR #3288 검토 기록

| 항목 | 내용 |
|---|---|
| 원 PR | [#3288](https://github.com/edwardkim/rhwp/pull/3288) |
| 작성자 / base | `kevin9327` / `devel` |
| 검토자 | `@jangster77` (GitHub review request 확인) |
| 원 head | `e26c9564191ba8e6bcbabc395ad8c2874da241e2` (2026-07-25 조회 참고값) |
| 규모 | +240/-3, 4 files, 4 commits |
| 관련 이슈 | #3287 (제목·처리 보고서 기준; GitHub closing reference는 없음) |
| 통합 보정 | `0ee48afa5` — 실패 JSON stdout 0-byte 계약과 help 노출 |
| 판단 | v2 통합 PR 수용 후보 |

## 범위와 검토

- `export-svg --json`은 render 결과의 page/path/byte 매니페스트를 내보낸다. 원 feature `f7406e16a`를 누적 적용했다.
- 쓰기 실패를 유발하면 원 구현은 exit 1인데도 stdout에 부분 JSON을 내보냈고 help에도 `--json`이 나타나지 않았다.
  성공 응답으로 오인될 수 있는 partial JSON은 계약 위반이다.

## 보정과 검증

- v2의 `0ee48afa5`는 JSON mode에서 렌더/쓰기 실패면 stdout을 비워 두고 stderr와 non-zero exit만 남기며, help에
  `--json`을 노출한다. 일반 사람용 실패 요약은 유지한다.
- 쓰기 불가 output을 통한 direct CLI 재현에서 보정 후 exit 1/stdout 0 bytes를 확인했고,
  `render_manifest_json_contract` 4 passed와 누적 full release-test 전체 성공을 확인했다.
- renderer 결과물의 형식이 아니라 CLI 응답 계약만 변경하며 renderer/layout/sample/golden은 바뀌지 않아 visual
  sweep과 baseline 등록은 대상이 아니다.

## 권고와 다음 조건

- **권고: v2 통합 PR로 수용.** source branch에는 추가 push하지 않는다. full CI 성공, 통합 PR의 최신
  mergeability, 사용자 PR·merge 승인이 필요하다.
