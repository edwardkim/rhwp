# PR #2174 검토 — 프론트 Phase 0 baseline freeze (postmelee, draft)

- 이슈: #2124 (#2022 umbrella / #2023 v2 계획 승인 조건 이행) / 2026-07-11
- 파일 25 (docs 19 + scripts 4 + studio deps 2) — **소스 코드 무접촉**, CI 1건 비-pass 는
  draft 전환용 무해 항목 확인.

## 1. 재현 검증 (메인테이너 로컬, WSL2)

| 검증 | 결과 |
|---|---|
| `frontend-metrics.mjs` 재실행 + `--compare` 자기대조 | **총량 3종 delta 0** (11,774 / 2,581 / 3,901 — 게시 스냅샷과 완전 일치, 환경 독립 재현 확인) |
| stale pkg 검출 → Docker fresh WASM(4m12s) → 재검증 | **binding gate 설계 의도대로 작동** — stale 에서 `getStructure` 누락 정확 검출, fresh 후 1/1 PASS |
| 계약 게이트 | extension-dist 3/3 · editor-embed 1/1 · studio build ✓ · studio 185/185 |

## 2. 안건별 판단

**① metrics schema/scope/provenance — 승인.** Rust v2.1(#2130)의 총량 지표·함수별
diff·통이동 비계상 원칙을 schema v2 로 정확히 이식 + provenance(commit/dirty/도구
hash/source clean)는 Rust 쪽보다 오히려 충실. 모집단/제외군 명문화 정합.

**② contract·WASM·font·security snapshot / gate 분류 — 승인.** 3등급(Local PASS /
Release·manual / Advisory) 분류가 v2 §3 이원화와 정합. WASM authority = repo Docker
fresh build 고정 + stale 검출 게이트는 이번 재현에서 실증. `postMessage('*')` 를
동결하지 않고 별도 security 설계로 분류한 판단 타당.

**③ SOLID 미채점 전환 — 승인.** 전체 54/100 단일 점수 폐기 → 평가 단위별 evidence
anchor + reviewer calibration 은 Rust 쪽 "산식 고정 후 측정" 교훈의 올바른 적용.

**④ merge 후 #2124 완료·#2125 착수 — 조건부 승인** (아래 수정 2건 반영 후).

## 3. 수정 요청 (경미 2건)

1. **`upstream/devel` 하드코딩** (frontend-metrics.mjs:897) — fork 환경 전제라
   본가 클론/메인테이너 환경에서 즉시 실패. `upstream/devel → origin/devel → 생략`
   fallback 권장 (Phase 0 도구의 재현성 목표와 직결).
2. **orders/20260710.md 충돌** — 본가 devel 전진분과 add/add. rebase 시 원격판 유지
   + append 로 해소 (관례).

## 4. 판단

**approve (수정 2건 반영 + draft 해제 후 merge).** merge 후 #2124 close·#2125 착수
동의. Rust 리팩토링 유보 방침(#1582, 07-11)과의 관계: 본 PR 은 리팩토링 실행이 아니라
**측정·계약 기준선**이라 방침과 상충 없음 — 오히려 프론트 쪽 "관측 인프라" 를 같은
철학으로 완성하는 작업. 단, **Phase B(대형 해체) 착수 시점**은 Rust 쪽과 동일한
경제성 검토(유입 대비 소득)를 먼저 거칠 것을 #2125 이후 조건으로 권고.
