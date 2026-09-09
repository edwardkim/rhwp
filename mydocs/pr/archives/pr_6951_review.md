# PR #6951 통합 self-review

## 판정과 범위

- **최종 판정: 메인터너 보정 후 수용 가능.** #6938 원 head를 단독 승인한 것이 아니라 복합 차트 보정이 포함된 통합 후보의 판정이다. 최신 head CI와 작업지시자의 merge 승인은 별도 조건이다.
- PR: [#6951](https://github.com/edwardkim/rhwp/pull/6951), 작성자 `jangster77`, base `devel`, head `review/planet6897-6938-6940-20260909`.
- 작성자 self-review이며 별도 reviewer를 지정하지 않았다. #6938/#6940 원 contributor의 출처 보존 체리픽 4개에 메인터너 보정과 검토 증적을 더했다.
- 세부 검토는 [#6938 review](../pr_6938_review.md), [#6940 review](../pr_6940_review.md), [통합 구현 기록](../pr_6938_6940_review_impl.md)을 따른다.
- 관련 범위: #6922 복합 차트 의미 복원, #6872 각주 numbering/빈 장식 문자 보존. #6941 인라인 사용자 문자 잔여는 별도다. issue/원 PR을 자동 close하는 지시는 하지 않았다.

## 커밋과 리베이스

- 리베이스 전 보정/증적 commit: `d3f8dff09`.
- 동기화한 `upstream/devel`: `144c224193f5508a66a7dc374036995ec6de2738`.
- 리베이스 후 code candidate: `e0dc1bc8a92ccd5d940419765d2e51bfe0a4d697`.
- 오늘할일을 포함해 모든 commit이 충돌 없이 적용됐다. 이 candidate를 upstream 작업 branch로 push하고 Open PR을 생성했다.
- 이 문서와 PR 채번/오늘할일 기록은 같은 PR의 문서 전용 trailing commit이다. contributor fork나 기본 branch에 직접 push하지 않았다.

## 완료한 검증과 생략

- 마지막 문자열 보정 전: fmt/check, workspace build, native/WASM/workspace-all-target Clippy 3종, manifest, release-test build 성공. 전체 nextest 9,346 passed, 0 failed, 46 skipped.
- 마지막 문자열 보정 후: 원본/최소 입력 metadata probe와 CLI build 성공. 기존 한컴 PDF를 사용해 원 HWP 3쪽 및 mixed_chart.hwp/hwpx 1쪽의 대표 패널을 직접 열어 복합 차트 복원을 확인했다.
- 사용자 지시에 따라 충돌 없는 리베이스 뒤 추가 테스트를 실행하지 않았다. 이전 전체 회귀/Clippy 성공을 마지막 문자열 보정 또는 리베이스 후 전체 검증으로 소급하지 않는다.
- 최신 PR head CI 결과는 이 문서 작성 단계에서 확정하지 않는다. source/binary/input/PDF/PNG SHA-256은 #6938 review에 있으며 검증 당시 미커밋 수정본을 git HEAD만으로 식별하지 않았다.
- 로그, 임시 Rust probe, 중간 SVG/raster/JSON, 불필요한 PDF와 파생 suite는 커밋에서 제외했다. 필요한 기존 기준 PDF, 최소 HWP/HWPX와 대표 PNG만 보존했다.

## 실제 시각 판정과 잔여

| 대상 | 직접 판독 | flagged | pixel_match | visual proxy | 결론 |
| --- | --- | --- | --- | --- | --- |
| 원 HWP | 3쪽 | 0/1 | 84.18302% | 14.14239% | 빨간 선/청록 막대/좌우 축 복원 |
| mixed_chart.hwp | 1쪽 | 0/1 | 93.77858% | 20.70838% | 같은 복합 차트 의미 복원 |
| mixed_chart.hwpx | 1쪽 | 0/1 | 93.77858% | 20.70838% | 같은 복합 차트 의미 복원 |

rsvg, 96 DPI, threshold 32 조건이다. 낮은 proxy와 flagged 0을 사람의 정확도/완전 일치로 해석하지 않는다.
범례/격자/외곽선/글꼴·배치 및 원 HWP 19 SVG쪽 대 기준 PDF 18쪽 차이는 남아 있다.
#6940은 한컴 원본/왕복 68쪽 텍스트와 대표 2쪽 raster가 일치했으며 rhwp 자체 렌더 전체의 일치 주장이 아니다.

## Merge 후 contributor PR comment 계획

- 정본: [Visual Sweep GitHub merge comment](../../manual/verification/visual_sweep_guide.md#github-merge-comment).
- 위 실제 페이지·수치·사람 판정·잔여와 검증 후보 구분을 기록한다. #6940은 A 10쪽/B 13쪽 pixel 및 proxy 100%, 한컴 PDF 간 비교이며 구조 flagged는 해당 없음임을 함께 기록한다.
- #6938 대표 파일은 `mydocs/pr/assets/pr_6938_maintainer_20260909/`의 `pr6938-original-p003-review.png`, `pr6938-mixed-hwp-p001-review.png`, `pr6938-mixed-hwpx-p001-review.png`다.
- #6940 대표 파일은 `mydocs/pr/assets/pr_6938_6940_20260909/`의 `pr6940-note-a-p010-review.png`, `pr6940-note-b-p013-review.png`다.
- raw URL은 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/<위-대표-파일-경로>` 형식으로 merge SHA에 고정한다.
- 최신 head CI와 별도 merge 승인을 확인하고 실제 merge 뒤 asset이 devel에 포함된 다음에만 승인 범위의 comment를 `--body-file`로 게시한다. 게시 후 API에서 Markdown/이미지를 확인한다.
- 현재 PR 생성까지만 수행했다. merge, 원 PR/이슈 close, contributor comment, 원격 branch 정리는 미수행이다.
