---
PR: #817
제목: fix — 1×1 래퍼 표 shortcut 다수 중첩 표 누락 수정 (closes #726)
컨트리뷰터: @oksure (Hyunwoo Park) — 5/11 사이클 14번째 시도 (PR #815 close 후 다른 본질)
처리: 옵션 C — devel 영역 영역 Task #688 이미 해결 + byte-identical SVG + 작업지시자 결정 영역 영역 close
처리일: 2026-05-12
---

# PR #817 처리 보고서

## 1. 처리 결과

✅ **close 완료** — 옵션 C (Task #688 이미 해결 + byte-identical SVG 입증 + 컨트리뷰터 분리 PR 가이드)

| 항목 | 값 |
|------|-----|
| 처리 | close + 분리 PR (Issue #726 진짜 본질) 재제출 요청 |
| 사유 | (1) Task #688 영역 영역 이미 해결 + (2) PR 적용 SVG 영역 영역 devel HEAD 영역 영역 byte-identical + (3) Issue #726 진짜 본질 (화살표 도형 미출력) 영역 영역 본 PR 무관 |
| Issue #726 | OPEN 유지 (분리 PR 영역 영역 후속) |

## 2. 본 PR 본질

PR 본문 영역 영역:
- table-vpos-01.hwpx 5쪽 nested 11×3 그리드 SVG 완전 누락 정정
- `find_map` 영역 영역 첫 nested table 만 반환 결함
- `nested_table_count == 1` 가드 영역 영역 다수 table 시 일반 경로

## 3. 작업지시자 시각 비교 — byte-identical 입증

본 환경 영역 영역 cherry-pick + 5쪽 SVG 내보내기:

```
=== before (devel HEAD, Task #688 가드) ===
text=343  polygon=0  image=0  path=2  lines=474  size=131K

=== after (PR #817 nested_table_count == 1 가드) ===
text=343  polygon=0  image=0  path=2  lines=474  size=131K

diff exit code: 0 (byte-identical)
```

→ 두 SVG 영역 영역 완전 동일. PR 영역 영역 본 환경 영역 영역 효과 부재.

## 4. devel HEAD 영역 영역 이미 해결됨 (Task #688)

**devel HEAD** (Task #688, PR #694, commit `40ecbe26`):
```rust
if cell.paragraphs.len() == 1 {  // paragraphs 수 가드
    ...
    // + 외곽 박스 border 렌더링 추가 정합 (exam_social.hwp pi=15 등)
}
```

table-vpos-01.hwpx p.5 영역 영역 셀[0] `paras=2` → 가드 false → shortcut 우회 → 일반 경로 영역 영역 모든 nested table 렌더링.

## 5. PR base 분석

PR #817 base = `30351cdf` (5/9, Task #688 머지 전). 본 PR 작성 후 Task #688 영역 영역 같은 본질 영역 영역 다른 방식 영역 영역 먼저 머지 → 본 PR 영역 영역 중복 정정 + 외곽 박스 border 회귀 위험.

## 6. Issue #726 영역 영역 진짜 본질

**Issue #726 본문**: 4대 그룹 사이 구분 도형 2개 SVG 미출력 — 본 PR 본문 (nested 11×3 그리드 누락) 영역 영역 다른 본질.

본 환경 점검 (devel HEAD = PR 적용 후 동일):
- `<polygon>` **0개** — 화살표 도형 미출력
- `<image>` **0개**
- IR 영역 영역 1개 다각형 (셀[18]) 존재

→ 셀 안 다각형/도형 SVG 렌더링 경로 누락 영역 영역 본 PR 무관.

### 두 결함 후보
- (a) SVG renderer 다각형 미출력 (`src/renderer/` 도형 분기)
- (b) HWPX 파서 다각형 1개 누락 (`src/parser/hwpx/`)

## 7. 컨트리뷰터 안내 (정중 톤)

[#817#issuecomment-4425741327](https://github.com/edwardkim/rhwp/pull/817#issuecomment-4425741327):
- byte-identical SVG 결과 명시
- Task #688 이미 해결됨 + 외곽 박스 border 정교한 정합 안내
- Issue #726 진짜 본질 (화살표 도형) 영역 영역 분리 PR 가이드
- 두 결함 후보 (a/b) 영역 영역 진단 권장

## 8. 본 환경 reset

cherry-pick + 시각 비교 → byte-identical 확인 → `git reset --hard origin/devel` 영역 영역 devel 무영향.

## 9. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure **20+ 사이클** (5/11 14번째 시도, PR #815 close 후) |
| `feedback_image_renderer_paths_separate` 권위 사례 강화 후보 | Issue #726 진짜 본질 영역 영역 셀 안 다각형 SVG/Canvas/paint json 4 backend 동기 정정 후속 |
| `feedback_process_must_follow` | PR base 5/9 영역 영역 작성 → Task #688 먼저 머지 → 중복 정정. base 갱신 영역 영역 점검 필요 |
| `feedback_diagnosis_layer_attribution` 권위 사례 강화 | **본 PR 본질** (nested 11×3 그리드 누락, Task #688 해결) vs **Issue #726 진짜 본질** (셀 안 화살표 도형 미출력) 영역 영역 두 본질 분리 진단 — PR 영역 영역 잘못 연결 (closes #726) |
| `feedback_visual_judgment_authority` 권위 사례 강화 | 작업지시자 영역 영역 5쪽 SVG 시각 비교 요청 → byte-identical 결과 입증 — 결정적 검증 영역 영역 PR 효과 없음 명확화 |
| `feedback_pr_supersede_chain` | PR #694 (Task #688) → Issue #726 (잔존 본질) → **PR #817** (close, 중복 정정) → 분리 PR (Issue #726 진짜 본질) (a) 패턴 |

## 10. 잔존 후속

- Issue #726 OPEN 유지 — 진짜 본질 (셀 안 화살표 도형 SVG 미출력) 영역 영역 분리 PR
- 후보 (a): `src/renderer/` 도형 분기 (4 backend 동기 정정)
- 후보 (b): `src/parser/hwpx/` table cell GenShape 파싱

---

작성: 2026-05-12
