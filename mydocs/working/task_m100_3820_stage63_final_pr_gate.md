---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-08
---

# Task #3820 Stage 63 — issue2007 최종 PR 게이트

## 기준 상태

- 기준 브랜치: `upstream/devel`
- 기준 SHA: `fcc3b2135fa782699b66b583ddf11fe9f748306e`
- Stage 62 수정 SHA: `1a9e05356`
- 최종 코드 게이트 SHA: `74fecfd68ae0b479d2af422be6401c0b17efc0ae`
- 작업 브랜치: `task/3820-3821-fidelity`
- 게이트 종료 상태: `upstream/devel` 위 ahead 89, behind 0

Stage 62에서 issue2007 물리 p14의 terminal 중첩 표 뒤 과잉 여백을 PDF 기준으로
복원했다. 이 Stage에서는 코드를 더 넓히지 않고, 최신 기준 위 최종 결과를 새 전용
target에서 처음부터 검증한다.

## 검증 범위

1. issue2007과 같은 분할 표 경로의 focused 회귀
2. issue2007 p7–p17 페이지별 visual sweep 및 PDF 직접 대조
3. 전체 release-test integration과 overflow-cell baseline
4. Native Skia, fmt, clippy, rustdoc, Studio TypeScript·unit
5. 새 WASM 빌드와 브라우저 E2E
6. Markdown link, LFS, branch ancestry와 clean 상태

Cargo 검증은 `CARGO_INCREMENTAL=0`과
`CARGO_TARGET_DIR=target/task-3820-stage63-final-pr-gate`를 공통으로 사용한다.
`cargo test --profile release-test --tests`는 장시간 실행을 정상으로 보고 최종 exit
code와 summary가 나올 때까지 종료하지 않는다.

## 완료 조건

- 모든 공식 게이트 실패 0, clippy warning 0
- issue2007 전체 17쪽과 p7–p17의 requested/completed/missing 일치
- p12·p14·p15의 블록 간격, p16·p17 상단 continuation, 표 경계가 공식 PDF와 일치
- 최종 검증 SHA·바이너리 SHA-256·명령별 결과와 증적을 이 문서에 기록
- 오늘할일과 PR review 문서를 archive 상태로 같은 PR에 포함할 준비 완료

## 최종 코드와 검증 순서

Stage 62 구현은 그대로 유지했다. 최종 clippy에서 두 개의
`bool.then(...).unwrap_or(0.0)` 표현에 `obfuscated_if_else` 경고가 발생해 동등한
명시적 `if/else`로만 정리했다. 분기 조건, 계산식, 반환값은 바꾸지 않았으며 정리
뒤 focused 및 전체 release-test를 다시 실행했다.

| 검증 | 최종 결과 |
|---|---|
| issue2007 focused integration | `15/15` 통과, 17쪽 유지 |
| #3637 split nested table focused | `3/3` 통과 |
| #3385·#3385b·#4224 PUA focused | `2/2 + 4/4 + 2/2` 통과 |
| `cargo build --release` | exit `0`, 4분 17초 |
| `cargo test --release --lib` | `3322` 통과, 실패 `0`, ignored `10` |
| `cargo test --profile release-test --tests` | 최종 코드 재실행 exit `0`, 실패 `0` |
| overflow-cell baseline | `overflow_cell_lines_do_not_grow` 통과, 126.91초 |
| Native Skia lib | `58/58` 통과 |
| Native Skia missing placeholder | `2/2` 통과 |
| Native Skia direct PDF | `4/4` 통과 |
| `cargo fmt --check` | exit `0` |
| `cargo clippy --all-targets -- -D warnings` | exit `0`, warning `0`, 43.94초 |
| `cargo test --doc` | `4/4` 통과, ignored `2` |
| Studio `npx tsc --noEmit` | exit `0` |
| Studio unit | `802/802` 통과 |
| fresh `wasm-pack build --target web --out-dir pkg` | exit `0`, 2분 16초 |
| 브라우저 #536·#4158·#4224 | 3개 E2E 모두 통과 |
| E2E manifest | tracked `88` / manifest `88`, 이상 없음 |

전체 integration은 clippy 표현식 정리 전 결과를 재사용하지 않았다. 최종 코드로
다시 실행해 이전 실패 축이었던 674개 샘플 overflow-cell 스윕을 포함한 모든
integration binary의 exit `0`을 확인했다.

## p7–p17 PDF 직접 대조

최종 코드 게이트 SHA에서 빌드한 release-test 바이너리로 다음 명령을 다시
실행했다.

