# PR #6880 검토: 자리차지 표 앵커 줄과 한컴 재저장 경계

## 최종 판정: 승인

메인터너 보정 완료 head를 승인한다. contributor 원 head와 보정 후 결과를 구분한다.
원본 절단 fixture의 절대 위치나 전체 원 문서 104쪽 fidelity가 일치한다고 판정하지 않는다.

## 대상과 처리 경로

- 원 PR: https://github.com/edwardkim/rhwp/pull/6880
- 작성자: planet6897, 검토자: jangster77. base: devel, source: planet6897/rhwp의 fix/6860-float-anchor-line-host-text.
- 경로: collaborator 매개 외부 PR + 로컬 검증 + 시각 증적 + review-only trailing + post_merge.
- 원 contributor head: 2c13b308ac53af1ff1270d2e674f3c9693a14b6b.
- 메인터너 보정: df1db1f20e113460a0ce8d042f4a1171014a7553.
- 최신 base 반영 code candidate: 58e41882137b154570c7056dc3051024a8f7168d.
- candidate의 base: c35e47ae4e647277f1d85d4fc570bac0728b655d.
- 문서 작성 전 diff: 14개 파일, +634/-9. 작성 시점 MERGEABLE/CLEAN이며 trailing head는 다시 확인한다.
- 관련 해결 대상: [#6860](https://github.com/edwardkim/rhwp/issues/6860). 별도 unsplit 경로 [#6879](https://github.com/edwardkim/rhwp/issues/6879)는 닫지 않는다.

## 발견과 보정

기존 절단 HWPX는 원 문서 쪽의 저장 vertpos를 보존하지만 한컴 PDF는 절단본을 재배치한다.
현재 rhwp의 기존 절단본 3쪽과 PDF 2쪽을 동일 저장 상태의 절대 위치 비교로 취급하지 않는다.
PR 본문의 절단본 2쪽 주장은 현재 검토 결과로 재확인되지 않았으며 이 기록이 실제 측정을 구분한다.
기존 fixture의 호스트 줄 순서·본문 내 배치 계약은 유지했다.

동일 저장 상태를 비교하기 위해 절단본을 한컴 HWP → HWPX로 재저장하고 그 HWPX에서 PDF를 생성했다.
이 비교본에서 마지막 표 행 첫 줄이 2쪽으로 밀리는 실제 차이를 보정했다.
첫 세 셀 문단의 저장 위치가 0 → 0 → 다음 한 줄 높이와 일치할 때만 행 분할 최소 유지 높이에
셀 패딩을 포함한다. 실제 쪽 공간 제한은 완화하지 않았다. 저장 reset 증거가 없는 대조군은 기존 기준을 유지한다.

## 완료한 검증

- 보정 제품 코드 df1db1f20: 전체 nextest 회귀 --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 --no-fail-fast, exit 0.
- 신규 두 HWPX 경로를 RHWP_SECURITY_SWEEP_SAMPLES_JSON에 지정해 정상 코퍼스 검사에 포함했다.
- #6860 기존 집중 5개, 신규 재저장 경계/대조군 2개, #6035 관련 3개: 모두 통과.
- native-skia #2225 집중 2개, fmt check, workspace all-target Clippy, WASM Clippy, workspace 빌드, manifest check: 통과.
- 실제 wasm-pack web release 빌드: 성공(2분 45초), pkg는 저장소 밖 임시 경로에 생성했다.
- 58e418821에는 이후 base의 CI·운영 문서 변경만 추가됐으며 제품 코드/fixture 보정은 없다. 중복 로컬 전체 회귀 대신 다음 exact head CI를 확인했다.

| workflow | run | 실제 결과 |
| --- | --- | --- |
| CI | [34239219541](https://github.com/edwardkim/rhwp/actions/runs/34239219541) | lint, Native Skia, archive A/B/C/D build/test, Build & Test 성공 |
| CodeQL | [34239219556](https://github.com/edwardkim/rhwp/actions/runs/34239219556) | Rust/Python/JS 분석 성공 |
| Adapter | [34239219542](https://github.com/edwardkim/rhwp/actions/runs/34239219542) | preflight 성공, worker expected skip |
| Proptest | [34239219536](https://github.com/edwardkim/rhwp/actions/runs/34239219536) | preflight 성공, worker expected skip |
| Render Diff | [34239219109](https://github.com/edwardkim/rhwp/actions/runs/34239219109) | preflight 성공, Canvas worker expected skip |

CI Impact Policy도 성공했다. skip을 실제 worker 실행 성공으로 바꾸어 기록하지 않는다.

## 시각 증적과 한계

- 절차: [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment).
- 입력: samples/issue6860/3067979-road-lighting-photometric-hancom2020.hwpx.
- 입력 SHA-256: 028c3d318e121120ed4b8a71ba11bbb343d724da3563d4133bd354ca4fb61b2a.
- PDF: pdf/3067979-road-lighting-photometric-hancom2020.pdf.
- PDF SHA-256: d2ef6b035034036f769a94a550efbfaf6a542d14846e16ad88ad3b53c1e59182.
- 원 절단본: HWPX, 마지막 저장 hancom-office-2020 11.0.0.2129. 재저장본: 한컴 2022 12.0.0.4605. MCP engine 2020 사용.
- MCP HWP 저장 job c1ce73bc-f3ae-4d43-9b37-9c95b037e797, HWPX 재저장 9f451ea8-48b4-42cf-9a03-e37ebc5b3c29, PDF ce9518f4-9a55-4ec8-b124-d29c61c53128: 성공, byte/hash 확인.
- 실행: python3 scripts/visual_sweep.py --hwp samples/issue6860/3067979-road-lighting-photometric-hancom2020.hwpx --pdf pdf/3067979-road-lighting-photometric-hancom2020.pdf --rhwp-bin target/pr-review/release-test/rhwp --key pr6880-maintainer --out /tmp/rhwp-pr6880-maintainer-sweep-20260908.
- 임시 compare/overlay/review: /tmp/rhwp-pr6880-maintainer-sweep-20260908/pr6880-maintainer/{compare,overlay,review}/{compare,overlay,review}_00{1,2}.png.
- rhwp/PDF 각각 2쪽, 직접 확인 2쪽, 자동 후보 0/2.
- 평균 pixel match 91.02424%, 내용 픽셀 보조값 8.99822%; 1쪽 8.34199%, 2쪽 9.65445%.
- 사람 판정: 마지막 행 첫 줄이 PDF처럼 1쪽에 남고 2쪽에 중복되지 않는다. 표 하단 높이·글자 픽셀 차이는 남으므로 전체 fidelity 통과로 해석하지 않는다. 제목/호스트/표의 큰 절대 위치 차이는 동일 저장 상태의 비교본으로 분리했다.
- 대표 패널을 직접 열어 본문과 도구 라벨을 구분해 확인했다. [1쪽](../assets/pr_6880_maintainer_20260908/review_001.png), [2쪽](../assets/pr_6880_maintainer_20260908/review_002.png).
- 임시 SVG, JSON, 로그, 중복 PNG는 커밋하지 않았다.

## Merge 후 contributor PR comment 계획

최신 trailing head CI 및 일반 merge commit을 확인한 뒤 post_merge.md 순서로 진행한다.
[Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment),
실제 PR/devel CI, 보정 SHA, 2쪽/후보 0/2/평균 pixel 91.02424%/내용 픽셀 보조값 8.99822%와 위 한계를 적는다.
감사와 #6860 처리 사실을 함께 남기되 #6879 또는 원 문서 전체 해결을 주장하지 않는다.

원 PR과 #6860 코멘트에는 다음 두 이미지를 commit SHA 고정 raw URL로 직접 표시한다.

![1쪽](https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_6880_maintainer_20260908/review_001.png)
![2쪽](https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_6880_maintainer_20260908/review_002.png)

내용 픽셀 보조값은 높을수록 기준과 유사하고 낮을수록 위치·형태 차이가 크며, 사람 판정 정확도가 아님을 명시한다.
asset의 devel 포함과 기존 comment를 확인한 후 UTF-8 body-file로 한 번만 게시하고 API로 본문을 재조회한다.
#6901 실운영 결과와 #6904 리뷰·오늘할일은 사용자 승인된 옵션 2 후속 기록 PR에서 처리한다.
fork branch와 기본 작업공간, 공유 target/pr-review는 보존하고 이번 local review branch만 종료 시 정리한다.
