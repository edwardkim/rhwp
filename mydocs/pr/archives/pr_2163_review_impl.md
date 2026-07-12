# PR #2163 통합 재검토 실행 계획

## 검토 대상

- PR: #2163, Issue #2156
- 원 구현 커밋: `35244ab62b6883a08a7064bfc9aea1c381400612`
- 통합 cherry-pick: `a4fbea951`
- 메인터너 범위 교정: 통합 브랜치의 Stage 4 후속 커밋

## Stage 1 - P1 보완 완료

1. `haansoft_latin_override`를 검증된 함초롬바탕/HCR Batang 정확한 별칭으로 제한했다.
2. positive/negative alias 회귀 테스트를 추가했다.
3. 돋움 수치 매핑은 별도 HWP/HWPX+PDF 근거를 갖춘 후속 과제로 분리했다.

## Stage 2 - 플랫폼 검증 완료

1. native focused test와 `svg_snapshot`을 실행해 통과했다.
2. `wasm-pack build --target web --out-dir pkg`를 실행해 통과했다.
3. Studio에서 내장 메트릭 경로로 `width_ladder.hwpx`를 로드해 1쪽 렌더와 console clean을 확인했고,
   문서의 WASM 범위 설명을 실제 동작과 맞췄다.

## Stage 3 - 통합 PR 준비

1. 통합 branch diff와 기준 PDF visual sweep을 재검토했다.
2. 깨끗한 `target` 전체 검증을 통과했다.
3. 통합 PR #2170은 `c95d8fd`로 merge됐고, 최신 GitHub Actions 성공 및 #2156 auto-close를 확인했다.

## 작업지시자 확인 사항

- 함초롬돋움 별도 측정/매핑을 후속 font-fidelity 과제로 관리할지.
- 통합 PR의 전체 검증 실행과 생성·remote push 승인 여부.
