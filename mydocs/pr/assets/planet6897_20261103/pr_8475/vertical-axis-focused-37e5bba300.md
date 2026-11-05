# 세로쓰기 글자 보존의 수정 전후 집중 검증

수정 전 source `c732203d4bd8fb7ae6ef158f1bacd210bd8509d9`, 보정 source `37e5bba3007614d9304c3dbf863503a8c7340405`.

같은 검출 함수 SHA-256 `36ac322371a4920c60688e8a6a8e9d1d6384a772c6205200924c8b26bc76236c`를 대조했다. 검출 assertion을 완화하지 않았다.

| 실행 | PASS | FAIL | exit | 실제 로그 SHA-256 |
| --- | ---: | ---: | ---: | --- |
| 수정 전 새 관계 검사 | 0 | 2 | 101 | `c12c00f12411abe656762c59aa954a9dfff84e7245ee8b5017fd8bfcd7b119d7` |
| 수정 후 `vertical-axis-fix-v5-focused-issue_8470_vertical_cell_content_axis` | 2 | 0 | 0 | `ac2f781736938380633c26ccade7edc31228e12d05b460e34e418836fd422663` |
| 수정 후 `vertical-axis-fix-v5-focused-issue_8470_vertical_cell_table_no_split` | 4 | 0 | 0 | `61a7728d4ad9332313cde139c309d39be570b6ddb7fe2ca864cd37b48dafae5a` |
| 수정 후 `vertical-axis-fix-v5-focused-issue_6029_vertical_cell_line_budget` | 1 | 0 | 0 | `d4df6487c7dc9e75970aca703be2137714b855da1c0c87e51eb5ea995d2e8f97` |

최소 글꼴 환경: 격리된 공개 글꼴 5개, 설치 OTF 의존 없음. 컴파일 8잡, 공유 target `/home/tsjang/rhwp/target/pr-review`.

모든 수정 후 검사는 기본 Rust 옵션, `release-test` profile과 `native-skia` feature로 실행했다. 진단용 cfg를 넣은 결과로 통과를 대신하지 않았다.

실제 실행 예:

```bash
bash /home/tsjang/rhwp/output/pr-review/planet6897-20261103/run_minfont_vertical_worktree.sh \
  node scripts/run-rust-test.mjs --cargo-test issue_8470_vertical_cell_content_axis -- \
  --profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia
```

작업 디렉터리는 `/home/tsjang/rhwp-worktrees/review-pr8475-vertical-axis-fix`였다. 두 기존 검사 모듈도 같은 실행 경로에서 순차 실행했다.

수정 전·후 검출 함수는 다섯 글자의 누락·중복, 동일 세로줄, 셀 내부 포함, 위아래 내용 순서와 비겹침을 검사한다. 추가로 모델의 합성 줄도 같은 단일 세로줄인지 검사해 paint만 고치는 우회를 차단했다. 절대 px나 설치 글꼴의 특정 폭으로 기대값을 정하지 않았다.

초기 실행의 `llms.txt` 누락·workspace 오류와 공유 target에서 이전 lib를 Fresh로 사용한 실행은 판정에서 제외했다. 새 helper가 실제 dep-info에 포함된 기본 옵션 재컴파일 뒤의 결과만 위 표에 기록했다.

| 보정 파일 | 실제 SHA-256 |
| --- | --- |
| `src/document_core/commands/document.rs` | `8ad96f48f0b7c3050914cc608789e881d85e75ad5a387fac77bbaa7d23f9c7c6` |
| `src/document_core/commands/document/cell_line_frame.rs` | `c0212f6926272569c756b33a60b674ac809bdbccf4c0a009e7e944d4da4dbde8` |
| `src/renderer/composer.rs` | `45850e66f64edcfe354736e0fc9f85e22a776543854168e8d2269a8287f8101b` |
| `src/renderer/composer/vertical_cell_lines.rs` | `54e4390b715a3d496e631509a9607cc7ead61fc07ff09021baf2e998c3f4c37a` |
| `src/renderer/layout/table_cell_content.rs` | `88b09b382a690b76dde140456859197e0b707352f607c1935b4481b999998595` |
| `tests/cases/issue_8470_vertical_cell_content_axis.rs` | `61514b0e147a2cdb3c8cd951128c66895131ca64f39432ec625eb0ee508f0d05` |

이 집중 검사만으로 최종 승인을 주장하지 않는다. [Native 전쪽·판독](vertical-axis-37e5bba300/native/README.md)과 이후 fresh WASM·필수 전체 검증을 함께 확인한다.
