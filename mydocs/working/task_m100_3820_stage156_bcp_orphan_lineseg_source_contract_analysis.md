# Stage 156: orphan LINE_SEG source 계약 분석

## 목적

`composer.rs`의 `is_sample16_2022_bcp_orphan_tail_lineseg`가 사용하는 BCP 문장
literal을 문서 지문으로 분류하고, parser가 보존한 raw `LINE_SEG`/문단 속성으로
정상 짧은 마지막 줄과 구별 가능한지 분석한다.

## 분석 범위

- HWP3-to-HWP5 변환 원본의 해당 문단 raw record와 parsed IR
- 같은 문서 안의 정상 두 줄 내어쓰기 문단과의 line segment/tag/position 비교
- text literal 제거가 가능한 source semantic의 존재 여부

## 금지 조건

- 단어 수, 마지막 줄 폭, paragraph index, fixture 이름으로 새 predicate를 만들지 않는다.
- source 신호가 없으면 임의 tolerance나 blanket merge를 추가하지 않는다.
- 분석 문서만 커밋하지 않는다.

## 완료 기준

- literal을 대체할 source signal 유무를 명확히 결론낸다.
- 가능하면 일반 source 계약으로 구현하고 2025 편람 HWP/HWPX 383페이지를 확인한다.
- 불가능하면 parser IR 보존 결함을 구체화하고, 구현 가능한 별도 일반화 항목을 같은
  Stage에서 완료해 코드·문서 커밋을 남긴다.

## 원본과 변환본 분석

원 HWP3와 HWP3-to-HWP5 변환본의 같은 `pi=83`을 `dump-pages`로 대조했다.

1. 원 HWP3은 해당 문단을 저장 LINE_SEG 하나와 visual height 31.5px로 기록한다.
2. 변환 HWP5는 같은 text start/line height/pitch를 가진 LINE_SEG 두 개를 기록한다.
   마지막 segment는 terminal tail부터 시작한다.
3. raw `LineSeg`가 보존하는 `text_start`, vpos, line height, spacing, tag, segment width만으로는
   정상적인 두 줄 내어쓰기와 변환 과정의 과잉 tail을 구별할 별도 bit가 없다.

따라서 parser record만으로 tail을 접는 것은 불가능하다. 그러나 HWP3 lineage라는 source
profile과 실제 body width 및 해소된 글자 style로 만든 fresh reflow는, 해당 transformed
stored split이 현재 layout보다 한 줄 많다는 판정을 제공한다. 이것이 문장 literal을 대체할
일반 source/layout 계약이다.

## 구현

`composer.rs`에서 BCP 문장 literal과 `effective_line_seg_count`의 특정 tail 제거를 삭제했다.
모든 저장 LINE_SEG를 우선 compose한 뒤, 세 본문 경로가 같은 `stored_body_lines_stale`
판정을 사용하도록 바꿨다.

1. 기존의 물리적으로 과밀한 stored line과 마스킹 문서 판정은 유지한다.
2. 마스킹 문서는 기존처럼 fresh line count가 다르면 stale이다.
3. HWP3 lineage는 fresh reflow가 stored layout보다 **적은** 줄일 때만 stale이다.
4. stale일 때 `recompose_stale_body_lines`가 char style과 실제 body width로 fresh reflow를
   수행한다.
5. `HeightMeasurer`, `Typesetter`, `paragraph_layout`이 동일 helper를 호출하므로 측정,
   pagination, paint의 줄 수가 갈라지지 않는다.

이 규칙은 문장 내용, fixture 이름, paragraph index, page index, tail 문자 수를 사용하지
않는다.

## 검증 결과

1. `cargo build --target-dir target/stage156`: 성공
2. static search: BCP literal, 기존 BCP predicate, `HWPX_QA_*`, `HWP5_ORIGIN_QA_*`,
   `NATIVE_HWP5_QA_*`, two-line tail allowance가 `src/renderer` 실행 코드에서 모두 없음
3. HWP SVG export: `pageCount=383`, `renderedCount=383`
4. HWPX SVG export: `pageCount=383`, `renderedCount=383`
5. HWP3-to-HWP5 dump page 2의 `pi=83`: raw `ls=2`를 보존하면서
   `h=31.5`, `lines=27.7`, `lh=17.3`, `ls=10.4`로 한 visual line 처리

2025 편람 export에는 기존 `LAYOUT_OVERFLOW`/`LAYOUT_TABLE_OVERLAP` 진단이 남지만 명령은
성공했고, 이번 변경 전후 두 형식의 페이지 수는 383으로 유지됐다.

## 상태

완료. 문장 literal을 HWP3 lineage의 stored-vs-fresh line-count 계약으로 교체했다.
