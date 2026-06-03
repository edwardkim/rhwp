# Task #288 단계 1 보고서 — 원인 조사 (탭 가설 무효 확인)

## 타스크

[#288](https://github.com/edwardkim/gotrib/issues/288) — 수식 SVG 레이아웃: 빈 runs 줄의 TAC 수식 x 위치 (# Phase 2)

- 단계 1 목적: HWP composed text 에 `\t` 가 존재하는지, compose 가 어떻게 다루는지 확인 → 접근 방침 확정

## 측정 방법

`examples/task288_probe.rs` 임시 바이너리로 `samples/exam_math_8.hwp` 문단 0.0 의 원본 `text`, `char_offsets`, `controls`, `line_segs`, 그리고 `compose_paragraph` 결과를 직접 덤프.

```
cargo run --example task288_probe --release -- samples/exam_math_8.hwp
```

## 측정 결과

### 원본 텍스트 (char-by-char)

```
text_len(chars)=21, char_count(HWP)=54, controls=4, line_segs=3

[ 0..15] "(가) 모든 자연수 에 대하여"
[ 16] U+000A \n          ← line break 1
[ 17] U+000A \n          ← line break 2
[ 18..20] "이다."
```

**`\t` (U+0009) 문자는 존재하지 않음.** 연속된 `\n\n` 사이에 큰 수식 TAC 이 삽입돼 있는 구조.

### line_segs 와 char_offsets

```
ls[0] ts=0,  lh=1150           → "(가) ..." 줄
ls[1] ts=41, lh=4095           → 큰 수식 전용 줄
ls[2] ts=50, lh=1150           → "이다." 줄

char_offsets (UTF-16 offset per char):
[10]=26 → [11]=35   9단위 점프 (작은 수식 `n`이 8-byte 차지)
[16]=40 → [17]=49   9단위 점프 (큰 수식이 40-48 차지)
```

UTF-16 offset 40 = 첫 \n, 41-48 = 큰 수식 (8 units), 49 = 두 번째 \n. ls[1].ts=41 은 큰 수식 시작점.

### compose_paragraph 결과

```
line[0] char_start=0  runs=1  text="(가) 모든 자연수 에 대하여"  has_line_break=true
line[1] char_start=17 runs=0  (has_line_break=true, line_height=4095)
line[2] char_start=18 runs=1  text="이다."

tac_controls:
  pos=11 (작은 n)   → line 0  [0, 17)
  pos=17 (큰 수식)  → line 1  [17, 18)
```

line 1 의 char 범위 `[17, 18)` 은 두 번째 `\n` 하나뿐. 이 `\n` 은 `has_line_break=true` 로 처리되어 runs 에서 제거 → **runs=[]**. 큰 수식은 `pos=17` 에서 이 줄 소유로 분류되어 tac_controls 에 담김.

## 결론 — #288 의 전제 무효

1. **`\t` 문자는 HWP composed text 에 존재하지 않는다.**
2. ls[1] 의 빈 runs 는 "탭이 탈락해서" 생긴 게 아니라 **단독 수식 줄 구조의 자연스러운 결과** 다.
3. 이슈 #288 에 적힌 (A) 휴리스틱 / (B) compose 계층 수정 모두 "탭 반영" 을 전제로 했으나 전제 폐기.

## PDF 대비 현재 x 재해석

- 현재 `inline_x = col_area.x(71.80) + margin_left(30.27) + |indent|(31.20) = 133.27`
- 이 값은 **line 2 "이다."의 x 와 완전히 동일** (line 2 도 같은 계산)
- PDF 에서는 큰 수식이 "이다." 보다 더 들여쓰기 됨 → **수식만 특별 정렬**

## 가설 재설정

한컴이 display 수식(ls 단독 차지)에 적용하는 것은 **탭이 아닌 특수 규칙**:

- **(가설 1)** 첫 탭 스톱 위치 사용 — `tab[0] pos=13600 HU=181 px` 를 들여쓰기로 사용 (탭 문자 없이도)
- **(가설 2)** `segment_width` 기준 수식 중앙 정렬 — ParaShape.alignment 무시
- **(가설 3)** HWP 내부 "display equation" 특수 처리 (일종의 `\displaystyle` 정렬)

단계 2 에서 샘플을 추가 조사해 가설을 좁혀야 한다.

## 접근 방침 재검토

### 폐기
- (A) 휴리스틱 (tab_stops → inline_x): 탭이 없으므로 동작 무관. 다만 "탭 스톱 위치를 참조" 로 재해석하면 가설 1 의 구현 방법이 됨
- (B) compose 계층의 `\t` 보존: `\t` 가 없으므로 무효

### 새 후보
- **(D) 첫 탭 스톱 위치 사용** — 빈 runs TAC 수식 줄에서 `tab_stops[0].pos` 가 있으면 그 위치로 이동. (단, PDF x=162-182 와 탭 pos=181 px 가 맞는지 추가 검증 필요)
- **(E) 세그먼트 내 중앙 정렬** — `inline_x = col_area.x + (segment_width - tac_w) / 2 + column_start`
- **(F) HWP 스펙 / reference 렌더러 분석 후 정확한 규칙 도출** — 시간이 걸리지만 근본적

우선순위: **(F) 간단 분석 → (D) 또는 (E) 구현 → 회귀 검증**

## 추가 측정 필요 (단계 2 입구)

1. 현재 SVG 에서 "이다." 의 x 는 133.27, 큰 수식도 133.27. 하지만 한컴 PDF 에서 **"이다." 도 같은 x 에 있는지**? 만약 PDF 에서 이다.도 133 근처라면 큰 수식만 들여쓰기 수정.
2. `exam_math_008.hwp` (같은 구조를 가진 다른 문서) 의 PDF 를 직접 측정해 규칙을 교차 검증.
3. `tab_def_id=6` 의 상세 — rhwp 에서 tab_def 를 제대로 파싱하는지, 여러 탭 스톱이 있는지.

## 산출물

- `examples/task288_probe.rs` (임시 — 단계 3/4 에서 제거)
- `mydocs/working/task_m100_288_stage1.md` (본 문서)

## 승인 요청

- 이슈 #288 본문의 "탭 반영" 전제가 폐기됐습니다. 
- 접근 방침을 재설정하기 위해 단계 2 에서 "PDF 에서 이다. 와 수식 x 를 정밀 측정" + "다른 샘플 교차 검증" 이 필요합니다.
- 또한 GitHub 이슈 #288 본문을 업데이트해 전제 오류를 바로잡을 필요가 있습니다 (규칙상 이슈 사실 왜곡 방지).

다음 중 어느 방향으로 진행할까요?

1. **이슈 #288 본문 갱신 + 단계 2 (규칙 역추정) 진행** — 제대로 된 분석 후 구현
2. **이슈 #288 종료 (현재 #287 수정으로 충분)** + 필요 시 새 이슈 재등록
3. **기타 지시**

판단 부탁드립니다.
