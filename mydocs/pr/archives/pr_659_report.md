# PR #659 처리 결과 보고서

**PR**: [#659 feat: ir-diff 표 속성 / 컨트롤 식별 비교 추가 (closes #653)](https://github.com/edwardkim/rhwp/pull/659)
**작성자**: @oksure (Hyunwoo Park, oksure@gmail.com) — 6번째 사이클 PR
**처리 결정**: ✅ **3 commits 보존 cherry-pick 머지**
**처리일**: 2026-05-07
**devel merge**: `0d8b0fe`

## 1. 처리 결과 요약

| 영역 | 결과 |
|------|------|
| cherry-pick 충돌 | 0 |
| author 보존 | ✅ Hyunwoo Park 3 commits |
| `cargo test --lib` | 1141 passed (회귀 0) |
| `cargo clippy --bin rhwp -- -D warnings` | ✅ 통과 |
| `cargo clippy --lib -- -D warnings` | ✅ 통과 |
| `cargo build --release` | ✅ 통과 |
| **결정적 게이트웨이 검증** | ✅ Issue #653 본질 영역 검출 |
| **메인테이너 시각 판정** | ✅ 통과 |

## 2. cherry-pick 영역

### 보존 cherry-pick (3 commits)

| commit | author | 영역 |
|--------|--------|------|
| `cf1296b` (orig `3645d69`) | Hyunwoo Park | feat: ir-diff 표 속성 / 컨트롤 식별 비교 추가 (closes #653) |
| `65817f2` (orig `8d5886c`) | Hyunwoo Park | feat: ir-diff LINE_SEG 전체 필드 비교 확장 |
| `ed13aa2` (orig `eb97d9b`) | Hyunwoo Park | address review: 출력 키 정합 + match 리팩터 + 문서 정정 |

본질 변경: `src/main.rs` (+140/-3, ir-diff 명령) + `CLAUDE.md` (+4/-1, 매뉴얼 정정).

## 3. 결정적 게이트웨이 검증 (작업지시자 지정)

`samples/hwpx/aift.hwpx` page 1 ir-diff (Issue #652 의 권위 자료 + Issue #653 의 발견 출처).

### BEFORE (현재 devel)

```
=== 비교 완료: 차이 255 건 ===
[차이] cc: A=32 vs B=56
[차이] char_offsets[0]: A=8 vs B=32
[차이] controls: A=3 vs B=4    ← 카운트만
[차이] char_shapes count: A=3 vs B=2
```

### AFTER (cherry-pick 후)

```
=== 비교 완료: 차이 259 건 ===  (+4건)

[차이] ctrl[0] type: A=cold vs B=secd      ← HWPX SectionDef 정합 안 됨 (Issue #653 본질!)
[차이] ctrl[1] type: A=tbl vs B=cold
[차이] ctrl[2] type: A=pgnp vs B=tbl
[차이] ctrl[1] type: A=tbl vs B=secd       (pi=1 영역)
```

### 전체 문서 sweep (aift.hwpx vs aift.hwp)

| 항목 | BEFORE | AFTER | 신규 검출 |
|------|:------:|:-----:|:---------:|
| 전체 차이 | 650 | **707** | +57 |
| `ctrl type 차이` | 0 | **4** | +4 |
| `tbl 속성 차이` | 0 | **48** | +48 |
| `pic 속성 차이` | 0 | **4** | +4 |

**Issue #653 본질 영역 정확 재현**:
- ✅ `tbl page_break: A=CellBreak vs B=RowBreak` 다수 — Issue #652 본질 #1 재현
- ✅ `ctrl[0] type: A=cold vs B=secd` — HWPX SectionDef 정합 안 됨 정확 재현
- ✅ `tbl treat_as_char` / `tbl wrap` / `tbl size` / `tbl outer_margin` / `tbl border_fill_id` 검출 가능

## 4. 회귀 차단 검증

### cargo test --lib (1141 passed)
회귀 0 입증.

### 코드 영역 분석
- 변경 영역: `src/main.rs` ir-diff 명령 (진단 도구) + `CLAUDE.md` 매뉴얼
- 무영향 영역: 코어 IR (`src/model/`), 렌더 (`src/renderer/`), 직렬화 (`src/serializer/`), HWP3/HWP5/HWPX 파서, WASM API
- 출력 포맷 변경: `controls: A=N vs B=M` → `controls count: A=N vs B=M` (메시지 키 변경, grep 의존 사용자 영향 가능 — 매우 좁은 영역)

## 5. 메인테이너 → 컨트리뷰터 선구현 패턴

본 PR 의 의의: **메인테이너가 5/5 PR #601 검토 영역에서 발견 → Issue #653 등록 → 외부 컨트리뷰터가 5/7 같은 일자에 선구현**.

| 영역 | 의의 |
|------|------|
| 메인테이너 사이클 단축 | Issue #653 직접 구현 미수행, 외부 흡수로 처리 |
| 공동체 비전 정합 | v1.0.0+ 부터 커뮤니티 참여 개방의 권위 사례 (`user_role_identity` 정합) |
| 활발한 컨트리뷰터 | @oksure 6번째 PR 누적 (PR #581/#582/#583/#600/#601/#602 + 본 PR) |

## 6. Copilot 리뷰 응답 정합

마지막 commit `eb97d9b` "address review" 에서 Copilot 4 코멘트 모두 반영:

| Copilot 코멘트 | 응답 |
|---------------|------|
| `border_fill` 출력 키와 필드명 불일치 | ✅ `border_fill_id` 로 정정 |
| `voff/hoff` 축약 vs 필드명 | ✅ `v_offset/h_offset` 명확화 |
| if-let 체인 → exhaustiveness 손실 | ✅ single match 리팩터 |
| help text / CLAUDE.md 와 구현 불일치 | ✅ 표/그림/도형 비교 범위 분리 명시 |

자체 검토 응답 패턴 정합 — `user_work_style` (자체 검토 응답 직접 결정 영역) 정합.

## 7. 미커버 영역 + 후속 권유

Issue #653 의 미커버 영역:
- ❌ **표의 `cells`** 비교 (개별 셀의 폭/높이/병합 등)
- ❌ **표의 `is_header`** 플래그 (PR #601 의 다중 제목행 분할 영역과 연관)

후속 권유 — 별도 PR 또는 후속 task 영역 분리. PR close 댓글에서 컨트리뷰터에게 안내.

## 8. 처리 절차

1. ✅ PR 정보 확인 (mergeable=MERGEABLE, BEHIND 5 commits)
2. ✅ 검토 보고서 작성 + 작업지시자 승인
3. ✅ BEFORE 게이트웨이 baseline 측정 (`output/pr659/before_p0.txt`)
4. ✅ cherry-pick 3 commits 보존 (충돌 0)
5. ✅ 결정적 검증 (cargo test/clippy/build)
6. ✅ AFTER 게이트웨이 검증 (`output/pr659/after_p0.txt`)
7. ✅ 작업지시자 시각 판정 통과
8. ✅ devel merge (`0d8b0fe`) + push
9. ✅ PR #659 close (한글 댓글 + 미커버 영역 후속 권유)
10. ✅ Issue #653 close (수동, closes 키워드 cherry-pick 자동 처리 안 됨)

## 9. 메모리 정합 영역

- `feedback_first_pr_courtesy` — @oksure 활발한 컨트리뷰터 (6번째), 첫 PR 표현 부적용
- `feedback_visual_judgment_authority` — 결정적 게이트웨이 검증 + 작업지시자 시각 판정
- `feedback_pr_comment_tone` — close 댓글 차분/사실 중심 + 컨트리뷰터의 정합 영역 인정 + 후속 권유 톤
- `user_work_style` — 외부 PR 옵션 분류 (옵션 A 보존 cherry-pick) + 자체 검토 응답 정합
- `user_role_identity` — 메인테이너 → 컨트리뷰터 선구현 패턴, v1.0.0+ 공동체 개방 비전 정합
- `project_external_contributors` — @oksure 누적 사이클 (Hyunwoo Park, oksure@gmail.com)

## 10. 본 환경 산출물

- `mydocs/pr/pr_659_review.md` — 검토 보고서
- `mydocs/pr/pr_659_report.md` — 본 처리 보고서 (archives 이동 대기)
- `output/pr659/before_p0.txt` — BEFORE 게이트웨이 baseline (page 1, 173 lines)
- `output/pr659/after_p0.txt` — AFTER 게이트웨이 (page 1, 177 lines)
- `output/pr659/after_all.txt` — AFTER 전체 문서 sweep
