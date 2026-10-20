---
kind: working
status: completed
last_verified: 2026-10-09
---

# #7688 2단계 — 출력 소비 계약의 재발 방지 절차

Issue: [#7688](https://github.com/edwardkim/rhwp/issues/7688).
사용자 지시는 잘못된 동작을 고치는 범위에서 향후 같은 문제가 생기지 않는 절차·방법을 마련하는 것이다.
이번 단계는 그 절차의 로컬 문서 반영이며 제품 결함 수정·자동 CI 신설의 완료를 뜻하지 않는다.

## 반영 범위

[출력 백엔드 소비 경로 검증](../manual/verification/renderer_backend_verification.md)을 작성하고
CONTRIBUTING, 시각 검증 거버넌스, 로컬 검증 4.3, 공통 조판 review 표와 문서 지도에 연결했다.
CONTRIBUTING의 “다른 렌더링 결과(참고)”는 변경 결과를 소비하는 해당 출력 경로의 필수 확인으로 바꿨다.
모든 PR에서 모든 backend를 전수 실행하도록 확대하지 않고 실제 소비 경로로 적용 범위를 정한다.

규칙의 근거와 범위 → 공통 조판/paint 결과 → font/resource·metric context·실제 합성 소비 →
구조·paint·독립 PDF 비교 → 수정 전 FAIL/후 PASS·정상 대조군 → 기존 PR의 근거 연결을 한 절차로 묶었다.
같은 helper나 family명, readiness, 파일 생성, 높은 점수로 실제 내용 보존을 대신하지 않는다.

현재 renderer baseline, Visual Sweep, adapter/trace 검사가 입증하는 범위와 공백을 구분했다.
새 자동 검사·CI workflow·baseline·허용치·제품 source·AGENTS.md는 변경하지 않았다.
독립 시각 근거 없이 현재 잘못된 출력을 새로운 regression/golden으로 고정하지 않는다.

## 검증

- 기준 upstream/devel: `6f66932a73ee6fd05a74a83bdc438c9499a89a12`. fetch 후 동일 SHA 확인.
- 1단계 기록 commit: `689d536de`. 변경 전 worktree clean 확인.
- `git diff --check`: 통과.
- `python3 scripts/check_markdown_links.py`에 변경된 장기 문서 6개와 CONTRIBUTING을 명시:
  7개 파일 상대 링크 이상 없음. 최종 새 가이드도 다시 확인했다.
- `scripts/check_document_metadata.py`는 인수와 무관하게 전체 장기 문서를 검사하므로 exit 1이었으나,
  오류 16건의 4개 원본 파일이 기준 SHA와 byte 단위로 동일함을 확인했다.
  같은 검사 모듈의 `validate_file`로 변경된 장기 문서 6개를 검사해 오류 0건·신규 오류 0건 확인.
  ignored 증적: `output/pr-review/renderer-backend-audit-20261009/procedure-metadata-verification.json`.
- renderer baseline `--help`와 실제 CLI·capture 호출을 대조해 새 가이드의 예시 옵션을 확인했다.
- 문서만 변경했으므로 Cargo·Docker WASM·시각 출력을 다시 실행하지 않았다.
  1단계 결과는 해당 source SHA의 과거 실행 증거이며 이번 문서만으로 제품 수정 후 증거로 바꾸지 않는다.

## 다음 구현의 적용 방법

Native 수식의 font/resource 전달을 우선 보정하고, 기존 본문 선택과 별도 수식 paint가 갈리는 반례를
실제 export 경로에서 확인한다. glyph coverage와 독립 출력의 기대값을 확인해 정식 회귀로 연결한다.
공통 crop은 원본 좌표 기준을 확인한 뒤 그 규칙을 소비하는 경로를 대조한다.
각 수정은 기존 90% 선행 조건·로컬 검증·fresh WASM·필요한 실제 화면 확인을 충족한 뒤 제출한다.
관련 #536 진행 중 구현은 이 기록으로 완료 처리하지 않으며 #7688은 제품 수정·미검증이 남아 열린 상태다.
