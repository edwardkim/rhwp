# 구형 수식의 아래첨자 글줄 기준선

Issue: #4138. 부모 `79ee0477b6`과 base `a7feb849e11642b1438b7e9ee460ff4c8a7c946e`에서
구형 HFT 아래첨자의 기준선 관계를 보정했다. **가로 배치와 전체 98% 목표는 미완료**다.
129쪽 raw 하락도 남아 있으므로 이 단계의 검사 통과를 PR 제출 가능으로 바꾸지 않는다.

## 독립 근거와 적용 경계

기존 입력 `saved-split-1x2.hwp` SHA-256
`4a96f1fc838eb7239d8c85b7698b9e04f83fc6b2f271c948ccae038a1c6c89b9`와
`pdf/issue4138-ascii-split-20261016-2024.pdf` 197쪽을 그대로 사용했다.
별도 6·9·12·18pt, 40조건의 한컴2024 Print에서도 아래첨자 28개를 대조했다.
본체와 아래첨자의 기준선 차이는 약 0.28em이고 자식 글자 크기는 본체의 약 0.64배다.
실제 원본 44·129·130쪽의 문자별 원점도 같은 관계다.
독립 문자 원점·출처 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue4138/legacy-subscript-frame-20261107/independent-baselines.json`)에는 글꼴 윤곽이나 프로그램을 넣지 않았다.
40조건 입력·PDF와 진단 스크립트는 ignored output에 남겼다.

기존 구현은 구형 글리프의 0.8em ascent를 일반 수식 fallback에 적용해 아래첨자를
본체보다 0.032em만 내렸다. 이제 같은 구형 face의 단일 글줄·동일 기준선·0.64배 자식임을
확인한 경우 두 글줄의 기준선 관계로 자식 y와 포함 높이를 함께 계산한다.
분수·중첩 첨자·다른 표시 글리프를 단일 글줄로 접지 않는다.
`script_subscript_geometry`를 아래첨자·동시 첨자가 공유하고, 측정 프레임과 실제 paint가
같은 자식 상자를 소비한다. 문자 수·원문·글자 크기·가로 전진은 이 단계에서 바꾸지 않았다.

기존 #8064의 `4dbb274ba4`·`38cb1ec05e`와 구형 표시 연결 `315de5dcb3`,
그룹/첨자 소유 `54874adab7`·`e9b373d755` 및 원 보고서를 먼저 대조했다.
현대 HYhwpEQ/HancomEQN 경로는 보존하며, 별도 현대 40조건의 전체 `LayoutBox`·페이지
배치가 부모와 바이트 직렬화 값까지 같은 것을 확인했다.

## 실제 검사와 전쪽 결과

정식 관계형 4건은 **수정 전 4 FAIL → 수정 후 4 PASS**다. 실제 원본의 `d_B`, `Q_n`,
`g_call`, `C_L`에서 본체·자식 소속, 기준선 차이, 자식 글자 크기와 프레임 포함을 검사한다.
가로 배치가 남은 쪽을 완료 페이지로 판정해 절대 px 기대값을 추가하지 않았다.
기존 112건·새 정식 4건·40조건 진단 1건을 포함한 집중 실행은 117건이다.
40조건을 40개의 독립 Rust test로 부풀려 세지 않는다.

| 쪽 | Native 수정 전 | Native 수정 후 | WASM 수정 전 | fresh WASM 수정 후 |
| --- | ---: | ---: | ---: | ---: |
| 44 | 97.84052% | 97.97440% | 97.89153% | 98.01130% |
| 45 | 98.92784% | 98.94138% | 98.94677% | 98.95601% |
| 129 | 94.83034% | 94.76447% | 94.98570% | 94.83414% |
| 130 | 95.95341% | 95.98723% | 95.90371% | 95.92003% |

양쪽 전197쪽을 192DPI·compat2024·Print·PDFium·raw 2px로 다시 산출했다.
변경된 PNG는 44·45·129·130쪽 네 쪽뿐이며 나머지 193쪽은 부모와 바이트가 같다.
누락·쪽수 차이는 없다. Native 최저 94.76447%, 미달44·129·130쪽이고,
WASM 최저 94.83414%, 미달129·130쪽이다. 21쪽의 앞 단계 결과는 그대로 유지했다.
Native의 자동 공급 캐시 바이트 차이는 같은 현재 환경·공급 파일로 전후 네 쪽을 다시
출력해 분리했다. 전후 공급 해시는 같고 각 PNG도 각각의 전쪽 결과와 같다.
WASM의 전쪽 전후 공급 기록은 동일하다.

## 직접 판독과 남은 보정

44·129·130쪽의 review/overlay를 전후 확대했다. 아래첨자의 세로 관계는 회복됐지만
원래부터 남은 복합 그룹·괄호·분수의 가로 전진과 위첨자 위치 차이는 남아 있다.
129쪽은 전체 raw가 오히려 낮아졌다. 이 하락을 숨기거나 관계형 검사 통과로 면제하지
않고 다음 가로 배치 보정에서 다시 전쪽 검증한다. 45쪽도 점수는 98% 이상이지만
`Q_n=Q/(N-1)`의 가로 위치 차이가 보여 완료 페이지로 판정하지 않는다.
기존 75자 로컬 수식 글꼴 공급 조건이며, 별도 85자 글꼴 진단 결과와 섞지 않았다.

| 경로·쪽 | 수정 전 review | 수정 전 overlay | 수정 후 review | 수정 후 overlay |
| --- | --- | --- | --- | --- |
| Native44 | [review](before/native-p044-review.png) | [overlay](before/native-p044-overlay.png) | [review](after/native-p044-review.png) | [overlay](after/native-p044-overlay.png) |
| Native129 | [review](before/native-p129-review.png) | [overlay](before/native-p129-overlay.png) | [review](after/native-p129-review.png) | [overlay](after/native-p129-overlay.png) |
| Native130 | [review](before/native-p130-review.png) | [overlay](before/native-p130-overlay.png) | [review](after/native-p130-review.png) | [overlay](after/native-p130-overlay.png) |
| WASM44 | [review](before/wasm-p044-review.png) | [overlay](before/wasm-p044-overlay.png) | [review](after/wasm-p044-review.png) | [overlay](after/wasm-p044-overlay.png) |
| WASM129 | [review](before/wasm-p129-review.png) | [overlay](before/wasm-p129-overlay.png) | [review](after/wasm-p129-review.png) | [overlay](after/wasm-p129-overlay.png) |
| WASM130 | [review](before/wasm-p130-review.png) | [overlay](before/wasm-p130-overlay.png) | [review](after/wasm-p130-review.png) | [overlay](after/wasm-p130-overlay.png) |

전쪽 원문: [이전 Native TSV](before/native-silhouette.tsv) · [현재 Native TSV](after/native-silhouette.tsv)
· [이전 WASM TSV](before/wasm-silhouette.tsv) · [현재 WASM TSV](after/wasm-silhouette.tsv).
45쪽: [Native review](after/native-p045-review.png) · [overlay](after/native-p045-overlay.png)
· [WASM review](after/wasm-p045-review.png) · [overlay](after/wasm-p045-overlay.png).

최종 소스·빌드·검사 출처는 validation.json (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue4138/legacy-subscript-frame-20261107/validation.json`)에 있다. WASM은 Mac의 locked
wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다. Docker 최적화·최종
전체 nextest·편람 게이트·실제 소비 경로 검증의 완료를 뜻하지 않는다. 다른 기존 미달
문서와 Native 공백 대조군의 차이도 남아 있으며, 전체 조건 완료 전 PR 제출을 보류한다.
