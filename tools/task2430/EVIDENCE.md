# #2430 HFT ASCII 메트릭 실측 검증 자료 (PR #2510 리뷰 대응)

리뷰 요청([#2510 리뷰 코멘트](https://github.com/edwardkim/rhwp/pull/2510))의
"실측 환경·재현성·HFT identity 증거"에 대한 기록. 측정 수행일 2026-07-21.

## 1. 실측 환경

| 항목 | 값 |
|---|---|
| OS | Microsoft Windows 11 Pro, Version 10.0.26200.8655 |
| 한컴오피스 한글 | HwpObject Version **12.0.0.4547** (한글 2022, COM `hwp.Version`) |
| Python / pyhwpx / PyMuPDF | 3.12.10 / 1.7.2 / 1.27.2 |
| 측정 스크립트 | `tools/task2430/hy_ascii_ladder.py` (preflight 포함 개정판) |

## 2. 다섯 face 의 HFT 실제 선택 가능 증거 (preflight)

`hy_ascii_ladder.py` preflight 는 CharShape 에 face+FontType=2(HFT)를 설정한 뒤
왕복 조회로 실제 해소 결과를 검증한다. **실존 HFT 는 FontType=2 가 유지**되고,
미설치 face 는 fallback 으로 FontType 이 변질된다(아래 negative-control).

측정 환경 로그 (`tools/task2430/measured/preflight_report.tsv` 커밋본):

```
한양신명조: readback=('한양신명조', FontType=2) OK
한양중고딕: readback=('한양중고딕', FontType=2) OK
한양견명조: readback=('한양견명조', FontType=2) OK
한양견고딕: readback=('한양견고딕', FontType=2) OK
휴먼명조:   readback=('휴먼명조',   FontType=2) OK
```

PDF 단 이중 확인: 각 face 프로브 PDF 의 임베드 폰트가 한양 4종=**Type3**(HFT
렌더 경로), 휴먼명조=**Type0 `INPILL+휴먼명조`**(cp949 복원) — 시스템 TTF
대체(`Haansoft *`) 혼입 없음. 스크립트가 저장 후 자동 검사한다.

## 3. Negative-control (미설치 HFT 지정)

```
$ python tools/task2430/hy_ascii_ladder.py --fonts "존재하지않는폰트XYZ" --out-dir ...
  [preflight] 존재하지않는폰트XYZ: readback=('존재하지않는폰트XYZ', FontType=6) ** HFT 미확인 **
[abort] HFT 미확인 face 1종: 존재하지않는폰트XYZ — TSV 를 생성하지 않는다
$ echo $?   # → 2  (ladder TSV 미생성, preflight_report.tsv 만 남음)
```

fallback 시 PDF 임베드는 `INPILL+Haansoft Batang`(Type0)으로 나타나며, 이
패턴은 측정 단계의 `check_pdf_fonts` 가 별도로 차단한다(exit 3).

## 4. 원자료·재현성·결정성

- **원자료**: `tools/task2430/measured/ladder_<face>.tsv` 5종 커밋. SHA-256:

  | face | sha256(ladder TSV) |
  |---|---|
  | 한양신명조 | `5eaec37c0332e864f7c9098e4ecafda21e6b3a33c1265fae08fabf2466587a25` |
  | 한양중고딕 | `522b52554b8af55922052021ef8da1654dca6e5b34a4c888594bb9ae96e814f7` |
  | 한양견명조 | `fadc0f7f33a25811ad39179d5dfa7a3562ad301ebc5b4024ce9377faefc24994` |
  | 한양견고딕 | `0f0f28534f62d698c8e359fd6196f4cd5401c7603d893c1b1c94e8cf2a1a7440` |
  | 휴먼명조 | `51873bec50c59bde2b453a914edc0c51dac2bdc0f48750c3ccbc5969503a05cf` |

- **커밋 배열과의 정확 일치** (COM 불필요 — 어느 OS 에서든 재검증 가능):

  ```
  $ python tools/task2430/gen_metrics.py --ladder-dir tools/task2430/measured --verify
  한양신명조 → HanyangSinMyeongJo: 95/95 exact match — OK
  한양중고딕 → HanyangJungGothic:  95/95 exact match — OK
  한양견명조 → HanyangKyunMyeongJo: 95/95 exact match — OK
  한양견고딕 → HanyangKyunGothic:  95/95 exact match — OK
  휴먼명조   → HumanMyeongJo:      95/95 exact match — OK   (exit 0)
  ```

- **결정성**: 동일 환경에서 COM 생성부터 2회 독립 실행(run1/run2), 5종 TSV
  전부 **byte-identical** (`diff -q` 무차이).

## 5. 직선 따옴표(0x22/0x27) 제외 사유

직선 따옴표는 한/글 편집기 자동 치환(스마트 따옴표) 대상이라 삽입 경로에
따라 측정 여부가 갈린다(문단별 삽입=치환됨, 일괄 삽입=유지). 커밋 테이블은
치환 환경에서 측정된 93자 실측 + 2자 보간이며, 재현 파이프라인도 동일하게
제외·보간한다(`EXCLUDE_AUTOCORRECT`). 일괄 삽입으로 얻은 실측치는 보간 대비
차이가 있어(예: `'` 신명조 보간 241 vs 실측 395), **10k 게이트를 통과해야
하는 별도 교정 후보**로 남긴다 — 본 PR 범위 밖.

## 6. 대표 문서 fixture (#2430)

r16/r17 대표 회귀 문서 `21868765`(안양시 행정기구 및 공무원 정원 조례
시행규칙 [별표 2], 자치법규 공표물)와 한컴 기준 PDF 를 저장소에 포함:

| 파일 | sha256 |
|---|---|
| `samples/21868765_별표2_보건소_분장사무.hwp` (60.5KB) | `ae694583e739ac48af97cb12ce573c2da9f4cb637721fdf84e5af4bf7ca17c13` |
| `samples/21868765_별표2_보건소_분장사무.pdf` (151KB, Producer=Hancom PDF 1.3.0.550, 4쪽) | `b4a85b70cdb8a41f3a55b6863481360b56b4d069e803da9b40211c56692168aa` |

출처: 자치법규정보시스템(ELIS) 공표 별표 서식 다운로드본(원명
`21868765_[별표 2] 보건소의 부서별 분장사무(...).hwp`). PDF 는 위 환경의
한글 2022 로 인쇄한 정답지.
