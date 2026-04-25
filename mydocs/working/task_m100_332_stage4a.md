# Task #332 Stage 4a — typeset fit 검사에 layout drift 안전 마진 도입 — 완료보고서

- **계획서**: `mydocs/plans/task_m100_332_impl.md`
- **브랜치**: `task332`
- **작성일**: 2026-04-25

---

## 변경 사항

### 코드 (`src/renderer/typeset.rs`)

```diff
     ) {
-        let available = st.available_height();
+        // Task #332 Stage 4a: layout drift 안전 마진.
+        // typeset 의 fit 추정과 layout 의 실측 진행은 폰트 메트릭/표 측정 다중성 등으로
+        // 미세하게 어긋날 수 있다 (~수 px). 마진을 빼서 보수적으로 fit 을 판정해
+        // layout 시점의 LAYOUT_OVERFLOW (clamp pile 트리거) 를 사전 차단한다.
+        const LAYOUT_DRIFT_SAFETY_PX: f64 = 15.0;
+        let available = (st.available_height() - LAYOUT_DRIFT_SAFETY_PX).max(0.0);
         ...
-        let base_available = st.base_available_height();
+        // Task #332 Stage 4a: partial split 시에도 동일 마진 적용
+        let base_available = (st.base_available_height() - LAYOUT_DRIFT_SAFETY_PX).max(0.0);
```

`typeset_paragraph()` 함수 진입 시 `available` 과 partial split 진입 후 `base_available` 모두 마진 차감. 마진 값 15px 은 21_언어 의 9.5px overflow + 첫 줄 line_height 14.7px 안전성 여유.

## 검증 결과

### 자동 테스트

```
cargo test --lib                  → 992 passed
cargo test --test svg_snapshot    → 4 passed, 2 FAILED (Stage 2 와 동일 baseline 차이)
cargo test --test '*' (기타)      → 모두 passed
```

### 21_언어 page 0 effect

`dump-pages` 결과:

| 단계 | 단 0 의 pi=10 partial 분배 | 단 0 used |
|------|---------------------------|-----------|
| Stage 3b | lines 0..3 (3 lines) | 1167.7px |
| Stage 4a | lines 0..2 (2 lines) | 1202.7px |

partial split 이 보수적으로 1 line 줄어듦 → typeset 결과는 더 정합. 그러나 LAYOUT_OVERFLOW 9.5px 잔존:

```
LAYOUT_OVERFLOW: page=0, col=0, para=10, type=PartialParagraph, y=1445.7, bottom=1436.2, overflow=9.5px
```

원인 분석: typeset 결과는 단 0 에 lines 0..2 (= 0, 1) 만 분배했지만 layout 이 line 2 까지 그리려 함. layout 의 partial 처리 로직이 typeset 의 `end_line` 결정을 정확히 반영하지 않아 line 2 가 두 군데(단 0 끝 + 단 1 시작) 모두 렌더되는 결함. **Stage 4b 의 stop drawing 정책이 단 0 의 line 2 렌더를 차단해 자연 해결.**

### 다른 샘플 회귀

| 샘플 | OVERFLOW | 비고 |
|------|----------|------|
| form-01 | 0 | 정상 |
| hwp-multi-002 | page=2 Table 31.3px | pre-existing (Stage 3b 와 동일) |
| multi-table-001 | 0 | 정상 |
| lseg-06-multisize | 0 | 정상 |
| aift | 3 건 (Table/PartialTable) | pre-existing (Stage 5 의 header 측정 통합으로 다룰 예정) |

회귀 없음.

## 다음 단계

Stage 4b: `paragraph_layout.rs:807-816, 2529-2542` 의 clamp pile 분기를 stop drawing + warn log 로 변경. typeset 의 보수적 분배 + layout 의 stop drawing 조합으로 글자 겹침 자체 차단.

## 미해결

- 마진 15px 의 적정성: pages 수 증가/표 위치 변동 가능성 → Stage 5 까지 누적 변화 측정 후 조정
- Table overflow (hwp-multi-002, aift) 는 본 마진 효과 밖 → Stage 5 에서 표 매니저 정합으로 해결
