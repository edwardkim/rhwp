# 구현 계획서 v2 — Task M100 #2214 CI 통합 보완

## 1. 배경과 범위

- 대상 PR: [#2241](https://github.com/edwardkim/rhwp/pull/2241)
- 회귀 유입: [#2195](https://github.com/edwardkim/rhwp/issues/2195) 병합 뒤
  `render_normalized`가 중첩 표 문서를 렌더 전용 복사본으로 보관하면서 deferred 편집의
  원본 문단과 시간적으로 분리됨
- 실패 계약: `issue2214_scoped_cache_coherence_preserves_transient_pagination`
  에서 원본 text는 174자지만 transient page tree는 편집 전 130자에 머묾
- 유지 계약: #2195의 NO_LS 실폭 조판, 셀 padding, 비-TAC 중첩 표 stretch와 관련 페이지
  정합 게이트는 되돌리지 않음

이번 보완은 전체 pagination을 매 입력 수행하지 않고 편집된 상위 문단의
`render_normalized` 파생 상태만 다시 만들고 관련 pointer-key cache만 무효화한다.
일반 paginator 및 render-normalized overlay 재설계는 후속 이슈로 분리한다.

## 2. 구현 단계

### Stage 1. 파생 상태 scoped coherence

1. 셀 편집 뒤 해당 섹션에 `render_normalized`가 존재할 때 편집된 상위 문단을 원본에서
   다시 복제한다.
2. 기존 정규화와 동일한 floating-stack 및 nested-table stretch 변환을 새 문단에 적용한다.
3. 교체 전 normalized owner table의 edited cell cache를 기존 #2214 owner-flag 규칙으로
   무효화하고 새 문단의 pointer key는 cold 상태로 둔다.
4. top-level composed가 영향을 받는 경우 해당 entry만 다시 구성한다.
5. full pagination과 전역 layout cache clear는 호출하지 않는다.

### Stage 2. 회귀 계약 보정

1. 44번째 상대 flow 경계와 #2195 이후 pagination 결과를 함께 고정한다.
   - transient/flush tree max 모두 174
   - page 0 cut/bounds는 `37 / 945.9`로 유지
   - full flush는 page 2 이후 continuation cut-chain을 재정렬
2. `cellFlowChanged=true`의 보수적 1회 flush 계약을 유지하고 첫 페이지의 cut 증가를
   경계 필수조건으로 사용하지 않는다.
3. HWP/HWPX 모두 normalized 복사본이 활성화된 상태에서 원본·tree·cursor가 일치해야 한다.

### Stage 3. 통합 검증과 PR 반영

1. 실패 단일 테스트와 #2214 crate-internal cache 테스트를 실행한다.
2. #2195 관련 페이지·조판 게이트와 전체 default-feature tests를 실행한다.
3. Rustfmt check, Clippy, Studio unit/build 및 #2214 E2E를 가능한 범위에서 실행한다.
4. 결과를 최종 보고서에 보정하고 PR 코멘트로 원인·수정·검증·후속 설계를 공유한다.

## 3. 중단 조건

- normalized 문단만 갱신해도 unrelated table cache를 보존할 수 없음
- 매 입력 전체 섹션 복제 또는 전체 pagination이 필요함
- #2195의 페이지 수·시각 게이트를 깨야만 #2214가 통과함
- pointer-key cache의 owner 관계를 안전하게 식별할 수 없어 전역 clear가 필요함

중단 조건이 발생하면 production 수정을 확대하지 않고 render-normalized revision/overlay
설계를 별도 이슈로 먼저 분리한다.
