---
kind: pr-review
status: accepted-pending-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-24
---

# PR #6018 review - visual_sweep 한글 라벨 폰트 fallback 보완 (#6016)

## 접수 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#6018](https://github.com/edwardkim/rhwp/pull/6018) |
| Issue | [#6016](https://github.com/edwardkim/rhwp/issues/6016) |
| 작성자 | [@jangster77](https://github.com/jangster77) |
| base | `devel` |
| head | `task_m100_6016_visual_sweep_cjk_label_font` |
| 검토 head | `9f69dafd6f37c3019e400ae96f368f5d33c7103b` |
| GitHub 상태 | `MERGEABLE`, GitHub CI 진행 중 |
| 판정 | **수용 권고 / 최신 CI 대기** |

## 변경 검토

PR #6015 후속 처리 과정에서 `mydocs/pr/assets/pr_6014_issue5712_p1_review.png`의 visual sweep
도구 라벨 중 한글 설명이 tofu 네모 박스로 표시됐다. 같은 증적의 문서 본문 한글과 기준 PDF 한글은
깨지지 않았으므로, 제품 renderer 회귀가 아니라 `scripts/visual_sweep.py`가 review/contact sheet PNG 위에
그리는 도구 라벨 폰트 선택 문제로 분리했다.

기존 `label_font()`는 macOS Arial 계열 경로만 확인하고 Linux에서는 Pillow 기본 폰트로 fallback했다.
현재 Ubuntu 서버에는 `Noto Sans CJK KR`와 `NanumGothic`이 설치되어 있었으나, fontconfig와 Linux CJK
폰트 경로를 조회하지 않아 사용할 수 없었다.

이번 보정은 다음 순서로 라벨 폰트 후보를 고른다.

- `RHWP_VISUAL_SWEEP_LABEL_FONT` 환경변수에 지정한 경로를 최우선으로 사용한다.
- `fc-match`가 있으면 `Noto Sans CJK KR`, `NanumGothic`, `UnDotum`, `Malgun Gothic`,
  `Apple SD Gothic Neo` family 후보를 조회한다.
- `platform.system()` 기준 현재 OS의 알려진 CJK 폰트 경로를 먼저 확인하고, 이후 다른 OS 후보를
  fallback으로 평가한다.
- 모든 TrueType 후보가 실패할 때만 `ImageFont.load_default()`로 fallback한다.

제품 renderer, HWP/PDF 변환, layout 알고리즘은 변경하지 않는다. 변경 범위는 visual sweep 증적 PNG의
도구 라벨 폰트 선택과, 같은 누락을 반복하지 않기 위한 PR review workflow 문서 보강에 한정된다.

## 절차 보강

누락 원인은 visual sweep 대표 PNG를 GitHub comment에 첨부하고 수치만 기록했지만, 증적 생성 도구가
덧그린 한글 라벨 자체의 판독성까지 별도 gate로 확인하는 절차가 부족했던 것이다. 따라서
`mydocs/manual/pr_review_workflow.md`와 `mydocs/manual/pr_review/visual_fixture_evidence.md`에 다음을
명시했다.

- 대표 review PNG를 실제로 열어 도구 라벨, 한글 glyph, metric text, overlay legend를 확인한다.
- 문서 본문 렌더 회귀와 visual sweep 도구 라벨 회귀를 분리해 판단한다.
- 도구 라벨 문제는 원 PR의 제품 fidelity 결함으로 오인하지 않고 별도 issue/후속 PR로 추적한다.
- `visual_accuracy_proxy_percent`는 자동 보조값이며, 사람이 읽는 시각 판정의 완전한 대체값으로 쓰지
  않는다.

## 로컬 검증

- `python3 -m unittest scripts.tests.test_visual_sweep`: 43 tests OK
- `python3 -m py_compile scripts/visual_sweep.py scripts/tests/test_visual_sweep.py`: 통과
- `python3 scripts/check_markdown_links.py mydocs/working/task_m100_6016_stage1.md mydocs/manual/pr_review_workflow.md mydocs/manual/pr_review/visual_fixture_evidence.md`: 3개 문서 링크 검사 통과
- `cargo fmt --all -- --check`: 통과
- `git diff --check`: 통과
- Ubuntu 서버에서 `configured_label_font_paths()`의 첫 기존 후보가
  `/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc`이고, `label_font()`가 `FreeTypeFont`로
  로드됨을 확인했다.

macOS와 Windows는 현재 로컬에서 실제 visual sweep 실행을 하지 않았고, OS별 후보 우선순위와 path
separator/env/fontconfig 계약을 mock 단위 테스트로 고정했다. 실제 host에서 다른 폰트 경로를 쓰는 경우에는
`RHWP_VISUAL_SWEEP_LABEL_FONT`로 명시할 수 있다.

## 권고

증적 생성 도구의 한글 라벨 판독성 문제를 제품 renderer 변경 없이 좁게 보정했고, 누락을 막는 workflow
문서 gate도 함께 강화했다. 최신 GitHub CI가 통과하면 merge 가능하다.
