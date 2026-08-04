# PR #3125 검토 기록 — 거대 표 pagination의 resumable step 분리

## 메타와 통합 판단

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#3125](https://github.com/edwardkim/rhwp/pull/3125) |
| 작성자 / 관련 이슈 | `postmelee` / [#2424](https://github.com/edwardkim/rhwp/issues/2424) (open 유지) |
| 원 PR 기준 | `devel` / head `d277f974928d3348efc7320d7295c5d962cca596` |
| 원 PR 최신 상태 | OPEN, MERGEABLE/CLEAN, required check 성공, maintainer 보류 코멘트 없음 (2026-07-24 확인) |
| 검토 브랜치 | `integrate/postmelee-20260724` |
| 통합 기준 | `upstream/devel@1b5950a95` |
| 누적 순서 | 1/3 — #3125 → #3130 → #3136 |
| 처리 결론 | 원 PR을 직접 merge하지 않고, 최신 `devel` 위 통합 PR 후보에 작성자 정보와 기능 커밋을 보존해 수용 권고 |

검토는 작업지시자가 기본 작업트리에서 진행을 볼 수 있도록 `devel` 위
`integrate/postmelee-20260724`에서 수행했다. 이 방식은
[PR 리뷰·통합 워크플로](../../manual/pr_review_workflow.md) 4.1.1절에 절차로 반영했다.

## 범위와 적용

#2424의 목표는 거대 셀/중첩 표 pagination을 한 번에 끝내지 않고 재개 가능한 단계로 나누는 것이다.
이 PR은 deferred pagination job·continuation fragment·삭제 뒤 재개 경로와 Studio IME anchor cache를
도입한다. 원 PR의 `Merge branch 'devel'` 커밋 `faa28a8`, `d277f974`는 현재 기준
`upstream/devel`에 이미 반영된 이력을 다시 들여오지 않도록 제외했다.

적용한 기능/문서 커밋은 다음과 같다.

```text
75f412f 7ca01ee 4ba2262 59def61 7338ad5 c4533cd 46afd83
7fc36e9 9263cf9 1e10d9e 063cdd3 04bdf0d
```

충돌은 `mydocs/orders/20260722.md`에서만 났다. #2424 완료 행은 유지하고, 이미 존재하던
#2431 기록을 덮어쓰지 않도록 병합했다. Rust 동작·테스트 충돌은 없었다.

## 누적 검증

아래 결과는 세 PR을 적용하고 메인터너가 충돌을 조정한 최종 통합 tree에서 한 번씩 실행한 결과다.
문서/asset 추가 뒤에는 코드가 바뀌지 않았으므로 전체 cargo suite를 다시 실행하지 않는다.

| 게이트 | 결과 |
| --- | --- |
| `git diff --check`, `cargo fmt --check`, `cargo check --lib`, release build | PASS |
| `cargo test --release --lib` | 2,888 passed, 0 failed, 7 ignored |
| `cargo test --profile release-test --tests` | 최초 1회 실행 완료; 실패 출력 없음 |
| Native Skia 공식 suite 및 `issue_2225_missing_picture_placeholder` | 각 1회 완료; 실패 출력 없음 |
| `render_p37_direct_pdf_export` | 4 passed |
| clippy / doctest | 각 1회 완료; warning·failure 출력 없음 |
| 사용자 실행 `wasm-pack build --target web --out-dir pkg` | PASS (2026-07-24) |

### 시각 검증

거대 표 pagination 영향 샘플
`samples/issue1949_giant_cell_nested_tables_perf.hwpx`와 기준 PDF
`pdf/issue1949_giant_cell_nested_tables_perf-2024.pdf`의 14–16쪽을 비교했다.

- 3쪽 모두 자동 후보 0건, frame/content-bottom/line-order/tail 후보 0건.
- pixel match 평균 92.323%, 최저 91.624%; p016은 92.390%, visual accuracy proxy 14.228%다.
  이 보조값은 폰트·anti-aliasing 차이가 크게 반영하므로 사람 판독과 함께 사용했다.
- p016의 표/문단 흐름·순서·페이지 경계에 조판 붕괴가 없음을 확인했다.
- 임시 산출물: `output/pr-review-3125-20260724/pr3125-issue1949/{compare,overlay,review}/…_016.png`
- 안정 검토 자산: `mydocs/pr/assets/pr_3125_postmelee_issue1949_p016_review.png`

선택 범위 밖의 exporter 진단 `LAYOUT_OVERFLOW` 로그는 있었지만 자동 시각 후보가 아니며,
이 검토의 14–16쪽 대상으로는 재현되지 않았다. #2424는 open으로 유지해 이후 거대 표 범위를 계속
추적한다.

## 리스크와 권고

#3130의 revision 기반 derived state가 같은 pagination 입력 경로를 바꾸므로, #3125 단독 merge보다
누적 통합 tree에서 확인하는 것이 안전하다. #3125의 원 PR CI는 green이지만, 최종 수용 기준은
통합 PR 최신 head의 required CI 성공이다.

통합 PR merge 뒤 원 PR을 supersede 처리할 수 있다. #2424는 추가 후속 범위를 갖는 open 이슈이므로
자동 close하지 않고, 원 PR close/comment는 merge 후 상태 확인 및 별도 승인 뒤에만 수행한다.
