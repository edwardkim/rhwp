# PR #8359 검토 — fix(layout): 기본 탭 자리를 구역 정의의 기본 탭 간격으로 정한다

## 최종 판정

**머지 보류 — 검증 진행 중.** 동일 입력의 최신 base/누적 후보 실행과 필수 검증을 완료한 뒤 판정을 갱신한다. 이 잠정 상태를 기여자 재작업 결론이나 새 회귀 검출로 사용하지 않는다.

## 접수와 범위

- 원 PR: https://github.com/edwardkim/rhwp/pull/8359, 작성자 `semanticist21`, Open·non-draft, base `devel`.
- 원 head: `2587dd58dac9a8bf312d7cd2f59b37e803d1a387`, reviewer `edwardkim`, 지정 시각 `2026-11-05T00:27:00.198412+09:00`; API 재확인 완료.
- 실제 검증 base: `22947f6a90c998e95fa51238d5b907a0ebd37162`. 충돌 simulation: CONFLICT (content): Merge conflict in src/renderer/style_resolver.rs.
- [공통 수행 계획](semanticist21_20261105_review_plan.md). CI 참고값: 동일 원 head의 제출된 check에 failure/cancelled 없음.
- 원 contributor의 주장과 독립 측정·메인터너 보정 결과는 별도로 기록한다. 조판 원칙·입력 commit·전쪽 Native/fresh WASM·실행/반례·필수 로컬 gate는 현재 검토 중이며 아직 완료로 판정하지 않는다.

## 다중 구역 반례와 메인터너 보정

- 위반 계약: `SectionDef.default_tab_spacing`은 현재 구역의 기본 탭 간격이며 공유 `ParaShape`의 문서 전역 속성이 아니다. HWPX `tabStop=8000`/`tabStopVal=4000`과 한컴 Mac 12.30의 16000 대조군(80pt)을 근거로 한다.
- 고정 base Native에서 두 번째 구역(8000)의 탭 간격은 53.3px였다. 원 기여의 첫 구역 전역 덮어쓰기를 통합한 진단 후보 `fd220910efe3e77aa311d8dad90af971c8b23d0f`에서는 첫 구역 16000과 두 번째 구역 8000 모두 106.7px가 되어 두 번째 구역에 회귀가 생겼다. 공개 `DocumentCore::set_section_def_native`·`get_cursor_rect_native`를 실제 호출한 결과이며 합성 한컴 Print 결과로 기록하지 않는다.
- 보정은 원본 구역값과 공유 문단 모양을 보존하고, 구역 composition → pagination 측정 → 페이지 배치에 동일한 scoped 폭을 전달한다. 단일 문단 편집·삽입도 현재 구역 범위로 재조판한다. TextStyle과 줄 채움은 같은 해소값을 소비한다.
- 구역 기본 폭이 달라지면 LayoutEngine의 셀·중첩 표 캐시를 비우고, 측정/배치의 single-line overflow 캐시도 새 폭의 namespace로 전환한다. 같은 폭의 연속 페이지에서는 캐시를 보존한다. 범위 종료·오류 반환 시 이전 폭을 복원한다. `ResolvedStyleSet` 전체를 구역마다 복제하지 않는다.
- 실제 보정 후 반례·저장 재열기·Native/fresh WASM 검증은 진행 중이며 아직 통과로 판정하지 않는다.
