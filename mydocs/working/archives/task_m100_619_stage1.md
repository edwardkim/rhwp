# Stage 1 단계별 보고서 — Task #619

TypesetEngine partial-split 에 다단 vpos-reset forced break 추가

- 브랜치: `local/task619`
- 이슈: https://github.com/edwardkim/rhwp/issues/619

## 1. 변경 내용

`src/renderer/typeset.rs::typeset_paragraph` 의 partial-split 루프 (line 1077–1093) 의 inner fit 루프에 vpos-reset forced break 검출을 추가했다.

```rust
for li in cursor_line..line_count {
    // [Task #619] 다단 paragraph 내 vpos-reset 강제 분리.
    // line_segs[li].vertical_pos == 0 (li>0) 은 HWP 가 해당 line 을
    // 다음 단/페이지 최상단에 배치하도록 인코딩한 신호.
    // 다단 한정 적용 — 단일 단은 partial-table split 회귀 (issue #418) 차단 위해 미적용.
    if st.col_count > 1
        && li > cursor_line
        && para.line_segs.get(li).map(|s| s.vertical_pos == 0).unwrap_or(false)
    {
        break;
    }
    let content_h = fmt.line_heights[li];
    if cumulative + content_h > avail_for_lines && li > cursor_line {
        break;
    }
    cumulative += fmt.line_advance(li);
    end_line = li + 1;
}
```

## 2. 적용 범위 및 회귀 차단 설계

| 조건 | 의도 |
|------|------|
| `st.col_count > 1` | 다단 섹션 한정. 단일 단은 partial-table split (issue #418) 회귀 차단 |
| `li > cursor_line` | 세그먼트 첫 줄 제외. forced break 후 cursor 가 vpos-reset line 부터 시작할 때 즉시 break 되어 무한 루프되는 것 방지 |
| `vertical_pos == 0` | HWP vpos-reset 신호. 다음 단/페이지 최상단 배치 의도 |

## 3. 빌드 검증

```
$ cargo build --release
   Compiling rhwp v0.7.9 (/Users/planet/rhwp)
    Finished `release` profile [optimized] target(s) in 1m 05s
```

## 4. 즉시 검증 — 대상 파일

### 4.1 페이지네이션 결과 변화

```
$ rhwp dump-pages samples/21_언어_기출_편집가능본.hwp -p 7
변경 전: 단 1: PartialParagraph pi=181 lines=0..9  vpos=77316..1816 [vpos-reset@line8]
변경 후: 단 1: PartialParagraph pi=181 lines=0..8  vpos=77316..0    [vpos-reset@line8]
```

```
$ rhwp dump-pages samples/21_언어_기출_편집가능본.hwp -p 8
변경 전: 단 0: PartialParagraph pi=181 lines=9..13 vpos=1816..7264
변경 후: 단 0: PartialParagraph pi=181 lines=8..13 vpos=0..7264 [vpos-reset@line8]
```

line 8 (vpos-reset 위치) 이 페이지 9 단 0 으로 이동.

### 4.2 LAYOUT_OVERFLOW_DRAW 제거

```
$ rhwp export-svg samples/21_언어_기출_편집가능본.hwp -p 7
변경 전:
  LAYOUT_OVERFLOW_DRAW: section=0 pi=181 line=8 y=1453.2 col_bottom=1436.2 overflow=17.1px
  LAYOUT_OVERFLOW: page=7, col=1, para=181, type=PartialParagraph, overflow=26.6px
  LAYOUT_OVERFLOW: page=7, col=1, para=181, type=Shape,            overflow=26.6px

변경 후:
  (LAYOUT_OVERFLOW_DRAW 미발생)
  LAYOUT_OVERFLOW: page=7, col=1, para=181, type=PartialParagraph, overflow=2.4px
  LAYOUT_OVERFLOW: page=7, col=1, para=181, type=Shape,            overflow=2.4px
```

- **`LAYOUT_OVERFLOW_DRAW`** (실제 텍스트 글리프 그리기 기준 오버플로) — **사라짐**. 본 Task 의 핵심 증상 "반쯤 안 보임" 해결.
- **`LAYOUT_OVERFLOW`** (PartialParagraph/Shape bbox 기준) — 26.6px → 2.4px 로 크게 감소. 잔여 2.4px 는 line_spacing trail 까지 포함한 bbox 기하학 차이로, 텍스트 글리프 자체는 단 안에 들어감 (별도 검토 사항).

### 4.3 페이지 9 첫 줄 검증

```
$ grep -oE 'translate\([0-9]+\.[0-9]+,2[0-9][0-9]\.[0-9]+\)' output/svg/p21_after/21_언어_기출_편집가능본_009.svg | sort -u | head -5
translate(128.53333333333336,222.2266666666667)  ← 페이지 9 단 0 첫 줄 (line 8)
translate(128.53333333333336,246.44000000000003)
translate(128.53333333333336,270.6533333333333)
translate(128.53333333333336,294.8666666666666)
```

페이지 9 본문 영역 상단 (col_top=209.8, baseline 반영 ~222) 에 line 8 텍스트가 정확히 배치됨.

## 5. 잔여 사항

- **PartialParagraph/Shape bbox 잔여 2.4px**: 본 Task 범위 외. 별도 이슈 분리 후보. Stage 3 회귀 검증에서 동일 패턴이 다른 샘플에도 있는지 확인.

## 6. 다음 단계

Stage 2 (한컴 PDF 비교 + 전체 회귀 가드 샘플 점검) 진행 승인 요청.
