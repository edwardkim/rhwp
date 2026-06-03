# Task #1001 구현 계획서 (v2 — Scope 확장)

수행 계획서: [`task_m100_1001.md`](task_m100_1001.md)

## 변경 이력
- v1: 격차 A (페이지 번호 외곽선) + B (페이지 분할) — 5 stage
- **v2**: scope 확장 — 격차 A + B (styling) + C (spacing drift) — 6 stage

## 1. 구현 범위

### 격차 A — 페이지 번호 외곽선 안/밖 ✅ 완료
- `src/renderer/layout.rs` `build_page_borders` bit 1/2 처리

### 격차 B — 변환본 styling 단순화 (Primary 신규)

**진단 대상**:
- 섹션 헤더의 "■ … ■" 장식: 어디서 그려지는가 (paragraph border? char shape? 텍스트 자체?)
- 회색 띠: paragraph background fill? 또는 별도 도형?
- 점선 박스 외곽: paragraph border 의 dash style? table border?
- 타이틀 포함 외곽선: 한컴은 그리고 rhwp 는 그리지 않음 — 외곽선 영역 계산 격차

**Fix 후보** (Stage 4 진단 후 결정):
- 변환본 식별 + styling element 선택적 무효화
- 또는 한컴 simplify 규칙 mimicking

### 격차 C — Paragraph spacing drift

**진단 대상**:
- HWP5 변환본의 ParaShape `spacing_before` 가 HWP3 원본의 2배인 raw data 확인
- HWP5 의 spacing 단위 vs HWP3 의 spacing 단위 (Hwp_unit 변환 규칙)
- Task #998 override 로직 (line_segs 누락 케이스) 의 동작 원리 재확인

**Fix 후보** (Stage 4 진단 후 결정):
- C1: 변환본 식별 + 일관 50% 보정
- C2: Task #998 override 확장 (line_segs 있는 paragraph 도 포함)
- C3: ParaShape 파싱 단위에서 변환본 패턴 정정

## 2. 변환본 식별 신호

격차 B/C 모두 "HWP5 변환본" 만 영향받아야 함 (일반 HWP5 회귀 차단). 식별 신호 후보:
- 파일 metadata (generator, creator 등 — HWP5 header)
- ParaShape 의 spacing_before / line_segs 패턴 (변환본 특유)
- pgbf attr=0x01 + bit 1/2 = 0 의 한컴 default 패턴 (격차 A 와 연결)
- BIN_DATA / styles 의 변환본 시그니처

Stage 4 에서 신호 정확도 측정 + 선정.

## 3. 디버그 인프라

기존:
- `RHWP_DEBUG_PAGE_BORDER` (Stage 3 확장 완료)
- `RHWP_DEBUG_LAYOUT`
- `dump`, `dump-pages`, `ir-diff`

신규 (Stage 4 도입 예상):
- `RHWP_DEBUG_CONVERT_DETECT` — 변환본 식별 결과 출력
- `RHWP_DEBUG_PARA_SPACING` — paragraph spacing_before 의 raw / 정정 후 값 출력

## 4. 구현 단계 (Stage 4-6)

### Stage 4 — 격차 B/C 정밀 진단 (대규모)

**4-1. 격차 B 진단**
- `rhwp dump samples/hwp3-sample16-hwp5.hwp -p 0 -s 0` 으로 페이지 1 paragraph 의 char shape, border_fill 분석
- 같은 영역을 HWP3 원본과 `ir-diff` 로 비교
- 한컴이 변환본 출력 시 무시하는 styling 요소 카탈로그 작성
- HWP3 원본의 styling 정보가 HWP5 로 어떻게 전달되는지 추적

**4-2. 격차 C 진단**
- ParaShape `spacing_before` raw data 비교: HWP3 원본 vs HWP5 변환본
- HWPX 변환본의 `<hp:paraPr>` `<hp:margin>` 비교
- spacing_before 의 단위 (HWPUNIT16) 변환 추적
- Task #998 의 override 로직과 정합/충돌 확인

