---
PR: #729
제목: Task #143 — LaTeX 명령어 호환 확장 (2차)
컨트리뷰터: @oksure (Hyunwoo Park) — 20+ 사이클 핵심 컨트리뷰터
처리: 옵션 A — PR HEAD squash cherry-pick + no-ff merge
처리일: 2026-05-10
머지 commit: eb3f9fd4
후속: Issue #762 (exam_math 영역 inf/sup/lim 회귀 정정 통합)
---

# PR #729 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (PR HEAD squash cherry-pick + no-ff merge `eb3f9fd4`)

| 항목 | 값 |
|------|-----|
| 머지 commit | `eb3f9fd4` (--no-ff merge) |
| Squash commit | `f65ffd03` (7 commits 통합) |
| closes | #143 |
| 시각 판정 | LaTeX 입력 ✅ + exam_math 회귀 발견 ⚠️ → PR #762 통합 후속 |
| 자기 검증 | equation 114 PASS + cargo test ALL GREEN + sweep 168/170 same |

## 2. 정정 본질 — 6 files, +735/-11

### 2.1 `src/renderer/equation/symbols.rs` (+87/-5)
- **FontStyleKind 5개 추가**: Blackboard / Calligraphy / Fraktur / SansSerif / Monospace
- **기호 별칭 ~80개**: 관계/논리/화살표/연산자/큰 연산자/기호/함수/장식/괄호

### 2.2 `src/renderer/equation/parser.rs` (+599/-3)
- **구조 명령어**: `\frac`, `\dfrac`, `\tfrac`, `\text`, `\operatorname`, `\binom`, `\overset`, `\underset`, `\stackrel`, `\phantom`
- **환경 파싱**: `pmatrix`, `bmatrix`, `vmatrix`, `Bmatrix`, `Vmatrix`, `cases`, `aligned`, `split`, `gather`, `gathered`, `array`, `smallmatrix`
- **LaTeX 호환 테스트 41건 신규** (43 → 84)

### 2.3 `src/renderer/equation/tokenizer.rs` (+40)
- `\\` (줄바꿈), `\, \: \; \!` (간격), `\{ \} \| \#` (이스케이프)

### 2.4 3 렌더러 동기 정정 (`canvas/svg/skia` +9/-3)
- `feedback_image_renderer_paths_separate` 정합 — exhaustive match 영역 영역 동시 갱신

## 3. 채택 접근 (메인테이너 권고 정합)

- 이슈 #143 영역 의 원래 설계: **듀얼 토크나이저** (별도 파일)
- 본 PR: 메인테이너 권고 영역 영역 **기존 파서 확장** 채택 — 신규 인프라 도입 부재 영역 영역 위험 좁힘

## 4. 본 환경 cherry-pick

### 4.1 7 commits squash (개별 cherry-pick 충돌 영역 영역 squash 채택)

원본 commits:
```
04187512 feat: LaTeX 이스케이프 중괄호 + 각도 괄호 지원
d672f954 feat: \overset/\underset/\stackrel + 환경 확장
45fdb335 feat: LaTeX 호환 명령어 대폭 확장
0264fe18 fix: Copilot 리뷰 반영 — Bold italic 보존 + \text 공백 제한 문서화
69b0b84d fix: Skia renderer FontStyleKind 매치 누락 수정 (CI 빌드 실패 수정)
d96ddcc4 Task #143: LaTeX 명령어 호환 확장 (2차)
d53b7acf Task #143: LaTeX \begin{env}...\end{env} 환경 파싱 (3차)
```

개별 cherry-pick 영역 영역 commits 영역 영역 누적 변경 영역 영역 영역 영역 의 충돌 영역 영역 발생 — PR HEAD `pr729-head` 영역 영역 squash cherry-pick 채택 (`git cherry-pick --no-commit 60aeaa8d..pr729-head`) 영역 영역 단일 commit `f65ffd03` 영역 영역 적용. 충돌 0건.

## 5. 결정적 검증

| 검증 | 결과 |
|------|------|
| `cargo build --release` | ✅ 통과 (31.54s) |
| `cargo test --release` (전체) | ✅ ALL GREEN, failed 0 |
| `cargo test --lib renderer::equation` | ✅ **114 PASS** (43 → 84 신규 41건 + tokenizer/symbols 영역) |
| 광범위 sweep (7 fixture / 170 페이지) | ⚠️ **168 same / 2 diff** (exam_math_014/016.svg) |
| WASM 빌드 (Docker) | ✅ 4.61 MB |

### 5.1 광범위 sweep 회귀 (exam_math 2건)

`samples/exam_math.hwp` 영역 영역 페이지 14/16 영역 영역 SVG byte diff:
- BEFORE: `<text>∞</text>` (∞ 기호)
- AFTER: `<text>inf</text>` (3글자 텍스트)

