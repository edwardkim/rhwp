# PR #9369 리뷰 — Visual Sweep에서 한컴 Print 영수증 대조 절차 안내

## 최종 판정

**승인.** 작성자 자체 검토 대상 `45dcafa180f18e381be1daa827419a35bb67d0ee`에서 가이드의 링크와 대상 절을 확인했다. 이 기록을 포함한 최종 head의 필수 검사와 최신 devel 병합 가능 여부를 확인한 뒤 병합한다.

## 접수와 변경 범위

[PR #9369](https://github.com/edwardkim/rhwp/pull/9369)는 postmelee의 collaborator self PR이며 대상은 devel이다. [이슈 #9367](https://github.com/edwardkim/rhwp/issues/9367)의 기준 PDF 출처 확인 경로를 안내한다. 변경 파일은 [Visual Sweep 가이드](../../manual/verification/visual_sweep_guide.md) 한 개다. [Print 영수증 절차](../../manual/mcp_hwp2024Convert_usage.md#print-영수증을-남기고-다시-확인하기)에서 설명하는 생성 기록과 원본·PDF 해시 검증을 연결한다.

## 검증 입력과 결과

- 문서 diff, 공백 검사, 대상 파일과 절 제목을 확인했다. 검토 시점의 devel `54d6aca1d7381900688b6bcd1292e892cd1a9bf6`과 코드 후보의 병합 시뮬레이션이 통과했다.
- 검토한 코드 head의 [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/45189023314/job/137742263766)는 성공했다.
- 원본·기준 PDF나 조판·렌더링 구현, 회귀 기대값을 바꾸는 변경이 없어 검증 입력 커밋 확인과 시각 대조는 비해당이다. 새 출력 프로필의 승인 기준은 [이슈 #9370](https://github.com/edwardkim/rhwp/issues/9370)에서 다룬다.
