# Task M100 #4969 W10-Q2-C — activation matrix와 owner 재감사

## 판정

Q2-C activation matrix를 승인받았고 dormant 구현은 **qualified**다. Q2-C 단독 제품 활성은 **기각**한 상태를
유지하며, cluster-aware paragraph/break transaction은 Q2-D emitted-run sidecar와 함께 원자적으로 활성화한다.

기계 판독 정본은
[`w10_q2_c_activation_matrix.json`](../tech/investigations/issue-4969/w10_q2_c_activation_matrix.json)이다.

## 왜 Q2-C를 단독 활성화할 수 없는가

현행 composer에는 폭 owner가 두 층으로 남아 있다.

1. `BreakToken`에는 K0 `base_width`와 W9가 갱신하는 `width`가 함께 있다.
2. frozen scalar 경로는 `width`를 쓰지만, 실제 live `fill_one_interval`은 `base_width`에
   `KerningParagraphBreakSession::boundary_pair_adjustment`를 더한다.
3. 최종 `LineBreakResult`는 `start_idx`·`end_idx`만 남기고 measurement `Arc`를 보존하지 않는다.
4. `paragraph_layout`은 나중에 emitted text를 다시 측정해 W9 scalar `layout_positions`를 만들며, common shaping
   cluster 결과를 받을 필드가 아직 없다.

따라서 Q2-C에서 shaped line boundary만 live로 열면 line selection은 common shaper가, bbox·다음 run origin·paint는
nominal/W9가 소유한다. 이는 “line-break candidate와 emitted run이 같은 결과를 소비한다”는 보호 불변식에 정면으로
위배된다. Q2-C는 transaction과 convergence를 검증하되 제품 소비자를 0으로 유지하고, Q2-D가 sidecar를 연결할 때
함께 활성화한다.

## 최초 activation matrix

`eligible`은 아래 조건을 **모두** 만족할 때만 가능하다. 하나라도 실패하면 typed reason을 남기고 승인된 문단
transaction 전체를 W9 K1, 그마저 불가능하면 K0로 되돌린다.

| 축 | 최초 eligible 조건 | 최초 excluded·reason |
| --- | --- | --- |
| feature detection | 직접 저장된 옛한글 Jamo 범위 `1100–11FF`, `A960–A97F`, `D7B0–D7FF`가 있음 | 일반 Latin liga는 `latin-default-liga-not-approved` |
| source | 같은 segment의 모든 scalar가 동일 exact portable handle·registry generation 사용 | source 없음·identity 불일치·fallback font 혼합 |
| 방향 | `horizontal-tb`, LTR, bidi level 0을 좁은 Hangul 문자 집합으로 증명 | RTL·방향 미확정·vertical |
| 문자열 | model text와 shaping text가 1:1로 같음 | PUA expansion·field/display projection·legacy 제품명 projection |
| style | 동일 slot·size·장평, variation empty, bold/italic/super/sub false | synthetic style·variation·style/source split 횡단 |
| 간격 | 자간 0, condense-min-space 0 | cluster별 자간 semantics가 미확정인 비영 자간·공백 압축 |
| 제어문 | inline control·tab·char overlap·rotation 없음 | control 경계 횡단·tab·회전·겹침 |
| feature | `kern`은 HWP kerning flag를 0/1로 명시, GSUB owner는 common shaper 하나 | shaping 적용 뒤 W9 GPOS 중복 |
| bounded | text·glyph·cluster 각 4,096 이하, context 총 cache 상한 이내 | 상한 초과·missing glyph·비정상 advance |
| break | 모든 후보와 확정 line boundary가 cluster boundary | cluster 내부 절단·token projection 불가 |
| convergence | initial shape 뒤 final-line reshape, 최대 재결정 2회 안에 동일 boundary·fit | 2회 안에 수렴하지 않음 |
| publication | Q2-D emitted-run `Arc` sidecar가 같은 measurement를 소비 | Q2-C 동안에는 항상 `publication-owner-pending` |

좁은 Hangul 문자 집합을 쓰는 이유는 버전 분기가 아니라 bidi authority의 기능 탐지다. 첫 lane은 옛한글 Jamo와
같은 segment의 현대 Hangul 음절만 허용한다. 중립 구두점·다른 script가 섞이면 별도 segment로 나누고, 안전하게
분리할 수 없으면 문단 전체를 rollback한다.

