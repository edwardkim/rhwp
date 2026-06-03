# Stage 2 단계별 보고서 — Task #619

대상 파일 수정 검증 + 한컴 PDF 비교

- 브랜치: `local/task619`
- 이슈: https://github.com/edwardkim/rhwp/issues/619

## 1. 대상 파일 검증 — `samples/21_언어_기출_편집가능본.hwp`

### 1.1 페이지네이션 변화

```
$ rhwp dump-pages samples/21_언어_기출_편집가능본.hwp -p 7
변경 전: 단 1: PartialParagraph pi=181 lines=0..9  vpos=77316..1816 [vpos-reset@line8]
변경 후: 단 1: PartialParagraph pi=181 lines=0..8  vpos=77316..0    [vpos-reset@line8]

$ rhwp dump-pages samples/21_언어_기출_편집가능본.hwp -p 8
변경 전: 단 0: PartialParagraph pi=181 lines=9..13 vpos=1816..7264
변경 후: 단 0: PartialParagraph pi=181 lines=8..13 vpos=0..7264 [vpos-reset@line8]
```

### 1.2 LAYOUT_OVERFLOW_DRAW 제거

```
변경 전: LAYOUT_OVERFLOW_DRAW: pi=181 line=8 overflow=17.1px  ← 텍스트 글리프 자체 오버플로
         LAYOUT_OVERFLOW (PartialParagraph) overflow=26.6px
         LAYOUT_OVERFLOW (Shape)            overflow=26.6px

변경 후: (LAYOUT_OVERFLOW_DRAW 미발생)                          ← 핵심 증상 해결
         LAYOUT_OVERFLOW (PartialParagraph) overflow=2.4px    ← bbox 잔여 (텍스트는 단 안)
         LAYOUT_OVERFLOW (Shape)            overflow=2.4px
```

### 1.3 페이지 9 첫 줄 정상 배치

```
$ grep 'translate(128.5..,2[0-9][0-9]\.\.)' output/svg/p21_after/21_언어_기출_편집가능본_009.svg
translate(128.5,222.2)  ← 페이지 9 단 0 첫 줄 (line 8) — col_top=209.8 + baseline ≈ 222
translate(128.5,246.4)
translate(128.5,270.6)
```

페이지 9 단 0 첫 줄에 line 8 텍스트가 정상 배치됨.

## 2. 한컴 PDF 비교

`pdftotext -layout` 으로 한컴 2010 / 2020 PDF 페이지 8/9 텍스트 추출 후 비교.

### 2.1 페이지 8 단 1 의 pi=181 부분 줄 수

| 환경 | pi=181 줄 수 | 페이지 9 첫 줄 |
|------|--------------|---------------|
| 한컴 **2010** PDF | 5줄 (line 0..5) | "인도하고…" (line 5) |
| 한컴 **2020** PDF | **8줄 (line 0..8)** | "토속 신앙의 영향을…" (line 8) |
| **변경 전 (rhwp)** | 9줄 (line 0..9, **overflow 17.1px**) | "로 쉽게…" (line 9) |
| **변경 후 (rhwp)** | **8줄 (line 0..8)** ✓ | "토속 신앙의 영향을…" (line 8) ✓ |

### 2.2 결론

- 변경 후 결과는 **한컴 2020 PDF 와 정확히 일치**.
- HWP 파일의 `pi=181 line_segs[8].vertical_pos = 0` 인코딩 의도는 한컴 2020 의 분포와 동일하다 (line 8 부터 다음 단 시작).
- 한컴 2010 PDF 는 다른 알고리즘 (line 5 에서 분할) 을 사용하지만, **HWP 인코딩 신호 (vpos-reset) 를 존중하는 본 변경의 방향은 한컴 2020 정합성 향상**.
- 메모리 노트 `feedback_pdf_not_authoritative.md` 룰 ("PDF 200dpi 는 보조 ref. 한컴 2010/2020 환경 차이 함께 점검") 에 따라 두 환경 모두 점검 완료.

## 3. 잔여 사항

- **PartialParagraph/Shape bbox 2.4px overflow**: 텍스트 글리프는 단 안 (LAYOUT_OVERFLOW_DRAW 미발생). bbox 가 line_spacing trail 까지 포함하는 기하학적 잔여. 본 Task 의 핵심 증상과 무관 — 별도 이슈로 분리 후보.

## 4. 다음 단계

Stage 3 (회귀 검증 — cargo test, clippy, 다단 가드 샘플) 진행.