**4-3. 변환본 식별 신호 결정**
- HWP5 header / file_version / generator 등 metadata 조사
- 변환본 특유 패턴 식별

**Stage 4 산출물**: `mydocs/working/task_m100_1001_stage4.md` — 진단 보고서

### Stage 5 — 격차 B/C Fix 후보 평가 + 적용

**5-1. Fix 후보 평가**
Stage 4 진단 결과로 후보 결정. 각 후보의 회귀 risk + 적용 범위 + 한컴 정합도 평가.

**5-2. Fix 적용**
- 격차 B: 1~3 파일 수정 예상
- 격차 C: 1 파일 수정 예상 (typeset.rs 또는 layout.rs)
- 단위 검증: sample16-hwp5 페이지 1-5 한컴 정합

**Stage 5 산출물**: `mydocs/working/task_m100_1001_stage5.md` — Fix 보고서

### Stage 6 — 회귀 검증 + 시각 판정 + 최종 보고서

**6-1. 단위 / 정적 검증**
- `cargo test --release --lib` 전체 (현재 baseline: 1306 passed)
- `cargo clippy --release -- -D warnings`

**6-2. SVG sweep (회귀 측정)**
| Sample | 검증 항목 |
|--------|----------|
| `hwp3-sample16-hwp5.hwp` | 페이지 1-5 한컴 정합 (격차 A+B+C) |
| `hwp3-sample16.hwp` | 외곽선 + 페이지 번호 유지 (Task #987) |
| 시험지 (`exam_*`) | paper-based + spacing=0 회귀 차단 |
| `aift.hwp` | paper-based + spacing>0 회귀 차단 |
| `biz_plan.hwp`, 통합재정통계, 복학원서 | paper-based 일반 회귀 차단 |
| 일반 HWP5 (변환본 아닌) | styling/spacing 회귀 차단 |

**6-3. WASM 빌드**
- `docker compose --env-file .env.docker run --rm wasm`
- rhwp-studio 실제 렌더링 확인

**6-4. 시각 판정**
- 작업지시자 한컴 정합 시각 판정 (sample16-hwp5 페이지 1, 16, 17 등)

**6-5. 최종 보고서**
- `mydocs/report/task_m100_1001_report.md`
- 격차 A/B/C 각각의 root cause + fix + 회귀 검증 결과
- 잔존 / 후속 권고

## 5. 회귀 risk 매트릭스

| 변경 영역 | 영향 sample | 회귀 risk | 완화책 |
|----------|-----------|----------|--------|
| `build_page_borders` bit 1/2 (Stage 3 완료) | 모든 외곽선 sample | 중 | Stage 6 sweep 검증 |
| 변환본 식별 분기 | HWP5 변환본만 (의도) | 식별 신호 정확도 | Stage 4 정확도 측정 |
| styling element 무효화 (격차 B) | 변환본만 | 일반 HWP5 영향 시 위험 | 식별 신호 + sweep |
| spacing_before 정정 (격차 C) | 변환본만 | Task #998 정합 유지 | line_segs 케이스 분리 처리 |

## 6. 단계별 산출물 / 커밋 단위

각 stage 완료 시:
- `mydocs/working/task_m100_1001_stage{N}.md` + 관련 소스/문서 → 1 커밋
- 작업지시자 승인 후 다음 stage 진행

## 7. 잔존 분리 (Out of Scope)

본 Task 에서 다루지 않을 항목 (필요 시 별도 issue):
- HWPX 변환본 (`samples/hwp3-sample16-hwp5.hwpx`) — 같은 패턴인지 확인만 (Stage 4)
- 다른 HWP3 → HWP5 변환본 sample 들 (HWP3 → HWP5 변환 패턴 일반화는 별도 task)
- HWPX sample18-hwp5 +7 페이지 inflate (별도 기존 issue)
