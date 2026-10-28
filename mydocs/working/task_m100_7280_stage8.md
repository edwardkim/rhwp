# Task #7280 Stage 8 — R2f 문단 줄 후보 스캔 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2e](task_m100_7280_stage7.md), 시작 head `1806b25ad`.
- 상태: R2f 구현·고정 SHA 집중 검증 완료. R2 전체 완료나 제출 준비 완료가 아니다.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.

## 1. 책임과 범위

`typeset_paragraph`의 현재 fragment 줄 후보 스캔을 `paragraph/scan.rs`로 분리한다.
`scan_lines`는 문단 IR·기존 FormattedParagraph·현재 줄 범위/예산/강제 경계와
`LineScanPage` 관측값만 읽는다. TypesetEngine/TypesetState를 받지 않고 페이지를 만들지 않는다.
페이지 상태의 읽기는 `state.rs::paragraph_line_scan_page`가 맡는다. 이 값은 루프 내부에서
매번 만들므로 다음 단/쪽 전환 뒤 오래된 상태를 재사용하지 않는다.

스캔은 `end_line`, `cumulative`, `used_saved_tail_vpos_fit`를 함께 반환한다.
순서: 강제 경계 → 다단/TAC 그림 reset → 줄 높이 초과 시 저장 꼬리/각주 증거 평가 →
tail fit chain → 허용 여부 → advance 누적/끝 줄 갱신. 기존 조건·상수·주석·연산 순서를 유지한다.
예산 이내 줄과 첫 줄은 기존대로 overflow 증거 평가를 거치지 않는다.

HWPX 저장 reset fragment 조회도 동일 입력 관점으로 이동한다. 앞선 문단 강제 경계 선택 경로는
parent의 기존 signature wrapper를 유지하고 그 호출 시점에서 관측값을 전달한다.
profile은 Copy인 불변 값이며, body 높이는 기존 read-only layout 조회다. 스캔 도중 상태 쓰기가
없으므로 읽는 시점을 시작으로 모아도 값이 달라지지 않는다. 페이지 전환 시점은 바꾸지 않는다.

조판 오류 수정·기존 heuristic의 정당화가 아니다. 테스트/기대값/baseline/ignore·IR·public API는
변경하지 않는다. 공유 helper의 parent 의존은 명시적으로 남기며 최종 소유권 정리는 후속이다.

## 2. 생산·소비 경로

| 단계 | 위치와 보존 계약 |
| --- | --- |
| 입력 예산 | parent while 루프의 page_avail, spacing_before, drift 차감은 그대로 |
| 후보 계산 | scan_lines가 원래 순서대로 content 높이를 검사하고 line_advance를 누적 |
| 경계 보정 | 기존 split::refine_split_boundary가 end/cumulative를 수용; saved-tail flag는 재계산하지 않음 |
| 높이 소비 | parent가 보정된 줄 범위로 part_line_height/spacing_after를 계산; 전체 fit 재확인에 cumulative와 flag 사용 |
| 적용·이월 | Full/PartialParagraph 추가 → trimmed spacing 초기화 → current_height 전진 → 필요 시 단/쪽 전환과 cursor 갱신 |

표 셀/rowspan의 컷·이어받기와 각주 예약 자체는 이번 범위가 아니다. 본문 줄 스캔의 각주 높이
조회만 이동한다. 남은 fit/배치 직접 쓰기와 표 문단 흐름 조정은 아직 R2 이행 항목이다.

## 3. 검증 계획과 한계

`output/7280/stage8/verify-scan.mjs`로 스캔·HWPX helper를 원래 위치에 복원해 비교하고
입력/반환과 snapshot 필드 매핑을 확인한다. 공백/후행 쉼표 이외의 조건 변경은 허용하지 않는다.
기존 typeset/composer 테스트와 저장 경계·dirty 사다리 회귀를 집중 실행한다.
각주 여백 helper의 경계 검사는 실제 전체 문서 모든 각주 경로의 검증을 뜻하지 않는다.