## dormant transaction 설계

### 1. 입력 동결

- W9 registry의 immutable snapshot과 generation을 재사용한다.
- 원문 scalar, style/source segment, hard boundary, K0 base positions를 bounded하게 보존한다.
- target segment는 Q2-B `HorizontalShapingContext`가 측정하고, non-target segment는 기존 W9/K0 owner를 유지한다.
- target segment가 `applied`이면 그 segment에는 W9 pair adjustment를 다시 적용하지 않는다.

### 2. cluster-aware paragraph measurement

- paragraph-level `range_width(start, end)`는 target cluster 경계에 맞을 때만 common result를 사용한다.
- non-target 범위는 W9/K0를 사용하되 hard boundary를 넘는 하나의 width 요청은 만들지 않는다.
- 글자 단위 한글 token이 하나의 old-Hangul cluster 내부에 있으면 token을 cluster range로 합친다. 가짜 개별
  `char_widths`를 만들지 않는다.
- 긴 token fallback은 scalar 증가가 아니라 다음 cluster boundary 후보만 순회한다.

### 3. bounded line 결정

1. paragraph context shape로 cluster 후보 폭을 만든다.
2. live cursor와 동등한 token·space 규칙으로 line boundary를 고른다.
3. 확정 line segment를 독립적으로 다시 shape해 boundary context를 반영한다.
4. width·fit이 달라지면 최대 2회 재결정한다.
5. 불일치·cluster 절단·상한·비수렴이면 pristine K0 token에서 문단 전체를 다시 계산한다.

Q2-C 결과에는 line range와 각 final shaped segment의 `Arc`를 함께 보존한다. 다만 Q2-D 전에는 이를 `LineSeg`나
`TextRunNode`에 게시하지 않고 공개 fixture test만 소비한다.

## 보호 불변식

1. Q2-C 동안 live composer·layout·paint consumer는 0이다.
2. target run은 common shaping 또는 W9/K0 중 하나만 소비한다.
3. cluster 내부 scalar boundary는 line candidate가 될 수 없다.
4. 실패 뒤에는 이미 수정한 token·cursor를 재사용하지 않고 pristine K0 입력에서 다시 시작한다.
5. 부분 segment 성공을 남긴 채 같은 문단의 실패 segment만 fallback하지 않는다.
6. initial paragraph shape와 final-line reshape가 같은 source generation·settings identity를 사용한다.
7. retry는 최대 2회이며 exhaustion은 typed rollback이다.
8. 원문은 in-memory cache key 밖으로 직렬화하지 않는다.
9. Q2-D sidecar 전에는 shaped line boundary를 제품 `LineSeg`에 게시하지 않는다.
10. Latin liga·RTL·variation·vertical·비영 자간·condense는 별도 근거 없이 확대하지 않는다.

## 구현 절편 제안

### Q2-C0 — paragraph segmentation·eligibility

`shaping_context.rs`에 bounded segment request와 activation decision을 추가한다. exact source·style·직접 Jamo·hard
boundary를 판정하고 실패 reason을 고정한다. 제품 consumer는 없다.

### Q2-C1 — cluster-aware width owner

target shaping과 non-target W9/K0를 합친 paragraph `range_width`와 cluster boundary index를 만든다. 글자 단위 token을
cluster로 합치고 내부 범위를 거부하는 공개 fixture를 추가한다. 제품 consumer는 없다.

### Q2-C2 — line transaction·convergence

live cursor와 독립된 dormant transaction에서 initial line decision, final-line reshape, 최대 2회 수렴과 전체 rollback을
검증한다. W9 K1·K0 fallback 결과와 line range를 대사한다. 제품 consumer는 없다.

### Q2-D — 원자적 제품 activation

Q2-C 결과의 final run `Arc`를 emitted `TextRunNode` internal sidecar로 전달하고 bbox·다음 run origin·LayerBuilder가
같은 결과를 소비할 때만 Q2-C line boundary publication을 함께 연다.

## 승인 요청 범위

다음 승인은 Q2-C0부터 Q2-C2까지 dormant transaction 구현만 연다. composer·layout·paint 제품 출력 변경,
`TextRunNode` sidecar, public schema, backend replay, Latin liga 확대는 승인 범위가 아니다. Q2-C2 결과 뒤 Q2-D
원자적 activation을 별도로 승인받는다.
