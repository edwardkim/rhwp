# PR #7161 검토 — 저장 줄의 표·그림 소유 및 #6701 해소 확인

## 접수와 범위

- PR: [#7161](https://github.com/edwardkim/rhwp/pull/7161), 작성자 `jangster77`의 self-review. reviewer를 별도로 지정하지 않았다.
- base route: `collaborator_self_merge.md`.
- modifiers: `intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`, `rework_and_exceptions.md`, `review_only_fast_pass.md`, `post_merge.md`.
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`와 위 기본·보조 문서를 읽었다.
- 검토 head: `68af356c36147e3a4c38c6e2caef52157e5958bd`. 제품 코드 `a95bae7fe`, 뒤 commit은 #6701 재검증 자료다.
- 제출 base `769582fc8`; trailing 준비 시 최신 base `0c9e28a48`. base 전진은 #7152 Studio 영어 메뉴·툴바 및 그 증적이며 renderer/Rust/Cargo 변경이 아니다. source를 반복 merge/rebase하지 않는다.
- 작성 시점 참고값: OPEN, non-draft, devel 대상, MERGEABLE/CLEAN. 2 commits / 24 files / +1,766 −12. 대부분 증적 JSON이며 제품 Rust 4파일과 test 1파일이다. 대형 PR 경로의 코드·시각 검토와 merge simulation을 적용한다.
- [#6706](https://github.com/edwardkim/rhwp/issues/6706)은 신규 코드 수정, [#6701](https://github.com/edwardkim/rhwp/issues/6701)은 기존 해소 재검증이다. 다른 열린 이슈는 종료 범위가 아니다.

## 검증과 시각 판정

[코드 CI](https://github.com/edwardkim/rhwp/actions/runs/34948644100)의 head가 위 검토 SHA임을 API로 확인했다.
Full Build & Test, lint, Native Skia, Archive A/B/C/D가 success이고 CodeQL, Render Diff, Adapter inter-diff,
Proptest도 완료됐다. GitHub CodeQL 종합 neutral은 완료 상태이며 실패가 아니다.
[정확한 job URL·완료 결과·시각 지표](../assets/pr7161_code_ci_visual.json)를 보존했다.

로컬에서는 집중 102/102, 전체 nextest 9,885/9,885(51 skip), native·WASM32·workspace Clippy,
Native Skia lib 4,112 및 그림 2/PDF 4, fresh WASM을 통과했다. [#6706 분석·명령·출처](../../working/task_m100_6706_stage1.md)의
source hash가 제출 commit과 일치함을 확인했다. 이번 trailing은 문서만 변경하므로 같은 회귀를 다시 실행하지 않았다.

#6706 native/WASM 18쪽을 직접 비교했고 이번 최종 검토에서도
[수정 후/한컴 비교 PNG](../assets/issue6706/p018_after_pdf.png)를 열었다. 표 제목 x=107.1px,
그림 (108.1,340.2)px로 같은 쪽에서 순서·유일 소유·그림 전체 표시·뒤 제목을 보존한다.
64쪽 중 18쪽만 바뀌며 다른 63쪽 SVG/tree와 OVR5 142쪽 tree는 전후 동일하다.
#6701 [현재 26쪽](../assets/issue6701/current_p026.png)도 다시 열었다. 과거 +32.76px까지 누적된
흐름 오차가 마지막 본문 0.00px로 해소됐다. #6706 전후 synam 35쪽 SVG는 동일하므로 신규 수정 성과로 계산하지 않는다.

| Visual Sweep | 직접 확인 페이지 | 구조 후보 | pixel match | 내용 픽셀 일치율 보조값 |
|---|---|---|---:|---:|
| #6706 최종 native | 18 | 0/1 | 76.77876% | 29.36386% |
| #6701 현재 native | 25–27 | 0/3 | 평균 90.16914% | 평균 46.59265% |
| #6701 현재 26쪽 | 26 | 0/1 | 96.54723% | 76.02539% |

수치는 높을수록 raster가 비슷한 자동 보조값이며 사람 판정 정확도가 아니다. #6706 그림의 색상·기존 글꼴과
테두리, #6701 셀 내부 약 −1.8px 및 기관명 자형·굵기, 앞뒤 쪽 위치·테두리 차이는 남는다.
구조 후보 0건이나 전체 회귀 통과를 문서 전체의 PDF 완전 일치로 해석하지 않는다.

## 공통 조판 원칙 준수

| 검토 항목 | 확인한 코드·증거 | 판정 |
|---|---|---|
| 구현 근거와 일반성 | 원시 UTF-16 위치 0/8/18이 가시 0/0/2로 겹쳐 원 소유 줄을 잃음. `composer.rs:1532`에서 유효한 저장 줄과 실제 충돌을 검사하며 문서 ID 분기나 그림 clamp 추가 없음. 독립 기준은 기존 한컴 PDF. | 충족 |
| 측정·배치 일관성 | 같은 `stored_tac_line_assignment`를 `typeset.rs:2257`의 높이 측정과 `paragraph_layout.rs:4502`의 줄별 TAC 필터/정렬 폭 및 `:7018` run 방출이 소비함. native/WASM 결과 동일. | 충족 |
| 분할·이어받기 계약 | 컷·rowspan·fragment 배분 알고리즘 변경 없음. 소유 줄과 그 높이 소비만 변경하며 뒤 제목과 나머지 63쪽 보존 확인. 새 분할 컷 경계 주장 없음. | 비해당 |
| 줄 소속과 점유 높이 | 저장 줄 수·높이·너비·시작 불일치 및 계산 줄은 기존 경로. `equation_tac_flow.rs:26` 수식 전용 재조판 제외. 본문 column_start가 자체 여백인 경우 중복 적용을 제거하고 셀 좌표와 구분. | 충족 |
| 사례와 독립성 | DocumentCore::from_bytes → build_page_render_tree의 #6706 실물 64쪽/18쪽/표·그림 유일 소유/뒤 제목 검사. #6754 같은 줄 그림+표, #5727 뒤 줄 TAC, #7103·#1139 수식/그림 등 집중 102 통과. 좌표 기대값은 PDF에서 산출. | 충족 |
| 기준값 변경 | 기존 golden·baseline·허용치 수정 없음. | 비해당 |
| 주장과 검증 범위 | source·입력 hash/로그와 직접 PNG 확인. #6701 기존 해소, 남은 font/color 차이와 성능 미측정을 구분. | 충족 |

호출 경로 변경의 동작 검증은 새 #6706 integration test의 실제 DocumentCore 입력·렌더 경로와 위 기존
반례로 확인했다. 단순 helper 반환값 테스트로 대체하지 않았으며, 소유 복원 조건 밖의 기존 재조판을 유지했다.

## 검증 입력 커밋 확인

판정: **충족**. 아래 실제 실행 파일과 검토 commit `68af356c36147e3a4c38c6e2caef52157e5958bd`의 blob이
byte-identical임을 확인했다. 원본·PDF는 이미 Git에 있는 파일을 재사용했고 복사·재명명하지 않았다.
PDF 생성 버전과 전체 검증 해시는 [#6706 provenance](../assets/issue6706/visual_provenance.json) 및
[#6701 provenance](../assets/issue6701/verification.json)에 있다.

| 경로 | SHA-256 |
|---|---|
| `samples/hwp3-sample16-hwp5.hwpx` | `49e3e809eb41e22b2c059383db32b0cf038787269b5c523d1ff59d1a52b4340c` |
| `pdf/hwp3-sample16-hwp5-2022.pdf` | `b246ed9ac7050afd099f297f4ed489ea7fdd6fa38eb4340ece9676cf0fe75e3a` |
| `samples/synam-001.hwp` | `1dce9356ec316407b6c684d5a11190a44bb26da643a7749626763e781ab0c13b` |
| `pdf/synam-001-2022.pdf` | `2f430884f916f00e65796beeb524b65b0f0c4aac48c6283431c40a97a2325fc8` |

## 최종 판정

**승인**. 검토한 범위의 로컬·Full CI·직접 시각 증거를 충족하며 새 blocker는 없다.
사용자가 CI 완료 후 후속 처리를 승인했다. review·오늘할일·완료 CI 증적만 single-parent trailing commit으로
범위의 로컬 merge simulation을 수행해 최신 base와 충돌 없이 exit 0, merge tree 공백 검사·변경 Markdown 2개 링크 검사·기존 오늘할일 보존을 확인했다. 최종 push 직전에 고정 base/head를 다시 대조한다.
최신 trailing head의 required checks와 실제 review-only 재사용, MERGEABLE/CLEAN을 다시 확인한 뒤 정상 merge한다.
이 문서는 GitHub self-approve event를 대신 생성하지 않는다. merge SHA·실제 issue close·duration 결과는 후속 comment로 남긴다.

## Merge 후 contributor PR comment 계획

1. 이 self PR의 병합과 [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 Markdown direct link로 게시한다.
2. 실제 merge SHA에 고정한 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/issue6706/p018_after_pdf.png`와 `.../issue6701/current_p026.png`를 이미지로 포함한다. source에서 검증한 asset이 devel merge blob에 있는지 먼저 확인한다.
3. 위 18쪽/26쪽 후보 수·pixel match·내용 픽셀 일치율과 사람이 확인한 위치/소유 개선 및 남은 색상·폰트 차이를 함께 적는다. 자동 보조값의 높고 낮음과 사람 판정과의 차이를 설명한다.
4. 최신 trailing CI의 candidate 재사용/aggregate, merge SHA, #6706 신규 수정/#6701 기존 해소의 종료 상태를 적는다. 각 이슈에도 같은 범위의 별도 후속 설명을 남긴다.
5. UTF-8 Markdown 파일을 `--body-file`로 게시하고 API에서 내용·permalink를 재조회한다. 중복 comment가 있으면 추가하지 않는다.
6. 병합 후 `Refresh nextest target duration data`만 확인하고 검증 CI를 시작하지 않는다. devel을 fast-forward한 뒤 이번 임시 upstream/local branch와 `/Users/tsjang/rhwp-issue-6706`, 전용 `target/issue-6706-20260915`를 소유·미사용 확인 후 정리한다. 기본 checkout의 누적 후보·dirty 변경과 공유 target은 보존한다.
