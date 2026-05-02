# PR #472 검토 문서

**제목**: Task #470 + #471 + #473: cross-column vpos-reset/박스 stroke_sig + 그림 crop scale 정정
**작성자**: planet6897 (Jaeook Ryu / Jaeuk Ryu)
**Base/Head**: `edwardkim/devel` ← `planet6897/local/devel`
**상태**: OPEN, mergeable
**커밋 수**: 67 (대부분 이미 cherry-pick 머지됨, 본질 신규 6 commits)

## PR 본질 분석

PR 제목은 "Task #470 + #471 + #473" 으로 3 Task 를 명시. 67 commits 중 메인테이너가 이미 cherry-pick 머지한 commits 다수 — 본질 신규 commits 만 추출.

### 신규 본질 commits (6개)

| Commit | Task | 영역 | 변경 |
|--------|------|------|------|
| `7e632e1` | **Task #470** | 다단 cross-paragraph vpos-reset 검출 완화 | `typeset.rs:415, 439` (다단/단일 단 분기) |
| `64cb32d` | **Task #471** | cross-column 검출 stroke_sig 비교 | `layout.rs:1670-1699` (bf_id → stroke_sig) |
| `8f5a079` | **Task #473** | 그림 crop 변환 scale 기준 정정 (75 HU/px) | `svg.rs:2385-2404` |
| `c9a919a`, `f3e26af`, `8e39f4a` | (머지 commits) | — | — |

## Task별 정밀 분석

### Task #470 — 다단 cross-paragraph vpos-reset 검출 완화

**의도**: Task #321 cross-paragraph vpos-reset 가드가 `cv == 0` 만 인정 → 컬럼 헤더 오프셋 (cv=9014 등) 으로 시작하는 다단의 새 컬럼 reset 미감지.

**케이스**: `samples/21_언어_기출_편집가능본.hwp` 1p pi=10 ("적합성 검증이란…") 이 col 0 에 강제 삽입되어 56.2px overflow.

**정정**:
- 다단 (col_count > 1): `cv < pv && pv > 5000` (HWPUNIT vpos 단조 증가 가정)
- 단일 단: 기존 `cv == 0 && pv > 5000` 유지 (issue_418 partial-table split 회귀 차단)

**검증**:
- 21_언어_기출 OVERFLOW 13 → 10 ✓
- exam_science 5 → 0, exam_social 4 → 1 (총 -11건 추가 해소)
- hwpspec(단일 단), exam_kor, exam_eng 회귀 0

**본 #431 (synam-001 15페이지) 와의 연관**: 본 결함이 **PR #424 (Task #412) 의 vpos 보정 변경 후 회귀**. Task #470 가 cross-paragraph vpos-reset 영역의 후속 정정 — **본 #431 결함 정정 가능성 있음** (검증 필요).

### Task #471 — cross-column 검출 stroke_sig 비교

**의도**: Task #468 cross-column 박스 인접 검출이 bf_id 동등 비교로 동작했으나, 머지(Task #321 v6) 는 stroke_sig 기준 → bf_id 다르더라도 visual 동일하면 한 그룹.

**케이스**: 21_언어_기출 1p 좌측 단 (가) 박스가 pi=6(bf=7) + pi=7~9(bf=4) 머지로 g.0=7. composed[10].bf=4 비교로 4 != 7 → partial_end 미설정 → 4면 stroke 단일 Rectangle → 하단 가로선 발생.

**정정**: `bf_id` 비교를 `stroke_sig` 비교로 변경.

**의존성**: Task #470 적용 후 노출된 회귀 정정 — Task #470 와 묶여서만 의미.

### Task #473 — 그림 crop 변환 scale 기준 정정 ⚠️

**의도**: `compute_image_crop_src` 가 `original_size_hu` 사용 시 일부 케이스 결함. 21_언어_기출 12p `<보기>` 표 내부 그림이 orig=12 HU/px (vs 75) 로 viewBox 산출 어긋남.

