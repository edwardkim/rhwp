---
kind: report
status: active
canonical: mydocs/report/task_m100_3702/README.md
last_verified: 2026-08-01
---

# #3702 처리 기록 — 편집 --verify 내장 (#3630 P2)

## 문제

편집 → 저장 → "잘 됐겠지"는 에이전트 실패 유형 2(#3630)의 원료다. 저장물이 실제로
의도한 문서인지 확인하려면 별도 재독 호출을 스스로 조립해야 했다.

## 구현

- `edit_verify_report(doc, out_bytes, cross_format)` 단일 헬퍼 — 저장 바이트를
  즉시 재파싱해 메모리 IR와 `diff_documents` 대조, `{identical, diffCount}` 봉투 반환.
  - 재파싱 실패는 판정 불가가 아니라 실패로: `identical:false + reparseError` (저장물은 남긴다).
  - 교차 포맷 저장(hwp→hwpx 등)은 `strip_cross_format_noise` 로 포맷 고유 잡음 제거 후 판정.
- CLI 3종 `edit fill-fields / replace-text / set-cell` 에 `--verify` 플래그:
  - 봉투에 `verify:{identical,diffCount}` 동봉, **identical=false 면 봉투 출력 후 exit 3** —
    판정은 데이터, 프로세스 종료코드와 모순 없음.
  - 미요청 시 `verify:null` (기존 소비자 무해·하위호환).
- `mcp-serve` `hwp_doc_save` 에 `verify:true` 인자 — 같은 헬퍼 재사용, 세션 저장도 동일 봉투.

## 실측 (evidence.txt 원문)

- fill-fields/set-cell/hwp_doc_save 3계열 모두 `verify:{"diffCount":0,"identical":true}` 라이브 확인.
- set-cell 좌표는 하드코딩이 아니라 export-tables 재독으로 고른 실존 최상위 표(index 1) — 이
  양식은 최상위 표 index 가 0에서 시작하지 않음을 실측으로 확인(계약 테스트도 동일 방식).

## 검증

- 신규 `edit_verify_contract` 4건 green (fill 봉투-exit 정합 / 미요청 null / set-cell 수용 /
  세션 save verify 보고) · fill 7·replace 4·set-cell 5 무회귀 · clippy 0 · fmt clean.

## 남은 것

- replace-text 라이브 실측은 fill·set-cell 과 같은 경로(동일 헬퍼)라 계약 테스트로 갈음.
- identical=false 실물 재현(의도적 손상 주입)은 단위 아닌 통합 픽스처가 필요 — v2 후보.
