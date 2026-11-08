---
kind: evidence
status: local-stage-verified
canonical: mydocs/manual/pr_review/visual_fixture_evidence.md
last_verified: 2026-11-06
---

# #7642 수학 명령·인용 글리프의 공통 투영

구현 커밋은 `0184227683a6219fddfd5749d230f7220f482130`, 고정 upstream base는
`170b886f167b705b6e630c5b34bb638d92fae0a9`다. 이 준비본은 전체 회귀·lint/build의
최종 결과를 기다리고 있으며 아직 저장소의 정식 증적 경로에 복사하지 않았다.

## 실제 보정과 검사

독립 한컴 2020 Print에서 여섯 크기의 삼각형 명령은 Haansoft Batang U+2206,
인용 작은따옴표는 HYhwpEQ U+E050으로 출력된다. 이 공통 glyph·face·논리 전진을
LayoutBox에서 보존한다. 원문 명령과 인용 밖 미분 prime, 일반 math face, legacy 경로를
구분하며 혼합 인용 문자열은 기존 가로 합성을 사용한다.

최종 검사 본문을 고정 base와 후보 rlib에 연결한 결과는 **5 PASS / 2 FAIL → 7 PASS**다.
실패 두 건은 표시 글리프의 식별이다. 고정 좌표 검사는 수정 전부터 통과하므로 기존 정상
좌표의 후퇴를 보호하며, 좌표 자체를 이번 보정으로 개선했다고 주장하지 않는다.
현재 Native 정식 suite의 7건도 통과했다.

수식 진단 전3쪽 raw는 Native **100 / 99.98115 / 99.95925%**, fresh WASM
**100 / 99.99209 / 100%**다. 양쪽 98%와 직접 판독을 확인한 후 독립 Print의
원점(72pt)을 96px로 환산한 ±2px 검사를 추가했다. 원본 전진·소유·내용 순서·glyph/face는
별도의 관계형 검사로 확인한다.

## 전쪽 결과

같은 입력·독립 Print·96DPI·compat2022·정본 2px raw 판정을 사용했다.
고정 base와 후보의 Native/fresh WASM 전쪽 TSV 24개를 함께 보존한다.
각 TSV는 전쪽을 새로 산출했고, base/후보 PDF PNG의 SHA-256도 페이지별로 동일하다.
허용 하락은 사용자가 지정한 **0.1%p**다. 모든 조건에서 쪽수 일치·누락0을 확인했다.

| 조건 | 각 경로 쪽수 | Native 최저 raw | fresh WASM 최저 raw | 90% 미달 | 0.1%p 초과 하락 |
| --- | ---: | ---: | ---: | ---: | ---: |
| prime-width | 3 | 99.95925% | 99.99209% | 0 | 0 |
| nov-original | 21 | 98.78595% | 98.77736% | 0 | 0 |
| nov-resaved | 21 | 99.03802% | 99.02457% | 0 | 0 |
| science | 4 | 97.11295% | 96.91722% | 0 | 0 |
| rectangles | 50 | 98.31507% | 98.32714% | 0 | 0 |
| rowbreak8539 | 3 | 100.00000% | 100.00000% | 0 | 0 |

총 **각102쪽, 양쪽204쪽**이다. 원본 11월 입력과 한컴 재저장 입력은 구분했다.
새 devel #8542/#8539의 이어받기 행 대조군은 양쪽 전3쪽 100%를 유지한다.
과학 시험지 등의 기존 미세 기준선·가장자리 차이는 점수만으로 완전 일치로 표시하지 않는다.

## source와 실행 경로

- Native/fresh WASM producer는 parent `ae80442566` 위 실제 공통 glyph 보정에서 만들었다.
  최종 구현 커밋의 runtime 파일은 producer와 모두 동일하며, 이후 추가한 것은 회귀 검사와
  주석이다. 대응은 `producer-commit-correspondence.json`에 기록했다.
- Mac에서 locked wrapper의 `--target web --out-dir pkg --no-opt`로 fresh WASM을 빌드했다.
  로컬 대체 빌드이며 Docker 최적화 결과가 아니다.
- root pkg·Studio public의 JS/WASM을 확인하고, 새 브라우저의 실제 초기화 fetch 응답 hash를
  관찰했다. 별도 재요청이나 캐시 응답으로 대체하지 않았다.
- `.log`·임시 driver·중간 진단은 ignored output에 보존한다. 아래 이미지와 TSV는 정본
  Visual Sweep이 생성한 파일의 바이트를 그대로 보존한다.

## 대표 이미지

| 주장 | Native | fresh WASM |
| --- | --- | --- |
| 18pt 삼각형·인용 기호와 ABC | [review](native-prime-width-p003-review.png) · [overlay](native-prime-width-p003-overlay.png) | [review](wasm-prime-width-p003-review.png) · [overlay](wasm-prime-width-p003-overlay.png) |
| 실제 11월 문서의 삼각형 명령 | [review](native-nov-original-p017-review.png) · [overlay](native-nov-original-p017-overlay.png) | [review](wasm-nov-original-p017-review.png) · [overlay](wasm-nov-original-p017-overlay.png) |
| 재저장본의 최저 페이지·양단 내용 | [review](native-nov-resaved-p018-review.png) · [overlay](native-nov-resaved-p018-overlay.png) | [review](wasm-nov-resaved-p018-review.png) · [overlay](wasm-nov-resaved-p018-overlay.png) |
| 새 devel 이어받기 행의 소속·하단 | [review](native-rowbreak8539-p003-review.png) · [overlay](native-rowbreak8539-p003-overlay.png) | [review](wasm-rowbreak8539-p003-review.png) · [overlay](wasm-rowbreak8539-p003-overlay.png) |

검사 helper 중복 로딩을 보정한 커밋 `dce48c65da5ae2dc2edbc3498009a5681bc68144`의 실제 source로 전체 nextest를
다시 실행하여 **12,649 PASS / 0 FAIL / 45 skip**을 확인했다(테스트 422.229초,
컴파일 포함 889.113초). Native 집중 11 PASS, Node 계약 26 PASS이며 Native feature·default·
WASM·all-target Clippy, workspace build, fmt·manifest·unit tier 모두 exit 0이다.
필수 Native Skia는 같은 runtime에서 lib 4,039 PASS / 12 ignore, PNG 2 PASS,
직접 PDF 22 PASS다. 이후 바뀐 것은 검사·주석과 helper의 로딩뿐이다.
두 기호 검사의 assertion 본문과 helper 구현은 그대로임을 해시로 확인했다.
`helper-commit-correspondence.json`과 `glyph-validation-summary.json`이 해당 대응을 기록한다.
최종 제출 head의 편람 Native gate와 다음 오래된 이슈·원격 통합은 아직 미완료다.
이 로컬 단계 완료만으로 이슈를 종료하거나 PR 준비 완료라고 하지 않는다.