**정정 방식**: `original_size_hu / img_px` 가 75 ± 5% 안일 때만 orig 사용 (역호환), 아니면 75 HU/px fallback.

⚠️ **본 작업 사이클의 Task #477 와 동일 영역 — 충돌 발생**:

| 정정 방식 | 본 Task #477 (이미 머지) | Task #473 (PR #472) |
|----------|-------------------------|---------------------|
| 룰 | 항상 75 HU/px (단일 룰) | 75 ± 5% 안일 때만 orig, 아니면 75 fallback |
| 분기 | 없음 (가장 단순) | 2 케이스 분기 |
| `original_size_hu` 인자 | 사용 안 함 | 사용 (5% 안일 때) |
| 작업지시자 통찰 | "이건 휴리스틱이 아닙니다. 룰입니다." | (분기 룰) |
| 단위 테스트 | exam_kor (75 HU/px), kwater pi=31 (신규) | exam_kor 보존, 가상 입력 갱신 |

**메인테이너 정정 (Task #477) 이 더 단순한 룰** — 작업지시자가 "이건 룰이다" 통찰로 결정. Task #473 의 분기 정정은 충돌이며, 본 #477 룰이 더 정합.

## 영역 충돌 점검

본 사이클 누적 정정과의 영역:

| 영역 | 본 작업 사이클 | PR #472 |
|------|--------------|---------|
| `svg.rs::compute_image_crop_src` | Task #477 (75 HU/px 단일 룰) | Task #473 (분기 룰) — **충돌** |
| `typeset.rs` cross-paragraph vpos-reset | (변경 없음) | Task #470 — 신규 정정 |
| `layout.rs` cross-column 박스 | (변경 없음) | Task #471 — 신규 정정 |

## 처리 옵션

### A. Task #470 + #471 만 cherry-pick (Task #473 제외)

본 작업 사이클의 Task #477 정정 (단일 룰) 보존. Task #473 의 정정 의도는 이미 해결됨.

**장점**:
- 본 #477 룰 보존 (작업지시자 통찰 정합)
- Task #470/#471 은 새 정정으로 흡수 — 본 #431 결함 정정 가능성도 점검

**단점**:
- 컨트리뷰터의 Task #473 정정 노력 흡수 안 됨 — 명시적 댓글 안내 필요

### B. PR #472 전체 머지 (Task #473 포함)

**문제점**:
- 본 Task #477 (단일 룰) 가 Task #473 (분기 룰) 로 회귀
- 작업지시자 통찰 ("이건 룰이다") 무시
- 권장 안 아님

### C. Task #470 + #471 cherry-pick + Task #473 의 단위 테스트만 흡수

**장점**:
- 본 #477 정정 보존
- Task #473 의 회귀 검증 (21_언어_기출 12p) 단위 테스트는 유익

**단점**:
- 작업 복잡도 증가

## 권장 처리

**옵션 A** — Task #470 + #471 만 cherry-pick. Task #473 은 본 #477 정정으로 이미 해결됨을 PR 댓글에서 안내.

## 본 #431 (synam-001 15페이지) 연관 점검

본 #431 결함의 회귀 origin: PR #424 (Task #412) - cross-paragraph vpos-reset 가드 변경.

Task #470 의 정정이 cross-paragraph vpos-reset 영역의 후속 정정 → **PR #472 의 Task #470 cherry-pick 후 본 #431 결함 잔존 여부 점검 필수**. 정정이 전 자동 해결되면 본 #431 작업 불필요.

## 다음 단계

1. 작업지시자 옵션 결정 승인
2. Task #470 + #471 cherry-pick (옵션 A)
3. 검증 게이트 (cargo test, svg_snapshot, issue_418, clippy, WASM 빌드)
4. 본 #431 결함 재현 점검 — synam-001.hwp 15-16페이지 dump-pages + SVG 추출
5. PR #472 결과 보고서 + 댓글 + close
