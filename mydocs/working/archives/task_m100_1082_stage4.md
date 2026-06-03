# Stage 4 보고서 — Task #1082: C군 근본 원인 정밀 특정 (다단 미주 vpos 간격 과소 계상)

- 브랜치: `local/task1082` (조사, 소스 clean)

## engine.rs 는 oracle 아님 (정정)
engine.rs 는 미주 레이아웃 코드 부재(`engine.rs:740-741` endnote_paragraphs 빈 Vec, 채우는 코드
없음) → **미주(정답/해설)를 아예 렌더 안 함**. 0 overflow 는 정답이 아니라 **콘텐츠 누락**
(typeset 20p vs engine 11p ≈ 누락 미주 ~9p). typeset.rs 가 미주를 렌더하는 것이 옳다.

## 렌더는 정상 (DIAG_EN 확인)
- build_single_column 의 `paragraphs.len()=993`(본문 488 + 미주 ~505 combined) → 미주 아이템
  정상 해석.
- `vpos_page_base_init`(layout.rs:2233): **컬럼별로 첫 아이템 vpos 에서 base 설정**(미주 컬럼
  68227 / 133141 / 203860 ... 증가) → 렌더는 컬럼별 vpos 정규화 정확.

## 근본 원인 (typeset 미주 누적의 vpos 간격 과소)
- 미주 paragraphs 는 `vertical_pos += endnote_start`(typeset.rs:1423) 로 **연속 offset vpos**
  부여(컬럼 간 리셋 없음). 렌더는 컬럼 page_base 로 정규화하므로 컬럼 used = px(마지막 아이템
  bottom_vpos − 컬럼 첫 아이템 first_vpos).
- typeset 미주 누적(typeset.rs:1485-1500): `en_advance = advance.max(height_for_fit)`,
  `advance = 미주 para 내부 vpos span(last.vpos+lh+ls − first.vpos)`. 이는 **미주 para 자체
  높이**일 뿐 **미주 간 vpos 간격(다음 미주 first − 현재 first)** 이 아님.
- dump-pages 측정: 미주 vpos 간격(pi 818→819 = 2522HU, 819→820 = 3827HU)이 para 내부 span 보다
  큼(빈 줄/문단 간격). typeset 이 그 차이를 누락 → **컬럼당 미주 과충전** → 렌더 vpos 정규화
  시 컬럼 높이 초과 → overflow(최악 ~900px).

## 수정 방향 (다음 — 신중 구현 필요)
typeset 미주 누적을 렌더와 동일한 **vpos-absolute(컬럼 첫 아이템 기준)** 로:
`current_height = px(현재 미주 bottom_offset_vpos − 컬럼 첫 아이템 first_offset_vpos)`,
컬럼 advance 시 base 리셋.
- 난점: 미주가 본문과 컬럼을 공유하는 전환(body→endnote)에서 컬럼 첫 아이템이 본문 para 일 수
  있어 base 추적이 body 레이아웃과 얽힘. #1062(단단 미주) 회귀 위험.
- 안전 구현: vpos-delta-to-prev(이전 배치 아이템 first_vpos 와의 px 차) 누적으로 렌더 정합 +
  전수 sweep/골든 회귀 검증.

## 상태
- pi 인덱싱 정합 완료(커밋 0b247163, dump-pages 미주 [미주] 표시). C군 근본 원인 정밀 특정 완료.
- 실제 수정은 미주 누적 vpos-delta 정합 — 본문/미주 컬럼 base 추적 + #1062 회귀 검증이 필요한
  집중 구현. 다음 단계로 진행.