```text
python3 scripts/visual_sweep.py \
  --hwp samples/basic/issue2007_nested_cell_pagination_42065.hwp \
  --pdf pdf/basic/issue2007_nested_cell_pagination_42065-2020.pdf \
  --key task3820-stage63-final-p7-p17 --pages 7-17 \
  --rhwp-bin target/task-3820-stage63-final-pr-gate/release-test/rhwp \
  --dpi 180 --out output/task3820-stage63-final-p7-p17
```

- 전체 문서 SVG/render-tree: `17/17`
- 선택 페이지 requested/completed/missing: `11/11/0`
- raster/compare/overlay/review: 각각 `11/11`
- 자동 구조 후보: `0`쪽
- pixel threshold: `32`, 평균 pixel match: `90.35502%`
- 평균 visual proxy: `10.78162%`, 최저 p11 `6.50971%`

낮은 proxy 수치를 자동 합격으로 해석하지 않았다. 각 원본 크기 review에서 rhwp와
한컴오피스 2020 PDF를 직접 대조해 다음을 확인했다.

- p7의 불필요한 표 선과 제목 중복이 없고 p8 하단에 추가 선이 없다.
- p9–p15의 분할 표 상·하단 경계와 다음 block 소유권이 PDF와 같다.
- p12·p15 제목 뒤 저장 간격과 p14 중간 block 간격은 CSS `1px` 이내다.
- p10·p11의 continuation 내용이 소실·중복되지 않는다.
- p16·p17 상단 문단이 잘리지 않고 PDF와 같은 페이지에서 시작한다.

대표 및 원장 증적:

- [p7–p17 contact sheet](../pr/assets/task_m100_3820_stage63_final_pr_gate/review_contact_sheet.png)
- [실행 provenance](../pr/assets/task_m100_3820_stage63_final_pr_gate/run_manifest.json)
- [완료 summary](../pr/assets/task_m100_3820_stage63_final_pr_gate/summary.json)
- [페이지별 overlay 지표](../pr/assets/task_m100_3820_stage63_final_pr_gate/overlay_metrics.json)
- [구조 지표](../pr/assets/task_m100_3820_stage63_final_pr_gate/metrics.json)
- [자동 후보 목록](../pr/assets/task_m100_3820_stage63_final_pr_gate/flagged_pages.json)
- 페이지별 합성본: `review_007.png`부터 `review_017.png`

입력과 실행 provenance SHA-256은 다음과 같다.

| 대상 | SHA-256 |
|---|---|
| HWP | `bebd4ce3691246b0fb3ae332e1d40bc51d9035cddb9fc3d378466b6a8a2b5626` |
| 공식 PDF | `9b0390f856bb9ad43337679babf6677209b7c7ab678b6616fcc6d6d5551ff1c4` |
| `scripts/visual_sweep.py` | `fd201f988e0cca9d8fe11a08e9e54551a44fa013413252e4fcc04485e90b336f` |
| 최종 release-test 바이너리 | `e0debebb26ffb23a90b32e9086a3b59e580b9ec754ffa233a2e14d6e744d2b8e` |

증적 묶음은 약 19MiB이고 최대 파일은 약 2.55MB다. `git check-attr` 결과 전부
`filter: unspecified`이며 저장소 LFS 대상인 `pdf-large/**/*.pdf`에 해당하지 않아
일반 Git 파일로 보존한다.

## #4253과의 경계

[PR #4253](https://github.com/edwardkim/rhwp/pull/4253)과 정확히 같은 경로는
`mydocs/manual/cli_commands.md` 하나뿐이다. 이 작업은 fidelity compare 설명부,
#4253은 front matter·CLI 사용 예·changelog를 수정해 hunk가 겹치지 않는다.
`gh pr diff 4253 | git apply --check --verbose -`도 현재 작업 기준 exit `0`이었다.
renderer, issue2007·#3637 회귀, 작업 문서와 증적에는 공통 경로가 없다.

#4253이 먼저 병합돼 현재 branch를 새 `upstream/devel` 위로 rebase해야 하면, 해당
PR이 추가하는 `tests/edit_render_diff_gate.rs`를 포함해 새 코드 SHA에서 필수
게이트를 다시 확인한다. 단순 base 전진만 있고 현재 code head를 바꾸지 않으면
PR workflow의 trailing docs fast-pass 규약을 따른다.

## 결론

Stage 60–62에서 수정한 issue2007 분할 중첩 표의 페이지 경계·간격은 p7–p17
PDF 직접 대조와 전체 회귀에서 유지됐다. 최종 코드 게이트 실패는 0이며, PR 번호를
받은 뒤 오늘할일과 archive PR review를 같은 source branch의 trailing 문서
커밋으로 추가할 준비가 됐다.
