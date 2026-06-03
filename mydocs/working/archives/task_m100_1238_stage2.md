# Stage 2 진행 보고서 — Task #1238: min-gap 클램프 (pagination 동기화 미완)

- **이슈**: #1238 (M100)
- **브랜치**: `feature/issue-1238-between-notes-margin`
- **단계**: Stage 2 / 3 (구현 + 검증, **부분 성공·미완**)
- **작성일**: 2026-06-02
- **선행**: `plans/task_m100_1238_impl_v2.md` (min-gap 모델), `tech/between_notes_multiline_1238.md` §7

## 1. 구현 내용 (render 측 min-gap 클램프)

- **typeset.rs**: between-notes(`between_notes > 0`) 경계마다 **새 미주 첫 문단의 local idx +
  마진 HU** 를 `endnote_between_notes_min_gap` 에 수집 → `PaginationResult` 로 전달.
- **layout.rs**: 세터 `set_endnote_between_notes_min_gap`(base+local→마진맵), 조회
  `endnote_between_notes_min_gap_px`, 직전 미주 content bottom 추적 `endnote_prev_content_bottom`.
- **paragraph_layout.rs**: `layout_composed_paragraph` 진입부에서 새 미주 첫 문단(start_line==0)의
  시작 y 를 `max(y, prev_endnote_bottom + between_notes_px)` 로 클램프(가산 아닌 **max**).
  같은 단 연속 흐름(`y_start + 0.5 >= prev_bottom`)에서만 — 컬럼 상단 신규 진입 제외.
- **rendering.rs**: 세터 배선.

## 2. 검증 결과 — 회귀 4건 → 2건

| 테스트 | v1(가산) | v2(min-gap) |
|--------|---------|-------------|
| issue_1139 문15 [24,32] (3-09월) | ✗ (52.9) | **✅ 통과** |
| issue_1189 oct q18→19 [205,235] (3-10월) | ✗ (192.6) | **✅ 통과** |
| issue_1189 oct 기타 | ✗ | **✅ 통과** |
| **issue_1189 nov pages10_12 (3-11월 pi=475)** | — | ✗ overflow +7.6px |
| **issue_1189 nov page17 (3-11월 문28 수식)** | — | ✗ 수식 y 이탈 |

- **min-gap 모델 자체는 정답**: 외부 가드(3-09·10월) 전부 통과, no-op 동작 확인.
- 남은 2건은 **모두 타깃 파일 3-11월 내부**.

## 3. 잔여 블로커 — render/pagination seg-vs-line vpos drift

- render 클램프는 새 미주 첫 문단을 gap 만큼 아래로 밀어 **실제 콘텐츠 변위**로 만든다.
- 그러나 pagination 의 **페이지 fit 판정**(`typeset.rs` `compute_en_metrics` 의 `en_fit`)은
  between-notes 를 **직전 문단의 trailing**(overflow 허용, `fit = advance - trailing_ls_px`,
  HeightMeasurer L141 "마지막 줄 line_spacing 제외")으로 처리 → gap 을 "무시 가능 공간"으로 봄.
- 결과: render 가 gap 자리에 콘텐츠를 넣으면 pagination 이 계획한 페이지 하단을 초과(pi=475 +7.6px).
- 더 정밀히는 pagination 예약치 = 마지막 **LINE_SEG**(`vpos+lh+ls`) vs render 클램프 기준 =
  마지막 **그려진 LINE**(`y+line_height`)+gap → 되감김/빈 trailing seg 케이스에서 ~7.6px 어긋남.

## 4. 동기화의 난점 (왜 단순 en_fit 가산이 안 되나)

- `en_fit` 에 gap 을 더하면 **이미 gap 이 예약된 문서(3-09·10월)는 이중 예약** → 통과 중인
  doc-specific 테스트 회귀. (3-09·10월은 baseline render 가 이미 26.5/52.9 표시 = pagination 이
  이미 예약했다는 증거. 3-11월 文22·pi=475 만 미예약.)
- "예약됨/미예약" 을 typeset 에서 깔끔히 구분하려면 render 의 정확한 위치 산정(seg-vs-line,
  되감김)을 재현해야 함. `compact_endnote_separator_profile`/문29·30/`inline_object_overestimate`/
  `new_endnote_between_notes_px` 등 문서별 특례가 밀집한 구간.

