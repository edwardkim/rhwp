# PR #4921 검토 - 문서 의미 diff 라이브러리

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#4921](https://github.com/edwardkim/rhwp/pull/4921) |
| 작성자 | `kevin9327` (`kevin`) |
| 검토 방식 | #4931 누적 통합을 위한 archive review |
| base / head | `devel` / `feat/docdiff-engine` |
| source candidate | `ae61bc401618ef8195f05119711211a54540edc9` |
| 통합 commit | `a6cb98e1bc5e8043651925c0cce36bec1864037d` |
| 규모 | 6 files, +1,736 / -0 |
| 작성 시점 상태 | `OPEN`, `MERGEABLE`, `CLEAN` |

## 범위와 판단

- 문단 앞 삽입을 대량 변경으로 오인하지 않도록 LCS 기반 정렬과 typed node path를 제공하는 `docdiff` 모듈을
  추가한다.
- 기존 `roundtrip::diff_documents`와 CLI `ir-diff`를 즉시 대체하지 않고 라이브러리 계약을 먼저 신설한다.
- 결정적 순회, finding 상한의 정직한 `truncated`, 추가·삭제 덩어리의 수정 승격은 자동화 소비자에 필요한
  안정적인 결과 계약이다.

## 검증

- source candidate의 Build & Test와 기본 feature 세 shard·slow shard는 성공했다.
- CodeQL 분석 job은 성공했고 roll-up은 `NEUTRAL`이며, Native Skia와 frontend/WASM job은 변경 영향에 따라
  skipped였다.
- #4931 누적 tree의 전체 `release-test` integration 회귀는 종료 코드 `0`으로 통과했다.

## 위험과 권고

기존 CLI를 새 라이브러리로 이관할 때 출력 호환과 max-finding 경계의 별도 contract test가 필요하다. 현재
범위에서는 독립 모듈 추가로 제한되어 있어 #4931 통합 merge를 권고하며, 원 PR은 merge 뒤 supersede 처리한다.
