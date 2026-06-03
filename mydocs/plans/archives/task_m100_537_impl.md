# Task #537 구현 계획서

**제목**: 21_언어_기출.hwp TAC 표 직후 첫 문단의 trailing line_spacing 누락으로 줄간격 좁아짐
**브랜치**: `local/task537`
**이슈**: https://github.com/edwardkim/rhwp/issues/537
**수행계획서**: `mydocs/plans/task_m100_537.md`

---

## 1. 변경 대상 파일

| 파일 | 변경 내용 | 예상 LOC |
|------|----------|---------|
| `src/renderer/layout.rs` | `vpos_lazy_base` 산출 로직 수정 (1497-1507) — IR vpos 직접 anchor 우선 | +15 / -8 |
| `src/renderer/layout/tests.rs` 또는 `integration_tests.rs` | lazy_base anchor 단위 테스트 추가 | +50 (신규) |
| `mydocs/working/task_m100_537_stage{1,2,3}.md` | 단계별 보고서 | 신규 |
| `mydocs/report/task_m100_537_report.md` | 최종 보고서 | 신규 |

**중요**: `paragraph_layout.rs` 의 `is_full_paragraph_end` trailing-ls 제외 로직(2645-2654, Task #479)과 `prev_tac_seg_applied` 가드(layout.rs:1434)는 **건드리지 않는다**. lazy_base anchor 가 sequential drift 를 무효화하는 방식으로 우회.

## 2. 코어 변경 설계 (Stage 2)

### 2.1 현재 코드 (`src/renderer/layout.rs:1490-1509`)

```rust
let (base, is_page_path) = if let Some(b) = vpos_page_base {
    (b, true)
} else if let Some(b) = vpos_lazy_base {
    (b, false)
} else {
    // 지연 보정: 첫 보정 시점에서 기준점 산출
    // sequential y_offset에서 역산하여 기준 vpos 결정
    let y_delta_hu = ((y_offset - col_area.y) / self.dpi * 7200.0).round() as i32;
    let lazy_base = prev_vpos_end - y_delta_hu;
    if lazy_base < 0 {
        (prev_vpos_end, false)
    } else {
        vpos_lazy_base = Some(lazy_base);
        (lazy_base, false)
    }
};
```

### 2.2 변경안 (A안)

`prev_pi` 가 신뢰 가능한 IR vpos 기준점을 가지면(즉 prev_pi 의 first seg vpos > 0 OR prev_pi 가 column 첫 항목), **그 first vpos** 를 `lazy_base` 로 직접 사용. sequential drift(즉 trailing-ls 제외로 인한 716 HU 누락) 를 lazy_base 에 반영하지 않는다.

```rust
let (base, is_page_path) = if let Some(b) = vpos_page_base {
    (b, true)
} else if let Some(b) = vpos_lazy_base {
    (b, false)
} else {
    // [Task #537] lazy_base anchor: prev_pi 의 first_vpos 가 신뢰 가능하면
    // sequential drift 를 무시하고 IR vpos 절대 좌표를 그대로 사용.
    // sequential y_offset 역산 방식은 paragraph 내부의 trailing-ls 제외(Task #479)
    // drift 를 base 에 동결시켜, 후속 paragraph 모두를 1 ls(=716 HU) 만큼
    // 위쪽으로 시프트하는 부작용이 있음.
    let prev_first_vpos = paragraphs.get(prev_pi)
        .and_then(|p| p.line_segs.first())
        .map(|s| s.vertical_pos)
        .filter(|&v| v >= 0);
    let prev_first_y = prev_first_vpos
        .and_then(|_| {
            // prev_pi 의 line[0] 이 실제 렌더된 y 를 직접 알 수는 없으므로
            // 안전한 anchor 후보: prev_pi 의 first_vpos 만 가져오고,
            // anchor 식은 col_anchor_y(= col_area.y at base) 가 col_area.y 와 동일하다는
            // 가정 하에 사용. (lazy_path 는 col_area.y 기준)
            prev_first_vpos
        });

    let y_delta_hu = ((y_offset - col_area.y) / self.dpi * 7200.0).round() as i32;
    let fallback = prev_vpos_end - y_delta_hu;

    let lazy_base = match prev_first_y {
        // prev_pi 의 first vpos 를 anchor 로 직접 사용 (drift-free)
        // 단, 그 anchor 가 negative 가 되지 않게 검증
        Some(b) if b >= 0 && b <= prev_vpos_end => b,
        _ if fallback >= 0 => fallback,
        _ => prev_vpos_end,
    };
    vpos_lazy_base = Some(lazy_base);
    (lazy_base, false)
};
```

### 2.3 정합성 분석

`end_y = col_area.y + (vpos_end - lazy_base)/75`

- 현재 코드: `lazy_base = 716` (drift) → end_y 가 IR_절대 − 9.55 px 위치
- A안: `lazy_base = pi=38.first_vpos = 40716` → end_y = col_area.y + (vpos_end − 40716)/75

A안에서 검증:
- pi=39 vpos_end = 46164 → end_y = 209.76 + (46164 − 40716)/75 = 209.76 + 72.64 = **282.40**

…잠깐, 이 값은 col_area.y 기준이지만 pi=38 line 0 이 209.76 + (40716 − base)/75 = 209.76 + 0 = 209.76 이 되어 모순.

**수정**: lazy_path 는 col_area.y 가 vpos=lazy_base 위치라고 가정한다. 그러나 col_area.y(=209.76) 는 실제로는 pi=30 line 0 (vpos=0) 위치이지 pi=38 (vpos=40716) 위치가 아님.

따라서 A안의 anchor 는 **prev_pi.first_vpos** 가 아니라, **column 의 첫 paragraph 의 first_vpos 와 col_area.y 의 관계** 를 유지해야 한다. 즉 lazy_path 에서 `col_area.y ≡ vpos = lazy_base` 가 invariant.

### 2.4 수정안 (A'안 — 더 정확)

drift 의 본질: `y_delta_hu` 계산 시 `y_offset` 이 trailing-ls 제외로 716 부족 → `lazy_base = prev_vpos_end - (vpos_delta - 716) = correct_base + 716`.

해법: `y_delta_hu` 산출 시 prev_pi 의 trailing-ls 만큼 보정.

```rust
let trailing_ls_hu = paragraphs.get(prev_pi)
    .and_then(|p| p.line_segs.last())
    .map(|s| s.line_spacing.max(0))
    .unwrap_or(0);
let y_delta_hu = ((y_offset - col_area.y) / self.dpi * 7200.0).round() as i32
    + trailing_ls_hu;  // ← sequential drift 보정
let lazy_base = prev_vpos_end - y_delta_hu;
```

**검증**:
- pi=39 처리 시: y_offset = 815.73, col_area.y = 209.76 → raw_delta_hu = 45448
- prev_pi(=38) trailing_ls = 716 → y_delta_hu = 46164
- prev_vpos_end (pi=38 last seg) = 44348 + 1100 + 716 = 46164
- lazy_base = 46164 − 46164 = **0** ✓
- 이후 end_y(pi=39) = col_area.y + (46164 − 0)/75 = 209.76 + 615.52 = **825.28 px**
- 그 baseline = 825.28 + 12.47 = **837.75 px** (= IR 정확값)

✓ drift 해소.

다른 시나리오 검증:
- prev_pi 가 trailing-ls 가 0 (Task #479 와 무관) → 보정 없음, 기존 동작 유지.
- prev_pi 가 빈 문단 (line_segs 없음) → trailing_ls=0, 기존 동작.
- prev_pi 가 여러 줄 → 마지막 줄만 trailing → 정확.

### 2.5 채택안

**A'안**: trailing_ls 만큼 y_delta_hu 보정. 패치 LOC ≈ +5 / −0.

```rust
let trailing_ls_hu = paragraphs.get(prev_pi)
    .and_then(|p| p.line_segs.last())
    .map(|s| s.line_spacing.max(0))
    .unwrap_or(0);
let y_delta_hu = ((y_offset - col_area.y) / self.dpi * 7200.0).round() as i32
    + trailing_ls_hu;
let lazy_base = prev_vpos_end - y_delta_hu;
```

## 3. 단계 분할

### Stage 1 — Baseline 캡처 + 단위테스트 추가

목표: 현재(buggy) 동작과 회귀 검증 기준선을 코드 레벨에서 고정.

작업:
1. `samples/21_언어_기출_편집가능본.hwp` 의 페이지 2, 3, 5, 6, 8, 9, 그리고 23/24/27/29번 문제 포함 페이지 SVG 를 `output/svg/task537_baseline/` 에 보존 (git 추적 X, 별도 보관)
2. `RHWP_VPOS_DEBUG=1` 출력에서 `base=716`(또는 그에 준하는 drift) 가 발생하는 paragraph index 목록 확보
3. 단위테스트 추가 위치 결정:
   - 후보: `src/renderer/layout/tests.rs` 또는 새 파일 `src/renderer/layout/lazy_base_tests.rs`
   - 케이스: 가짜 column 에 (TAC-like trailing-ls=716 paragraph) → (next paragraph) 시퀀스를 구성해 lazy_base 결과가 0 이어야 함을 확인
4. **현재(미수정) 상태에서 실패하는 테스트** 를 작성 → Stage 2 의 수정으로 통과되도록 함 (TDD)
5. Stage 1 보고서: `mydocs/working/task_m100_537_stage1.md`
6. 커밋 후 승인 요청

### Stage 2 — A'안 적용 + 11곳 수정 확인

목표: 코드 패치 적용 + 본 task 직접 대상 11곳 모두 수정 확인.

작업:
1. `src/renderer/layout.rs:1497-1509` 패치 적용 (위 2.5절)
2. Stage 1 단위테스트 통과 확인
3. `samples/21_언어_기출_편집가능본.hwp` 다시 SVG 출력 → 11곳 모두 `① → ② gap` 이 IR vpos delta 와 일치하는지 정량 확인
4. `RHWP_VPOS_DEBUG=1` 출력에서 `base=0`(또는 base 가 drift 없는 값) 으로 변경되었는지 확인
5. 한컴 PDF (`samples/21_언어_기출_편집가능본.pdf`, `21_언어_기출_편집가능본-2010.pdf`, `21_언어_기출_편집가능본-2020.pdf`) 와 시각 비교 (200dpi)
6. 두 한컴 환경이 다르면 별도 issue 등록 후 작업지시자 결정 요청
7. Stage 2 보고서: `mydocs/working/task_m100_537_stage2.md`
8. 커밋 후 승인 요청

### Stage 3 — 광범위 회귀 검증 + 최종 보고

목표: 다른 샘플의 회귀 없음 보장.

작업:
1. `cargo test` 전체 통과 확인
2. `cargo clippy --all-targets -- -D warnings` 통과 확인
3. 회귀 검증 샘플 SVG 출력 + 한컴 PDF 비교:
   - `samples/synam-001.hwp` (TAC 표 다수)
   - `samples/복학원서.hwp` (BehindText/Square wrap)
   - 추가: `samples/` 디렉터리에서 표/그림/그림자/다단을 포함한 샘플 5~8 종 무작위 선정
4. `dump-pages` 출력에서 column `diff` 가 본 task 대상 페이지 (21_언어 페이지 2,3,5,6,8,9 포함) 에서 0 으로 수렴하는지 확인 (수정 전 −123 px 이던 페이지 2 col 0 의 diff 가 0 에 수렴하면 강한 증거)
5. 회귀 발견 시:
   - 회귀 위험 룰 적용: 수정 범위 축소 검토, 또는 변경 가드 조건 추가 후 재검증
   - 회귀가 다른 본질적 버그를 노출시킨 경우 별도 issue 등록 후 결정
6. 최종 보고서: `mydocs/report/task_m100_537_report.md`
   - 수정 전후 정량 비교 (gap 측정 표)
   - 단위테스트 결과
   - 회귀 검증 결과 (샘플별 OK/문제)
   - 한컴 2010/2020 환경 비교 결과
7. `mydocs/orders/{yyyymmdd}.md` 갱신 (해당 날짜 파일이 있으면)
8. 커밋 후 승인 요청 → merge 시점은 작업지시자 결정

## 4. 커밋 단위

각 단계마다 1~3 커밋:
- Stage 1: "Task #537 Stage 1: lazy_base drift baseline + 단위테스트 추가"
- Stage 2: "Task #537 Stage 2: lazy_base trailing-ls 보정 (A'안 적용)"
- Stage 3: "Task #537 Stage 3: 광범위 회귀 검증 + 최종 보고서"

각 커밋 메시지에 `closes #537` 는 Stage 3 마지막 커밋(또는 merge 커밋)에만 포함.

## 5. 롤백 전략

- 각 단계 끝에 git tag (`task537-stage{N}-checkpoint`) 부여
- Stage 3 회귀 발견 시 Stage 2 의 패치만 revert 하여 lazy_base anchor 결정 로직 재설계 가능
- 최종 merge 후 회귀 발견 시 `local/task537_v2` 브랜치로 후속 수정

## 6. 검증 명령 모음

```bash
# 빌드
cargo build --release

# 본 task 대상 페이지 SVG
./target/release/rhwp export-svg samples/21_언어_기출_편집가능본.hwp -o /tmp/diag537 -p 1
./target/release/rhwp export-svg samples/21_언어_기출_편집가능본.hwp -o /tmp/diag537

# vpos 디버그
RHWP_VPOS_DEBUG=1 ./target/release/rhwp export-svg \
  samples/21_언어_기출_편집가능본.hwp -o /tmp/diag537 -p 1 2>&1 | grep VPOS_CORR

# column diff 확인
./target/release/rhwp dump-pages samples/21_언어_기출_편집가능본.hwp -p 1 2>&1 | grep "단 0"

# 회귀 검증
cargo test
cargo clippy --all-targets -- -D warnings
./target/release/rhwp export-svg samples/synam-001.hwp -o /tmp/diag537/synam
./target/release/rhwp export-svg samples/복학원서.hwp -o /tmp/diag537/bokhak
```

---

**작업지시자 승인 요청 사항**:
1. A'안 (trailing_ls 보정) 채택 — Stage 2 코드 패치 (위 2.5절) 의 정확성
2. 단위테스트 위치/방식 (TDD: Stage 1 에서 실패 → Stage 2 에서 통과)
3. Stage 3 회귀 검증 샘플 범위 (synam-001, 복학원서 + 5~8 종 무작위)
4. 커밋/태그 전략

승인 후 Stage 1 부터 시작합니다.
