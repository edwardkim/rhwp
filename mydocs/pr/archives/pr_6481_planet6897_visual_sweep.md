---
kind: visual-sweep-record
status: not-run
canonical: mydocs/manual/verification/visual_sweep_guide.md
last_verified: 2026-08-30
pr: 6481
source_prs: [6413, 6422, 6445, 6447, 6455, 6470, 6471, 6479]
---

# PR #6481 planet6897 통합 visual sweep 기록

## 현재 상태

이 문서는 시각 검증 계획과 미실행 상태를 기록한다. visual sweep은 아직 실행하지 않았고, 대표 PNG, 기준 PDF, pixel match, `visual_accuracy_proxy_percent`, flagged page 수는 없다. 원 PR에 포함된 before/after PNG와 golden SVG는 contributor 증적이며 통합 head 직접 시각 검증 결과가 아니다.

따라서 이 문서는 수용 증적이 아니며, #6481 또는 원 PR #6413, #6422, #6445, #6447, #6455, #6470, #6471, #6479의 merge 권고 근거로 사용할 수 없다.

## visual sweep 필요 범위

| 원 PR | issue | 사용자-visible 주장 | 필요 상태 |
|---|---:|---|---|
| #6413 | #6298 | TAC 표 좌측·우측 경계 | 필요 |
| #6422 | #6299 | wrap 행 소비와 표 행 배치 | 필요 |
| #6445 | #6312 | float anchor와 paragraph flow | 필요 |
| #6447 | #6300 | forced break와 페이지 경계 | 필요 |
| #6455 | #6442 | cell margin과 카드·페이지 배치 | 필요 |
| #6470 | #6443 | 비용 상세 열의 텍스트·폭·SVG | 필요 |
| #6471 | #6310 | Zoom image fill과 CMYK JPEG 출력 | 필요 |
| #6479 | #6465 | footer logo line placement | 필요 |

## 실행 전 준비

1. 각 regression test와 report에서 통합 head의 원본 HWP/HWPX fixture와 기준 PDF 출처를 확인한다.
2. 원본마다 `rhwp info --json`으로 format과 `lastSavedWith`를 기록하고, 기준 PDF가 없으면 저장 제품에 맞는 MCP engine으로 수신한다.
3. 원본과 기준 PDF의 SHA-256, 생성 서비스 버전, 페이지 수, 수신 경로를 기록한다. 서버 URL, IP, token, `.env.local` 내용은 기록하지 않는다.
4. [PDF/SVG visual sweep 가이드](../../manual/verification/visual_sweep_guide.md#github-merge-comment)의 compare, overlay, review PNG 절차를 통합 head에서 실행한다.
5. 대표 review PNG는 `mydocs/pr/assets/pr_6481_*` 안정 경로에 저장하고 실제 이미지를 열어 라벨·한글 glyph·overlay legend를 확인한다.

## 수용 gate

각 대상에서 command, 임시 output 경로, 페이지 수, 자동 후보 수, `pixel_match`, `visual_accuracy_proxy_percent`, 사람의 판정과 남은 차이를 기록한다. 최신 #6481 CI와 Render Diff가 성공해도 visual sweep이 이 문서에 기록되지 않으면 최종 판정은 `수용 보류`다.

GitHub push, PR update, review, comment, close, merge는 작업지시자의 별도 승인 뒤에만 수행한다.
