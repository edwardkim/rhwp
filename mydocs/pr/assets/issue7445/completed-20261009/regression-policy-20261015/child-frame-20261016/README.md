# 부모 셀 분할과 자식 물리 프레임·그림 줄 소유 보정

Issue: #7445, #4138. base `375aff29498166260a22796d54e67f508c84aada`.
C59는 parent `6a679a55ea1a5fd5604db09e01d7205686bad2bc`와 두 layout source의 보정이다.
전체 제출 게이트 완료 보고가 아니라, 이 세 입력의 전쪽 검증과 회귀 추가 근거다.

## 독립 입력과 기준

| 입력 | 입력 SHA-256 | 독립 한컴 Print PDF | PDF SHA-256 | 쪽수 |
| --- | --- | --- | --- | ---: |
| `samples/issue1949_giant_cell_nested_tables_perf.hwp` | `ef10261cd29325116028e4f4f3e6be1a72c675eb771bddfd8484e7fe5aa94b4e` | `pdf/issue1949_giant_cell_nested_tables_perf-hwp-2024.pdf` | `9fb08dea604813eed90a0449f29682d400be54dacd413352cc4f680f37cd4a44` | 115 |
| `tests/fixtures/issue4138/tab-period-split.hwp` | `d9bc6b13d54f5179c2cf57e6a2310a28efce4f8f6d6e74137a5828c9878a24b6` | `pdf/issue4138-tab-period-split-20261015-2024.pdf` | `687f1404e56a764258c93fb0e59d9fb1dd870b506d36451fbfa833fd207f8294` | 197 |
| `../issue4138-current-production/saved-split-1x2.hwp` | `4a96f1fc838eb7239d8c85b7698b9e04f83fc6b2f271c948ccae038a1c6c89b9` | `pdf/issue4138-ascii-split-20261016-2024.pdf` | `5a82290de1571c692c7246fe6b57ae136602ae52c2068f2568fffc6c91bfd571` | 197 |

현재 source로 실제 원본 편집·저장한 산출물도 마지막 입력과 byte-identical이다.
이전 실패 입력을 새 입력으로 바꾸어 수용한 것이 아니다. 원본115/저장197 검사를 유지했다.

## 같은 조건의 전쪽 실제 PNG

canonical Visual Sweep,192DPI,compat2024,Print profile,threshold32,2px 이웃 관용90%다.
Native Skia PNG와 fresh WASM Canvas PNG를 각각 직접 출력했다. PDF는 `pdftoppm -thinlinemode shape`다.
같은 명시적 빈 FontEnvironment와 독립 전체 한글 표본에서 공급한 HFT를 사용했다.
HFT SHA `7257416630313e6989c7690847f2c372a3d5106a2c215a6e16ee03573fe995dd`의
11172자 윤곽·폭·bearing을 별도 표본과 전수 대조했다. 대상 정답 PDF를 글꼴 제작에 사용하지 않았다.
글꼴은 커밋하지 않는다. WASM은 locked wrapper의 `--target web --out-dir pkg --no-opt` Mac 로컬 대체 빌드다.
루트pkg·Studio public·동결 pkg의 JS/WASM hash가 일치한다. 실제 Studio 초기화 검증을 대신하지 않는다.

| 입력 | Native 최저 | fresh WASM 최저 | 각 경로 쪽수 | 90% 미만 |
| --- | ---: | ---: | ---: | ---: |
| 원본 | 90.62416% | 90.50161% | 115/115 | 0 |
| 과거 분할본 | 90.27729% | 90.21662% | 197/197 | 0 |
| 현재 생성본 | 90.27729% | 90.21662% | 197/197 | 0 |

각 backend509쪽,합계1018쪽이다. raw TSV·producer·전체 로그의 정본은 ignored
`output/pr-review/7445-completed-20261009/regression-policy-audit-20261015/rebase-375aff29-20261016/`의
`candidate-c59/{producer,wasm-producer}.json`, `giant-whole-c59/{scope,results,state}.json`와
각 입력·backend별 `silhouette.tsv`다. source hash로 후보와 후속 commit의 runtime 동등성을 확인한다.

