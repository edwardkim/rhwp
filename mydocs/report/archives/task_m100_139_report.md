# 최종 보고서: 수식 렌더링 고도화 (분석/방향 수립)

- **타스크**: [#139](https://github.com/edwardkim/rhwp/issues/139)
- **마일스톤**: M100
- **브랜치**: `local/task139`
- **작성일**: 2026-04-14
- **모드**: 플랜 모드 — 분석/방향 수립, 소스 코드 수정 없음

## 1. 타스크 개요

HWP 수식 처리 1차 구현(85~90% 완성)을 기반으로, 수식 입력 방식·폰트·레이아웃의 고도화 방향을 수립한다.

## 2. 산출물 목록

| 단계 | 산출물 | 위치 |
|------|--------|------|
| 1단계 | LaTeX vs 한컴 수식 비교 분석 | `mydocs/tech/equation_latex_comparison.md` |
| 2~3단계 | 수식 폰트 조사·선정 및 적용 방안 | `mydocs/tech/equation_font_selection.md` |
| 4단계 | 수식 레이아웃 정밀화 방안 | `mydocs/tech/equation_layout_spec.md` |
| 단계별 보고 | 1단계 완료보고서 | `mydocs/working/task_m100_139_stage1.md` |
| 단계별 보고 | 2~3단계 완료보고서 | `mydocs/working/task_m100_139_stage2.md` |
| 단계별 보고 | 4단계 완료보고서 | `mydocs/working/task_m100_139_stage4.md` |
| 피드백 | 엔지니어링 포인트 피드백 | `mydocs/feedback/task139-eq-01.md` |

## 3. 주요 결정 사항

### 3.1 수식 입력 방식 — 듀얼 토크나이저 (전략 A)

- 기존 한컴 파서(tokenizer.rs, parser.rs) **무수정 유지**
- LaTeX 전용 토크나이저+파서를 별도 추가 → 공통 AST(EqNode)로 합류
- **핵심 근거**: AST 이하 레이어(layout, render)가 완전히 문법 독립적 → 토크나이저/파서만 확장하면 됨
- UI: 명시적 모드 토글(한컴 ↔ LaTeX)을 1순위로, 자동 감지는 보조 수단

### 3.2 수식 폰트 — Latin Modern Math + Pretendard

| 역할 | 폰트 | 근거 |
|------|------|------|
| 주 수식 폰트 | **Latin Modern Math** | HyHwpEQ와 동일 Computer Modern 뿌리, MATH 테이블, MathJax 4 권장 |
| 기호 폴백 | **STIX Two Math** | 최대 글리프 커버리지(5,200+), macOS 기본 탑재 |
| 한글 폴백 | **Pretendard** | OFL, rhwp에 이미 woff2 번들, 모든 OS 커버 |

**font-family 체인**:
```
"Latin Modern Math", "STIX Two Math", "Cambria Math", "Pretendard", serif
```

**HWP 수식 폰트 처리**: `font_name` 속성(HyHwpEQ, HancomEQN 등)은 **무시**, rhwp 수식 폰트로 통일

### 3.3 수식 레이아웃 — TeX 표준 대비 검증

- `SCRIPT_SCALE(0.7)`, `FRAC_LINE_THICK(0.04)` → TeX 표준과 **일치** (보정 불필요)
- `BIG_OP_SCALE(1.5)` → TeX 표준(~1.2)과 **25% 차이** (한컴 측정 후 보정 필요)
- `MATRIX_COL_GAP(0.8)`, `MATRIX_ROW_GAP(0.3)` → 한컴 측정으로 보정 필요
- 표준 수식 12종 측정 절차 수립 완료

## 4. 피드백 반영

| 피드백 | 반영 내용 |
|--------|----------|
| 폰트 로딩 시점과 레이아웃 틀어짐 | Canvas 경로에서 `document.fonts.load()` 호출로 폰트 로딩 완료 보장 후 렌더링 시작 |
| LaTeX 자동 감지 엣지 케이스 | 첫 글자 `\` 감지 → 보조 수단 격하, UI 명시적 모드 토글을 1순위로 채택 |

## 5. 후속 구현 타스크 분리 계획

| 후속 타스크 | 내용 | 근거 문서 |
|------------|------|----------|
| **수식 폰트 적용** | svg_render.rs, canvas_render.rs에 font-family 적용, woff2 번들 | `equation_font_selection.md` |
| **수식 레이아웃 보정** | 한컴 기준값 측정 → 레이아웃 상수 조정 | `equation_layout_spec.md` |
| **LaTeX 파서 구현** | 듀얼 토크나이저 방식 LaTeX 입력 지원 | `equation_latex_comparison.md` |
| **수식 편집 UI 개선** | 듀얼 모드, 자동완성, 템플릿 확장 | `equation_latex_comparison.md` |

## 6. 전략적 의의

- **한컴 호환 + LaTeX 지원 + 웹 네이티브(WASM)** — 이 조합을 제공하는 수식 편집기는 현재 없음
- 한컴 디벨로퍼 포럼에서 사용자들이 LaTeX 지원을 강력히 요청 중이나 한컴 측은 구체적 일정 미공개 → rhwp가 선점 가능
- AI(LLM)가 LaTeX 수식 출력에 최적화되어 있어, LaTeX 입력 지원 시 AI 협업 시나리오 확장
