# PR #2314 검토 — 편집 시 저장 vpos 리셋 보존 (lpaiu-cs, #2299)

- 검토일: 2026-07-17 / 12파일 +784/−38 / CI 11 green / BEHIND
- 요지: `recalculate_section_vpos` 재설계 — 저장 vpos 리셋(단/쪽 경계
  인코딩) 보존 + 문단 경계 gap(셀 경로 boundary_gaps 동일 산식) +
  placeholder 저지선 2종. 편집·필드 경로 전용이라 무편집 로딩 무회귀 선언.

## 검증 (로컬 재실증)

| 게이트 | 결과 |
|--------|------|
| 신설 핀 4건 patch-revert FAILED 실증 | 재현(소스 원복 시 실패) → 복원 후 8/8 |
| 편집-스윕 (devel 기준) | **3건 변동 재현** (shortcut 7→9 · SO-SUEOP 46→45 · biz_plan 6→7) — PR 주장 정합 |
| 편집-스윕 (PR head) | 위 3건 해소 — 단 **treatise sample.hwp 7→8 신규 출현** |

## 발견 — PR 기인 편집 회귀 1건 (수정 요청)

**`samples/basic/treatise sample.hwp` (1.4MB, 각주 문서): para0 1자 삽입
시 7→8쪽** — devel 에서는 안정(7→7), PR head 에서 3회 반복 결정적 재현.
컨트리뷰터의 "80건 스윕 80/80 안정" 주장과 상충 (treatise 는 <2MB 로 스윕
범위 내였을 문서).

추정 축: 각주 문서의 저장 vpos 감소 패턴(각주 영역 좌표계)이 단/쪽 경계
인코딩으로 오인 보존되어 가짜 넘김 생성 — 보존 감지("저장 first 가 직전
저장 end 보다 감소")의 반례 후보. 정밀 귀속은 컨트리뷰터 몫으로 이관.

## 구조 검토 (발견 외 건전)

- 하드코딩 클린(샘플명은 주석 실측 서술만), 셀 경로(boundary_gaps ·
  ignore_reset_at 관례) 대칭 준수, 저지선 논리 문서화 충실.
- 기존 핀이 중간 설계 결함을 잡아 교정한 기록 — 관례 정합.
- 컨트리뷰터 스스로 요청한 **한컴 실기 대조**(편집 시 단/쪽 유지, Windows
  부재)는 작업지시자 확인 항목.

## 판단

**CHANGES_REQUESTED 1건 후 재검토** — treatise 회귀 재현 절차 공유,
80건 스윕 방법론 차이 확인 요청 포함. 접근 자체(저장 신호 보존)는 저장
지오메트리 신뢰 계보(#2112 계열)와 방향 정합.


---

## 재검토 v2 — 보강 커밋 a1a23850 (2026-07-17)

CHANGES_REQUESTED(treatise 7→8 회귀 + 스윕 방법론) 에 대한 컨트리뷰터 보강을
merged tree(devel 2e65eb95 + a1a23850) 로컬 재검증.

### treatise 쟁점 — 컨트리뷰터 반론이 옳았음 (좌표 실측 확증)

'X' 삽입 시 para0 이 1줄→2줄 실성장(end 5320→8840). 
- **PR**: para1 first 6120→**9640** — 저장 간격 800HU(문단 위 간격) 정확 보존, 겹침 없음, 8쪽 = **정당 성장**
- **devel**: para1 first 6120→**8840** — abutment 재계산으로 저장 간격 800HU **소실**(간격 압축 누적으로 7쪽 유지). 원 리뷰의 "7쪽 유지가 정답" 판단은 devel 간격 소실 버그를 기준으로 삼은 오판

### 편집-스윕 devel 대조 (574건 <2MB, 1자 삽입 전후 페이지 수)

| | devel | PR merged tree |
|---|---|---|
| 변동 | **70건** | **11건** |
| PR 에서만 변동 | — | treatise 1건 (정당 성장 확증) |
| 양쪽 공통 (기존 동작) | 10건 | 10건 |
| devel 에서만 (PR 이 해소) | 60건 | — |

devel 회귀 3건 유지 확인: shortcut 7→7 (단 밴드 보존), biz_plan 6→6, SO-SUEOP 46→46 (devel 은 46→45).

### 게이트 (merged tree)

- 신규 회귀 테스트 issue_2299 8/8 / `cargo test --tests` release-test 전 통과 / fmt / clippy 0
- CI (a1a23850): preflight·Canvas visual diff·CodeQL·Native Skia 전 pass (Frontend/WASM 은 경로 필터 skip)

### 판단

**merge 권고.** 원 리뷰의 수정 요청 2건 모두 해소 — ①treatise 는 회귀가 아니라
devel 간격 소실의 교정임이 좌표로 실증 ②스윕 방법론 차이는 devel 대조표로 해명
(devel 가짜 변동 60건 해소가 본 PR 의 실효). 한컴 실기 대조(단/쪽 유지)는 앞서
작업지시자 확인 완료.
