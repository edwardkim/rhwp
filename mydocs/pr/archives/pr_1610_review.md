# PR #1610 처리 보고서 — is_hwp3_origin 오탐지 제거 (-1쪽 갭 요인 A, cherry-pick 통합)

- PR: https://github.com/edwardkim/rhwp/pull/1610
- 제목: `Task #1608: is_hwp3_origin 오탐지 제거 (네이티브 HWPX 부당 HWP3 tolerance 차단)`
- 작성자: planet6897 (collaborator)
- 연결: Closes #1608 (=Task #1600 -1쪽 갭 요인 A), #1609 위 스택
- base ← head: `devel` ← `planet6897:pr/devel-1608-squash`
- 처리일: 2026-06-28

## 1. 처리 결정

**cherry-pick 통합 후 PR close.** -1쪽 갭 요인 A — `is_hwp3_origin` 오탐지로 모던 HWPX 가
부당 tolerance(+21px)를 받아 한글보다 1쪽 적게 렌더되던 문제를 정정한다. #1609 머지 후 본 PR 이
orders/문서 add/add 충돌로 CONFLICTING(소스 무충돌)이 되어, #1608 신규 단일 커밋만 통합한다.

## 2. 충돌

#1608 신규 커밋(`33680b1f`)은 단일 커밋이며 #1609 스택 중복 없음. 충돌은
`mydocs/tech/investigations/issue-1600/render_minus1_page_gap.md` add/add 1건(요인 A 해소 섹션)뿐. 소스(parser/test)는
충돌 없이 적용. → #1608 쪽(요인 A 해소 섹션 추가) 채택으로 해소.

## 3. 통합 내용 (devel 위 cherry-pick 1커밋, 작성자 보존)

| 파일 | 내용 |
|---|---|
| `src/parser/hwpx/mod.rs` | `is_hwp3_origin = (head version=="1.4")` 판정 + tolerance 부여 블록 제거. `hwpml_version` 무손실 보존은 유지 |
| `src/parser/hwpx/header.rs` | `parse_hwpx_hwpml_version` 오버핏 docstring 정정 |
| `tests/issue_1608_hwpx_native_no_hwp3_tolerance.rs` | 네이티브 HWPX tolerance==0 가드 (RED→GREEN) |

핵심: HWPML head version "1.4"는 **스키마 버전**일 뿐 HWP3→HWPX 변환 지표가 아니다. 네이티브
한글2022 HWPX(version.xml major=5 minor=1 "Hancom Office Hangul")도 head 1.4 라 거의 모든
모던 HWPX 가 HWP3-origin 으로 오탐지되어 부당 tolerance(1600 HU)를 받았다. 진짜 변환본과
네이티브의 메타데이터 판별자가 없어(조사 확정), 파싱 시점 tolerance 부여를 제거(방향 3).

## 4. 검증 (로컬)

| 항목 | 결과 |
|---|---|
| GitHub CI (Build&Test/CodeQL/Analyze/Canvas visual diff) | 전부 pass |
| 충돌 시뮬레이션 | 문서 1건(해소) |
| 신규 `issue_1608_hwpx_native_no_hwp3_tolerance` | 통과 |
| `hwpx_roundtrip_baseline` / `visual_roundtrip_baseline` | 4/4 / 3/3 (HWP3 변환본 16→16 무변동) |
| 전체 `cargo test --tests` | **FAILED 0건** (lib 1975 passed) |
| fmt / clippy | clean |

## 5. 시각 판정 주의

통제셋 일치 60→66(net +6), −1쪽 29→21(8 해소)은 컨트리뷰터 한글 오라클(`render_page_gate.py`,
로컬 root) 측정이다. 로컬에선 baseline 게이트로 회귀 없음만 확인했고, 페이지 정합 권위는
작업지시자 환경(`feedback_self_verification_not_hancom`). PR 이 회귀 2건(36395325·36382819,
부당 tolerance 가 우연히 맞추던 네이티브 = 요인 B 잔존, net +6 우세로 수용)을 정직하게 명시.

## 6. 후속

- 요인 B(footer 콘텐츠 ~60px 누적 부족, 21건)는 본 이슈 범위 밖 — 별도 layout-fidelity 조사.

## 7. 산출물

- 본 처리 보고서: `mydocs/pr/archives/pr_1610_review.md`
