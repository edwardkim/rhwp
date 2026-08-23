# Task M100 — #5769 후속 2: 구역 설정 "문서 전체(all)" 범위 역연산화

- 일자: 2026-08-23
- 브랜치: `feat/5769-zorder-inverse` (#5915 스택, 후속 1과 동일)
- 이슈: edwardkim/rhwp#5769 "다음 작업 대상 3. 구역 설정 문서 전체 범위"

## 무엇을 했나

구역 설정 다이얼로그의 적용 범위 "문서 전체" 가 스냅샷(슬롯 1)으로 기록되던
마지막 잔여 분기를 **다구역 raw 저널 속성쌍 커맨드**(슬롯 0)로 전환했다.
Stage 4 가 현재 구역만 역연산화했던 것의 완결이다.

## 설계 판정 — Rust 확장 불필요

실측(`queries/rendering.rs:2607`): `set_section_def_all_native` 은
**구역 루프의 `apply_section_def_json` + 마지막 단일 `recompose_and_paginate`**
뿐이다. 즉 all 은 독립적인 구역별 적용의 묶음이라, 저널 계약("그 구역의 캡처와
복원 사이 그 구역 편집 금지")이 구역 단위로 성립하는 한 TS 조합으로 역연산화
된다. 새 네이티브·EXEMPT 없음.

## 구현

- `SetSectionPropsAllCommand`(command.ts):
  - execute: 모든 구역 before 사전 수집(다이얼로그가 함) → **전 구역 캡처 선행**
    → `setSectionDefAll(after)` 한 번(네이티브 효율 유지). 실패 시 캡처 전량 폐기.
  - undo: 구역별 old 재적용(raw 재무효화) → 캡처 복원(캡처 순서 유지).
  - redo: execute 재실행 = 재캡처(저널 소비 계약 준수).
  - no-op: 모든 구역 before==after → 캡처·적용 없이 기록 생략(#2370).
  - type='setSectionProps' — 양식 모드 게이트는 종전 snapshot 차단과 동일하게
    default 차단(동작 변화 없음).
- 다이얼로그 'all' 분기: `applyThroughRouter(snapshot)` →
  `applyCommandThroughRouter(command)` 전환. services 미주입 fallback 유지.
- 슬롯 효과: all 범위 설정 스냅샷 1슬롯 → **0**. section-settings 의 스냅샷
  진입점은 이로써 0곳이 됐다.

## 검증

- Rust 게이트(`issue_5769_stage4_setter_convergence.rs` 확장, 7종):
  `section_def_all_multi_section_with_journal_converges_byte_exact` — 2 구역
  합성 문서에서 TS 순서 그대로(전 구역 before 수집 → 캡처×2 → all 적용 →
  구역별 old 재적용 → 복원×2) 왕복 뒤 **저장 바이트 완전 수렴** + 적용 실재
  단정. 기존 6종도 회귀 없음 — **7/7**
- clippy `-D warnings` clean · fmt --check 통과
- npm test **1072 pass / 0 fail**: 공용 헬퍼 가드 2종(dialog-apply-standard,
  undo-layout-dialogs)을 헬퍼 계열 확장에 맞게 갱신 — snapshot 과 command
  어느 쪽이든 "공용 헬퍼 경유" 자체를 핀하도록(표준화 의미 보존)

## 메모

- all 적용 시 구역별 before 가 서로 다를 수 있어 커맨드는 `[{idx, before}]`
  배열을 든다 — redo 는 after 하나면 충분하다(all 은 균일 적용).
- 남은 구역 설정 후속: pageSetup/pageMargin 은 여전히 스냅샷 잔류(#4956 재래핑
  불수렴 실증 유지, #5890 개선 시 재판정).
