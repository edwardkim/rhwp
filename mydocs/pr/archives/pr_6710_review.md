---
kind: pr-review
pr: 6710
reviewed_at: 2026-09-04
source_head: 4a1eb7c27552dd9dd619a9aef1b9aaaf997a6fdb
maintainer_correction: bc7baa359
---

# PR #6710 검토 - 저장 첫 조각의 source-frame allowance

## 판정: 메인터너 보정 후 수용 가능

**보정 상태: 메인터너 보정 완료.** 원 PR `#6710`은 저장된 첫 조각의 초과 허용치를
origin marker 유무가 아니라 source-frame 계약으로 적용하도록 `typeset` 처리를
보정한다. 원 PR head `4a1eb7c27552dd9dd619a9aef1b9aaaf997a6fdb`는 현재 통합 후보에
다음 commit으로 체리픽되어 있다.

| 구분 | 통합 후보 commit |
| --- | --- |
| source-frame allowance 보정 및 회귀 테스트 | `c340bd7a8` |
| 전·후 PNG 보고 자료 | `61cd71fb9` |
| 정식 fixture 등록 메인터너 보정 | `bc7baa359` |

## 검토 범위

- `src/renderer/typeset.rs`에서 저장 첫 조각 allowance의 적용 조건을 origin marker와
  분리한다.
- `issue_4658_ir_diff_pagecount`와
  `issue_5057_profile_agnostic_source_frame_allowance` 회귀 계약이 함께 수정됐다.
- 원 PR의 required `Build & Test`는 2026-09-04 조회 시 성공이었다.
  [Checks](https://github.com/edwardkim/rhwp/pull/6710/checks)

## 메인터너 보정

원 회귀 테스트는 비공개 Windows 경로와 환경 변수로 sample을 찾고, 자료가 없으면
성공처럼 반환했다. 메인터너 보정 `bc7baa359`로 다음을 완료했다.

- 원본을 `samples/issue5057/21484591-gimcheon-sewage-ordinance.hwp`로 정식 sample로
  등록했다.
- `MANIFEST.json`, `README.md`, `.gitattributes`로 공개 fixture 계약을 명시했다.
- 테스트가 저장소 sample을 반드시 읽도록 바꾸고 private-path 탐색과 silent skip을
  제거했다.

## 전·후 PNG의 실제 범위

`mydocs/report/5057-origin-marker-profile/before_p7.png`와 `after_p7.png`는
renderer 변경 전후를 보여 주는 보조 보고 자료다. 이 두 PNG만으로 Hancom 정본과
일치한다고 주장하지 않으며, 외부 기준 PDF를 대체하지 않는다.

| 자료 | 경로 | 의미 |
| --- | --- | --- |
| 변경 전 | `mydocs/report/5057-origin-marker-profile/before_p7.png` | 기존 renderer 출력 |
| 변경 후 | `mydocs/report/5057-origin-marker-profile/after_p7.png` | 보정 renderer 출력 |

## 실행한 검증

다음은 보정 뒤 현재 통합 후보에서 성공한 로컬 호환/통합 검증이다. GitHub required
CI 또는 nextest 공식 full lane을 실행한 것으로 표기하지 않는다.

```sh
node scripts/rust-test-suite-manifest.mjs --prepare
CARGO_TARGET_DIR=target/pr-review/green-ci-batch-20260904-full \
  cargo test --profile release-test --tests
```

## 보조 N-up sweep 기록

Hancom 2010 저장 원본의 `printMethod=4` 출력은 물리 PDF 시트와 논리 페이지가 일대일이
아니다. 아래 자료는 물리 시트의 좌우 영역을 논리 A4 크기로만 균일 변환해 구조·프레임·흐름
후보를 검사한 보조 기록이다. 이는 Hancom 정본과의 완전한 시각 동치나 Studio 직접 비교의
대체 근거가 아니다.

| 자료 | 경로 | SHA-256 / 결과 |
| --- | --- | --- |
| Hancom 2020 기준 PDF | `pdf/issue5057-21484591-2020.pdf` | `78ef349ce8936a7cfaa4e671c1ca1318e31cda3b91c303ebc86b91c1660bae54` |
| 논리 페이지 매핑 | `../assets/pr_6683_6705_20260904/visual-6709-6710/nup-logical-a4-normalized-page-map.json` | 13 논리 페이지 |
| 대표 contact sheet | `../assets/pr_6683_6705_20260904/visual-6709-6710/issue5057-a4-normalized-contact-sheet.png` | 13/13 완료, 규칙 후보 0 |

## 병합 전 남은 조건

1. `samples/issue5057` 원본을 `hwp2024-mcp-convert` client의 `engine 2020`으로
   변환한 기준 PDF를 확정한다.
2. 그 기준 PDF와 현재 `rhwp-studio`의 동일 페이지를 직접 대조해, source-frame
   allowance가 기대한 쪽 경계를 보존하는지 시각 증적으로 남긴다.
3. 보정 commit을 포함한 최종 통합 PR head에서 required CI, mergeability,
   `mergeStateStatus=CLEAN`을 재확인한다.

위 조건을 충족하기 전에는 원 PR을 직접 병합하거나 수용 완료 댓글을 남기지 않는다.
