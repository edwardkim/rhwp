# #8109 1단계 — Helvetica 선언·대체 글꼴과 양 저장 소비 진단

- 날짜: 2026-10-21 (Asia/Seoul)
- 승인: 작업지시자의 "진행을 승인합니다". [수행계획](../plans/task_m100_8109.md)의 1단계다.
- 검증 source/base: `80968497c2c178eeecb3b2e1d04d086ad27a0dc7`, `task_m100_8109`.
- 판정: **1단계 진단 완료, 2단계 구현 승인 대기**. 제품 수정·정식 회귀 추가·PR 제출은 아직 하지 않았다.

## 결론과 선행 구현

원본 Helvetica 선언을 유지하고 표준 대체 글꼴을 Arial로 지정하면 한컴 Print는 실제 ArialMT를
사용한다. 현재 rhwp는 HWP 재열기에서 그 정보를 paint에 연결하지 않고 HWP→HWPX 재저장에서도
잃는다. 이 두 경계를 보정한 뒤, 프로그램이 없는 PDF Base14 Helvetica에 명시적 대체를 부여한다.
점수만 맞추는 이름 교체나 전역 Helvetica alias는 사용하지 않는다.

선행 #7998 → [PR #8120](https://github.com/edwardkim/rhwp/pull/8120)의 실제 통합 SHA가
위 기준이다. PR 본문의 글꼴 예외와 [self-review](../pr/archives/pr_8120_review.md),
[stage33](task_m100_7998_stage33.md)·[stage37](task_m100_7998_stage37.md),
[최종 입력·Print/TSV](../pr/assets/issue_7998/final-p5/README.md)를 현재 코드와 대조했다.

