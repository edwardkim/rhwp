# R42 리베이스·충돌 처리 및 변경 보존 보고

## 결과

`review/integration-20261106-planet6897-open`을 최신 확인 base
`1e16a19aede5ccb2657d35974940eb596286cdf1`로 리베이스했다. **코드 충돌 0건**이며, 양쪽을 합친 사전
`git merge-tree --write-tree` 결과와 리베이스 직후 tree가 정확히 같다.
이는 충돌 없는 적용과 내용 보존 확인이며, 새 head의 실제 기능 검증 완료를 뜻하지 않는다.

| 구분 | 고정값 |
| --- | --- |
| 리베이스 전 head | `3fcc14d95f865cf4e7237dcf79e26579fea858da` |
| 이전 base | `e4ffabea38062e716f6c333cc950fa31b42b4dd7` |
| 새 base | `1e16a19aede5ccb2657d35974940eb596286cdf1` |
| 리베이스 직후 head | `896708a4a02c4138064ab3d426a487bbce52fc02` |
| 사전 병합 tree = 실제 결과 tree | `1f4bd4917ff8bd61d9c0af6b9d25f2609142cb29` |
| 시작 / 완료 | `2026-11-07T13:01:31.754797+00:00` / `2026-11-07T13:04:28.703459+00:00` |

## 충돌 처리 경과

사전 병합 시뮬레이션은 종료 코드 0이다. 실제 rebase는 중간에 `.git/index.lock`이
존재해 1회 정지했고 해당 pick이 todo에 재등록됐다. 재개 전 실제 lock 파일이 이미
사라진 것을 확인한 뒤 `git rebase --continue`로 끝냈다. lock 파일을 삭제하거나 커밋을
skip·amend하지 않았다. 코드 충돌 해결을 위해 한쪽 구현을 버린 파일은 없다.
원래 136개 replay 항목이었으며, 실패한 pick을 재등록한 재개 로그의 137 표기를
추가 체리픽이나 중복 적용으로 해석하지 않는다.

## 변경 보존 확인

- 고유 원 기여 commit **56개**의 cherry-pick footer와 순서가 리베이스 전후 일치한다.
- 이번 통합에서 사용하는 원본·독립 PDF **141개 경로의 Git blob이 모두 같다**.
- 리베이스 전후 달라진 **163개 경로는 새 base가 추가·수정한 경로 집합과 정확히 같다**.
- 전체 결과 tree까지 사전 병합 tree와 같아, 리베이스가 기존 통합의 별도 보정을 누락하거나
  새 base의 변경을 덮어쓰지 않았음을 파일 내용 수준에서 확인했다.

## 새 base에서 반영한 기능과 후속 검증

[PR #8738](https://github.com/edwardkim/rhwp/pull/8738)의 표·셀 단색 배경 저장 보존과
Windows Vite fixture 경로 보정이 반영됐다. Rust 제품 변경은 `html_table_import.rs`에 있는
공통 `create_border_fill_from_json`이다. 파일 이름과 달리 HTML 가져오기 전용이 아니며,
표·셀·문단·머리말/꼬리말·각주의 속성 편집 경로가 공유한다. 기존 채움의 색상·무늬를 부분
편집에서 보존하고 새 단색의 무늬 없음은 `-1`로 만든다. Studio의 두 배경 설정 대화상자도
같은 sentinel을 사용한다. 새 저장 계약 Rust/Studio 검사와 기존 `issue_7606` 대조 검사가 함께 들어왔다.
해당 PR의 원본·독립 Print 7개 조합 및 기존 증적을 먼저 재사용하고, 새 base와 후보의 실제
저장·다시 열기·출력을 대조한다. 기존 통합의 개선과 [PR #8389](https://github.com/edwardkim/rhwp/pull/8389)·
[PR #8397](https://github.com/edwardkim/rhwp/pull/8397)의 표시값 무하락 조건도 유지한다.

실행 중이던 R41 Rust 검사는 별도 worktree의 고정 source
`851e4074514935e4c6a7377f71a97f0cca454191`에 대한 실행이다. 이 로그나 과거 Native/WASM
결과를 R42의 새 head를 실행한 결과로 바꿔 적지 않는다. 영향 분석·필수 검증·개별 최종
리뷰를 완료한 뒤 PR을 제출한다.

새 대화상자의 실제 메서드·속성 전달과 셀 편집 대상 경계는
`node --test tests/issue-8733-solid-fill-pattern.test.ts tests/cell-target-issue8102.test.ts`로
검사해 **21건 통과, 실패·skip 0건**을 확인했다. 실제 WASM 저장과 전쪽 시각 검증의 대체
증거로 사용하지 않으며, 실행 로그는
`output/pr-review/planet6897-integration-20261106/logs/r42-studio-new-base-contracts.log`에 있다.

## 실제 로그

로그는 ignored `output/`에 두고 커밋하지 않는다. 검토 종료 후 정리 전까지 다음 경로에서 확인한다.

- 최초 rebase: `output/pr-review/planet6897-integration-20261106/logs/r42-rebase.log`
- 재개·완료: `output/pr-review/planet6897-integration-20261106/logs/r42-rebase-continue.log`
- 사전 병합: `output/pr-review/planet6897-integration-20261106/logs/r42-rebase-merge-tree.log`
- 구조화 기록: 같은 디렉터리 상위의 `r42-rebase-record.json`·`r42-rebase-preservation.json`