## 직접 판독과 중요한 좌표

[review.png](review.png)는 독립 Print / 수정 전 C55 Native / 수정 후 C59 Native 순서다.
[WASM 비교](wasm-review.png)도 같은 구성이다. 두 분할본177쪽은 Native99.82990% / fresh WASM99.83114%다.
두 독립 PDF의177·178쪽 raster hash도 각각 같다. 표 종료 괘선·제목·본문 순서와 미세 간격을
확대 판독했고,앞 쪽에 보이던 다음 쪽 그림 일부가 없어졌다.
[178쪽 대조](next-page-review.png)에서 두 원문 그림·후속 본문이 보존됨을 확인했다.
178쪽은98.23931/98.26963%다. 잔여 raster·글꼴 차이는 있으며100% 픽셀 동일을 주장하지 않는다.

새 `tests/cases/issue_4138_child_frame_ownership.rs`는 다음을 보호한다.

- 원본29쪽: 중첩 표 뒤 빈 줄의 전진은 원본 호스트 줄높이·줄간격·바깥여백 관계로 검사한다.
  실제 gap22.06667px는 독립 원본 속성 구간19.77333–22.30667px 안이다. 양쪽97.78%여서 px 핀을 만들지 않는다.
- 두 분할본177쪽: 독립 PDF의72pt를96CSSpx로 환산한 괘선 끝525.185343,
  제목x87.792419·baseline566.101318을 각각±2CSSpx로 검사한다.
  capture192DPI의2raster px 관용 점수와96CSSpx 좌표 허용오차를 구분한다.
  내용·순서 관계를 함께 유지한다. 같은 핵심 동작을 검증하는99%대177쪽을 우선 선정했다.
  100%인197쪽은 이 중첩 표 종료·뒤 제목 경계를 소유하지 않아 대체하지 않는다.
- 원문2286의 둘째 저장 줄이 소유한 그림은177쪽의 공백 첫 줄에서 paint하지 않는다.
  177쪽 그림0·178쪽 두 그림의 단일 배치·순서·비겹침과 앞 문단의 누락/중복 없음을 검사한다.

정식3건은 C59에서3PASS(1.407초), 보정 직전6a679a55에서1PASS2FAIL(0.801초)이다.
실패는 제목baseline587.24와 앞 쪽 그림 paint로 재현됐다. build 실패를 결함 재현으로 세지 않는다.
기존9묶음45건도 C59에서45PASS0FAIL이며,#7902 원래 물리 프레임7건을 포함한다.
전체 nextest·범위 lint/Skia/Studio/Gym·나머지 문서·실제 소비 경로 검증은 아직 진행 대상이다.
기존104쪽 px 핀은 원래96DPI 조건에서 재확인했다. HWP와HWPX 모두 Native99.98469% /
fresh WASM99.98462%이며 [현재 직접 비교](original104-review.png)에서 표·제목·본문·그림을 판독했다.
HWP는 위9fb08dea Print,HWPX는 `pdf/issue1949_giant_cell_nested_tables_perf-hwpx-2020.pdf`
SHA `ba065afdd3ace7fd6b87fe82848e6cae3d01915a1606e4d8a70e4c2153d57d73`이다.
독립 기준선374.310954/425.455037/882.235840 CSSpx가 양쪽 PDF에서 동일하다.
처음 HWP의 다른 과거2020 Print(0c2530d9)를 사용한 진단은96.53475/96.47616%였다.
그 결과를 최종 Print 조건의 실패로 섞지 않는다. 192DPI와96DPI 점수도 서로 대체하지 않는다.
`px104-audit96-final-reference`에 정확한 원본/PDF/compat/TSV/PNG를 보존했다.
기존 ±2CSSpx와 관계 검사를 유지했으며 HWPX 전115쪽은 이 선택1쪽 확인으로 완료하지 않는다.
