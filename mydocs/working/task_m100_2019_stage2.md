# #2019 Stage2 findings — 다층 근본원인 규명(계측 완료) + 즉흥수정 부적합 판정

- 브랜치: `fix/2019-through-wrap-overlay-vpos` (소스 변경 없음 — 조사·계측 후 원복)
- 대상: `74312 벤처투자 시행규칙(안).hwpx` — rhwp 81p vs 한글 18p (4.5× 과분할)

## 결론 요약

74312 과분할은 **세 개의 얽힌 메커니즘**이며, 지배 메커니즘(③)은 가장 복잡한 함수
(`process_multicolumn_break`, 200+줄 Task 특수케이스)에 있어 **즉흥 수정 시 광범위 회귀 위험**이
크다. 도형 높이(①)·단나누기(②) 부분수정은 국소 개선(near-empty 35→25p)이나 총 페이지 미수렴
(81→80). **정식 다단계 타스크로 인계 권장.**

## 계측으로 확정한 사실

활성 엔진 = TypesetEngine. 페이지 생성 호출부는 `#[track_caller]` 계측으로 정확히 특정:
- **advance@단나누기 핸들러**(21회) + **force@쪽나누기 핸들러**(16회).
- 문서 구조: **[단나누기](Column) 71개** + [쪽나누기](Page/Section) 17개. 저장 stored vpos 최대
  ≈1,276,150 HU ÷ 쪽당 77,013 HU ≈ **16.6p ≈ 한글 18p** (콘텐츠 세로 범위는 정상).

### ① 부동 개체 높이의 흐름 예약 (수정 가능·저위험, 부분효과)
- pi=86 앵커의 stored `LINE_SEG.line_height = 14545 HU (= 51.3mm = 194px)` = **부동 글상자 높이 자체**.
  `format_paragraph`(typeset.rs:~10145)가 이를 `total_height`로 합산 → `typeset_paragraph`가
  흐름 `current_height`에 예약 → 오버플로 조기분할. 부동 개체(통과/tac=false)는 Paper 절대위치로
  별도 렌더되므로 이중 계상.
- **수정안**: `para_is_floating_overlay_anchor`(빈 텍스트 + 전 컨트롤 부동 통과/글앞/글뒤 tac=false)
  게이트로 line_height 를 빈문단 fallback 으로 대체. **결과: near-empty 35→25p, 총 81→80p.**

### ② 부동 폼 앵커의 단나누기 → 단일 단 페이지 분할 (수정 가능, 총효과 net-zero)
- 71개 [단나누기] **전부 빈 텍스트 + 부동 개체 앵커**(텍스트 문단엔 0개). rhwp 는 단일 단에서
  단나누기를 페이지로 변환(21회 advance), 한글은 안 함.
- **수정안**: 단나누기 핸들러에서 floating-overlay-anchor 면 억제. **일부 발동하나 억제 자리에
  내용이 재누적돼 인근 재분할 → 총 페이지 불변**(다른 메커니즘이 지배).

### ③ ColumnDef zone 전이의 stored-vpos 오프셋 (★지배, 고위험·미수정)
- 증가하는 단나누기 문단(pi=39/74/82/83)의 컨트롤은 도형이 아니라 **[단정의](ColumnDef)** —
  문서가 **1단↔2단을 71회 전환**. 이들은 `has_diff_col_def=true` → `process_multicolumn_break`.
- `process_multicolumn_break`는 페이지를 직접 만들지 않고 같은 페이지에 zone 을 쌓으나,
  `vpos_zone_height`가 **대형 stored vpos(165310+)를 zone 오프셋으로 사용** → `current_zone_y_offset`
  누적 → 본문이 아래로 밀려 오버플로·페이지 증가.
- 즉 **폼의 절대 vpos(≈17p 캔버스 좌표)가 흐름 오프셋으로 사용**되어 71회 단-zone 전환과 결합,
  80p 로 팽창. `process_multicolumn_break`는 수십 개 Task 특수케이스(#853/#866/#874/#702…)가 얽혀
  즉흥 수정 시 회귀 위험 큼.

## 배제한 가설 (계측 근거)
- vpos-reset 강제분할 트리거(typeset.rs:2606): **0회 발동**.
- 명시적 쪽나누기: 17개뿐(한글 18p와 정합, 정상).
- 스타일 page_break_before: 전부 false.
- 높이 오버플로: ① 수정 후 제거됨.

## 권장 (정식 다단계 타스크)
근본은 **stored LINE_SEG vpos 가 부동 폼의 절대 배치를 인코딩**한다는 점. 두 방향:
1. (권장) 부동 폼 앵커 영역에서 stored vpos 를 **페이지 배치에 직접 신뢰**(→ ≈17p) 하고 흐름
   오프셋·단나누기·개체높이 예약을 모두 무력화하는 통합 게이트.
2. 또는 ①+②+③을 각기 부동-앵커 게이트로 개별 무력화(③이 난제).
- 어느 쪽이든 `process_multicolumn_break`의 vpos_zone_height 경로 재설계 + hwpdocs 부동개체 다수
  문서 광범위 회귀검증 필수. #2004(전면이미지 스택)와 동급 난이도.

## 상태
- 소스 원복(clean). Stage1 하네스/기준선 커밋 유지. 본 findings 커밋 + #2019 코멘트로 RCA 인계.
