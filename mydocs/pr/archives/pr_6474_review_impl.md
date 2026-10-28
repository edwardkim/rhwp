---
kind: pr-review-implementation
status: active
pr: 6474
source-pr: 6415
---

# PR #6474 구현 검토 - #6415 page-count Oracle 보정

## 단계별 처리

1. 최신 `upstream/devel@bd78a53122e4b532eeee330b2788cbc858dad2b0` 위에서 원 PR #6415 head를
   검토했다.
2. baseline TSV, page-count gate, gate 문서, 재생성 도구의 의도는 유지했다.
3. #6415의 pair-index 및 newest-engine 추론은 #6466의 source-relative fail-closed 계약과 충돌하므로
   적용하지 않았다.
4. 재생성 도구가 기존 `oracle_pdf_selection.py`의 `engine_for_product`와 `choose_canonical`을 직접
   사용하도록 보정했다.
5. 새 upstream branch `fix/6415-oracle-pdf-selection-20260830`에 검증한 세 커밋만 push하고
   `devel` 대상 PR #6474를 생성했다.

## 수용 기준

- 원본의 저장 제품이 2024인 경우에만 2024 PDF를 선택한다.
- 저장 제품이 2022 이하이거나 메타데이터가 없으면 2020 PDF를 선택한다.
- 원본 상대 경로, 확장자, engine이 맞는 canonical PDF가 없으면 추측하지 않고 실패한다.
- HWPX 382 대 Hancom PDF 384 차이는 기존 격차로 남아 있으며, 이 PR의 수용 근거가 아니다.

## 로컬 검증

- mandatory Rust lint bundle과 manifest check를 통과했다.
- page-count partition 회귀 테스트 16개가 모두 통과했다.
- Python 문법 검사와 whitespace diff gate를 통과했다.

## 병합 전 확인

- 최신 PR head의 required CI와 Rust CodeQL이 success 또는 정책상 expected skip인지 확인한다.
- `mergeable=MERGEABLE`, `mergeStateStatus=CLEAN`을 재확인한다.
- renderer fidelity 수용 주장은 별도 직접 PDF/visual evidence 없이는 하지 않는다.
