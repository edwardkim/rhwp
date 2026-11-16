# #9047·#9050 종료 정리 기록

## 중간 정리

- 2026-11-13T14:42:08+09:00: #9047 MERGED·후속 기록·#8951 댓글을 확인하고, 원격 `review/integration-7620-sacru2red-followup-tests`를 exact SHA `68c2441bfd947609676b841e7ff43bf1c64983ab` lease로 삭제했다. 같은 로컬 브랜치도 병합 조상을 확인하고 삭제했다.
- 같은 경로의 worktree는 #8975 새 head 검증에 재사용 중이며 `review/pr-8975-updated-cellbreak-evidence`가 체크아웃돼 있다. 후속 PR #9050이 아직 실행 중이므로 전체 worktree/output/debug 정리는 완료로 표시하지 않는다.
- 원 fork branch·다른 worktree·사용자 글꼴·로컬 crates.io 소비 환경은 보존한다.

## 최종 정리 시작 — 2026-11-13T15:27:31+09:00

활성 파일 참조와 해당 공용 target의 Cargo/rustc/nextest/wasm-opt 사용은 0건이다. 유휴 cwd는 삭제 보류 근거로 사용하지 않았다. 원본 head가 현재 devel의 조상이며 작업 트리는 clean이고 이 원격 branch를 사용하는 열린 PR은 0건이다.

| 삭제 경로 | 삭제 전 할당 bytes |
| --- | ---: |
| `/home/edward/mygithub/rhwp-worktrees/review-integration-7620-sacru2red-followup-tests` | 11859677184 |
| `/home/edward/mygithub/rhwp/output` | 2091876352 |
| `/home/edward/mygithub/rhwp/target/pr-review/debug` | 13082714112 |

삭제 전 디스크 여유: 115530612736 bytes.

## 최종 정리 완료

- 전용 원격 branch는 exact-head lease로 삭제했다. 로컬 branch와 전용 worktree도 제거했다.
- 검토 소유 `pr8951-head`·`pr8975-head`·`pr8975-current` tracking ref는 예상 SHA를 대조하고 삭제했다.
- 저장소 루트 `output/` 전체와 기본 작업공간 `target/pr-review/debug` 전체를 삭제했으며 두 경로와 전용 worktree의 부재를 확인했다. 백업·압축·이관은 하지 않았다.
- 삭제 후 디스크 여유: 142579118080 bytes (132.79 GiB), 관측 증가 25.19 GiB.
- `target/pr-review/release-test`·다른 target profile·사용자 글꼴·`local/crates-io-consumption`은 보존했다.
- 다른 작업의 `pr-8296-seal-pua`, `review-integration-20261110-hyunmyungjanelee`, `task-8863-base-renderer`, `task-8863-char-shapes` worktree는 이번 검토 소유가 아니므로 유지했다.
- #8975 감사 댓글은 [게시 댓글](https://github.com/edwardkim/rhwp/pull/8975#issuecomment-6845852907)이며 실제 GitHub 페이지에서 비교 PNG 4개가 각각 2382×1146으로 로드되는 것을 확인했다. 게시 본문·원본 PNG SHA도 검증했다.

#8951의 footer 11쪽 게시 이미지도 실제 GitHub 페이지에서 2382×1160으로 로드됨을 별도 확인했다. 두 원 PR은 최신 head의 MERGED이고 #7620은 OPEN 유지한다.
