# 2~3단계 완료보고서: 수식 폰트 조사·선정 및 적용 방안 설계

- **타스크**: [#139](https://github.com/edwardkim/rhwp/issues/139)
- **마일스톤**: M100
- **브랜치**: `local/task139`
- **작성일**: 2026-04-14
- **구현 계획서**: `mydocs/plans/task_m100_139_impl.md`

## 산출물

- `mydocs/tech/equation_font_selection.md` (10개 장)

## 완료 내용

### 1. 한컴 수식 폰트 히스토리 조사

출처: [leesj.me/hwp-custom-equation](https://leesj.me/hwp-custom-equation/)

- 한컴 수식 폰트 변천: HSUSR.HFT → HyHwpEQ (v1.10~v1.13) → HancomEQN
- **핵심 발견**: HyHwpEQ는 **CMU Serif(Computer Modern)** 기반. 이탤릭 대문자는 로만체를 skew 처리로 생성
- 사용자 불만: 로만체 굵기 불일치, 부등호 곡선, 근호 돌출 등

### 2. 오픈 라이선스 수식 폰트 12종 비교 조사

| 폰트 | 라이선스 | 글리프 | CM 유사도 | MATH 테이블 |
|------|---------|--------|----------|------------|
| Latin Modern Math | GUST | 4,802 | ★★★★★ | ✓ |
| STIX Two Math | OFL | 5,200+ | ★★ | ✓ |
| XITS Math | OFL | ~3,550 | ★★ | ✓ |
| Libertinus Math | OFL | ~4,000 | ★★★ | ✓ |
| Fira Math | OFL | 2,094 | ★ | ✓ |
| CMU Serif | OFL | ~1,000+ | ★★★★★ | ✗ |
| 기타 6종 (New CM, Asana, TG 계열) | — | — | — | ✓ |

### 3. 최적 폰트 선정

| 순위 | 폰트 | 역할 | 핵심 근거 |
|------|------|------|----------|
| **1순위** | **Latin Modern Math** | 주 수식 폰트 | HyHwpEQ와 동일 CM 뿌리, MATH 테이블, MathJax 4 권장 |
| **2순위** | **STIX Two Math** | 기호 폴백 | 최대 글리프 커버리지, macOS 기본 탑재 |
| **한글 폴백** | **Pretendard** | 한글 처리 | OFL, rhwp에 이미 woff2 번들, 모든 OS 커버 |

### 4. 수식 내 한글 처리 방안

- 수식 내 한글 사용 사례: CASES 조건 설명, 단위/주석, 한글 변수
- Latin Modern Math에 한글 글리프 없음 → font-family 체인에서 Pretendard로 자동 폴백
- 별도 한글 감지 로직 불필요 — 브라우저의 font-family 체인 동작으로 충분

### 5. HWP 수식 폰트 처리 방침

- HWP 파일의 수식 `font_name` 속성(HyHwpEQ, HancomEQN 등)은 **무시**
- rhwp의 수식 전용 폰트 체인으로 통일
- 근거: 저작권 폰트 번들 불가, 사용자 환경에 미설치, CM 계열 대체로 시각적 일관성 유지

### 6. 최종 font-family 체인

```
"Latin Modern Math", "STIX Two Math", "Cambria Math", "Pretendard", serif
```

### 7. 적용 방안 설계

| 수정 대상 | 변경 내용 | 예상 규모 |
|----------|----------|----------|
| `svg_render.rs` | 모든 `<text>` 요소에 EQ_FONT_FAMILY 상수 추가 | ~20줄 |
| `canvas_render.rs` | `ctx.font`에 수식 폰트 반영 | ~15줄 |
| `web/fonts/` | Latin Modern Math woff2 추가 | 파일 1개 (~200-300 KB) |
| `rhwp-studio/` | `@font-face` 선언 | ~10줄 |
| 웹 폰트 로딩 | woff2 번들링 + `local()` 우선 참조 | — |
| `export-svg` | `--embed-fonts` 시 Latin Modern Math 서브셋 포함 | — |

## 다음 단계

4단계: 수식 레이아웃 정밀화 방안 설계 → `mydocs/tech/equation_layout_spec.md`
