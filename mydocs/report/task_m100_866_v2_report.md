# 최종 결과 보고서 — Task #866 v2 (M100)

대상: shortcut.hwp 의 페이지내 다단 zone 전환·점선 단 구분선·`<...>` 소제목 zone 간격 정밀화. PR #868 (Task #853/#866) 이후 사용자 보고 잔존 사항 처리.
GitHub Issue: edwardkim/rhwp#866 · 브랜치: `pr-task853`

## 결과 요약

| Stage | 내용 | 결과 |
|---|---|---|
| 1 | pi=94 `<편집 화면 분할에서>` 회귀 수정 (Distribute 다단 마지막 컬럼 `[단나누기]` → `process_multicolumn_break` 로 라우팅) + PDF↔SVG 정밀 측정 | 커밋 `7f3ea171` |
| 2 | 헤더 띠/`<...>` 소제목 zone 전환 간격 +16px (`solo_zone_pad`) | 커밋 `70d793df` |
| 3+4 | zone 별 점선 단 구분선 + `column_break_pad` (배분 다단 마지막 컬럼 단나누기 신규 zone 에 +16px) | 커밋 `4221050f` |

### 핵심 변경
1. **`typeset.rs::paginate_section` (Stage 1)**: 다단 zone 마지막 컬럼 `[단나누기]` 의 새 페이지 강제 push 를 **배분(Distribute) zone 한정** 으로 `process_multicolumn_break` 라우팅 — 같은 페이지 여유 있으면 이전 밴드 아래로, 부족할 때만 새 페이지.
2. **`typeset.rs::process_multicolumn_break` + `layout.rs::build_columns` (Stage 2/4)**: `solo_zone_pad` 게이트 — (a) 1단/간격=0 zone 진입·이탈, (b) `[단나누기]`-induced 새 zone — 둘 중 하나라도 참이면 `+16px(=1200 HU)` 추가 세로 여백.
3. **`layout.rs::emit_zone_column_separators` 신규 (Stage 3)**: page 전역 build_column_separators 가 페이지 내 다단 zone 의 ColumnDef 를 못 보던 결함 정정 — zone_layout.column_areas + zone 별 y 범위 기준 emit. `구분선 type=6/7` → Dot 매핑 확장.

## 시각 정합 (shortcut.hwp ↔ `pdf/basic/shortcut-2022.pdf`)

| 항목 | PR #868 후 | Task #866 v2 후 | 한컴 PDF | 평가 |
|---|---|---|---|---|
| 페이지 수 | 8 | **7** | 7 | ✓ 정합 |
| `LAYOUT_OVERFLOW` | 4 | **1** | — | ✓ 거의 해소 (잔존 1) |
| pi=94 `<편집 화면 분할에서>` 위치 | 4쪽 단독 zone | **3쪽 zone_y≈235** | 3쪽 같은 위치 | ✓ 정합 |
| 본문 2단 zone 점선 단 구분선 | 미렌더 | **렌더 (모든 본문 zone)** | 회색 점선 | ✓ 정합 |
| 3쪽 페이지 구성 | overflow | **`<그림 넣기에서>` 까지** | `<그림 넣기에서>` 까지 | ✓ 정합 |

## 검증
- `cargo test --release` 34 suites / FAILED 0. svg_snapshot 8/8 — 골든 무변경.
- shortcut.hwp 7쪽 SVG 측정 → 점선 구분선 x=561.25, `#aeaeae`, dotted, width 1.9.
- 회귀 보호 검증 (`exam_math.hwp` / `21_언어_기출_편집가능본.hwp` 페이지 수) — Stage 1 의 Distribute 게이트로 신문형 다단 영향 0.

## 잔존 (미수정 — 후속 이슈 권장)

1. **4쪽 `<스타일에서>` zone gap** — 16px solo_pad 적용 후도 한컴 PDF 대비 ~10~20px 부족. 20px 상향 시도 → 페이지 수 7→8, LAYOUT_OVERFLOW 1→18 회귀. 16px 가 회귀 없이 적용 가능한 최대치. 정밀화는 Hancom 편집기 cross-check 또는 transition 유형별 차등 모델 필요.
2. **6쪽 `도구` 띠** — ColumnDef 1단/간격=**1mm**(=3.78px) 로 solo/tac_band 게이트(`< 0.5`) 미발동 → 좁음. 게이트 임계값 `5.0` 상향 시도 → pi=209 가 단독 페이지로 분리 (페이지 수 7→9). 별도 게이트 또는 1단 미사용 spacing 무시 규칙 필요.
3. **`LAYOUT_OVERFLOW` 1건 잔존** — 4쪽 일부 zone 의 본문 미세 초과 (~10~30px). 위 잔존 정밀화 후 자연 해소 가능성.
4. **pi=36 형 헤더 띠 line0 텍스트** — `place_table_with_text` 의 `tac_wrap_split` 이 띠 위 16px 라인에 cell 텍스트("파일" 등) 를 별도 렌더. 한컴 PDF 는 띠 1개만 표시 (220dpi crop 확인). 제거 시 다른 transition 측정 깨질 수 있어 보류.

## 커밋 (브랜치 `pr-task853`)
- `7f3ea171` — Stage 1 pi=94 회귀 수정 + 측정
- `70d793df` — Stage 2 solo_zone_pad +16px
- `4221050f` — Stage 3+4 점선 zone 구분선 + column_break_pad

## 문서
- `mydocs/plans/task_m100_866_v2_impl.md` — 구현 계획서
- `mydocs/working/task_m100_866_v2_stage1.md` ~ `_stage3.md` — 단계별 보고서
- `mydocs/report/task_m100_866_v2_report.md` — 본 최종 결과 보고서
