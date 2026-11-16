# f8234f6f90 리베이스와 기능 보존 검증

## 최종 제품 검증 상태

제품 source `c95f9ade4e8acf9d901c01136ef2b9bacd60c497`의 최종 검증은 [검증 기록](verification-record.md)에 통합했다. 아래 source833 및 실행 중 표시는 당시 단계별 역사적 기록이며 현재 후보의 미완료 판정이 아니다. 새 굵게 반례와 일반 대조군의 결론은 [HFT 서식 범위 보정](hft-bar-style-coverage-analysis.md)을 따른다. 원격 CI·PR·병합 결과는 아직 별도 확인 대상이다.

## 리베이스 결과

- 이전 base: `53da4d2da28ae0cacf8feac3a989cc6a69fb2622`.
- 검증 완료된 이전 head: `433c74a915daa6ca335710955f433757817f82e3`.
- 새 base: `f8234f6f90cb25d9601c9d5d762e0a565aea19e4` — PR #9083의 다섯 원본 389쪽 보정.
- 리베이스 직후 head: `b780590f6f6f2d4f6800ac6e8e36546f6a52c014`.
- `git rebase upstream/devel`: 종료 코드 0, 충돌 파일 0개.
- 원 기능·메인터너 보정 28개 커밋을 모두 순서대로 재적용했고 임의로 생략하지 않았다.
- 이전 검증 head는 원격 `review/pr-9075-rowbreak-integration`에 고정해 당시 source·증적 SHA를 보존했다.
  새 base 검증 전에는 PR을 생성하거나 리베이스된 head로 덮어쓰지 않는다.

## 새 base의 적용 범위

PR #9083은 저장 번호·RIGHT 탭·그림 캡션·병합 칸의 완료 행·각주 줄 소유와 그라데이션/밑줄을 보정했다.
공통 layout/typeset와 paint JSON·CanvasKit 소비 코드가 바뀌었으므로 이전 source43의 결과를
리베이스된 source의 새 실행으로 기록하지 않는다. 해당 PR의 다섯 입력과 독립 PDF는 새 base에 포함돼 있다.

| 입력 식별자 | 독립 PDF 쪽수 | 보존할 주요 동작 |
| --- | ---: | --- |
| CBTA | 204 | 저장 번호 경계·RIGHT 탭·캡션·118/162/166쪽 병합 행 |
| 권고안 | 22 | 목차·그라데이션과 CanvasKit의 실제 반경 |
| 영어 시험지 | 8 | 반각 공백·밑줄 원점 |
| HWP 컨트롤 API | 105 | 원본 표 외곽·본문·그림 소유 |
| 각주 표 | 50 | 주석 전용 줄과 표 이어받기 |

## 완료 전 조건

- 새 source의 컴파일과 원 PR 첫 줄·내용 보존 경계를 먼저 실행한다.
- 새 base에서 반영한 관련 회귀와 독립 Print의 주요 페이지를 같은 조건으로 확인한다.
- 원 문서 14쪽, 상향한 정본 36개 데이터셋·3,542행과 실제 편람 gate를 확인한다.
- 실제 영향의 소스/호출 경계를 대조하고 충분한 기존 증거만 재사용한다.
- 새 head의 필수 Rust·Native·WASM·Studio 검증과 생성 바인딩의 일치를 완료한다.
- 하락이 있으면 구현을 보정하고 현재 source에서 재검증한다. 보호 기준을 낮추지 않는다.

현재는 새 source의 필수 검증을 수행 중이다. 전체 로컬 검증과 원격 CI·PR·머지 완료가 아니다.
원시 리베이스·range-diff 로그는 ignored output에 보존하며 정식 문서에는 결과를 요약한다.

## 새 source의 Native 선행 확인

2026-11-15T09:11:48.550927+09:00 기준으로 source `833c22d0de3e6ec8cbe090a1358a6d457bac8039`의 실제 Native 빌드를 완료했다.
Native binary SHA-256은 `25ac5b6d9eb366ac78177e0849fcdbf706e1b9f36b2b25bec50ba999d24b17e0`이다.
같은 원본·독립 PDF·기존 명시적 글꼴 환경과 Print·compat 2022·96 DPI에서 전 14쪽의
공식 raw 표시값이 모두 100.00%다. 전체 텍스트 보존 감사·같은 쪽 직접 판독과 fresh WASM,
상향된 보호 전쪽·새 base의 다섯 원본 및 전체 회귀 검증은 후속 실행 중이다.
부분 완료를 제출 준비 완료로 표시하지 않는다.

### Native 경계 검사와 기존 직접 판독 재사용

