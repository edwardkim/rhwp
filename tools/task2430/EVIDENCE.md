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

아래는 측정 환경에서 5종을 per-face 실행했을 때의 preflight **콘솔 stdout**
로그다(파일 형식이 아니라 실행 시 표준출력):

```
한양신명조: readback=('한양신명조', FontType=2) OK
한양중고딕: readback=('한양중고딕', FontType=2) OK
한양견명조: readback=('한양견명조', FontType=2) OK
한양견고딕: readback=('한양견고딕', FontType=2) OK
휴먼명조:   readback=('휴먼명조',   FontType=2) OK
```

> **커밋 아티팩트 상태(재보존 완료, 2026-07-21, #2677)**: 위 5행 블록은
> *콘솔 로그*이며, 이제 커밋된 `tools/task2430/measured/preflight_report.tsv`
> 파일에도 **5종 identity 행이 모두 보존**된다(한양신명조·중고딕·견명조·견고딕
> = FontType=2, 휴먼명조 = FontType=2). 최초 커밋 시점에는 per-face 프로세스
> 분할 실행에서 매 실행이 파일을 덮어써 마지막 face(휴먼명조) 1행만 잔존했으나,
> `hy_ascii_ladder.py` preflight 를 requested_face 기준 **누적 병합**으로
> 개선(#2510 리뷰 후속 커밋 #2675)한 뒤 동일 Windows + 한컴(한글 2022) COM
> 환경에서 5종 per-face 를 재실행해 5행 아티팩트를 재보존했다(재실행 시
> `ladder_<face>.tsv` 5종은 커밋본과 byte-exact 일치 확인 — 측정 결정성).
> `--verify` 재현성(아래 §4)은 커밋 TSV↔배열 일치만 보증하며 HFT vs fallback
> identity 는 이 preflight 아티팩트로만 입증되는 점에 유의한다.

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

- **원자료**: `tools/task2430/measured/ladder_<face>.tsv` 5종 커밋. SHA-256
  (**LF 체크아웃 = Git 저장 바이트 기준**. 저장소는 TSV 를 LF 로 저장하므로
  아래 값은 `git show HEAD:<path> | shasum -a 256` 또는 LF 체크아웃 파일의
  해시와 일치한다. 최초 커밋 EVIDENCE 는 Windows CRLF 바이트 해시를 실어
  LF 체크아웃과 어긋났던 것을 #2510 리뷰 후속에서 LF 기준으로 정정했다):

  | face | sha256(ladder TSV, LF) |
  |---|---|
  | 한양신명조 | `35e546ea1f1788faf0fe657f78dd88e7fe57db017cbab164a3e710831fbb6164` |
  | 한양중고딕 | `8d1d628bf0ed49797945af68e1103dce2f68c4620ebc14712c1626c2176d416e` |
  | 한양견명조 | `7f56f4f7735f2eb724974296237a02fb56e47f56b11c99e247b0e2022d1629a8` |
  | 한양견고딕 | `b7535277bd6bfb05dc8e206e77b0a814a5b4c30d1cb536ba45372a92351cd19a` |
  | 휴먼명조 | `7e4c2a8d8734f2c7809135e08f18063df12d1dd26a9801fe5260ea3f934038f9` |

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
