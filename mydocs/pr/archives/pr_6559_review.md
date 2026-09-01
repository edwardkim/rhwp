# PR #6559 검토 - 어울림 표의 저장 프레임 꼬리 확장 상한

- 검토일: 2026-09-01
- 작성자: `planet6897`
- 원 PR/source head: `c0e24543a1217aa0e1ee6b689ba15c61ed25520c`
- 누적 순서: 2/3 (`#6554 -> #6559 -> #6560`)
- 통합 적용 commit: `2d7e32457c3ec903ba924f1fbd4252a8065f4ed9`
- 통합 기준: `upstream/devel@6d3fd65a30cd8e6755b18a1ab44c5035279b110f`
- conflict: `tests/fixtures/ir_field_sweep_baseline.tsv` 1건
- 판정: 메인터너 보정 후 수용 가능

## 코드 판정

기존 저장 프레임 꼬리 확장 상한은 `mid_frame_only` 갈래에만 적용됐다. PR은 옆으로 본문이 흐르는
`TextWrap::Square` 표에도 같은 bounded branch를 적용해, 제한 없는 확장이 행 예산을 늘리고 표를 한 쪽에
밀어 넣는 경로를 막는다. 382쪽 편람의 기존 보호 사례는 모두 `TopAndBottom`이므로 이 갈래에 들어오지
않으며 관련 핀에서도 쪽수를 유지했다.

## conflict 처리

원 PR의 GitHub 상태는 `CONFLICTING/DIRTY`였다. 충돌은 Rust source가 아니라 IR 기준선 한 파일에만
있었다. 최신 `devel`의 `cae16410d`가 이미 #6524와 #6542 행을 사전순 위치로 재산출했으므로 incoming의
꼬리 두 행을 그대로 합치면 #6524를 중복·과거 위치로 되돌리게 된다.

따라서 최신 사전순 기준선을 유지하고 신규 fixture의
`issue6549/16418295_square_rowbreak_table.hwp list_header_width_ref 17` 한 행만 #6542 뒤에 추가했다.
누적 head에서 IR field sweep 전건을 다시 실행해 4/4 통과했으므로 충돌 표식을 제거한 것만으로 판정하지
않았다. 원 contributor branch는 수정하거나 force-push하지 않았다.

## 누적 검증

- `square_rowbreak_table_splits_instead_of_overfilling_the_page`: 통과, 결과 2쪽
- #3931: 5/5 통과
- #3930: 3/3 통과
- #5801: 2/2 통과
- oracle page-count와 off-canvas 각 16 partition 통과
- IR field sweep 4/4 통과, 전건 비교 848.94초
- 누적 head `ee4d7c4ad196b6642a4bfc14bd8fa9d11bb2994a`의 필수 Rust lint 묶음 전부 통과
- `release-test` 전체 회귀 8,917/8,917 통과, 46건 정책상 skip, 실패 0건(313.667초)
- Native Skia 전체 `--lib`, 누락 이미지 placeholder 2/2, 직접 PDF export 4/4 통과
- Docker 29.7.2 WASM 빌드 통과(6분 15초, `/app/pkg` 생성)
- 로컬 `cargo-nextest` 0.9.137은 권장 0.9.140보다 낮아 `junit.report-skipped` 설정 경고가 있었지만
  test selection과 실행 결과에는 영향이 없었다. 이 환경 차이는 통합 PR에 명시한다.
- Draft 통합 PR #6563의 code candidate head `ee4d7c4ad196b6642a4bfc14bd8fa9d11bb2994a`에서 Full CI가
  성공했다. CI run `33480418819`, CodeQL `33480418938`, Render Diff `33480418671`, Proptest
  `33480418840`, Adapter inter-diff `33480418917`이 모두 녹색이었다.

## engine 2020 직접 시각 검증

원본 `samples/issue6549/16418295_square_rowbreak_table.hwp`의 SHA-256은
`b002108391f6b078cff3ff4570bd3ff0f41a2cc44597ae4f63a35ba4f9974bf1`이고 마지막 저장 제품은
`hancom-office-2020`(`11.0.0.5178`)이다. 정책상 engine 2020 PDF가 기준이어야 한다.

