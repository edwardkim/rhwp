# PR #7377 처리 계획

| 단계 | 상태 | 내용 |
| --- | --- | --- |
| 1 | 완료 | #7333 재현 HWP·한컴 PDF·기존 RHWP PDF와 50쪽 visual sweep으로 범위와 잔여를 분리했다. |
| 2 | 완료 | renderer 저장 줄·footer·도형 affine/marker 보정과 focused/full 검증을 완료했다. |
| 3 | 완료 | 8쪽 포함 모든 line의 선택 우선순위·cellPath·선택 쪽을 보정하고 실제 드래그 Undo E2E를 추가했다. |
| 4 | 완료 | head `3a78dbe0c`을 upstream 임시 branch에 push하고 PR #7377을 만들었다. |
| 5 | 완료 | self-review와 작업 기록을 trailing commit으로 추가한다. |
| 6 | 대기 | 최신 head CI, mergeability, 작업지시자 승인 뒤 merge한다. |
| 7 | 대기 | merge SHA 고정 visual asset comment, #7333/PR follow-up, devel 동기화와 소유 artifact 정리를 수행한다. |

곡률 지정은 #7375가 소유한다. #7333의 해결 범위를 그 기능으로 확장하지 않는다.
