# Stage 4 단계별 보고서 — Task #554

> **이슈**: [#554](https://github.com/edwardkim/rhwp/issues/554)
> **단계**: Stage 2-3 — 회귀 검증 + 신규 회귀 테스트
> **상태**: 완료, 작업지시자 승인 대기
> **작성일**: 2026-05-03

---

## 1. 신규 회귀 테스트 (`tests/issue_554.rs`)

### 1.1 테스트 12건

| 테스트 | 검증 내용 |
|--------|----------|
| `hwp3_sample4_hwp5_36p` | HWP5 변환본 36p 정합 |
| `hwp3_sample5_hwp5_64p` | HWP5 변환본 64p 정합 |
| `hwp3_sample5_hwpx_64p` | HWPX 변환본 64p 정합 (hwpml=1.4) |
| `hwp3_sample_hwp5_15p_known_limit` | 잔존 -1 over-correct 회귀 가드 |
| `hwp3_sample_hwpx_15p_known_limit` | 잔존 -1 over-correct 회귀 가드 |
| `hwp3_sample_hwp3_16p` | HWP3 원본 회귀 0 |
| `hwp3_sample5_hwp3_64p` | HWP3 원본 회귀 0 |
| `task554_no_regression_2022_kuglip` | 단순 -1600 적용 시 -5 회귀였던 케이스 회피 |
| `task554_no_regression_exam_kor` | 휴리스틱 임계값 근처 케이스 안전 분리 |
| `task554_no_regression_aift` | 단순 적용 시 -1 회귀였던 케이스 회피 |
| `task554_no_regression_2025_donations_hwpx` | hwpml=1.5 직접 작성본 미적용 |
| `task554_no_regression_exam_science` | Task #546 정합 (4 페이지) |

### 1.2 결과

```
test result: ok. 12 passed; 0 failed; 0 ignored
```

## 2. 회귀 검증

### 2.1 자동 테스트

| 검사 | 결과 |
|------|------|
| `cargo test --lib` | **1113 passed** (회귀 0) |
| `cargo test --test issue_554` | **12 passed** (신규) |
| `cargo test --test svg_snapshot` | 6/6 passed |
| `cargo test --test issue_546` | passed (Task #546 정합) |
| `cargo test --test issue_418/501/505/530` | 모두 passed |
| `cargo clippy --lib -- -D warnings` | **0건** |

### 2.2 광범위 fixture sweep (`samples/*.hwp{,x}`)

총 80+ fixture 중 한컴 정답 확인된 fixture 검증:

| 파일 | rhwp 결과 | 한컴 정답 | 평가 |
|------|----------|----------|------|
| hwp3-sample.hwp | 16 | 16 | ✅ |
| hwp3-sample4.hwp | 39 | 36 | ⚠️ HWP3 자체 회귀 (Task #554 범위 밖) |
| hwp3-sample5.hwp | 64 | 64 | ✅ |
| hwp3-sample-hwp5.hwp | 15 | 16 | ⚠️ -1 잔존 |
| hwp3-sample4-hwp5.hwp | 36 | 36 | ✅ |
| hwp3-sample5-hwp5.hwp | 64 | 64 | ✅ |
| hwp3-sample-hwpx.hwpx | 15 | 16 | ⚠️ -1 잔존 |
| hwp3-sample5-hwpx.hwpx | 64 | 64 | ✅ |
| **hwp-3.0-HWPML.hwp** | **122** | **122** | ✅ (작업지시자 확인) |

다른 80+ fixture 모두 baseline과 동일 (휴리스틱 false positive 0).

## 3. Task #554 진전 요약

### 3.1 정합 달성 (5건)

- hwp3-sample4-hwp5.hwp: 38 → **36** ✓
- hwp3-sample5-hwp5.hwp: 68 → **64** ✓
- hwp3-sample5-hwpx.hwpx: 68 → **64** ✓
- hwp-3.0-HWPML.hwp: 변동 없이 **122** ✓ (휴리스틱 false positive 회피)
- HWP3 원본 (sample, sample5): 보정 충돌 없음 ✓

### 3.2 잔존 (2건, 의도된 trade-off)

- hwp3-sample-hwp5.hwp: 17 → 15 (정답 16, **-1 over-correct**)
- hwp3-sample-hwpx.hwpx: 17 → 15 (정답 16, **-1 over-correct**)

**원인**: 단일 -1600 HU 보정의 본질적 한계. 페이지 break 알고리즘이 줄 단위로 결정되어 -400 ~ -1600 범위 모든 보정값이 동일한 효과 (Stage 2-2 sweep 진단). sample은 -1, sample4/5는 -2/-4 만큼 줄어들어야 하는데 단일 보정은 모두 동일 줄 수 흡수.

**해결**: typeset.rs 페이지 break 알고리즘 정밀화 (별도 task).

### 3.3 별도 issue 권고

- **별도 task A**: typeset.rs 마지막 줄 tolerance 본질 구현 (sample -1 over-correct 정밀화)
- **별도 task B**: hwp3-sample4.hwp HWP3 자체 회귀 (39 vs 36, +3) — Task #554 범위 밖

## 4. 코드 변경 누계 (Stage 1 ~ Stage 2-3)

| 파일 | 변경 |
|------|------|
| `src/parser/hwpx/header.rs` | +28 LOC (`parse_hwpx_hwpml_version` 신규) |
| `src/parser/hwpx/mod.rs` | +13 LOC (HWPX 휴리스틱 + 보정) |
| `src/parser/mod.rs` | +37 LOC (`apply_hwp3_origin_fixup` 신규 + 두 진입점에 호출) |
| `tests/issue_554.rs` | +120 LOC (12 회귀 테스트) |
| `mydocs/plans/task_m100_554{,_impl}.md` | 수행/구현 계획서 |
| `mydocs/working/task_m100_554_stage{1,2,3,4}.md` | 단계별 보고서 |
| `src/bin/check_compat.rs` | 임시 진단 도구 (Stage 2-4 에서 처리) |

## 5. 작업지시자 승인 요청

본 Stage 2-3 보고서 검토 후:

1. **회귀 테스트 (`tests/issue_554.rs`) 12건 승인**
2. **광범위 회귀 검증 결과 승인** (80+ fixture 중 hwp-3.0-HWPML 등 정합 확인)
3. **Stage 2-4 (정리 + 최종 보고서) 진행 승인**:
   - `src/bin/check_compat.rs` 처리 결정 (삭제 vs 정식 명령 통합)
   - `mydocs/orders/20260503.md` 갱신
   - `mydocs/report/task_m100_554_report.md` 작성

승인 후 Stage 2-4 (최종) 진행한다.
