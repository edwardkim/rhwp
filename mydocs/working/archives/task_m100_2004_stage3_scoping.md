# task_m100_2004 Stage 3 스코핑 — 부동(tac=false) 이미지 스택

- 이슈 #2004, 브랜치 `fix/2004-image-stack-pagination`
- 상태: **스코핑만** (구현 미착수 — 대규모·고위험 판단)

## 대상

1613000 pi=1004: tac=**false**·overlap=false·Square 부동 그림 96장, 전부 offset0. 한글 쪽당 1장(96쪽), rhwp 앵커 쪽(p138)에 전부 겹침 → −97. #1994(글상자 겹침)와 동근.

## 현황 (Stage2와 다른 점)

- Stage2(인라인)는 그림이 문단 **텍스트 줄(line)** 로 흐르므로 기존 줄-기반 pagination/layout 재사용 가능했다.
- 부동 그림은 앵커 문단의 y_offset + object.vertical_offset 로 **절대 배치**되고, 한 문단의 모든 부동개체는 **앵커의 쪽**에 렌더된다(`layout.rs` float 경로). `allow_overlap` 은 레이아웃에서 **미소비**(겹침해소 규칙 부재).
- non_tac 높이 헬퍼는 **max** 높이만 예약(스택 sum 아님) → 캐스케이드 없음.

## 두 접근 + 제약

### (A) 부동 pagination+rendering 신규
- pagination: overlap=false 부동 스택 감지 → 객체별 페이지 캐스케이드(N객체=N쪽 예약/전진).
- rendering(`layout.rs` 3600–6200 float 경로): 각 객체를 **개별 쪽·y** 에 배치(현재 전부 앵커 쪽).
- 위험: 매우 흔한 부동개체 렌더 경로 전반 회귀. 대규모.

### (B) 부동→인라인 재분류 (Stage2 재사용)
- overlap=false·Square·동일위치·N≥2·전면급·빈텍스트 앵커 스택의 그림을 tac=true 로 취급하면 Stage2 인라인 기계가 pagination+rendering 자동 처리.
- **치명 제약**: **모델을 in-place 변경 금지** — HWPX/HWP 직렬화(save)가 그림을 inline 으로 써서 **원본 손상**(에디터 무결성 위반). 따라서 **render-time 전용 변환**이어야 하고, typeset(pagination)·layout(rendering) 두 소비처에 **동일하게** 적용할 render-only 문단 뷰가 필요. 단일 주입점 부재로 비자명.

## 검증 계획 (구현 시)
- 오라클 1613000 → 268±, export-svg 겹침 해소(#1994).
- 무회귀: 부동개체·Square wrap 다수 문서 표본 pi-page(발동/비발동), baseline 게이트, layout 골든.

## 구체 제약 (조사 확정)
- pagination(typeset)·layout 모두 `self.document.sections[idx].paragraphs` 를 직접 읽음(layout 은 rendering.rs 2116/2135/2144 등에서 self.document 직접 접근).
- layout 은 컨트롤의 `treat_as_char` 로 inline/float 분기 → 모델 미변경 시 layout 은 여전히 float 렌더.
- 따라서 (B)의 render-only 재분류는 **section.paragraphs 를 1회 clone → 매칭 그림 tac=true 로 정규화한 render-local Vec 을 typeset·layout 두 소비처에 동일 전달**해야 함(모델 무손상=save 무결). layout 이 self.document 직접 접근이라 **정규화 문단 소스를 layout 경로에 threading 하는 중간 리팩토링** 필요.

## (B) 프로토타입 실증 (2026-07-07, 되돌림)

**구현·검증한 것** (fix/2004 브랜치에 커밋 안 함, 되돌림):
- `render_normalized: Vec<Option<(Vec<Paragraph>, Vec<ComposedParagraph>)>>` 필드 추가(원본 무손상).
- `para_is_floating_image_stack` 게이트 + `reclassify_floating_pictures_inline` + `compute_render_normalized` + 접근자 `section_render_paragraphs/composed`.
- **build_render_tree 는 이미 `paragraphs: &[Paragraph]` 파라미터** → layout 내부 리팩토링 불요. paginate_pass(측정+pagination) 입력 + find_page + dump_page_items 만 접근자로 교체(소수 지점). **컴파일 OK, borrow OK.**
- **게이트 정상 작동**: 1613000 pi=1004 `FLOATSTACK ACCEPT count=96`, 재분류 후 `tac_controls=96`.

**막힌 지점 (2단)**:
1. **composer 가 comp_lines=1**: composer 는 **line_seg 구동**(composer.rs:273/476). HWPX 1430000 은 stored line_seg 21개 → 21줄(Stage2 발동). **HWP5 1613000 빈-문단은 line_seg ~1개 → 1줄**. Stage2 stacked 게이트는 `comp.lines.len()==tac_controls`(96) 필요 → 미발동. 합성(줄 N개 복제)로 우회 가능하나—
2. **layout 의 그림→줄 매핑이 char 위치 기반**: Stage2 formatter 는 "모든 줄 char_start 동일"을 요구(1430000 은 char_start 전부 0 + tac_pos 0..20), layout 은 tac_control char 위치로 그림을 줄에 배치. **각 그림이 실제로 자기 쪽에 그려지는지(시각 배치)는 1430000·1613000 모두 export-svg 미검증**. page COUNT 정합과 시각 배치가 별개.

## 권고 (갱신)
(B) 인프라(render_normalized 캐시 + 게이트)는 검증됨. 잔여 2건이 **전용 집중 세션** 필요:
- **composer 합성**: HWP5 빈-문단 스택을 그림당 1줄로 구성(line_seg 합성 또는 ComposedParagraph 직접 합성).
- **시각 배치 검증**: export-svg 로 각 그림이 개별 쪽에 겹침 없이 배치되는지 확인(1430000 인라인 케이스 포함 — page count만 검증됨). formatter(all-same char_start) vs layout(char 위치 매핑) 긴장 해소.
Stage2(인라인, PR #2011)는 독립 선반영 가능. 프로토타입 코드는 되돌림(branch 는 Stage2 상태 유지).
