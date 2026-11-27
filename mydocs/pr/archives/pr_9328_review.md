# PR #9328 자기 리뷰 — 본문과 표 셀의 메모 위치로 이동

## 최종 판정

**승인.** 메모 스택 #9332에서 이 PR의 동작과 정확한 코드 head CI를 확인했다. 병합 전에는 이 기록이 들어간 최신 head의 필수 검사, 스택의 선형 이력과 작업지시자의 승인을 확인한다.

## 접수 정보

| 항목 | 확인한 값 |
| --- | --- |
| PR과 이슈 | [#9328](https://github.com/edwardkim/rhwp/pull/9328) · [#9260](https://github.com/edwardkim/rhwp/issues/9260) |
| 작성자 및 검토 경로 | `postmelee` collaborator 자기 리뷰 |
| 검토한 코드 head | `ff1a6e7f0e845472c601d77e583f9af77e57b399` |
| 직접 base | `3e97980785dcee996646ed399f46e65fbaaee261` |

## 변경과 검토 범위

메모 목록의 쪽 번호와 이동 명령이 같은 좌표를 사용한다. 기존 범위는 선택하고 지점 메모는 캐럿을 배치한다. 이동·읽기는 저장 상태와 undo 기록을 바꾸지 않는다.

## 검증 입력과 결과

- 본문, 표와 빈 셀, 이어지는 표 및 중첩 표의 기준 위치 이동, HWP·HWPX 2회 저장·재열기와 메모 범위 보존을 확인했다.
- 브라우저 E2E는 재현 코드가 본문과 표를 만들며, HWP·HWPX 저장과 재열기를 실제 Studio에서 실행했다. 문서 조판이나 기본 렌더 출력 변경은 비해당이다.
- 최종 누적 Studio 코드에서 TypeScript와 단위 테스트 2,838개가 통과했다(2개 건너뜀). 스택을 정렬하며 바뀐 문서의 변경 범위, 최신 devel과의 병합 시뮬레이션 및 문서 링크를 검사했다.
- 해당 코드 head의 `Build & Test`와 관련 GitHub Actions가 성공했다. [CI](https://github.com/edwardkim/rhwp/actions/runs/45985550160) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/45985550386) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/45985550480) · [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/45985550210) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/45985550417)
