# PR #8725 검토 — Kang-T 표 칸 그림 편집 3건 수용

## 최종 판정

**승인.** 정확한 head `07510f602a6081d90c94397fe29613420f3e7f63`의 필수 CI 39개 check/status를 확인한 뒤 merge `331521a3b1fb5de5dfa39b0707c0b295b90f9dc4`로 수용했다. 원 PR 3건은 자동 MERGED이며 source SHA를 보존했다.

작성자 self-review를 완료했습니다. 검토 head는 `07510f602a6081d90c94397fe29613420f3e7f63`, 최신 기준은 `a7feb849e11642b1438b7e9ee460ff4c8a7c946e`입니다. **최종 판정: 승인.** 원 기여 SHA 3개를 보존하며, 동일 head의 39개 check/status와 CLEAN·merge-tree를 재확인했습니다.

- 공통 계약: 편집 재조판의 정렬 기준선, 칸 문단 삽입·실패 원자성·기본 오프셋, 제거 뒤 재번호된 실제 소유 경로, 실제 포인터가 잡은 점과 도착 원점·Undo snapshot의 연결을 대조했습니다. 문서별 좌표·높이 우회는 없습니다.
- 로컬: 최소 글꼴 전체 12,778건·Native 공식 4,064건·집중 34건·Studio 2,745건과 필수 fmt/Clippy/build/suite·타입·관련 E2E를 통과했습니다.
- 시각·실사용: 원 13쪽 Native/fresh optimized WASM 최저 99.13472%/99.09683%, 독립 Print·변경 영역 직접 판독을 완료했습니다. 최신 devel 정상 대조군 20쌍과 새 글꼴 사례 6쌍 PNG 동일, Native 편람 383쪽 후퇴 0쪽입니다. CDP 19222 실제 앞/뒤·같은 쪽·칸→본문 이동, 실제 소유·선택·포인터·Undo/Redo·HWP 저장 재열기를 확인했습니다.
- 보안: CodeQL 전체 성공이며 기존 #478·#483·#484는 10월 31일 생성·해당 파일 Git blob 불변을 확인했습니다. 신규 경고는 없고 기존 분류·탐지 설정을 바꾸지 않았습니다.

| 필수 원격 검사 | 실제 성공 실행 |
| --- | --- |
| CI (Archive 12,584 passed·기존 45 skipped, Native·Lint·Frontend 포함) | [42308262394](https://github.com/edwardkim/rhwp/actions/runs/42308262394) |
| CodeQL | [42308262423](https://github.com/edwardkim/rhwp/actions/runs/42308262423) |
| Render Diff | [42308262421](https://github.com/edwardkim/rhwp/actions/runs/42308262421) |
| Adapter / Proptest | [42308262441](https://github.com/edwardkim/rhwp/actions/runs/42308262441) / [42308262352](https://github.com/edwardkim/rhwp/actions/runs/42308262352) |
| 최종 CI Impact Policy | [42315950854](https://github.com/edwardkim/rhwp/actions/runs/42315950854) — success |

[원본/PDF·전쪽 TSV·확대 판독·실제 CDP 증거](https://github.com/edwardkim/rhwp/blob/07510f602a6081d90c94397fe29613420f3e7f63/mydocs/pr/assets/kang-t-20261106/README.md)를 근거로 수용합니다. 작업지시자가 CI 성공 후 merge·원 PR/이슈 완료 반영·감사 댓글·기록·devel 동기화·종료 정리까지 승인했습니다. 병합 직전 같은 head와 CI를 다시 확인하고 그 순서를 따릅니다.


## 후속 기록

[원격 검증·운영 기록](../assets/kang-t-20261106/remote/completion.md), [개별 #8566](pr_8566_review.md)·[#8595](pr_8595_review.md)·[#8632](pr_8632_review.md). 이 파일은 소스·테스트·workflow를 바꾸지 않는 maintainer 운영 기록이다.
