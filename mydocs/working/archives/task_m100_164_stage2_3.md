# 단계별 완료 보고서 — Task #164 Stage 2.3

**이슈**: [#164](https://github.com/edwardkim/生/rhwp/issues/164)
**단계**: Stage 2.3 — 다문단 + 소프트 라인브레이크 + 탭 정식 직렬화
**브랜치**: `feature/task164-hwpx-serializer`
**완료일**: 2026-04-17

---

## 목표

사용자가 직접 생성한 레퍼런스(`samples/hwpx/ref/ref_mixed.hwpx`)의 실제 XML 구조를 역공학하여 다문단·소프트 브레이크·탭 처리를 정확히 구현한다.

## 레퍼런스 분석 결과

`samples/hwpx/ref/ref_mixed.hwpx` section0.xml 분석:

1. **Enter (하드 브레이크)** → `<hp:p>` 요소 여러 개 (각 문단별)
2. **Shift+Enter (소프트 브레이크)** → 같은 `<hp:t>` 내 `<hp:lineBreak/>` **혼합 콘텐츠**
3. **Tab** → `<hp:tab width="3028" leader="0" type="1"/>` **속성 필수**
4. **vertpos 증분** = 1600 HWPUNIT (vertsize 1000 + spacing 600)
5. 첫 문단만 `<hp:secPr>`/`<hp:ctrl>` 포함, 나머지는 단순 런+linesegs

## IR 매핑 관행

```
section.paragraphs 여러 개  →  하드 문단 경계 (여러 <hp:p>)
paragraph.text 내 '\n'      →  소프트 라인브레이크 (<hp:lineBreak/>)
paragraph.text 내 '\t'      →  탭 (<hp:tab width=... leader="0" type="1"/>)
```

## 구현 내용

### 변경 파일

- `src/serializer/hwpx/section.rs` — 전면 재작성
  - `render_paragraph_parts()`: 단일 문단을 `(<hp:t>, linesegs, 다음 vert_cursor)`로 변환
  - 다문단 처리: `</hp:p></hs:sec>` 직전에 추가 `<hp:p>` 삽입
  - vert_cursor 누적 추적
- `src/serializer/hwpx/mod.rs` — `multi_paragraph_emits_multiple_hp_p` 테스트 추가
- `examples/hwpx_dump_text.rs` — `stage2_mixed.hwpx` 산출 (레퍼런스 재현)
- `samples/hwpx/ref/ref_mixed.hwpx` — 사용자 제공 레퍼런스 저장

## 검증

### 단위 테스트 — 10/10 통과
- `tab_and_linebreak_emitted_inline` — 혼합 콘텐츠 + tab 속성 검증
- `linesegs_emitted_per_linebreak` — vertpos 증분 1600 검증
- `multi_paragraph_emits_multiple_hp_p` — 3개 문단 → 3개 `<hp:p>` 검증

### 한글2020 시각 검증
- `output/stage2_mixed.hwpx` 정상 오픈
- "첫째 줄 / 줄바꿈A / 줄바꿈B / 탭[TAB]뒤 / 끝" 4문단 + 소프트 브레이크 + 탭 모두 정확히 표시 ✅

## 다음 단계

Stage 2 실질 기능 완료. Task #164 최종 보고서 작성 및 PR 준비 단계로 진행 가능.