최소 공개 글꼴 격리에서 원 PR의 세 검사와 새 전체 내용 보존 검사, 총 네 검사가 모두 통과했다.
쪽수 14·5쪽 첫 조각·6쪽 나머지 두 줄·7쪽 후속 제목과 원본 셀 전체 내용의 누락·중복 방지를
확인했다. suite 내 나머지 340개는 focused 필터로 실행하지 않았으며 전체 회귀의 skip으로 합산하지 않는다.

실제 집중 검사 로그의 SHA-256은 `f838966042301dde414c4478aa8916b88e6ed734c8ab1240e513fe1028bfddaf`이다.
새 Native 바이너리로 재출력한 전 14쪽 PNG는 이전 source43의 실제 PNG와 각각 바이트가 같다.
이에 따라 이전 독립 Print의 같은 쪽·같은 배율 직접 판독을 재사용한다. 컴파일·IR·SVG 해시로
PNG 재출력을 대신하지 않았으며 전체 14쪽의 새 실루엣 측정도 완료했다. fresh WASM은 별도 검증 중이다.

## 최적화 WASM·Studio 선행 검증 완료

source `833c22d0de3e6ec8cbe090a1358a6d457bac8039`의 최적화 WASM binary SHA-256은 `d40ecc2600f0f5a37e1783555d6575050044d529f18a7a9fba83c885162f632d`이다.
실제 Canvas browser23개가 통과했고, 새 package로 재출력한 원 문서 전14쪽도 모두 raw100.00%다.
Native·WASM 각각 전14쪽의 PNG가 이전 직접 판독 PNG와 바이트가 같아 그 판독을 재사용한다.
표 조각의 원문·소유 감사와 같은 쪽 review/overlay 재산출도 완료했다.

같은 package를 root pkg와 Studio public 실행 경로에 동기화했다. JS 바이트는 같고 두 타입 선언의
생성 순서가 바뀌어 실제 생성본으로 갱신한다. 바인딩/embed3개·타입 검사·Node2,796개·프로덕션
빌드가 모두 통과했다. 원시 로그·중간 JSON은 output에만 보존한다. 보호 전쪽·새 base의 같은
Linux 조건 전389쪽 대조와 전체 Rust 검증은 아직 실행 중이므로 PR 완료를 선언하지 않는다.

## Native 보호 전쪽과 추가 직접 판독

새 Native18개 데이터셋 전1,771쪽은 상향된 정본8f073d 대비 하락0·추가 상승0이다.
실제 필수 편람 HWPX Native gate383쪽도 완료했으며130쪽의 기존 WMF 차이를 포함한
최저99.96%를 유지한다. 전체 WASM 보호·같은 환경의 새 base 대조·전체 Rust는 별도 진행 중이다.

원 문서4쪽의 Native/WASM review·overlay를 각각 직접 열어 조문 대비표의 글줄·밑줄·
점선·두 열 경계와 쪽표시를 추가 확인했다. 두 경로의 raw는100.00%다.

보호 Native PNG도 이전 실행과 전쪽 대조했다.1,582쪽은 바이트가 같고189쪽은 차이가 있어,
동일한 공식 raw 표시값만으로 RGB 동일성을 선언하지 않았다. 변경 영역 중 편람112·116·117쪽,
shape-group-02 1쪽·sample16 HWPX1쪽·8040 insert4쪽의 같은 쪽 review/overlay를 직접 확인했다.
본문·표·제목·내용 순서와 외곽 소유를 보존하며 작은 가장자리·농도 차이를 구분했다.
편람116쪽 치수 화살표의 PDF 대비 선폭·농도 차이는 이전 Native에도 존재한다. 이전/새 출력의
변경 bbox는 x462~480의 단위 문자 영역에 한정되어 화살표 영역은 이전 출력과 동일하다.
이 보존 검증을 새로운 RGB 완전 일치 또는 기존 그래픽 차이의 해결로 표현하지 않는다.

권고안4쪽은 새 base와 후보의 실제 WASM paint JSON 전체가 바이트로 같고, 공통 반경 생산과
CanvasKit 소비 코드도 같다. 기존 실제 CK 반경 검증을 이 범위에서 재사용한다. 이 Node 검사는
host font 없이 paint 계약을 확인한 것이며 새 CK 캡처·일치율을 산출했다고 주장하지 않는다.

## 새 base 주요 페이지의 직접 판독

[다섯 입력의 실제 Native/fresh WASM 14개 review·overlay 쌍](final/rebase-f823-reading/README.md)을 직접 확인했다. CBTA 118·162·166쪽, 권고안 4쪽, 영어 시험지 2쪽, API 35쪽, 각주 표 37쪽이다. 원 PR #9075의 원 문서 증적과 구분한다. 같은 환경 전쪽 대조와 전체 Rust 완료는 별도 조건으로 유지한다.

## Native/fresh WASM 특별 보호 전쪽 완료

