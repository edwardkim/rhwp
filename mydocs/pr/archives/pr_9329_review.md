# PR #9329 자기 리뷰 — 여백 메모 카드와 안내선

## 최종 판정

**승인.** 메모 스택 #9332에서 이 PR의 동작과 정확한 코드 head CI를 확인했다. 병합 전에는 이 기록이 들어간 최신 head의 필수 검사, 스택의 선형 이력과 작업지시자의 승인을 확인한다.

## 접수 정보

| 항목 | 확인한 값 |
| --- | --- |
| PR과 이슈 | [#9329](https://github.com/edwardkim/rhwp/pull/9329) · [#9261](https://github.com/edwardkim/rhwp/issues/9261) |
| 작성자 및 검토 경로 | `postmelee` collaborator 자기 리뷰 |
| 검토한 코드 head | `c49c4a97fb3820e20cec107fe3d5dcaa3a5a4261` |
| 직접 base | `0234e44f3b00bdf8fdff645cb52d0d0f827bf109` |

## 변경과 검토 범위

메모를 본문 옆 별도 열에 읽을 수 있고 각 카드에서 대응하는 본문 위치를 찾을 수 있다. 표시 설정은 Studio 상태에 두고 저장 내용과 분리한다.

## 검증 입력과 결과

- 여백 카드 배치와 연결선, 확대 후 카드 간 겹침 방지, 여러 쪽 표·중첩 셀의 위치 이동을 확인했다. 보기 전환 전후 HWPX 저장 값과 dirty 상태를 비교했다.
- 브라우저 E2E는 재현 코드가 본문과 표를 만들며, HWP·HWPX 저장과 재열기를 실제 Studio에서 실행했다. 문서 조판이나 기본 렌더 출력 변경은 비해당이다.
- 최종 누적 Studio 코드에서 TypeScript와 단위 테스트 2,838개가 통과했다(2개 건너뜀). 스택을 정렬하며 바뀐 문서의 변경 범위, 최신 devel과의 병합 시뮬레이션 및 문서 링크를 검사했다.
- 해당 코드 head의 `Build & Test`와 관련 GitHub Actions가 성공했다. [CI](https://github.com/edwardkim/rhwp/actions/runs/46016240559) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/46016240827) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/46016240660) · [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/46016240672) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/46016240644)
