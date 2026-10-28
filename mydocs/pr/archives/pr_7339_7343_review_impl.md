# PR #7339-#7343 통합 검토 및 메인터너 보정

## 범위와 결론

최신 `upstream/devel` `7a95e46e` 위에서 #7339, #7340, #7341, #7342, #7343의 최신 head를 다시
적용했다. #7338은 이미 `c1ac0f987`로 `devel`에 병합되어 다시 cherry-pick하지 않았다.

통합 head `a3f61dc5fe2a0b13b74239505f7d753164490c25`은 다음 두 원 PR 결함과 이미 병합된
#7338의 raw 호환성 회귀를 제한적으로 보정했다.

| 관련 PR | 원 head의 문제 | `a3f61dc` 보정 |
| --- | --- | --- |
| #7338 | semantic `pageBreak`가 이미 같은 경우에도 비표준 raw bit를 정규화 | semantic 값이 같으면 raw record bit를 보존 |
| #7339 | offset만 비교해 같은 실제 가로 위치의 정렬 셀을 비중첩으로 오판 | padding, inner width, align, margin을 반영해 실제 배치 비교 |
| #7340 | 뒤 candidate의 confirmation이 앞 cut에도 적용될 수 있음 | rewind/confirmed 길이를 함께 반환하고 현재 cut과 일치할 때만 허용 |

#7341의 최신 좁힌 gate와 #7342/#7343의 원 구현에는 추가 보정이 필요하지 않았다. 개별 판정은
[#7339](pr_7339_review.md), [#7340](pr_7340_review.md), [#7341](pr_7341_review.md),
[#7342](pr_7342_review.md), [#7343](pr_7343_review.md)를 따른다. 원 contributor PR들을 직접
merge하지 않고, 보정이 포함된 통합 head만 다음 원격 단계의 후보다.

## 완료한 검증

모든 아래 검증은 통합 head에서 완료했다.

| 항목 | 결과 |
| --- | --- |
| fmt, `git diff --check` | 통과 |
| focused regression suite 008/016/017 | 653 passed, 0 skipped |
| 전체 nextest | 10,156 passed, 50 skipped |
| native 및 wasm Clippy, workspace build, workspace all-target Clippy | 통과 |
| Rust test suite manifest | 1,404 sources, 6,025 static attrs, 48 targets, minimum 6,559 cases 통과 |
| Native Skia | lib 3,930 passed/13 ignored, 보충 2 및 4 contract tests 통과 |
| fresh no-opt WASM | 성공; `rhwp.js` `2b7e7bb01cbbff0cb0d3c9a3222c6cb187f9d9710045013bcfb077d7abef4133` |

Docker 표준 WASM은 Docker 및 `.env.docker`가 없어 실행하지 못했다. 이를 Docker 검증 성공으로
표시하지 않으며, fresh no-opt WASM은 보조 진단이다.

## 시각 검증과 asset 경계

저장소에 이미 커밋된 HWP/HWPX와 Hancom 기준 PDF만 사용했다. fresh WASM sweep의 원시 raster,
SVG, render-tree JSON, summary/metrics JSON, run manifest와 실행 로그는 검토 전용 외부 경로에만 두었다.
merge 후 PR 코멘트에 실제 표시할 최종 review PNG 6장만 `mydocs/pr/assets/`에 반영했다.

- #7339/#7340/#7343: chemical 표본 39, 40, 64, 65, 83쪽 완료, 구조 후보 0건.
- #7341: night guard 1쪽 완료, 구조 후보 0건.
- #7342: 7062 표본 5쪽 중 4쪽의 미주 gap 후보는 base와 동일; 2430 표본 17·27쪽은 후보 0건.

각 개별 review의 `Merge 후 contributor PR comment 계획`에 대표 asset, 실제 페이지와 수치, merge-SHA
고정 raw URL 형식을 기록했다. merge가 완료되어 asset이 `devel`에 존재하기 전에는 원격 코멘트를 게시하지 않는다.