저장 경계의 원 수정은 [PR #2538](https://github.com/edwardkim/rhwp/pull/2538)이다.
원 commit `157846ff3bd36823669ea0cfc7e60d86d25fa503` → 이식 commit
`4e122b69ca436405b0faaf27f96e562af818c469` → 통합 [PR #2670](https://github.com/edwardkim/rhwp/pull/2670),
merge `101708a819bafe6ce10d1b0eea704ad6b7acc737`의 본문·원 diff·후속 comment를 읽었다.
당시 수정은 HWPX substFont.face를 HWP alt_name으로 쓰는 한 방향의 이름 보존을 보호했다.
타입 byte의 모호성은 범위 밖으로 유지했고 렌더 소비·다시 HWPX로 저장하는 동작의 검증은 아니었다.
이 기존 이름 보존 계약은 유지한다.

## 현재 source 실행과 입력 신원

아래 빌드를 공용 target에서 3분57초·exit0으로 완료했다. 직전 fresh WASM도 현재 source와
제품 코드가 같다. 진단 중 제품 소스를 바꾸지 않았다.

```bash
CARGO_BUILD_JOBS=14 cargo build --locked --profile release-test \
  --features native-skia --lib --bin rhwp \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review
```

증거 루트는 ignored `output/pr-review/issue8109/stage1/`다. `producer-identity.json`에
Native·WASM/glue와 공급한 Arial/Batang/HANBatang 프로그램 SHA-256을 기록했다. 새 rlib에
진단 probe.rs를 링크했다. 이 helper를 정식 회귀로 계산하지 않는다.

공개 body-list.pdf의 SHA-256은
`7933b277f1456516fc078681f3ef1db5b0e147580a0e68c4086c43b0c96c8acc`다.
기존 공개 원본·저장본·Print 14개 역할 행을 inputs.tsv의 바이트·SHA와 대조했다.
PDF는 임베딩 없는 Type1 Helvetica/Helvetica-Bold를 선언하며, 기존 한컴 Print의 내장 face는
Haansoft Batang이다. pdffonts 관측과 문서 선언·공개 AFM 메트릭을 구분했다.

현재 Native로 원본을 열고 실제 X 삽입·직접 역삭제·양 저장·재열기를 실행했다. 새 저장본은
기존 입력과 바이트가 다르지만 글자모양 ID를 제외한 문단/줄 및 RenderTree는 각 대응 기존
저장본과 같다(`fresh-save-comparison.json`). ID 재배정과 geometry를 혼동하지 않는다.
**이 동일성으로 새 저장본의 한컴 Print를 실행했다고 주장하지 않는다.** 아래 시각 재현은
바이트가 검증된 기존 저장본과 대응 독립 Print를 사용했다. 바뀐 새 저장본의 Print는 2단계에서 검증한다.

## 생산·소비 경로와 실제 관측

| 위치 | 코드 및 실행 관측 |
| --- | --- |
| backend fonts/font_dict.rs의 FontInfo → layout/text_block.rs의 TextSpan | FontInfo는 font dictionary/프로그램을 처리하지만 TextSpan에는 프로그램 존재의 확정 상태가 없다. face 이름이나 디코딩 결과 None만으로 비임베딩을 추정하면 안 된다 |
| parser/pdf/lowering.rs:351 부근 | source family를 Font/CharShape에 등록한다. 실제 목록은 Helvetica 선언이며 대체 정보가 없다 |
| structured_lowering.rs:293, font_metrics_data/core14 | Helvetica AFM을 공통 측정에서 소비한다. AFM 전진폭은 실제 glyph program의 증거가 아니다 |
| renderer/style_resolver.rs:662 → Skia font lookup | non-embedded subst_font.face만 후보에 추가한다. 진단 HWPX는 Helvetica,Arial → 실제 Arial이지만 HWP 재열기는 Helvetica → 실제 Noto Sans KR이다 |
| serializer/doc_info.rs:459 → parser/doc_info.rs:297 | HWP 저장은 subst_font.face를 기록한다. 재열기는 alt_name으로만 읽고 별도 대체 유형 byte는 버린다. 이름이 CFB에서 사라진 것은 아니다 |
| serializer/hwpx/header.rs:204 | subst_font만 hh:substFont로 쓴다. HWP 재열기의 alt_name은 HWPX 재저장 때 누락된다 |
| model/style.rs 및 HWP3 생산자 | HWP3 변환은 alt_name에 원래 이름을 보관하기도 한다. 모든 alt_name을 같은 대체 의미로 해석하지 않고 HWP 표준 대체 필드의 생산 지점에서 정규화한다 |

선언 → AFM 측정 → 후보 체인 → 실제 Native face → 양 저장 → 재열기 trace를 연결했다.
원문·문단·글줄·최종 RenderTree는 *-ir.json, 실제 선택은 *-trace.json과 trace-summary.json에 있다.

## 독립 Print·전쪽 TSV와 확대 판독

기존 무편집 HWPX의 header.xml에만 substFont(face=Arial,type=TTF)를 추가한 진단본과,
이름 자체를 Arial로 바꾼 비교 대조군을 만들었다. 각 7개 언어 슬롯의 font 항목만 수정했고
문단·줄·크기·장평·원문은 그대로다. *-identity.json에 원본/변형 SHA·변경 항목을 기록하고
ZIP CRC를 확인했다. **수동 진단 입력이며 한컴 생성본·정상 회귀 fixture가 아니다.**

두 HWPX와 rhwp로 저장한 두 HWP, 총 4개는 각각 독립 한컴 2020 Print를 실행했다.
실제 engine2020·build11.0.0.9136·method0·hancom2020_pdf_driver_one_up 성공을
status.result로 확인했다. **네 Print 모두 내장 face가 ArialMT**다. input/output hash·job ID·
출력 방법은 print-jobs/*/{start,status,download}.json 및 pdffonts.txt에 보존했다.

Native/fresh WASM 각각 11입력/11쪽을 검증했다. 복원과 무편집의 입력/기준 PDF 바이트가
같은 2경로씩은 같은 backend의 캡처를 재사용했다. 실제 export는 backend별 9입력이다.
모든 원본·기준 PDF의 전체 쪽수는 1쪽, 누락·쪽수 차이는 0이다.

| 입력·상태 | Native raw | fresh WASM raw | 의미 판정 |
| --- | ---: | ---: | --- |
| PDF source → 원본 PDF | 99.93425% | 100% | 실제 Helvetica 프로그램 보유의 증거는 아님 |
| 기존 HWP/HWPX 무편집·복원 | 89.54211% | 90.70673% | 기존 Native 미달 재현, 실제 선택과 한컴 대체가 다름 |
| 기존 HWP/HWPX X 삽입 | 89.67777% | 90.89930% | 양 저장 형식의 같은 경계 재현 |
| 명시적 대체 Arial 진단 HWPX·HWP | 100% | 100% | **HWP Native trace는 Noto Sans KR. 높은 raw로 소비 실패를 면제하지 않음** |
| 이름 자체를 Arial로 바꾼 HWPX·HWP | 100% | 100% | Native도 Arial이나 원본 선언을 바꾸므로 선택할 구현 방식은 아님 |

명령은 visual.py, TSV 원문은 native/<key>/silhouette.tsv 및 wasm/<key>/silhouette.tsv,
실제 producer·명령·결과는 visual-results.json에 있다. 기존 6개 Native 미달을 PR 통과로 표시하지 않는다.

baseline Native/WASM과 대체 진단 Native HWPX/HWP의 compare·standalone overlay·review를
review/에 생성해 직접 열었다. baseline-zoom.png와 substitute-zoom.png는 같은 96dpi 원본을
같은 영역·4배로 확대했다. 기존 Print Batang의 serif와 Native/WASM sans의 차이, 대체 HWPX의
Arial 윤곽 및 HWP Native의 옅은 글자를 확인했다. 목록 2줄·내어쓰기·뒤 NEXT ITEM의 순서/원점은
유지된다. 수식이 없는 입력이며 글자·줄 간격도 판독했다. 2px raw100%가 이 차이의 부재를 뜻하지 않는다.

초기 PNG-pair 일반 모드 불허와 추가 WASM review의 Chrome 경로 누락은 도구·환경 실패다.
canonical 일반 모드와 명시 Chrome 경로로 실제 자료 생성을 완료했으며 제품 결함·통과에 포함하지 않았다.

## 2단계의 최소 수정 계약

1. **확정된 source에서만 정책을 생산한다.** 해당 PDF font resource의 Base14 Helvetica family/스타일과
   프로그램 선언 부재를 확인해 전달한다. 동명 Form resource, 실제 프로그램 있음, 선언됐으나 손상/읽기
   실패, 미확정을 구분한다. TextSpan 이름만으로 대체하지 않고 기존 backend의 유한 자원 해석을 재사용한다.
2. **원본과 fallback을 분리한다.** 원래 Font.name/source 선언을 유지하고 명시적 Arial 대체를 표준
   필드에 넣는다. 프로그램 있는 입력·명시적 Arial은 기존 계약을 유지한다. 전역 alias·크기/장평/alpha
   보정을 추가하지 않는다.
3. **HWP 대체를 생산 지점에서 공통 모델로 연결한다.** 독립 대체 이름·유형을 공통 subst_font로
   보존하고 기존 alt_name 계약도 유지한다. 양 serializer·style/paint가 같은 결과를 소비하게 한다.
   HWP3의 원래 이름 보존과 구분한다. 유형 byte는 원래 값과 사양 표21로 별도 검증한다.
4. **출력·재편집으로 닫는다.** HWPX→HWP→HWPX 및 역방향, 각 재열기 뒤 실제 편집에서 선언·
   대체·AFM/실제 face·문단/줄/소유를 확인한다. 같은 편집의 새 한컴 Print와 양 backend 전쪽 TSV·
   확대 판독 뒤에 정식 관계 회귀를 등록한다. 미해결 HWP face를 정상 결과로 고정하지 않는다.

위는 1단계 독립 실험으로 정한 구현 방향이다. 실제 정책 발행·프로그램 경계 반례·수정 전 FAIL/후 PASS의
정식 회귀·새 편집 Print와 최종 전체 검증은 다음 단계에서 완료한다. 현재 제품 해결·PR 준비 완료는 아니다.
