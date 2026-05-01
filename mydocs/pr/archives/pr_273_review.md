# PR #273 검토 — Task #267: right tab 선행 공백 처리 — 경로 1/2/3 통일

## PR 정보

- **PR**: [#273](https://github.com/edwardkim/rhwp/pull/273)
- **이슈**: [#267](https://github.com/edwardkim/rhwp/issues/267)
- **작성자**: @seanshin (Shin hyoun mouk)
- **base/head**: `devel` ← `feature/task267`
- **Mergeable**: ⚠️ CONFLICTING (`mydocs/orders/20260424.md` **문서 충돌만**, 코드는 충돌 없음)
- **검토일**: 2026-04-24

## 변경 요약

`KTX.hwp` 목차 2페이지에서 소제목 페이지 번호가 ~9.33px 밀리는 버그 수정. right tab 처리 코드 3경로의 **선행 공백 처리를 통일**.

### 핵심 변경 (코드 2개 파일)

| 파일 | 변경 | 경로 |
|------|------|------|
| `src/renderer/layout/text_measurement.rs` | +6 -7 | 경로 1 (3곳): EmbeddedTextMeasurer inline_tabs / tab_stops + WasmTextMeasurer tab_stops |
| `src/renderer/layout/paragraph_layout.rs` | +16 -6 | 경로 2 (pending_right_tab_est) + 경로 3 (pending_right_tab_render) |

### 테스트
- `tests/golden_svg/issue-267/ktx-toc-page.svg` 신규 (279줄, KTX.hwp 2페이지)
- `tests/svg_snapshot.rs` 에 `issue_267_ktx_toc_page` 추가
- `samples/KTX.hwp` 샘플 추가

## 루트 원인 분석

right tab 정렬 코드가 **3개 경로로 분산**되어 있고 탭 직후 선행 공백 처리가 불일치:

| 경로 | 케이스 | 코드 위치 |
|------|--------|-----------|
| 1 | 탭이 run 중간 (**소제목**) | `text_measurement.rs` `compute_char_positions` |
| 2 | 탭이 run 끝, 추정 패스 (**장제목**) | `paragraph_layout.rs:809` `pending_right_tab_est` |
| 3 | 탭이 run 끝, 렌더 패스 (**장제목**) | `paragraph_layout.rs:1177` `pending_right_tab_render` |

- **장제목** (경로 2/3): 선행 공백 자동으로 제거됨 → 올바름
- **소제목** (경로 1): 선행 공백이 seg_w 계산에 포함 → ~9.33px 좌편 이동 ❌

한컴 동작: right tab 직후 선행 공백을 **무시**하고 실질 텍스트의 우측 끝을 tab_pos에 맞춤.

## 수정 내역

### 경로 1: text_measurement.rs (3곳)

```rust
// BEFORE
1 => { // 오른쪽
    let seg_w = measure_segment_from(&chars, &cluster_len, i + 1, &char_width);
    x = (tab_target - seg_w).max(x);
}

// AFTER: right tab 분기에서만 leading space skip
1 => {
    let seg_start = {
        let mut s = i + 1;
        while s < chars.len() && chars[s] == ' ' && cluster_len[s] != 0 { s += 1; }
        s
    };
    let seg_w = measure_segment_from(&chars, &cluster_len, seg_start, &char_width);
    x = (tab_target - seg_w).max(x);
}
```

- **right tab (type=1)** 만 seg_start에서 공백 skip
- **center tab (type=2)** 은 기존 `i + 1` 유지 (가운데 정렬 의미 보존)
- 부가: `eprintln![DEBUG_TAB_POS]` 잔류 디버그 코드 제거

### 경로 2/3: paragraph_layout.rs (2곳)

```rust
// BEFORE: run_w 하나로 match
let run_w = estimate_text_width(&run.text, &ts);
match tab_type {
    1 => est_x = tab_pos - run_w,
    2 => est_x = tab_pos - run_w / 2.0,
}

// AFTER: match arm 내에서 각각 계산, right tab만 trim_start
match tab_type {
    1 => {
        let run_w = estimate_text_width(run.text.trim_start(), &ts);
        est_x = tab_pos - run_w;
    }
    2 => {
        let run_w = estimate_text_width(&run.text, &ts);
        est_x = tab_pos - run_w / 2.0;
    }
}
```

## 설계 검증

| 설계 요소 | 평가 |
|----------|------|
| right tab만 공백 skip | ✅ center tab은 기존 동작 보존 — 가운데 정렬 의미 정확 |
| 3경로 모두 동일 규칙 적용 | ✅ 장제목/소제목 결과 통일. 근본 해결 |
| 경로 1 seg_start 로직 | ✅ `chars[s] == ' ' && cluster_len[s] != 0` — 실제 공백 글리프만 skip. cluster 경계 존중 |
| 경로 2/3 trim_start() | ✅ Rust str 표준 — 유니코드 공백도 고려됨 |
| `eprintln![DEBUG_TAB_POS]` 제거 | ✅ 잔류 디버그 코드 정리 |
| Golden SVG | ✅ KTX.hwp 2페이지 전체 스냅샷 — 회귀 감지 가능 |

## 이전 시도 (실패) — 이슈 본문 기록

- **방안 A** (`measure_segment_from` 공백 제거): 경로 1만 개선, 경로 2/3 미적용
- **방안 B** (`trimmed_seg_w` 별도 계산): 경로 1만 적용, 경로 2/3 악화
- **available_width 통일**: 기존 문서 전체 정렬 변동 → revert
- **방안 C (본 PR)**: 3경로 모두 right tab만 공백 skip → 통일 ✅

## 메인테이너 검증 결과

### PR 브랜치 체크아웃 후 검증

| 항목 | 결과 |
|------|------|
| `cargo test --test svg_snapshot` | ✅ **5 passed / 0 failed** (issue_157 + issue_267 포함) |
| `cargo test --lib` 전체 | ✅ **963 passed / 0 failed / 1 ignored** |
| `cargo clippy --lib -- -D warnings` | ✅ clean |
| `cargo check --target wasm32-unknown-unknown --lib` | ✅ clean |

참고: PR 브랜치에 PR #266 커밋 3개가 포함되어 있어 `issue_157_page_1` 도 함께 통과.

## 충돌 분석 (CONFLICTING 상태)

`git merge origin/devel` 시뮬레이션:

- **충돌 파일**: `mydocs/orders/20260424.md` **단 1개 문서 파일**
- **코드 충돌**: 없음 — rust 파일 모두 자동 merge 성공
- **충돌 내용**: 양쪽 모두 "## 2. Task" 섹션을 추가. PR #273은 `Task #267` 섹션, devel은 `Task #280`/`Task #283` 섹션.

### 3-dot diff 검증 (실질 변경)

```
src/renderer/layout/paragraph_layout.rs
src/renderer/layout/text_measurement.rs
tests/svg_snapshot.rs
```

rust 실질 변경 파일이 3개이며, 모두 PR 설명대로 tab 처리 로직만 수정.

### 충돌 해결 방향

orders 문서 충돌은 **단순 순번/섹션 병합**으로 쉽게 해결 가능. 외부 기여자 PR이므로 2가지 선택지:

1. **A. 메인테이너 직접 해결** — orders 문서는 메인테이너 관리 영역으로 볼 수 있음
2. **B. 작성자 리베이스 요청** — CLAUDE.md 절차 엄격 적용

코드 품질·검증이 양호하고 충돌 규모가 문서 단일 파일이므로 **A 방식(admin merge 직접 해결)** 을 권장.

## 리스크 평가

| 리스크 | 판정 |
|--------|------|
| center tab 정렬 회귀 | ✅ type=2 분기는 변경 없음. 기존 동작 보존 |
| 다른 언어 공백(U+00A0 등) 처리 | ✅ 경로 2/3 `trim_start()` 는 유니코드 공백 고려. 경로 1 `chars[s] == ' '` 는 ASCII only지만 HWP 탭 시나리오에서 타당 |
| cluster 경계 위반 | ✅ `cluster_len[s] != 0` 체크로 cluster 중간 잘리지 않음 |
| Golden SVG 외 기존 회귀 | ✅ 기존 svg_snapshot 4건 모두 통과 |
| wasm32 호환 | ✅ WasmTextMeasurer도 동일 수정 |

## 문서 품질

CLAUDE.md 절차 준수:

- ✅ 수행계획서: `mydocs/plans/archives/task_m100_267.md` (이미 archives 이동됨)
- ✅ 구현계획서: `mydocs/plans/archives/task_m100_267_impl.md`
- ✅ 단계별 보고서: `stage1.md` + `stage2_3.md`
- ✅ 최종 보고서: `mydocs/report/task_m100_267_report.md` — **#266과 달리 존재함** 👍
- ✅ orders 갱신: `mydocs/orders/20260424.md` Task #267 섹션

## 판정

✅ **Merge 권장**

**사유:**
1. **3경로 통일** — 장제목/소제목 공통 근본 원인 해결. 이전 시도 A/B가 부분 개선이었다면 C는 근본 해결
2. **수정 범위 일관적** — 5곳 모두 동일 패턴 (right tab만 공백 skip, center tab 보존)
3. **Golden SVG 등록** — KTX.hwp 2페이지 전체 스냅샷으로 회귀 감지
4. 빌드/테스트/clippy/wasm 모두 통과
5. **CLAUDE.md 절차 완전 준수** — 최종 보고서 포함
6. 코드 충돌 없음 — 문서 1개만 충돌

**Merge 전략:**
- Admin merge (--admin) 로 처리 또는 orders 문서 충돌만 직접 해결
- 시각 검증: WASM 빌드 → `samples/KTX.hwp` 2페이지 소제목 페이지 번호 정렬 확인
