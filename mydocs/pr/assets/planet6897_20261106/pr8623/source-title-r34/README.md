# issue265의 원문 제목 소유와 저장 보존 계약 재검토

> R34 선행 검토의 역사적 기록이다. `5de92ecc46`에서 일반 ASCII 구두점까지 영문 슬롯으로
> 넓힌 해석은 R38에서 잘못으로 확인했다. `09ada34bf1`은 원래의 프로그램 선택을 복원했고,
> `d753f31aa8`은 HWP3 제목의 연속 run 기대값을 복원했다. 자연 글리프를 원문 주소로 찾는 개선과
> 독립 좌표는 유지한다. [현재 Git 분석과 실제 56건 통과](../../r39-git-restoration-analysis.md)를 따른다.
> 아래 수치·이미지는 당시 측정 이력이며 현재 보정본의 최종 시각 판정을 대신하지 않는다.


## 원 해결과 실제 실패

[PR #8191](https://github.com/edwardkim/rhwp/pull/8191)의 `60972534680a0c5e51e54ec189b92e252e81a867`은
원본 프로그램·제목 전진·그림과 표 뒤 본문을 보호했다. [PR #8623](https://github.com/edwardkim/rhwp/pull/8623)의
현재 적용 `1ab72b32062a5d694d1ad4e981434eaed365e731`은 자연 글리프 검사에 저장소 OFL 글꼴을
명시적으로 공급했다. 제목을 `한글+여는 괄호` 한 run으로 보는 전제는 이때 바뀌지 않았다.

당시 `5de92ecc46ff259341838fc8d0125a6a641b20fe`의 원문 슬롯 보정은 괄호를 영문 슬롯으로
분리한다. 실제 최소 공개 글꼴 환경의 기존 집중 검사 3건 중 2건은 이 run 묶음 전제와
같은 문자열을 찾는 선택 때문에 실패했고, 그림·표·후속 본문 검사는 통과했다.
원문 전체는 누락 없이 `4. 작업 할당 방식(Scheduling Algorithms)`이며 문자 시작점
3의 한글 → 11의 `(` → 12의 영문이 같은 기준선에서 점유 끝점으로 이어진다.
독립 Print에서도 원문 제목·괄호·끝 글자와 간격이 보존된다.

당시 검토에서는 한글·괄호·영문의 원문 소유와 순서·접합·기준선을 각각 확인하도록 고쳤다.
자연 글리프 검사는 문자열 묶음 대신 문단 121/문자 3으로 원문 글자를 선택했다.
글자 늘림의 실제 래스터 비교, 원본 프로그램 보존, 뒤 본문 관계와 독립 ±2px 기대 좌표는
유지한다. 기대 좌표를 현재 출력으로 복사하거나 허용치를 넓히지 않는다.

## 독립 기준과 측정

- 원본: `samples/issue_265.hwp`, SHA-256 `645525c8cd5ec11b1742ba7cfc759f68622861916233b5e982385cdb12f0ced2`.
- 원본 그대로의 독립 한컴 Print: `pdf/issue_265-hwp-2020.pdf`, SHA-256
  `c8e4218c7b80f1fb4f7877f5613eb6fd82fa1ca2b1ca3cde6266b81586357302`.
- 2026-11-07, compat 2022, Print, 96 DPI, 색상 경계 보정 전 2px raw. PDF를 재산출하지 않았다.
- Native 제품 `ef77d740e1`, 최적화 fresh WASM 제품 `99f1340628`. 후속 R35의 WASM 재검증은 별도다.
- 시각 비교는 시스템 전체 글꼴 공급과 명시적 빈 `FontEnvironment`를 사용했다.
  Rust 집중 검사는 공개 글꼴 5개만 보이는 최소 환경에서 별도로 실행한다.

실제 공개 `exportHwp`/`exportHwpx` 저장본은 원본 내용·조판 보존의 대조군으로 같은 원본 Print와
비교한다. 이것을 새 저장본 자체를 한컴에서 다시 인쇄한 PDF라고 부르지 않는다.
기존 2026-10-20 저장본 Print도 별도 대조했지만 캡션 번호가 1로 초기화된 과거 저장 결과여서
현재 저장본의 내용 정답으로 사용하지 않는다. 원본 Print의 그림 2·그림 4를 현재 저장에서도
유지하는지 확인한다. 제목·표의 기존 독립 좌표와 줄·문단 소유를 함께 검사한다.

| backend | 실제 입력 | 전체 쪽수 | 원본 Print 대비 전쪽 최저 | 전쪽 TSV |
| --- | --- | ---: | ---: | --- |
| native | issue265-original | 16 | 99.59% | [native-issue265-original.tsv](native-issue265-original.tsv) |
| native | issue265-hwp-save | 16 | 99.58% | [native-issue265-hwp-save.tsv](native-issue265-hwp-save.tsv) |
| native | issue265-hwpx-save | 16 | 99.59% | [native-issue265-hwpx-save.tsv](native-issue265-hwpx-save.tsv) |
| wasm | issue265-original | 16 | 99.51% | [wasm-issue265-original.tsv](wasm-issue265-original.tsv) |
| wasm | issue265-hwp-save | 16 | 99.50% | [wasm-issue265-hwp-save.tsv](wasm-issue265-hwp-save.tsv) |
| wasm | issue265-hwpx-save | 16 | 99.51% | [wasm-issue265-hwpx-save.tsv](wasm-issue265-hwpx-save.tsv) |

모든 경로에서 3·8·12쪽은 Native 100.00%/100.00%/99.97%, fresh WASM
99.95%/99.95%/99.93%다. 원본과 저장본은 각각 16쪽이며 한 쪽도 90% 미만이 없다.
같은 쪽의 제목·괄호·누적 끝점, 그림 캡션·뒤 본문·표 외곽·뒤 본문의 확대 판독을 별도로 했다.
raw 통과를 픽셀 동일이나 새 저장본의 독립 인쇄와 동일시하지 않는다.

- 원본 12쪽 [Native review](native-issue265-original-review-012.png) / [overlay](native-issue265-original-overlay-012.png).
- 원본 12쪽 [fresh WASM review](wasm-issue265-original-review-012.png) / [overlay](wasm-issue265-original-overlay-012.png).
- 실제 HWP 저장 3쪽 [Native review](native-issue265-hwp-save-review-003.png) / [overlay](native-issue265-hwp-save-overlay-003.png).
- 실제 HWP 저장 8쪽 [Native review](native-issue265-hwp-save-review-008.png) / [overlay](native-issue265-hwp-save-overlay-008.png).
- 실제 HWP 저장 12쪽 [fresh WASM review](wasm-issue265-hwp-save-review-012.png) / [overlay](wasm-issue265-hwp-save-overlay-012.png).

기본 실행은 `scripts/visual_sweep.py --silhouette-only --resume --compat 2022 --dpi 96`이며
입력·기준을 위 파일로 지정했다. 원본 전체 PNG와 공개 API 저장본의 실제 전체 PNG를 같은
원본 Print PNG에 대조해 TSV와 대표 패널을 산출했다. 실제 저장본 SHA-256은 HWP
`58341ecf833871d1333ae030c67408f26adf516bbfa3d985485a494b25335701`, HWPX
`b94db0caae07a9b247cfa1b5120237e27ec0b44aa5c84730c052fc4d4178fa9d`다.

R38 제품의 같은 원본·실제 저장본 전 16쪽 Native/fresh WASM 재검증은 진행 중이다.
원본 제목의 복원한 기대값을 포함한 최소 글꼴 집중 회귀는 실제 56건 모두 통과했다.
