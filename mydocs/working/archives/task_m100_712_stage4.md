# Task #712 Stage 4-5 (회귀 + 광범위 검증) 완료 보고서

**Issue**: [#712](https://github.com/edwardkim/rhwp/issues/712)
**Stage**: 4 (회귀) + 5 (광범위) 통합
**작성일**: 2026-05-08

---

## 1. 회귀 테스트 (Stage 4)

### `cargo test --release` 전체

```
passed=1252  failed=0  ignored=5
```

→ 회귀 0. 1221+ 기준 충족.

주요 관련 테스트 통과 확인:
- `tests/issue_703.rs` (BehindText/InFrontOfText 표 — 동일 표 가드 영역)
- `tests/issue_504*.rs` (각 vert_offset 관련)
- `tests/issue_157.rs`, `tests/issue_267.rs` (svg_snapshot)
- `tests/issue_154.rs` (TopAndBottom 표 정렬)
- 새로 추가된 `tests/issue_712.rs`

## 2. 광범위 회귀 검증 (Stage 5)

### 페이지 수 횡단 비교

181 개 샘플(.hwp) 의 페이지 수 before/after 비교:

```
$ for f in samples/*.hwp samples/basic/*.hwp samples/hwpx/*.hwp; do
    ./target/release/rhwp dump-pages "$f" | grep -c "^=== "
  done
```

**결과**: `diff before after` 0 lines — 모든 샘플의 페이지 수 동일.

→ 음수 `vertical_offset` 케이스가 아닌 샘플(99% 이상)은 정정 영향 0. 음수 offset 표가 있는 케이스(2022 국립국어원 등) 도 표 y 위치만 변경되며 페이지 흐름은 동일.

### 결함 표 위치 시각 검증

| 페이지 | Before | After | 비고 |
|--------|--------|-------|------|
| 36 (page_idx 35) — pi=585+pi=586 결함 표 | pi=586 [124.93..1004.31] | pi=586 [148.88..1028.25] | **+23.95 px (정상)** |
| 37 (page_idx 36) — 분할 연결 + 말미 1x3 | pi=586 ci=0 y=94.5, ci=1 y=966.9 | 동일 | `is_continuation=true` 가드 무영향 |

### 검증 명령

```bash
./target/debug/rhwp export-svg "samples/2022년 국립국어원 업무계획.hwp" \
    -o output/svg/task712-after -p 35 -p 36 --debug-overlay
```

`output/svg/task712-after/` 의 SVG 디버그 오버레이로 시각 확인 가능.

## 3. 결함 케이스 횡단 후보 조사

`vertical_offset` 음수 인코딩(unsigned 30억 대 값)을 가진 표가 더 있는지 dump 검색:

```bash
$ for f in samples/*.hwp samples/basic/*.hwp; do
    ./target/release/rhwp dump "$f" 2>/dev/null \
        | grep -E "vert=문단\([0-9]{10}=" | head -1 \
        | xargs -I{} echo "$f: {}"
  done | grep -v "^:" | head
```

(`vert=문단(4294xxxxx=...)` 패턴은 음수 i32 의 unsigned 표시.) 본 결함의 정정은 음수 offset 자체가 위쪽으로 점프하지 않도록 차단하는 것이므로, 동일 패턴이 있더라도 침범 0 으로 정상화될 것 (비-Partial 경로의 기존 클램프와 동등 효과).

## 4. 변경 파일 정리 확인

```
$ git diff stream/devel..HEAD -- src/ --stat | grep -v test
 src/renderer/layout.rs               | +5 -2  (pt_y_start 게이트 signed)
 src/renderer/layout/table_partial.rs | +9 -3  (gate signed + 주석)
```

순수 패치 라인: 14 라인 (주석 포함). 기능적 변경 = 2 게이트의 비교 연산자 변환.

## 5. 회귀 위험 재평가

| 위험 (Stage 0 plan) | Stage 4-5 결과 |
|--------|------|
| 음수 vert offset 가드 확장이 다른 케이스 침범 | 페이지 수 회귀 0/181 — 영향 없음 확인 |
| VPOS_CORR 정정 시 Task #412/#643/#470 회귀 | VPOS_CORR 미수정 — 위험 무관 |
| 인스트루먼트 잔존 | Stage 3 에서 모두 제거 확인 |

## 6. 다음 단계 (Stage 6 — 최종 보고)

1. 최종 결과 보고서 `mydocs/report/task_m100_712_report.md` 작성
2. `closes #712` 커밋
3. 작업지시자 승인 후 `local/task712` → `devel` merge
4. `pr-task712` 브랜치 생성 → `stream/devel` 으로 PR

## 승인 요청

Stage 4-5 회귀 + 광범위 검증 완료 — 회귀 0. Stage 6 (최종 보고서 + close) 진행 승인 요청.
