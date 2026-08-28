---
kind: pr-review-implementation
status: local-validation-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28
---

# open CI-green PR #6245~#6268 통합 검토 구현 기록

## 기준과 포함 범위

- 통합 브랜치: `review/open-ci-green-20260828`
- 최신 기준: `upstream/devel@1a43a507c9da`
- 포함 PR:
  - #6245 `9c53276c37c8` - #6194 머리 표 행 높이 과대 계상 보정
  - #6246 `37abb2599dca` - #6186 꼬리말 세로 정렬과 HWPX 왕복 보존
  - #6247 `6d3149551ea6` - `CellContext` 빈 경로 panic 방어
  - #6248 `d84f1e8a4fe1` - #6179 오른쪽 탭 뒤 TAC 개체 정렬
  - #6249 `e11ab9e89b07` - CIRCLED/GANADA 번호 포맷 OOB 방어
  - #6250 `9fc79fdd477b` - font/border 인덱스 OOB 방어
  - #6252 `31b697549bc6` - #6174 글상자 clip descender 잘림 보정
  - #6254 `00cac13820e3` - #6173 오른쪽 정렬 말미 공백 판정
  - #6259 `87447c260737` - #6167 TAC 표 자기 줄 leading 제거
  - #6260 `b3c67ae91cfb` - #6196 저장 단일 줄 셀의 과도한 자간 압축 억제
  - #6261 `91776e11acc3` - #6206 표 셀 안 쪽번호 재시작 수집
  - #6262 `612d3e78e67` - #6190 `TAG_INDENTATION` 없는 저장 lineSeg 들여쓰기 억제
  - #6265 `17fa0f782d0c` - #6192 셀 안 앞/뒤 그림의 host 문단 앵커 보정
  - #6268 `f58550248bdb` - #6208 문서 인쇄 방식(모아 찍기) 메타데이터 노출
- 제외:
  - #5953, #6059: draft
  - #6073: 실패 check가 있어 제외
  - #6270: 리베이스 시점의 최신 `upstream/devel`에 이미 merge되어 이번 통합 PR의 별도 체리픽
    대상으로 다루지 않는다.
  - #6083: 실패 check는 없지만 기존 메인터너 코멘트에서 현 상태 통합 보류/재작업을 요청했다.
    따라서 이번 CI-green 통합 대상에서 제외하고 `pr_6083_review.md`의 보류 판단을 유지한다.

## 체리픽과 최신성 확인

- 원 PR head를 `upstream/prNNNN-head`로 fetch한 뒤 PR 번호 순서로 `git cherry-pick -x` 적용했다.
- 적용 후 원 PR별 head SHA와 통합 브랜치의 최종 변경 내용을 비교했다. #6246은 force-push 이후
  commit patch-id 비교가 의미 없어진 상태라, 최신 PR head가 바꾼 파일의 최종 diff가 통합 브랜치와
  일치하는지 별도로 확인했다.
- 2026-08-28 재확인 시 모든 포함 PR은 non-draft, 실패 check 0건, 진행 check 0건이었다.
- 작업지시자 지시에 따라 이 목록 이후 새로 발견되는 PR은 승인 없이 자동 포함하지 않는다.
- #6246은 검토 중 head가 `89eeca512dcf`에서 `37abb2599dca`로 force-push 되었다. 최신 head와 비교해
  최종 diff가 일치하도록 stale conflict 보정 흔적을 제거했고, #6186 focused 2건을 다시 통과시켰다.

## 메인터너 보정

- #6245의 `ladder_pushed_following_line`이 모든 후속 문단을 `any()`로 훑으면, 여러 문단 뒤 누적
  `vpos`가 우연히 큰 값을 갖는 경우에도 자리차지 개체 흡수 증거로 오인할 수 있다.
- 통합 브랜치에서 `src/renderer/height_measurer.rs`를 보정해 바로 뒤의 실제 `lineSeg` 1개만
  확인하도록 좁혔다.
- 보정 커밋: `10ce6b419 fix(renderer): 사다리 흡수 판정 범위를 좁힌다`

## 로컬 검증