**회귀 본질** (Issue #762):
- `FUNCTIONS` HashMap 영역 영역 `("inf", "inf"), ("sup", "sup"), ("lim", "lim"), ("limsup", ...), ("liminf", ...), ("Pr", "Pr")` 추가
- 기존 hwpeq script 영역 영역 `inf` 영역 영역 ∞ 기호 영역 영역 영역 의미 (`lookup_symbol("inf") → "∞"`)
- PR 후 영역 영역 `is_function("inf") = true` 영역 영역 우선 매칭 영역 → 텍스트 "inf" 영역 출력
- `test_hwpeq_not_regressed` 영역 영역 본 케이스 영역 미커버

## 6. 작업지시자 시각 검증

### 6.1 LaTeX 본질 ✅ 통과
`samples/equation-lim.hwp` 예제 파일 영역 영역 LaTeX 형식 수식 입력 테스트 ✅ 통과 — PR 본질 정합 입증.

### 6.2 exam_math 회귀 ⚠️ 확인
페이지 14/16 영역 영역 의 `inf` 텍스트 영역 영역 작업지시자 영역 영역 확인 — 회귀 판정.

## 7. 처리 결정 — PR #762 통합 후속 (PR #723 영역 영역 (c) 패턴 정합)

### 7.1 결정
- PR #729 머지 유지 (rollback 미진행)
- exam_math 회귀 정정 영역 영역 후속 PR 영역 영역 통합 처리 영역

### 7.2 결정 근거
- LaTeX 본질 확장 (FontStyleKind 5개 + 구조 명령어 + 환경 파싱 + 기호 별칭 ~80개) 영역 영역 본질 정합 영역 영역 통과
- 회귀 영역 영역 영역 lookup 우선순위 영역 영역 영역 — 정정 영역 영역 단순 (parser 영역 영역 `lookup_symbol` 영역 영역 `is_function` 우선 점검 + hwpeq inf 회귀 가드 추가)
- 동일 컨트리뷰터 @oksure 영역 영역 5/10 사이클 영역 영역 다수 PR 등록 영역 영역 후속 PR 영역 영역 통합 영역 영역 합리

### 7.3 후속 등록 — Issue #762
- 제목: "exam_math.hwp 영역 LaTeX 호환 확장 영역 영역 `inf`/`sup`/`lim` lookup 우선 회귀"
- 처리 방향: parser.rs 영역 영역 `lookup_symbol` (∞) 영역 영역 `is_function` (inf 텍스트) 보다 먼저 점검 + hwpeq 회귀 가드 신규 추가
- @oksure 영역 영역 후속 PR 영역 영역 통합 영역 영역 처리 권장

## 8. 영향 범위

### 8.1 변경 영역
- 수식 렌더링 영역 영역 LaTeX 호환 명령어 (~80 기호 별칭 + 환경 파싱 + 구조 명령어)
- FontStyleKind 5개 추가 → 3 렌더러 동기 정합

### 8.2 무변경 영역 (sweep 영역 영역 168/170 same 영역 영역 입증)
- 다른 layout/render 경로
- HWP3/HWPX 변환본 영역 영역 시각 정합 (수식 컨트롤 외부)
- 6 fixture (2010-01-06 / aift / exam_eng / exam_kor / exam_science / synam-001) 영역 영역 회귀 0

### 8.3 회귀 (Issue #762 영역 영역 영역 후속 정정)
- exam_math 영역 영역 페이지 14/16 영역 영역 의 hwpeq `inf` ∞ 기호 영역 영역 → "inf" 텍스트 영역 변환

## 9. 후속 분리 (PR 본문 명시)

### 9.1 UI 명시적 모드 토글
이슈 #143 영역 영역 의 "UI 명시적 모드 토글" 영역 영역 rhwp-studio (프론트엔드) 범위 → 별건 이슈 영역.

### 9.2 exam_math 회귀 정정
**Issue #762** — `inf`/`sup`/`lim` lookup 우선 회귀 영역 영역 후속 PR 영역 영역 통합 처리 권장.

## 10. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure **20+ 사이클** 핵심 컨트리뷰터 (5/10 사이클 영역 영역 2번째 PR — PR #728 직후 영역) |
| `feedback_image_renderer_paths_separate` | **권위 사례 강화** — 3 렌더러 (canvas/svg/skia) 영역 동기 정정 + Skia renderer 영역 영역 누락 정정 영역 fix commit `69b0b84d` 영역 영역 자기 진단 |
| `feedback_process_must_follow` | 메인테이너 권고 영역 영역 채택 (듀얼 토크나이저 → 기존 파서 확장) — 신규 인프라 도입 부재 영역 영역 위험 좁힘 + 후속 분리 (UI 모드 토글 + Issue #762) 명시 |
| `feedback_visual_judgment_authority` | 작업지시자 시각 검증 — LaTeX 본질 ✅ + exam_math 회귀 발견 영역 영역 후속 PR 통합 결정 |
| `feedback_pr_supersede_chain` | **(c) 패턴 적용** — PR #729 머지 유지 + Issue #762 (exam_math 회귀) 후속 PR 통합 영역 영역. PR #723 → PR #732 영역 영역 동일 패턴 정합. |
| `feedback_hancom_compat_specific_over_general` | hwpeq 영역 영역 영역 LaTeX 영역 영역 영역 키워드 충돌 영역 영역 — 일반화 lookup 영역 영역 영역 case 가드 영역 영역 정합 (Issue #762 영역 영역 후속 정정 방향) |

## 11. 잔존 후속

- **Issue #762 OPEN** — exam_math 회귀 정정 통합 (@oksure 후속 PR 영역 영역 권장)
- **UI 명시적 모드 토글** — rhwp-studio 영역 영역 별건 이슈 등록 권장

---

작성: 2026-05-10
