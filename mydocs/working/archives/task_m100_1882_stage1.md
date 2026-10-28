# task_m100_1882 Stage 1 완료 보고서 — 갭② 팔레트

- 이슈: #1882 (C1c, #1431 Track C)
- 브랜치: `local/task1882`
- 단계: Stage 1 / 5 — 한컴 2022 기본 팔레트 교체 (`src/ooxml_chart/renderer.rs`)

## 변경 내용

1. **`DEFAULT_PALETTE` 교체** (renderer.rs): 녹색-우선(0x70AD47…) → 한컴 2022 실측 팔레트.
   - `#6183D7`(파랑) → `#FE813B`(주황) → `#B0B0B0`(회색) → `#FCD801`(노랑) — **실측**
     (`pdf/chart/` 정답지 PDF 픽셀 히스토그램: 막대 3시리즈 + 원형 4슬라이스).
   - 5~6번째 `#5B9BD5`(하늘)/`#70AD47`(초록) — **유추** (코퍼스에 4시리즈 초과 샘플 없음),
     7~8번째 기존 유지. 주석에 실측/유추 구분 명기.
2. **`scheme_color`(parser.rs) 의도적 무변경** — schemeClr는 문서 테마 참조 의미(코퍼스 미사용),
   변경 시 기존 테스트가 근거 없이 깨짐 (구현계획서 §2 Stage 1 승인 사항).

## 테스트 (TDD)

신규 `test_default_palette_hancom_order` — 색 미지정 3시리즈 렌더 → `#6183d7`/`#fe813b`/`#b0b0b0`
등장 순서 단언 + `#70ad47`(구 1순위) 미포함. **팔레트 교체 전 실행하여 실패 확인 후 교체 → 통과.**

## 검증

```
cargo test --lib ooxml_chart                              → 33 passed (기존 32 + 신규 1), 0 failed
cargo test --test issue_1431_scatter                      → 1 passed  (placeholder 0건 무회귀)
cargo test --test issue_1453_chart_3d_ofpie_routing       → 2 passed  (라우팅·percent 축 무회귀)
```

**시각 확인** (`output/poc/chart_c1c/stage1/묶은세로막대형.svg` ↔ `pdf/chart/세로막대형/묶은세로막대형-2022.pdf`):
시리즈 1/2/3이 정답지와 동일한 파랑/주황/회색으로 렌더. 잔여 차이는 제목 없음(Stage 3),
범례 하단(Stage 4), Y축 0~5(Stage 2 — 한컴 0~6 라벨 0,2,4,6)로 **모두 후속 단계 대상과 일치**.

## 다음 단계

Stage 2 — 갭④ 축 스케일: `nice_range`→`nice_axis`(min,max,step) + 경계 headroom + step 기반 눈금.
