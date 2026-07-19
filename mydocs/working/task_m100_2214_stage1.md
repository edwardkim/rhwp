# 1단계 완료보고서 — Task M100 #2214: 진단 + 정정 + 게이트

- 이슈: #2214 / 수행계획서: `task_m100_2214.md` (승인됨) / 브랜치: `local/task2214`
- 작성일: 2026-07-13

## 진단 — 근인 확정 (계획의 stale 경계 이분)

### 계층 귀속의 반전

수행계획서 시점의 가설(studio 캔버스 합성 층)은 **기각**됐다. E2E 계측
(신설 스크립트, headless Chrome)이 재현에 성공하며 결정적 단서를 냈다:

- 이슈 재현대로 `'1'` 연속 입력(숫자 run)일 때만 재현 — 이전 native 실증
  (한글 28자 일괄 삽입)이 놓친 조건은 **"편집 사이의 렌더/캐럿 조회"**였다.
- wasm `getCursorRectInCell` 자체가 deferred 상태에서 폴백값
  (84.1, 238.7 = para5 line1 시작)을 반환 → studio 가 아니라 native.

### 근인 격리 (native 재현 이분)

| 시퀀스 | rect(offset 180) |
|--------|------------------|
| 50자 연속 삽입 → 조회 | (629.7, **341.9**) 정상 |
| 30자 삽입 → **rect 조회 1회** → 20자 삽입 → 조회 | (84.1, **238.7**) 폴백 |

읽기 조회 한 번이 상태를 오염시킨다 = 캐시. **deferred 셀 편집이 호출하는
`invalidate_page_tree_cache_from` 이 #1949 셀 레이아웃 캐시(cell_units
포인터 키 메모이즈)를 비우지 않았다** (전체 무효화
`invalidate_page_tree_cache` 는 비움). 편집 사이 트리 빌드(studio 는 매
입력마다 렌더+캐럿 갱신)가 편집 전 셀 유닛을 캐시에 남기고, 이후 편집은
stale 셀로 렌더 — 줄 경계를 넘은 새 줄이 화면·캐럿에서 소실된다.

증상 전부가 이 근인으로 설명된다: Enter 복원(=full-edit flush 가 전체
무효화 경유), 줄 경계 이후에만 발생(줄 수 불변 편집은 stale 셀과 시각
차이가 줄 내부라 잘 안 보임), 800ms/1.5s 후에도 미표시(재렌더가 반복돼도
같은 stale 캐시).

## 정정 (1줄 + 주석)

`src/document_core/queries/rendering.rs` —
`invalidate_page_tree_cache_from` 에 `layout_engine.clear_layout_caches()`
추가. 포인터 키 캐시라 페이지 부분 무효화가 불가능해 전체 clear.
#1949 캐시의 목적(한 번의 전체 렌더 안에서 O(pages×cell) 방지)은 렌더
패스 내 재적재로 유지된다. **매 입력 전체 pagination 우회 아님** (이슈
완료 조건 준수) — deferred 경로 구조는 그대로.

## 검증

### 표적 테스트 (수정 전 FAILED 실증)

- **native**: `tests/issue_2214_deferred_cell_edit_stale_cache.rs` (2건) —
  "30자 → rect 조회 → 20자 → rect" 시퀀스에서 캐럿이 line5(y≈341.9)로
  와야 함 (수정 전: 폴백 238.7 FAILED 실증) + 페이지 0 트리에
  line5(char_start=129) run 존재 + #2185 핀(115쪽) 가드.
- **browser E2E**: `rhwp-studio/e2e/issue-2214-linewrap-display.test.mjs` —
  이슈 재현 절차 그대로(클릭→End→'1'×70) 새 줄 영역 잉크가 **2 rAF 안**
  표시 + 800ms/1.5s 유지 + 캐럿 y=341.9. 수정 전 실측: 잉크 무변화(71) +
  캐럿 폴백 → 수정 후: 입력 직후부터 잉크 2311, 전 시점 유지. 스크린샷
  실증 — 새 줄이 Enter 없이 표시되고 1.1.2 문단이 정상 시프트.

### 게이트

| 게이트 | 결과 |
|--------|------|
| fmt / clippy (release-test all-targets) | 통과 / 0 |
| 전수 `--tests --no-fail-fast` | **3065 passed / 0 failed** |
| OVR 5샘플 (±2px, 분리 폴더 `output/poc/issue2214/`) | 회귀 0건 |
| studio tsc / npm test | 클린 / 206/0 |
| WASM 빌드 (docker) | 성공 (E2E 는 새 wasm 으로 검증) |

## 처리 방침 (작업지시자 결정 2026-07-13)

postmelee 가 동일 근인(cold/warm 셀 캐시) 가설로 교차검증을 진행 중임이
확인되어 **본 정정은 보류**한다 (2안). `local/task2214` 브랜치는 유지하고
postmelee 의 교차검증·PR 을 기다린다. 격리 실증과 후보 정정 위치는 이슈
코멘트로 공유 완료. PR 도착 시 본 브랜치의 정정·테스트와 대조 검토한다.

## 최종 처분 (2026-07-17)

PR #2241(postmelee) merge 로 **supersede** — 표적 coherence 가 정확성
동치(본 진단의 조회-오염 프로브 이식 2/2) + 성능 보존으로 우월 판정.
`local/task2214` 브랜치 폐기, 본 진단 기록(근인 격리·E2E 계측)은 이슈
코멘트와 본 문서로 보존.