상향된 정본 `8f073dddd67cf8f44bea7c0315c3f9950ba73641e9393747cdbc52d24b33f9a1`을 기준으로 Native18개·fresh WASM18개, 전3,542행을 모두 완료했다. 공식 두 자리 raw 표시값의 하락0행·누락0행·추가 상승0행이다. 앞서 수용한 상승30행의 새 하한을 유지하며 더 낮은 과거 기준으로 돌리지 않는다. 전체 Rust·새 base의 동일 환경 비교 완료와 PR 생성은 아직 별도 조건이다.

## 같은 환경 778행 대조와 추가 보정 경계

다섯 원본을 Native/fresh WASM 전389쪽씩 실제 대조했다. 일반 페이지별 raw 하락은32행,
최대0.55%p(CBTA Native47쪽99.16%→98.61%)다.2%p 초과 하락은0행이고 CBTA118·162·166쪽은
양쪽98% 이상이다. 특별 보호36조건3,542행은 여전히 별도 기준 하락0행이다.

그러나 최대 하락 쪽의 실제 review/overlay와 render tree를 대조하니 굵은 en dash 다음
영문 시작점이 x384.8→394.1 CSSpx로 바뀌었다. `한양신명조 (HFT)`의 일반 U+2013을
1em으로 보충한 부분 표가 굵게 fallback에도 상속된 결과다. 허용폭 이내라는 이유만으로
배치 보존을 선언하지 않는다. 이 경계의 추가 보정과 일반 en dash 보호 대조를 완료한 뒤 제출한다.

- 원본: `samples/issue6764/1613000-202200037-air-traffic-controller-cbta.hwp`,
  Section0 문단501·글자모양36·U+2013 offset23, 기호 슬롯5·한양신명조·altType2·bold.
- 새 WASM 읽기 전용 font decision trace: HFT 부분 표 hit, baseAdvance1400HU·final1472HU,
  `boldFallback`으로 regularMetricAdvance를 상속했다. 실제 Native 두 render tree도 위 원점 차이를 보인다.
- 독립 PDF `pdf/issue7727/cbta-original-hancom2020-print-20261106.pdf`47쪽은
  `/F4`(xref26, HCI Poppy)의 ToUnicode `<0053> <2013>`와 CID83의 `/W=500`을 사용한다.
  PDF parser는 원본의 공백 있는 BaseFont 이름에 경고를 냈으나 이 CMap·폭과 실제 PDF raster는 확인했다.
- 기존 중첩 문서16쪽의 일반 en dash는 Type3 T6/HFT376의1000/1000em이다.
  굵게 대체 경로의500과 구분하며 일반 계약을 반각으로 되돌리지 않는다.
- [base review](final/rebase-f823-reading/en-dash-p047/base-review-047.png) ·
  [overlay](final/rebase-f823-reading/en-dash-p047/base-overlay-047.png),
  [후보 review](final/rebase-f823-reading/en-dash-p047/candidate-review-047.png) ·
  [overlay](final/rebase-f823-reading/en-dash-p047/candidate-overlay-047.png).

기준 측정 기록 프로세스가 종료된 두 결과는 완료 PNG와 묶음 manifest·출력 SHA를 먼저 확인했다.
문서 커밋으로 Git HEAD가 바뀌어 export `--resume`가 provenance를 거부했으므로 그 검사를
완화하지 않았다. 기존 실제 PNG·계산 체크포인트를 공식 `--png-pair --resume`로 재대조해
원래 묶음별 TSV와 동일한309행을 복구했다. 원본 재렌더·PDF 재인쇄·소스 변경은 없고,
기존 raster의 head·실제 base binary/WASM·글꼴·입력·PDF SHA를 그대로 보존했다.

## 기존 회귀와 실제 출력 반례의 구분

새 base 이후 최소 공개 글꼴 격리의 기본 전체 회귀는13,582 PASS·실패0·설정상 skip45로
종료했다. 실행973.590초이며 마지막 대형 표 검사도440.870초에 실제 PASS로 종료했다.
이는 아직 굵은 en dash 반례를 보정하기 전 source833의 결과다. 기존 회귀가 해당 원점 차이를
검출하지 못했다는 근거이며 추가 보정·새 반례 검사를 면제하지 않는다.

하락32쪽의 실제 render tree를 전부 대조했다.31쪽은 트리 전체가 같고 모든32쪽의 TextRun
문자열 순서도 같다.47쪽만 en dash 폭과 그 다음 TextRun 원점이 다르다. 나머지31쪽은 실제
base/Print/후보의 최대 paint 차이 영역을 골라 같은 배율로 확대하고 각 경로의 공식 overlay와
함께8개 모음으로 직접 판독했다. 글줄·내용·문장부호·표 소유를 유지하며 작은 가장자리·
농도 차이를 구분했다. crop은 진단이며 전쪽 TSV나 page gate를 대체하지 않는다.
