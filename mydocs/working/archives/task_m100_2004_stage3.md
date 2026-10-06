# task_m100_2004 Stage 3 완료보고서 — 부동(tac=false) 이미지 스택 페이지네이션

- 이슈 #2004, 브랜치 `fix/2004-image-stack-pagination`

## 근본원인 (Stage3 스코핑 확정)

1613000 pi=1004: 부동 그림 96장(tac=false·Square·overlap=false·182mm·동일 offset0), 앵커 쪽(p138)에 전부 겹침 → −97. `allow_overlap` 이 레이아웃 경로에서 미소비(겹침해소 규칙 부재). #1994(글상자 겹침)와 동근.

## 접근: render-전용 재분류 (B) + composer 합성

**모델 무손상(save 무결)** 을 위해 원본 `document` 는 건드리지 않고, render-전용 정규화본을 도입.

`src/document_core/`:
1. **`render_normalized: Vec<Option<(Vec<Paragraph>, Vec<ComposedParagraph>)>>`** (mod.rs) — 섹션별 재분류본. paginate 시 `compute_render_normalized` 로 재계산.
2. **게이트** `para_is_floating_image_stack` (rendering.rs): 모든 컨트롤이 tac=false·Square·overlap=false·전면급(높이>본문×0.5)·동일 세로오프셋 그림 + 가시텍스트 없음 + 개수≥2. → **부동개체 일반 문단 오검출 차단**.
3. **재분류** `reclassify_floating_pictures_inline`: 정규화본의 그림 tac=true.
4. **composer 합성**: HWP5 빈-문단은 line_seg 부족으로 `compose_paragraph` 가 1줄로 붕괴 → 그림 수만큼 줄을 합성(모두 char_start 동일)해 **Stage2 stacked 게이트(comp.lines.len()==tac_controls)** 가 발동하게 함. (HWPX 1430000 은 stored line_seg 로 이미 N줄.)
5. **배선**: `build_render_tree` 가 이미 `&[Paragraph]` 파라미터 → layout 내부 리팩토링 불요. paginate_pass(측정+pagination) 입력 + find_page + dump_page_items 를 접근자 `section_render_paragraphs/composed` 로 교체. direct 필드 접근으로 disjoint-borrow 유지.

→ 재분류된 부동 그림이 **Stage2 인라인 스택 기계(formatter + pagination hwp_authoritative off)** 를 그대로 재사용해 쪽당 1장 배치.

## 검증 (오라클 한글 2022)

| doc | base | 수정후 | 오라클 | |
|---|---:|---:|---:|---|
| 1613000 (부동) | 171 | **266** | 268 | **−97→−2** |
| 1430000 (인라인, Stage2) | 403 | 403 | 404 | 불변 |
| 1790387 (표) | 130 | 130 | 146 | 불변 |
| 1220000 (표) | 125 | 125 | 134 | 불변 |

- **시각 배치**: 1613000 stack 페이지(151·181·211) **각 1 image**(export-svg) → 각 그림 개별 쪽, 겹침 해소(#1994 동근 해소). 1430000(385·391·396)도 각 1 image 재확인.
- **무회귀**: 랜덤 60문서 페이지수 **전부 불변**(gate 미발동, 오검출 0). 랜덤 40문서 no-panic. **hwpx_roundtrip_baseline 4/4**(save 무결), renderer lib 763/0, document_core lib 203/0.

## 잔여
1613000 −2 는 미세 잔차(첫 140쪽 분산 드리프트 +2, Stage1 프로파일과 정합), 범위 밖. 표 행높이 계열(1790387 −16, 1220000 −9)은 #1937/#1842 별도.

## 결론
#2004 두 변종(인라인+부동) 모두 해소. #1994(글상자 겹침) 동근 해소. 지배 페이지손실(−97/−20) 제거.
