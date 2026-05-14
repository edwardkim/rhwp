# Task #901 구현 계획서 v2 — Runtime Wrap Engine

**이슈**: [edwardkim/rhwp#901](https://github.com/edwardkim/rhwp/issues/901)
**v1 계획서**: [task_m100_901_impl.md](task_m100_901_impl.md)
**Scope 확장**: 파일 포맷에 wrap zone 미인코딩된 paragraph 의 runtime wrap 계산

## 1. v1 → v2 전환 사유

### v1 완료 분
- Stage 1: paragraph 0 wrap zone cs/sw 분기 조건 확장 (커밋 `bac1dca5`)
- Stage 2+3: typeset wrap_anchor cs_only_match 추가 (커밋 `d973e9ff`)
- paragraph 0 "우/리/나/라" + paragraph 1 "대한민국" 한컴 정합

### v1 미해결
- paragraph 7 ("SK하이닉스가...") + 이후 본문 paragraph 들이 무용수 그림 (57.8×84mm Square wrap @ paper 122.2mm,164.0mm) 영역 침범

### 리버스 엔지니어링 결과 (mydocs/working/task_m100_901_stage2.md + 본 v2 분석)
- HWP/HWPX 두 포맷 모두 paragraph 7 의 wrap zone 미인코딩
- line_segs cs=0 sw=42520 (전체 폭 fallback)
- pic2.hwp paragraph 7 picture h_offset+width = 51024 > body_w 42520 (오버플로우 edge case)
- 한컴 편집기조차 이 case 에서 wrap zone 계산을 fallback 처리 → 파일에 전체 폭만 저장
- HWPTAG_PARA_LINE_SEG flags bit 에 wrap zone 표시 비트 없음 (한글문서파일형식_5.0 표 62)
- 한컴 뷰어는 **runtime wrap geometry engine** 으로 매 paragraph layout 시 picture collision + wrap zone 동적 계산

**결론**: 파일 포맷만으로는 paragraph 7 fix 불가. rhwp 에 runtime wrap engine sub-system 신규 구현 필요.

## 2. v2 Stage 분할

### Stage 4 — Runtime wrap engine 설계 + 최소 구현
**목표**: paragraph 가 자신의 Square wrap picture 를 보유하고 line_seg 가 wrap zone 미인코딩 (cs=0 sw=full) 상태일 때, runtime 에서 wrap zone 을 계산하여 적용.

**진단 항목**:
- [ ] 영향받는 paragraph 식별 알고리즘 (cs=0 sw=full + 자신의 Square wrap picture + line vpos overlap)
- [ ] picture body 좌표 변환 (paper→body coords, vert_rel/horz_rel 처리)
- [ ] line vpos 와 picture vertical body range overlap 검사

**최소 구현 영역**:
- `src/renderer/composer.rs` `compose_lines`: 신규 helper 도입. paragraph 자신의 Square wrap picture 검출 → 가상 wrap zone (cs_virtual, sw_virtual) 계산 → 영향 라인에 적용. 단 font metric 미보유로 text re-break 어려움 → 우선 effective_col_w 만 반환.
- `src/renderer/layout/paragraph_layout.rs`: composer 가 제공한 가상 wrap zone 으로 effective_col_x/w 재계산. 단 text re-break 안하면 chars overflow 우려 → 1 line 텍스트만 영향, 나머지 라인 자동 wrap 안됨.

**위험**: text re-break 없이 effective_col_w 만 좁히면 chars 가 그림 영역에 침범. 의미있는 시각 정합 미달성.

**산출물**: `mydocs/working/task_m100_901_stage4.md`

### Stage 5 — Text re-break + line synthesis
**목표**: 영향받는 paragraph 의 line_segs 를 runtime 에 재분할 (font metric 기반 char width).

**구현 영역**:
- `src/renderer/composer.rs`: paragraph 의 ComposedLine 들을 wrap zone 폭으로 re-wrap. 단 font metric 접근 필요 → composer signature 확장 (font ctx 인자 추가, 모든 호출자 영향).
- `src/renderer/typeset.rs`: paragraph layout 호출 전에 wrap zone 가산 (단계별 vpos 누적 의존성 변화).
- `src/renderer/layout/paragraph_layout.rs`: 가상 추가 라인 emit.

**위험**: composer signature 변경이 매우 큰 영향 (HWP3/HWP5/HWPX 모든 포맷, 1402 test 대다수). 단계 격리 어려움.

**산출물**: `mydocs/working/task_m100_901_stage5.md`

### Stage 6 — 회귀 검증
- [ ] cargo test --release --all-targets: 1402 passed 유지
- [ ] 모든 sample SVG visual 회귀 점검 (특히 wrap=Square 사용 sample)
- [ ] golden SVG 회귀
- [ ] pic2.hwp 페이지 1 한컴 PDF 시각 정합 (paragraph 7+ 본문)
- [ ] pic2.hwp 페이지 2 정합 유지

**산출물**: `mydocs/working/task_m100_901_stage6.md`

### Stage 7 — 통합 + 최종 보고서 + PR
- [ ] 최종 보고서 작성
- [ ] PR 생성 (작업지시자 승인 후)
- [ ] issue #901 회신

**산출물**: `mydocs/report/task_m100_901_report.md`

## 3. 위험 평가

| Stage | 위험 | 완화 |
|-------|------|------|
| 4 | runtime wrap zone 산출 알고리즘 정확도 | 다양한 picture position + wrap 조합 회귀 점검 |
| 5 | composer signature 확장 → 광범위 영향 | font ctx 인자 default fallback 처리, 점진적 마이그레이션 |
| 5 | line synthesis 시 vpos 누적 변화 → 페이지 수 회귀 | 모든 sample 페이지 수 검증 |
| 6 | 회귀 sample 다수 | Stage 별 commit + 단위 회귀 점검 |

## 4. 시간 추정

| Stage | 작업량 |
|-------|--------|
| 4 (설계 + 최소) | 1~2일 |
| 5 (text re-break) | 2~3일 |
| 6 (회귀 검증) | 1일 |
| 7 (통합 + PR) | 0.5일 |
| **합계** | **4.5~6.5일** |

## 5. Fallback 시나리오

Stage 5 진행 중 회귀가 심각하면:
- v1 분 (Stage 1 + Stage 2+3) 만 유지 + PR 생성 → #901 partial fix 마무리
- 나머지 (paragraph 7+ wrap) 는 별도 이슈 #903 으로 분리하여 더 큰 설계 기간 확보

## 6. 의사결정 요청

본 v2 implementation plan 자체 승인. 승인 시 Stage 4 (runtime wrap engine 설계 + 최소 구현) 진행.