고정 SHA review worktree에서 파생 suite 준비, fmt·고정 baseline 정책·native Clippy와
집중 테스트를 순차 실행한다. 최종 전체 회귀·WASM/workspace lint·workspace build·Native Skia·
fresh Docker WASM/직접 시각 검증은 통합 게이트에 남는다. 정적 보존/집중 통과를 시각 판정이나
#7280 제출 준비 완료로 승격하지 않는다. 원격 push·PR·댓글은 하지 않는다.

### 고정 SHA와 선행 검사

- 제품 SHA: `a17d342d8435722aefafc81601196b97800ea4f2`.
- review worktree: `/home/edward/mygithub/rhwp-review-7280-r2f`.
- 정적 보존 검사: `output/7280/stage8/extraction-proof.json` 통과.
- manifest: 고정 baseline 대비 1,382 sources / 5,965 static attrs / 48 integration targets 통과.
- unit-tier: 고정 baseline 대비 4,205 tests / 298 modules / ready 0 / support 87 /
  white-box 4,114 / cfg support items 28 통과.
- `cargo fmt --all -- --check` 통과.
- native Clippy(`--locked -- -D warnings`): exit 0, 54.78초.
- 로그: `output/7280/stage8/{prepare,manifest,unit-tier,fmt,clippy-native}.log`.

실행 환경은 기존 고정 review target, `CARGO_BUILD_JOBS=4`다. 파생 suite는 review worktree에서만
준비했다. 테스트를 옮기거나 추가하지 않았으며 제품 검증 이후 변경은 이 문서/계획 기록뿐이다.

집중 테스트 명령(review worktree):

```bash
CARGO_BUILD_JOBS=4 cargo nextest run --locked --cargo-profile release-test \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review \
  --lib --test regression_suite_011 --test regression_suite_019 --test regression_suite_028 \
  -E 'test(renderer::typeset::) | test(renderer::composer::) | test(issue_6031_ladder_sb_omitted_tail_overrun::) | test(issue_6542_mid_para_vpos_rewind_breaks_page::) | test(issue_6718_native_hwp5_zero_vpos_rewind::) | test(issue_6718_zero_rewind_in_split_paragraph::)' \
  --no-fail-fast
```

실제 prepare 결과의 suite 매핑으로 선택했다. #6031은 dirty 저장 사다리의 3/6쪽 하단 및
4쪽 첫 줄 소유, #6542/#6718은 render tree의 본문 하단과 저장 되감김 경계를 확인한다.
typeset 기존 테스트에는 saved-tail chain의 중단·각주 여백 허용/비허용 경계도 포함된다.
모든 포맷/다단/각주/그림 조합을 이번 집중 실행이 전수 검증한다는 뜻은 아니다.

집중 결과: **154건 통과 / 실패 0건**, 필터 비선택 4,539건, 7 binaries, exit 0.
빌드 4분 52초, 테스트 0.344초. 실행 ID: `ef26a9f0-efed-42dd-ae97-f147609a8725`.
로그: `output/7280/stage8/nextest-focused.log`.
`compare-focused.mjs` / `regression-comparison.json`에서 baseline 전체 실행 중 같은 필터로
선택한 154개 PASS 이름과 일치했다. 필터 비선택은 기존 ignore 50건과 별개이며, 이번에
전체 회귀를 재실행한 것은 아니다.

nextest 0.9.137(권장 0.9.140) 및 미사용 observation profile 설정 경고는 기준 실행과 같다.
현재 CI 도구 버전까지 같은 결과로 주장하지 않는다. review worktree의 tracked 변경은 없고,
파생 suite/manifest는 stage하지 않았다. 제품 SHA 이후 제품/테스트 변경은 없다.

## 4. 다음 절편

문단 후보 스캔과 경계 보정은 읽기 전용 계산으로 분리됐다. 실제 분할·배치 확정과 단/쪽 전환의
조정, 표 문단 흐름, state의 나머지 직접 쓰기 분리를 이어간다. 최종 전체 검증·직접 시각 증적을
남기기 전에는 R2 전체 또는 #7280을 완료로 판정하지 않는다.
