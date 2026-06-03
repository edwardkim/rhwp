# 단계별 완료 보고서 — Task #164 Stage 2.2

**이슈**: [#164](https://github.com/edwardkim/rhwp/issues/164)
**단계**: Stage 2.2 — 제어문자(탭/줄바꿈) 인라인 직렬화
**브랜치**: `feature/task164-hwpx-serializer`
**완료일**: 2026-04-17

---

## 목표

문단 텍스트 내 `\t`/`\n`을 `<hp:tab/>` / `<hp:lineBreak/>` 요소로 `<hp:t>` 안에 인라인 출력하여, HWPX 규격상 올바른 구조를 생성한다.

## 구현 내용

### 변경 파일

- `src/serializer/hwpx/section.rs` — `render_text_run()` 도입
- `src/serializer/hwpx/mod.rs` — 탭/줄바꿈 테스트 추가
- `examples/hwpx_dump_text.rs` — `stage2_ctrl.hwpx` 산출 추가

### 출력 패턴

한컴 레퍼런스(`samples/hwpx/2025년 2분기 …hwpx`)의 실제 인코딩을 따라 `<hp:t>` 내부에 혼합 콘텐츠로 출력:

```xml
<hp:t>첫째<hp:tab/>탭 뒤<hp:lineBreak/>둘째 줄<hp:tab/>Tab<hp:lineBreak/>셋째</hp:t>
```

## 검증

### 단위 테스트 — 8/8 통과
- `tab_and_linebreak_emitted_inline` 신규 추가

### 한글2020 시각 검증
- 파일 정상 오픈 ✅
- 줄바꿈은 조판부호(`↵`) 마크로 표시되지만 **실제 줄 나눔은 발생하지 않음**
- 탭은 시각적으로 붙어 렌더링됨

## 한계 (→ Stage 2.3로 연기)

`<hp:linesegarray>`에 `<hp:lineseg>` 1개만 있어 한컴이 "문단 전체가 한 줄"로 취급한다. 시각적 줄바뀜과 탭 스탑 반영을 위해서는 줄바꿈마다 lineseg를 동적으로 추가해야 한다.

## 다음 단계

**Stage 2.3** — `<hp:linesegarray>` 동적 생성 (줄바꿈 기준 lineseg 분할, textpos/vertpos 계산)
