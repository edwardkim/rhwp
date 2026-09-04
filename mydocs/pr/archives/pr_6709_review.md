---
kind: pr-review
pr: 6709
reviewed_at: 2026-09-04
source_head: 36b5500891e750be7680c2559e2c278d4cbbe175
maintainer_correction: bc7baa359
---

# PR #6709 검토 - 용지 기준 어울림 개체의 배제 밴드

## 판정: 메인터너 보정 후 수용 가능

**보정 상태: 메인터너 보정 완료.** 원 PR `#6709`은 용지 기준으로 배치된 어울림
개체의 배제 밴드를 계산하고, 개체 이동 뒤 본문을 되감도록 renderer를 보정한다.
원 PR head `36b5500891e750be7680c2559e2c278d4cbbe175`는 현재 통합 후보에 다음 두
commit으로 체리픽되어 있다.

| 구분 | 통합 후보 commit |
| --- | --- |
| 배제 밴드 계산 및 회귀 테스트 | `e6b9a3ed5` |
| 용지 기준 세로를 밴드-로컬 좌표로 변환 | `ffd47191e` |
| 정식 fixture 등록 메인터너 보정 | `bc7baa359` |

## 검토 범위

- `src/renderer/composer/line_breaking.rs`와 `src/renderer/float_placement.rs`에서
  용지 기준 float의 exclusion band와 본문 재조판 경로를 변경한다.
- command/text editing 경로는 본문 되감기와 연동하는 최소 범위로만 수정한다.
- 회귀 테스트 `issue_6202_paper_relative_float_exclusion`가 포함되어 있다.
- 원 PR의 required `Build & Test`는 2026-09-04 조회 시 성공이었다.
  [Checks](https://github.com/edwardkim/rhwp/pull/6709/checks)

## 메인터너 보정

원 테스트는 비공개 Windows 경로와 환경 변수에 의존해 공개 CI에서 sample을 보장하지
못했다. 메인터너 보정 `bc7baa359`는 다음을 수행했다.

- 기준 HWP를 `samples/issue6202/156483689-turmeric-industry-standardization.hwp`로
  정식 등록했다.
- `MANIFEST.json`, `README.md`, `.gitattributes`로 출처, SHA-256, binary 취급을
  저장소 계약으로 만들었다.
- 테스트가 정식 sample을 반드시 읽도록 바꾸고, 개인 경로 탐색과 silent skip을
  제거했다.

## 실행한 검증

다음은 보정 뒤 현재 통합 후보에서 성공한 로컬 호환/통합 검증이다. 이는 GitHub
required CI 또는 nextest 공식 full lane을 대체한다고 표기하지 않는다.

```sh
node scripts/rust-test-suite-manifest.mjs --prepare
CARGO_TARGET_DIR=target/pr-review/green-ci-batch-20260904-full \
  cargo test --profile release-test --tests
```

## 보조 N-up sweep 기록

Hancom 2018 저장 원본은 `printMethod=4` N-up PDF로 출력돼 물리 페이지와 논리 페이지의
1:1 pixel 판정을 만들지 않는다. 아래 자료는 물리 시트의 좌우 영역을 논리 A4 크기로만
균일 변환해 구조·프레임·흐름 후보를 검사한 보조 기록이다. 이를 Hancom 정본과의 완전한
시각 동치나 Studio 직접 비교로 주장하지 않는다.

| 자료 | 경로 | SHA-256 / 결과 |
| --- | --- | --- |
| Hancom 2020 기준 PDF | `pdf/issue6202-156483689-2020.pdf` | `3154313e2bbaf793dfe2f6c505768cffb6d1097d9019fb6cf8d50d7659c701a7` |
| 논리 페이지 매핑 | `../assets/pr_6683_6705_20260904/visual-6709-6710/nup-logical-a4-normalized-page-map.json` | 8 논리 페이지 |
| 대표 contact sheet | `../assets/pr_6683_6705_20260904/visual-6709-6710/issue6202-a4-normalized-contact-sheet.png` | 8/8 완료, 규칙 후보 0 |

## 병합 전 남은 조건

1. `samples/issue6202` 원본을 `hwp2024-mcp-convert` client의 `engine 2020`으로
   변환한 기준 PDF를 확정한다.
2. 기준 PDF와 현재 `rhwp-studio`의 같은 페이지를 직접 대조해, paper-relative
   float의 exclusion band와 본문 되감기 결과를 시각 증적으로 남긴다.
3. 보정 commit을 포함한 최종 통합 PR head에서 required CI, mergeability,
   `mergeStateStatus=CLEAN`을 다시 확인한다.

위 시각 증적 조건은 추가 renderer 보정 요구가 아니다. 충족 전에는 원 PR을 직접
병합하거나 수용 완료 댓글을 남기지 않는다.
