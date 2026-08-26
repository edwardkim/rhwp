---
kind: review
status: active
source_pr: 5531
---

# PR #5531 검토

## 접수

| 항목 | 값 |
| --- | --- |
| 원본 PR | #5531 `[렌더/이미지] 텍스트 EPS/AI 아트워크를 SVG 로 옮겨 그린다 (#4062)` |
| 작성자 | `planet6897` |
| 원본 head | `b06da17b6930bef0adcfc3d8f9c95f54aede63ec` |
| 누적 체리픽 | `2d869998012a1459f4cb856d9b3a812b7cbf2566`, `34a6add579eeb9fb66fcd1199e5ea5dea488c104` |
| 관련 이슈 | #4062 |
| 검토 경로 | maintainer 일반 + 접수·리뷰 기록 + 로컬 검증 + 다수 PR 누적 + 시각·fixture 증적 |

## 변경 및 메인터너 보정

- 텍스트 EPS/Adobe Illustrator 아트워크를 제한된 연산자 집합으로 SVG로 변환하는 `src/eps.rs`를 추가한다.
- image resolver가 DOS EPS의 WMF/TIFF preview를 우선 사용하고, preview가 없으면 AI-EPS 변환 SVG를 사용하도록 단일 진입점으로 통합한다.
- SVG·HTML·web canvas와 document query 경로가 같은 변환 결과를 사용하며, 회귀 검사는 `tests/cases/eps_artwork_to_svg_contract.rs`로 이동한다.
- 원본은 선행 기능 커밋과 후속 integration-test 이동 커밋으로 구성돼 있다. 후속 커밋만 적용하면 삭제된 모듈 충돌이 발생하므로 두 커밋을 원래 순서대로 체리픽했다.

## 검증 상태

- 로컬 Rust·frontend·시각 검증: 이번 단계에서는 실행하지 않았다.
- 시각 증적: EPS/AI 입력의 기준 PDF 또는 비교 asset을 아직 수집하지 않았다.
- 최종 조건: 통합 PR CI 성공과 EPS integration 계약 실행, SVG/HTML/Canvas 결과의 시각 검증이 필요하다.

## 권고

**메인터너 적용 순서 보정 반영, 시각·통합 검증 대기.**