## 5. pagination en_fit 예약 시도 — 반증 (blocker 는 예약이 아니라 drift)

`compute_en_metrics` 직후 새-미주-첫-문단 `en_fit` 에 between_notes_px 를 더해(같은 단 연속,
문29/30 제외) 페이지가 일찍 넘어가도록 시도 → **실패 2건 변화 없음**. 동시에 3-09·10월 가드는
계속 통과(이중예약 회귀 없음).

해석: pagination 의 **누적(acc=vpos-delta)은 이미 gap 을 포함**(`bottom_with_spacing` L2155)하므로
페이지 패킹은 이미 gap 을 반영한다. 문제는 **render 가 같은 흐름을 pagination 측정보다 ~7.6px
낮게 그리는 drift**(마지막 LINE_SEG vs 마지막 그려진 LINE). pi=475 는 새 미주 첫 문단이 아니라
문6 꼬리 continuation 이며, 위쪽 경계 클램프가 전체 흐름을 밀어 누적 drift 가 페이지 하단을 넘김.

→ **blocker 는 pagination 예약 부족이 아니라 render/pagination VPOS drift**
(`[[tech_lazy_base_trailing_ls_gate]]` 가 경고한 회귀 민감 영역). en_fit 예약 시도는 revert 함.

## 5.1 한컴 2022 PDF 시각 대조 (결정적) — gap 은 정답, 잔여는 drift

`pdf/3-11월_실전_통합_2022.pdf` page 10 (한글 2022, Linux 1차 정답지) 실측:

- 좌측 단에서 **문4→문5→문6 각 제목 앞에 균일한 between-notes gap 이 명확히 존재**. → min-gap
  모델이 PDF 권위로 **검증됨**(외부 테스트뿐 아니라 정답지 일치).
- 문6 본문은 좌측 단을 채우고 **우측 단으로 자연스럽게 이어짐(overflow 없음)**.

결론: 남은 2건은 **모델 오류가 아니라** render/pagination **acc 근사 drift(~7.6px)**. 올바른 gap
추가가 단-넘김 지점에서 기존 drift 를 가시화했을 뿐. 즉 **gap 은 그대로 두고, 단-넘김 정밀도
(acc↔render 정합)만 보정**하면 된다.

## 5.2 pagination 동기화 3차 시도 — 모두 회귀 (dense 영역 한계)

drift 보정(승인)으로 pagination 누적을 render 클램프와 정합하려 3가지 조건부 게이트 시도:

| 시도 | 변경 | 결과 |
|------|------|------|
| en_fit 가산 | 새-첫문단 fit 에 +gap | **무효** (overflow 변화 없음; pi=475 는 body 문단) |
| en_advance +full gap | 누적에 +between_notes | **5+ 회귀** (3-09월 등 이미 예약된 문서 이중가산) |
| en_advance +deficit | +max(0, gap − stored_vpos_gap) | **8 회귀** (prev_en_bottom_vpos 가 이미 bottom_with_spacing → 이중 차감, `capped_new_endnote_advance` 와 충돌) |
| **(구조) vpos_offset += extra_gap** | 새 미주 stored vpos 를 부족분만큼 하향 | **7 회귀** (extra_gap>0 인 많은 경계에서 gap 이 이미 vpos/렌더에 존재 → 과다 이동; question29/30·oct·2023) |

**판정**: pagination 누적부(`compute_en_metrics`)는 `capped_new_endnote_advance`·vpos-delta·
trailing_ls 관례로 **이미 between-notes 를 조건부 처리**한다. 단순 가산/차감은 그 불변식과
충돌해 다른 문서를 깬다. 안전한 정합은 이 서브시스템 불변식에 대한 깊은 이해(또는 render 가
자체 클램프 대신 pagination 이 계산한 정확 위치를 그대로 쓰도록 하는 구조 변경)가 필요하다.
3가지 시도 모두 revert 함.

## 6. 현재 상태 / 최종 평가 (승인 필요)

- **달성**: render 측 min-gap 클램프. 회귀 **4→2**, 외부 가드(3-09·10·23월) 전부 통과,
  타깃 문22 메커니즘 작동, **한컴 2022 PDF 로 모델 검증**(§5.1).