원 PR의 `mydocs/report/square-rowbreak-extension-6549/pages_before_after_oracle.png`는 한컴 2024
참고 자료로만 두었다. 검토자는 MCP client의 비동기 `start -> status -> download` 흐름으로 engine
2020 PDF를 새로 산출했다.

- MCP job: `1fb65248-8df5-4a7f-8e94-7428bd3d9b85`
- 요청/응답 engine·profile: `2020` / `2020`
- 한컴 version: `12.0.0.4605`
- backend/worker: `hwp-managed-direct-dll-host`, 32-bit
- 결과: success, 88,050 bytes, 2쪽 A4, PDF 1.6
- 기준 PDF: `pdf/16418295_square_rowbreak_table-2020.pdf`
- 기준 PDF SHA-256: `3e60433161a849b2177ab26b1853090872bf764cc91e3df34315f0ca45a7f10b`
- 비교 절차: [PDF/SVG visual sweep 가이드](../../manual/verification/visual_sweep_guide.md#github-merge-comment)
- 실행 command:

```bash
python3 scripts/visual_sweep.py \
  --key pr6559-square-rowbreak \
  --hwp samples/issue6549/16418295_square_rowbreak_table.hwp \
  --pdf pdf/16418295_square_rowbreak_table-2020.pdf \
  --pages 1,2 \
  --svg-rasterizer rsvg \
  --out output/pr-review-planet/6559-2020
```

| physical page | compare | overlay | review | pixel match | visual accuracy proxy | 자동 후보 |
|---:|---|---|---|---:|---:|---:|
| 1 | `output/pr-review-planet/6559-2020/pr6559-square-rowbreak/compare/compare_001.png` | `output/pr-review-planet/6559-2020/pr6559-square-rowbreak/overlay/overlay_001.png` | `output/pr-review-planet/6559-2020/pr6559-square-rowbreak/review/review_001.png` | 90.30743% | 12.62612% | 0 |
| 2 | `output/pr-review-planet/6559-2020/pr6559-square-rowbreak/compare/compare_002.png` | `output/pr-review-planet/6559-2020/pr6559-square-rowbreak/overlay/overlay_002.png` | `output/pr-review-planet/6559-2020/pr6559-square-rowbreak/review/review_002.png` | 99.41514% | 7.07413% | 0 |

두 review PNG를 직접 열었다. p1은 한컴과 같은 행까지 배치되고 하단 넘침이 없으며, p2에는 두 결과 모두
동일한 마지막 행 하나만 이어진다. 자동 후보는 0건이다. 낮은 visual accuracy proxy는 font·glyph의
픽셀 차이를 포함하므로 전체 fidelity 점수가 아니며, 이번 계약인 쪽수·행 분할·하단 안전성은 일치한다.

- 대표 asset: [p1·p2 review contact sheet](../assets/pr_6559_issue6549_2020_review.png)
- 대표 asset SHA-256:
  `79e5f479b1177a098de1f4fd5e471967605f93c8af685e804f36ff0605e93d0d`

원 contributor head는 최신 `devel`과 직접 merge할 수 없으므로 승인 대상으로 쓰지 않는다. 의미 기반
baseline conflict 해결과 위 검증을 포함한 integration head만 `메인터너 보정 후 수용 가능`이다.

## Merge 후 contributor PR comment 계획

- 원 head `c0e24543` -> 적용 `2d7e32457` -> 통합 merge SHA의 계보를 남긴다.
- baseline 충돌에서 중복 과거 행을 되살리지 않고 신규 행만 추가한 이유를 알린다.
- engine 2020 p1·p2, 자동 후보 0건, 위 지표와 행 분할 일치를 기록한다.
- 대표 PNG `mydocs/pr/assets/pr_6559_issue6549_2020_review.png`는
  `<merge-commit-sha>` 고정 raw URL로 표시한다.
- UTF-8 without BOM body file로 게시하고 API로 재조회한 뒤 원 PR을 중복 병합하지 않고 close한다.
- #6549는 신고된 표 분할 계약이 해결됐는지 본문과 후속 범위를 재확인한 뒤 별도 close한다.