| 검증 | 결과 |
| --- | --- |
| `cargo fmt --all -- --check` | 통과 |
| `node scripts/rust-unit-test-tiers.mjs --check` | 통과, 4,221 tests / 299 modules |
| `node scripts/rust-test-suite-manifest.mjs --prepare && --check` | 통과, 998 sources / 4,435 static test attrs / 32 suites + 9 exceptions / 최소 6,559 cases |
| focused nextest (#6174/#6186/#6190/#6192/#6196/#6208) | 통과, 9 pass / 8,520 skipped |
| #6186 최신 head 정렬 후 focused nextest | 통과, 2 pass / 8,527 skipped |
| `cargo test --locked --lib test_format_number --target-dir target/pr-review` | 통과, 5 pass |
| `cargo test --locked --lib index_matches_legacy_linear_scan_exhaustively --target-dir target/pr-review` | 통과, 1 pass |
| `cargo test --locked --lib degenerate_inferred_row_uses_base_grid_instead_of_expanding_last_cell --target-dir target/pr-review` | 통과, 1 pass |
| `cargo test --locked --lib cursor_rect --target-dir target/pr-review` | 통과, 16 pass / 5 ignored |
| `cargo clippy --locked --all-targets --target-dir target/pr-review -- -D warnings` | 통과, 58.12s |
| 전체 `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast` | 통과, 8,486 passed / 43 skipped / 10 slow, 912.908s |
| `CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg` | 통과, 9m 03s |
| Native Skia lib | 통과, 3,946 pass / 13 ignored + contracts 15 pass + ooxml-chart 165 pass + password 2 pass |
| Native Skia `issue_2225_missing_picture_placeholder` | 통과, 2 pass |
| Native Skia `render_p37_direct_pdf_export` | 통과, 4 pass |
| `git diff --check` | 통과 |

## 시각 증적

- #6245/#6194: `mydocs/report/header-row-picture-height-6194/after_p1.png`와 `oracle_p1.png`를 직접
  확인했다. 머리 표 높이와 아래 표 분리가 기준과 가까워졌고 겹침이 보이지 않는다.
- #6246/#6186: `mydocs/report/footer-band-valign-6186/after_p2.png`와 `oracle_p2.png`를 직접 확인했다.
  꼬리말 쪽번호가 밴드 안에서 아래 정렬 위치로 내려오며 HWPX 왕복 보존 test가 이를 가드한다.
- #6247: `mydocs/report/bug-layout-empty-path/after.png`를 확인했다. 빈 `CellContext` 경로는 panic
  대신 `Option` 흐름으로 빠진다.
- #6248/#6179: `mydocs/report/right-tab-tac-object-6179/p1_footer_after.png`를 확인했다. 오른쪽
  꼬리말 로고가 용지 밖으로 나가지 않는다.
- #6249: `mydocs/report/bug-circled/README.md`와 before/after SVG 증적을 확인했다. 방어적 OOB
  수정이라 정상 문서 출력 변화는 기대하지 않는다.
- #6250: `mydocs/report/bug-font-border/after.png`를 확인했다. font/border OOB 방어 성격과 맞고
  눈에 띄는 회귀는 없었다.
- #6252/#6174: `mydocs/report/textbox-clip-descender-6174/after_p1.png`와 `oracle_p1.png`를 확인했다.
  글상자 clip이 글줄 하단 획을 자르지 않는다.
- #6254/#6173: `mydocs/report/right-align-inline-object-space-6173/p2_textbox_after.png`를 확인했다.
  글상자 안 두 로고가 우단 안에 배치된다.
- #6259/#6167: `mydocs/report/leading-space-tac-table-6167/p38_table_after.png`를 확인했다. 표가
  본문 좌단 기준으로 돌아오고 오른쪽 열 잘림이 보이지 않는다.
- #6260/#6196: `mydocs/report/cell-overflow-spacing-6196/p4_cell_after.png`를 확인했다. before에서
  우측 경계를 넘던 내용이 after에서 셀 안에 들어오며, 판단은 해당 fixture의 단일 줄 셀 보정으로 제한한다.
- #6261/#6206: `mydocs/report/assets/issue_6206/pagenum-113424-p7.png`와
  `pagenum-156555538-p2.png`를 확인했다. 표 셀 안 `새 번호로 시작` 이후 쪽번호가 절대값으로 재시작한다.
- #6262/#6190: `mydocs/report/stored-lineseg-indentation-6190/p3_after.png`를 확인했다.
  `TAG_INDENTATION`이 꺼진 저장 lineSeg에 불필요한 첫 줄 들여쓰기가 얹히지 않는다.
- #6265/#6192: `mydocs/report/cell-overlay-para-anchor-6192/p4_after.png`를 확인했다. 셀 안 앞/뒤 그림이
  host 문단 흐름 기준으로 앵커된다.
- #6268/#6208: `mydocs/report/print-method-nup-6208/oracle_2up_vs_rhwp_portrait.png`와
  `samples/issue6208/print_method_nup.hwp`를 확인했다. 수용 판단의 중심은 `rhwp info --json`/contract test의
  인쇄 방식 노출이며, PNG는 2-up 문서 provenance 보조 증적이다.

## 코멘트 처리 계획

- 통합 PR 본문과 merge 후 코멘트에는 포함/제외 PR, #6245 메인터너 보정, #6246 force-push 최신 head
  정렬, 로컬 검증 숫자를 함께 적는다.
- 원 PR 또는 관련 issue별 후속 코멘트에는 각 개별 review 문서의 `코멘트 처리` 절을 기준으로
  수용/보류 판단과 대표 증적을 남긴다.
- 증적자료를 새로 산출해야 하는 renderer/layout/paint/HWP/PDF 사안은
  `mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment`를 정본으로 사용한다.
  임시 output 경로만 적지 않고 대표 `review_*.png`와 summary JSON을 `mydocs/pr/assets` 아래 안정
  파일명으로 보존한다.
- GitHub comment에서 이미지를 보여줄 때는 asset이 merge commit에 포함된 뒤
  `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-sha>/mydocs/pr/assets/<file>.png` 형식의
  SHA 고정 raw URL을 사용한다. 자동 일치율 수치는 보조값이며 사람 판정을 대체하지 않는다는 문구를
  함께 둔다.
- 이번 통합 범위 이후 새 PR 또는 새 push는 작업지시자 승인 없이 자동 포함하지 않는다.

## 결론

#6245/#6246/#6247/#6248/#6249/#6250/#6252/#6254/#6259/#6260/#6261/#6262/#6265/#6268는
통합 수용 권고다. #6245에는 메인터너 보정 1건을 포함했고, #6246은 force-push 이후 최신 head와
최종 diff를 맞춘 뒤 focused 재검증했다. 최종 통합 head 기준으로 renderer/layout 필수 로컬 검증과
시각 증적 확인을 완료했다.
