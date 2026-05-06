# PR #600 처리 보고서 — 핀셋 cherry-pick 머지 + 시각 판정 ★ 통과 + 별개 영역 이슈 #615 발견

**PR**: [#600 fix: Supplementary PUA-A (U+F02B1~F02C4) SVG 출력 정정 (closes #513)](https://github.com/edwardkim/rhwp/pull/600)
**작성자**: @oksure (Hyunwoo Park)
**처리 결정**: ✅ **commit 단위 cherry-pick 2 commits + devel merge + push + PR/Issue close**
**처리일**: 2026-05-06

## 1. 처리 결과 요약

| 항목 | 결과 |
|------|------|
| 결정 | ✅ commit 단위 cherry-pick (2 commits) + devel merge + push + PR close + Issue close |
| 시각 판정 | ★ **통과** (① ~ ⑳ 정상 출력 + KTX 회귀 0) |
| Devel merge commit | `9768819` |
| **PR mergeable** | UI MERGEABLE / 그러나 PR base 광범위 뒤 (PR #571/#599 패턴) |
| Cherry-pick 충돌 | 0 건 (본 사이클 처리분과 0 중첩) |
| Author 보존 | ✅ Hyunwoo Park (`oksure@gmail.com`) 보존 |
| Issue #513 | CLOSED (수동 close + 안내 댓글) |
| 광범위 페이지네이션 sweep | 164 fixture / 1,614 페이지 / 페이지 수 회귀 0 |
| 별개 영역 발견 | Issue #615 (U+F53A 옛한글 PUA 매핑 한컴 정답지 정합 안 함) |

## 2. 본질 결함 (PR 진단)

Task #509 의 `map_pua_bullet_char` 매핑이 `convert_pua_enclosed_numbers` (composer) 의 CharOverlap 변환에 막혀 도달 못 함. 결과: SVG 에 ① 대신 "1" (사각형 박스) 렌더링.

## 3. 본질 정정 (PR)

1. `composer.rs::pua_enclosed_border_type` — F02B1~F02C4 범위 제거 (CharOverlap 제외)
2. `paragraph_layout.rs::map_pua_bullet_char` — F02BA~F02C4 (⑩~⑳) 매핑 추가 (전체 20자 완성)
3. F02CE~F02E1 (반전 사각형) — CharOverlap 유지 (표준 Unicode 부재)

### 3.1 Copilot review 응답 (`aafe85a`)

- ⑩~⑳ 테스트 추가 (`supplementary_pua_a_maps_circled_digits`)
- 주석 ①~⑨ → ①~⑳ 갱신
- 폭 계산 정합 (`map_pua_bullet_char` 후 `estimate_text_width`)

## 4. PR base skew + commit 단위 cherry-pick

### 4.1 GitHub UI vs 실제 base

UI mergeable=MERGEABLE 표시지만 PR base diff 가 본 사이클 cherry-pick 모두 revert (PR #571/#599 패턴):
- PR #599 의 Skia 본질 모두 revert
- 메인테이너 후속 정정 모두 revert
- 본 사이클 cherry-pick 모두 revert (PR #561/#564/#567/#570/#575/#580/#584/#589/#592/#593)

### 4.2 본질 commit 영역 — 본 사이클 처리분과 0 중첩

본 PR 의 영역은 Supplementary PUA-A `F02B1~F02C4` (Issue #513 정정) — PR #592 의 SPUA-A 저영역 (`F0000~F00CF`) 과 **다른 코드포인트 영역** → 충돌 없이 cherry-pick 가능 (PR #571 close 결정과 다른 케이스).

## 5. cherry-pick 진행

### 5.1 대상 commits (2개, 충돌 0)

```
34f8547 fix: Supplementary PUA-A (U+F02B1~F02C4) SVG 출력 정정 (closes #513)
14f30e8 address review: ⑩~⑳ 테스트 추가 + 주석 갱신 + 폭 계산 정합
```

`Hyunwoo Park <oksure@gmail.com>` author 보존.

### 5.2 변경 영역

| 파일 | 변경 |
|------|------|
| `src/renderer/composer.rs` | -15 (F02B1~F02C4 CharOverlap 제외) |
| `src/renderer/layout/paragraph_layout.rs` | +30 (⑩~⑳ 매핑 + 폭 계산 정합 + 테스트) |

## 6. 결정적 검증 결과

| 게이트 | 결과 |
|--------|------|
| `cargo build --release` | ✅ Finished |
| `cargo test --lib --release` | ✅ **1134 passed** / 0 failed (회귀 0) |
| `cargo test --lib pua` | ✅ **12 passed** (`supplementary_pua_a_maps_circled_digits` ⑩~⑳ GREEN) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_546` | ✅ 1 passed |
| `cargo test --test issue_554` | ✅ 12 passed |
| `cargo clippy --release --lib` | ✅ 0건 |

## 7. 광범위 페이지네이션 회귀 sweep

| 통계 | 결과 |
|------|------|
| 총 fixture | **164** (158 hwp + 6 hwpx) |
| 총 페이지 | **1,614** |
| **fixture 별 페이지 수 차이** | **0** |

## 8. SVG 정량 측정 (PR 본문 100% 재현)

| Fixture | byte 차이 | 원문자 출현 (before/after) | 평가 |
|---|---|---|---|
| **pua-test** | 1/1 | **0 → 9** | ✅ PR 본문 명시 (9 SVG text) 100% 재현 |
| **mel-001** | 15/21 | 20 → 34 (+14) | ✅ PR 본문 정정 영역 |
| **kps-ai** | 1/80 | 32 → 34 (+2) | ✅ PR 본문 명시 정합 |
| **KTX** | 0/27 | 변경 없음 | ✅ PR 본문 명시 (변경 없음) 정합 |

## 9. 시각 판정 (★ 게이트)

### 9.1 SVG 자료

- `output/svg/pr600_before/{mel-001,kps-ai,KTX,pua-test}/`
- `output/svg/pr600_after/{mel-001,kps-ai,KTX,pua-test}/`

### 9.2 작업지시자 시각 판정 결과

> 시각 판정 통과

→ ★ **통과**. ① ~ ⑳ 정상 출력 + KTX 회귀 0 + mel-001 ①②③ 정상 표시.

## 10. 별개 영역 발견 — Issue #615

본 PR 시각 판정 중 별개 결함 발견:

- `samples/pua-test.hwp` 의 **U+0F53A** 가 `pua_oldhangul.rs:5022` 에서 자모 시퀀스 (ᄒᆞᆫ) 로 임의 변환
- **한컴 PDF 정답지는 "Basic-out, 매핑 표 외" 빈 공백 처리** (한컴 자체도 매핑 미지원)
- → 본 환경의 `pua_oldhangul.rs` 매핑이 한컴 정답지와 정합 안 함

작업지시자 시각 검증 후 명확화:
> PUA 가 계속 회귀되는 현상이 발생하는군요! 한군데 고치면 계속 다른 곳에서 터져버립니다.

→ **PUA 영역의 두더지 잡기 (whack-a-mole) 패턴** 인식. 본 사이클 PUA 정정 누적:
- PR #587 (HWP 5.0 0x18/0x1E swap)
- PR #562 (Task #555 옛한글 PUA → 자모 변환 후 폰트 매트릭스)
- PR #592 (Task #588 SPUA-A 저영역 0xF0000~F00CF)
- PR #600 (Task #509 후속 Supplementary PUA-A F02B1~F02C4) — 본 PR
- → Issue #615 (옛한글 PUA 매핑 한컴 정답지 정합 검증)

→ **PUA 매핑 영역 전체 재검증** 별도 광범위 task 후보 (메모리 룰 후보: PUA 매핑 표 통합 검증 패턴).

## 11. PR / Issue close 처리

### 11.1 PR #600 close
- 댓글 등록 (한글, cherry-pick 결과 + 결정적 검증 + 광범위 sweep + SVG 정량 + 별개 영역 안내 + 컨트리뷰터 협업 인정)
- close 처리

### 11.2 Issue #513 수동 close
- 안내 댓글 + 별개 영역 (Issue #615) 안내

### 11.3 Issue #615 신규 등록 + 본질 분석 갱신
- 작업지시자 시각 검증으로 한컴 정답지 영역 ("Basic-out, 매핑 표 외") 확인
- `pua_oldhangul.rs:5022` 의 자모 시퀀스 매핑이 잘못된 영역 식별

## 12. 메모리 정합

- ✅ `feedback_visual_regression_grows` — 시각 판정 게이트 정합 운영 (★ 통과)
- ✅ `feedback_hancom_compat_specific_over_general` — 케이스별 명시 가드 (F02B1~F02C4 vs F02CE~F02E1 분리)
- ✅ `feedback_rule_not_heuristic` — 코드포인트 범위 + 명시 매핑
- ✅ `feedback_pdf_not_authoritative` — 권위는 작업지시자 한컴 환경 + 한컴 PDF 정답지 ("Basic-out, 매핑 표 외" 영역 식별)
- ✅ `feedback_per_task_pr_branch` — Issue #513 단일 본질 PR 정합
- ✅ `feedback_pr_comment_tone` — close 댓글 차분/사실 중심 + 한글 답변 (PR #599 학습 영역)
- ✅ `feedback_check_open_prs_first` — PR OPEN 상태 확인 후 진행
- ✅ `feedback_small_batch_release_strategy` — 본 사이클 (5/1 ~ 5/6) 누적 22번째 PR
- ✅ **PR #571/#599 패턴과 다름** — base skew 동일하지만 본질 영역 0 중첩으로 cherry-pick 가능

## 13. 본 PR 의 본질 — Task #509 후속 정정 + PUA 회귀 패턴 인식

본 PR 의 처리 본질에서 가장 우수한 점:

1. **Task #509 본질 후속 정확히 식별** — 매핑 우선순위 결함 (CharOverlap → map_pua_bullet_char) 정확 추적
2. **케이스별 명시 가드** — F02B1~F02C4 (CharOverlap 제외) + F02CE~F02E1 (CharOverlap 유지) 명시 분리
3. **Copilot review 응답 정합** — 3개 코멘트 모두 commit `aafe85a` 로 응답 (테스트 + 주석 + 폭 계산 정합)
4. **PR #571/#599 와 다른 base skew 처리** — 본 사이클 처리분과 0 중첩 영역 식별로 commit 단위 cherry-pick 가능
5. **PUA 회귀 패턴 인식의 시작점** — 본 PR + Issue #615 가 본 사이클 PUA 정정 누적의 두더지 잡기 패턴 인식 트리거

## 14. 본 사이클 사후 처리

- [x] PR #600 close (cherry-pick 머지 + push + 한글 댓글)
- [x] Issue #513 수동 close (안내 댓글)
- [x] Issue #615 신규 등록 (U+F53A 옛한글 PUA 매핑 한컴 정합 안 함)
- [x] 처리 보고서 (`mydocs/pr/archives/pr_600_report.md`, 본 문서)
- [ ] 검토 보고서 archives 이동 (`mydocs/pr/pr_600_review.md` → `mydocs/pr/archives/pr_600_review.md`)
- [ ] 5/5 또는 5/6 orders 갱신
