# Stage 4 완료 보고 — Task M100-5769: 구역 설정 역연산화 + setter 수렴성 실증

- 일자: 2026-08-23
- 브랜치: `fix/5769-delete-inverse` @ `2d788651`
- 계획서: `mydocs/plans/task_m100_5769.md` Stage 4
- PR: https://github.com/edwardkim/rhwp/pull/5915

## 실증 결과가 계획을 정교하게 만들었다

계획서는 pageSetup·pageMargin·sectionSettings 세 곳 모두를 old/new 속성쌍으로
전환하는 것이었다. 실증(`tests/cases/issue_5769_stage4_setter_convergence.rs`,
표본 hongbo) 결과 둘로 갈라진다:

| setter | 속성쌍만 | + raw 저널 | 원인 |
|--------|---------|-----------|------|
| set_section_def | 불수렴(@1144) | **수렴(diff None)** | passthrough 소실 → IR 재구성 직렬화 |
| set_page_def / setPageMargin | 불수렴(len 잔류) | 불수렴 | 재래핑([#4956])이 한컴 line_segs 교체 |

핵심 발견: section_def의 불수렴은 **속성과 무관**하다 — 같은 값을 한 번 적용만
해도 동일 델타가 난다. 즉 raw 스트림+봉인만 되돌리면 수렴한다.

## 구현

### Rust (fc694641)
- `section_raw_journal.rs`: SectionRawCapture 저널(조각 저장소 패턴 준용).
  capture(변경 전)/restore(old 재적용 후, Some 상태 거부 전제 검증)/discard.
- wasm 바인딩 3건(captureSectionRaw 등), #2724 EXEMPT 등재 3건.
- 수렴·불수렴 단정 테스트 6종 — 불수렴 단정은 트립와이어(#5890 개선 시 경보).

### TS (2d788651)
- `SetSectionPropsCommand`: before/after SectionDef + raw 저널.
  execute=캡처→적용, undo=old 재적용→raw 복원. 스냅샷 슬롯 0.
- `applyCommandThroughRouter`(dialog-apply.ts): 커맨드 변주 공용 헬퍼 —
  실패 처리 규약 공유, "다이얼로그 직접 라우팅 금지" 원칙 유지.
- section-settings-dialog: 현재 구역 적용 → 커맨드 경로. 문서 전체(all)는
  다구역 저널 필요로 스냅샷 잔류(코드에 명시).

## 검증
- Rust: cargo check OK, 단정형 6/6 (임시 타겟 실측)
- TS: npm test 1066 pass / 0 fail, 가드(dialog-apply-standard,
  undo-layout-dialogs, mutation-routing) 18/18
- tsc: 신규 에러는 stale 바인딩 클래스 3건(CI가 WASM 직접 빌드해 해소)

## 남은 것
- 문서 전체(all) 범위의 다구역 저널 — 필요 시 별도 태스크.
- pageSetup/pageMargin 역연산화는 #5890(직렬화 충실도) 또는 조판 정합이
  전제돼야 가능 — 장기 로드맵 항목으로 이동.
