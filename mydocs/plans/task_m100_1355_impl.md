# 구현계획서 — Task #1355: 해설(미주) 세로 여백 누적 → 오버플로

- **이슈**: #1355 (M100)
- **브랜치**: `local/task1355`

## 배경 (코드 경로)

`src/renderer/layout.rs` 메인 배치 루프(2900~3410)에서 미주 제목(문NN) 앞 간격을 제어:

- `should_preserve_endnote_title_gap` (3193~): `prev_endnote_title_gap_px`(20mm급 "미주 사이"
  간격)를 보존해 `min_y = y_before_vpos + gap`로 제목을 내림 → **간격 주입**
- `compact_endnote_title_gap_after_single_equation_tail` (252~): **직전이 단일 수식**
  (`inline_equation_count==1`) & `question>=29` & `ordinal<=13`일 때만 간격 압축

문30은 직전이 **빈 문단**(`count==0`)이라 압축 게이트 미통과 → 보존 경로로 과다 여백.

## 단계 (4단계)

### Stage 1 — 근본원인 계측 (소스 임시 계측, 커밋 제외)

- 임시 디버그 로깅으로 문30(p18) 배치 시 다음 캡처:
  - `prev_endnote_title_gap_px`, `consumed_gap`, 빈 문단(pi=927) 높이/spacing,
    `should_preserve_*`/`compact_*` 분기 진입 여부, 최종 y_offset
- p21→p22 오버플로 케이스도 동일 캡처
- **목표**: +40px 초과분이 (a) 보존 gap 과대 인지 (b) 빈 문단 높이 중복 인지 코드로 특정
- 산출물: `_stage1.md` (계측 결과 + 원인 확정). 임시 계측 코드는 제거.

### Stage 2 — 한컴 정합 기준 + 게이트 설계

- PDF 픽셀 측정으로 정답 gap 산출 (p18 문30 = 45px ≈ 0.35×20mm 추정 검증)
- 일반 tail(빈 문단/텍스트) 뒤 제목 gap 보정 게이트 설계:
  - 기존 `compact_*` 확장 또는 신규 조건부 게이트
  - **전면 통일 금지**, 기존 단일 수식 케이스(#1302) 무영향 보장
- 산출물: `_stage2.md` (설계 + 영향 범위)

### Stage 3 — 구현

- 설계된 조건부 보정 적용 (`layout.rs`)
- 회귀 가드 테스트 추가 (p18 문30 gap, 문24 본문 수용, p21 잔류)
- 산출물: 소스 + 테스트, `_stage3.md`

### Stage 4 — 검증

- p18 문30 gap 45px±5 정합, 문24 본문 내 수용
- p21 `(ⅰ)~(ⅲ)에서` p21 잔류 (p22 미넘김)
- `cargo test` 전체 + 기존 미주 회귀 테스트 통과
- 다른 해설 샘플 시각 회귀 없음
- 산출물: `_stage4.md`, `_report.md`

## 커밋 전략

- Stage별 소스+테스트+보고서를 `local/task1355`에서 커밋
- 기능 변경과 포맷 변경 분리, 무관 rustfmt diff 금지