- **미해결**: 3-11월 내부 2건 — render/pagination **acc 근사 drift(~7.6px)** 로 인한
  page10 overflow·page17 수식 위치. pagination 동기화 3차 시도 모두 회귀(§5.2).
- 코드: **uncommitted WIP** (2건 실패 → 머지 불가).

### 구조 변경(승인) 시도 결과
- **간단 버전(vpos_offset += extra_gap)**: §5.2 마지막 행 — 7 회귀. 다수 경계에서 gap 이 이미
  vpos/렌더에 존재해 과다 이동. → 단순 stored-vpos 하향으로는 불가.
- **남은 구조 경로(대규모)**: render 의 미주 문단 base_y 를 "누적 incoming y" 대신 **절대 stored
  vpos 앵커**로 전환(pagination 과 동일 좌표). 미주 렌더 위치 산정 전반을 바꾸므로 ~30개 미주
  테스트에 광범위 영향 → 회귀 위험 큼, 신중한 단계적 작업 필요.

### 결론 / 선택지 (총 5개 동기화 접근 모두 회귀 — 단순 보정 불가 확정)
1. **현 render-only WIP 커밋 + 2건 known-issue**: 모델·PDF 검증 가치 보존, drift 후속 분리.
2. **대규모 구조 리팩터**(render 미주 좌표를 절대 vpos 앵커로): 깊은 작업, 별도 타스크 권장.
   → **후속 이슈 생성됨: #1246** (render 미주 좌표 절대 vpos 앵커 전환).
3. **baseline revert**: 모델/PDF/시도 기록만 문서 보존, 구현은 후속 재착수.

## 7. stream/devel 리베이스 후 결정 — #1238 을 #1246 에 흡수

현 stream/devel(914ee139, task 1209 미주 VPOS 되감김 포함) 기준으로 squash·rebase 후 재검증:

- **호재**: 1209 의 미주 VPOS 보정으로 기존 known-issue 중 `nov_page17` 가 **자동 해소**.
- **악재**: render 클램프가 **devel 의 기존 통과 테스트 2건을 회귀**시킴(클린 devel 에서 둘 다 통과 확인):
  - `issue_1189_2022_nov_pages10_12` (pi=475): page10 overflow +7.6px (cascade)
  - `issue_1209_2022_sep_page10_question12` (문12): 클램프가 1209 safe-vpos-backtrack 위치(396)를
    418.69 로 강제. rewind 게이트(local/internal_vpos_rewind, large_vpos_jump)로도 미검출.

**근본 결론**: render min-gap 클램프는 새 미주 첫 문단을 `prev_bottom + between_notes` 로 강제하나,
한컴/렌더러는 미주를 **stored vpos 기반**(1209 backtrack, rewind-tail)으로 배치 → 클램프가 vpos
위치를 침범. **render 클램프 접근 자체가 vpos 위치 시스템과 양립 불가**. 단독 머지 시 기존 테스트 회귀.

**결정(작업지시자)**: #1238 을 **#1246 에 흡수**. render 클램프 구현은 머지하지 않고, #1246
(render 미주 좌표 절대 vpos 앵커 전환)에서 클램프 hack 없이 vpos 차원에서 gap 을 실현 →
1209 backtrack 과 양립. 본 브랜치(`feature/issue-1238-between-notes-margin`)는 **검증된 모델 + PDF
근거 + 5개 시도 + render-클램프 레퍼런스**로 #1246 의 출발점이 된다.

- 코드: **uncommitted WIP** (2건 실패 → 머지 불가). render 측 min-gap 은 검증됨, en_fit 시도는 제거.
- 다음 후보:
  1. **render 측 drift 정합**: 클램프 기준을 render 의 drift 된 `prev_content_bottom` 대신
     pagination 의 예약 위치(마지막 LINE_SEG bottom 기반)와 일치시켜 ~7.6px 오버슈트 제거.
     VPOS_CORR/lazy_base 영역이라 조건부 게이트 필수(무조건 적용/제거 둘 다 회귀 위험).
  2. **한컴 PDF 시각 판정**: page17 수식·pi=475 의 현재 이동 결과를 한글 2022 PDF 와 대조해
     정답/회귀를 작업지시자가 판정 → drift 정합 방향(어느 위치가 옳은지) 확정.
