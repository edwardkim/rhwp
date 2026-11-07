# PR #8711 원격 처리안

아직 실행하지 않은 승인 대상이다. 제출 후보의 로컬 필수 검증을 모두 마치고 정확한 제출 head를 확정한 뒤 작업지시자에게 이 범위를 요청한다.

1. 원 PR head `57f897a3bd844aadd883d7afb31def4904acb8ca`와 최신 `upstream/devel`을 다시 조회한다. 검토 이후 소스가 바뀌었으면 변경을 대조하고 필요한 검증을 보완한다. 준비한 후보는 원 source head의 자손이며 강제 push를 사용하지 않는다.
2. 기존 upstream head `review/pr-8226-save-recheck`에 최신 base 통합·테스트 주석 보강·검토 증거를 fast-forward push한다. 새 PR을 만들지 않는다. 본문은 원 기여·협업자 검토를 보존하고 메인테이너 실제 결과와 정확한 head SHA의 TSV/review/overlay 이미지를 보충한다.
3. 새 head의 required check·CI aggregate·CodeQL·Render Diff·Adapter·Proptest·정책 감사를 확인한다. 기존 source head의 녹색 CI를 새 head의 성공으로 표시하지 않는다. workflow 취소·재실행·보안 재분류는 이 안에 포함하지 않는다.
4. 최신 base merge simulation, 같은 head check, mergeability를 재확인하고 self-review를 기록한다. 승인된 정확한 head에 `--match-head-commit`을 적용해 merge commit으로 병합한다. source 원 기여와 협업자 commit을 보존한다.
5. merge SHA와 duration 갱신 결과를 확인한다. archive review·운영 증거·오늘할일을 `mydocs/`만 포함한 운영 기록 commit으로 devel에 반영하고 기본 devel을 fast-forward 동기화한다. 제품 코드·test·workflow·golden을 운영 기록 push에 섞지 않는다. 이번 PR은 CI 정의/정책을 바꾸지 않아 `post_merge.md` 7.1.1의 옵션 2 대상이 아니다.
6. [원 PR #8226](https://github.com/edwardkim/rhwp/pull/8226)은 체리픽 수용이므로 자동 MERGED를 가정하지 않는다. 이미 완료됐는지 조회한 뒤, OPEN이면 실제 merge SHA와 credit 대응을 설명하는 감사 댓글을 게시하고 대체 통합 완료로 close한다. [#8711](https://github.com/edwardkim/rhwp/pull/8711)에도 postmelee/semanticist21의 기여, 실제 검증·시각 증거를 포함한 감사 댓글을 게시한다. 댓글 전에는 SHA로 고정한 raw PNG의 형식·해시를 확인하고 게시 뒤 API와 실제 이미지 표시를 검증한다.
7. #4680·#4687은 현재 CLOSED이며 이 PR로 전체 문제를 새로 해결했다고 주장하거나 상태를 바꾸지 않는다. 다른 issue/PR도 자동 종료하지 않는다.
8. 현재 검토에서 만든 clean한 local worktree·review branch·PR fetch ref를 제거한다. 저장소 루트 `output/` 전체와 공용 `target/pr-review/debug`는 실제 빌드/테스트 사용 종료·정식 증거 보존 뒤 자동 정리하고 외부 정리 영수증을 남긴다. 기본 작업공간과 `pr-8296-seal-pua`는 보존한다. **기존 upstream `review/pr-8226-save-recheck`는 협업자가 만든 branch이므로 자동 삭제하지 않는다.** 원 contributor fork branch도 보존한다.

초기 원격 승인은 이 전체 범위를 요청한다. 승인 전에는 push·본문/댓글 게시·approve·merge·원 PR close·devel 운영 기록 push를 수행하지 않는다.
