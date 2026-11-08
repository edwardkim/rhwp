# sample16 원본 수식 소유와 읽기 순서의 독립 Print 증거

Issue: #7642. 기존 #6527 검사의 부정확한 wrapper 전제를 교정한다.

## 실제 입력과 독립 Print

| 입력 | 전체 쪽수 | Native 전쪽 raw 최저 | fresh WASM 전쪽 raw 최저 | 6쪽 Native / WASM raw |
| --- | ---: | ---: | ---: | ---: |
| sample16-hwp3 | 64 | 94.88041% | 94.94043% | 98.31869% / 98.32426% |
| sample16-hwpx | 64 | 95.03221% | 95.06438% | 98.27434% / 98.27433% |

HWP3 원본은 `samples/hwp3-sample16.hwp`이며, 같은 디렉터리의 README와
print-provenance.json에 원본 변경 없는 MCP engine2020 Print 출처가 있다.
HWPX는 `samples/hwp3-sample16-hwp5.hwpx`, 대응 독립 Print는
`pdf/issue7642-sample16-print-refresh-20261012/hwp3-sample16-hwp5-hwpx-2024.pdf`다.
모두 기존 commit에 포함되어 있으며 새로운 중복 사본을 만들지 않는다.

검증 code는 cc4255a58d8112c07013e89eda1b1a065c824fea다. 인쇄 프로필/Cairo
96dpi·시스템 폰트와 assets/fonts를 사용했다. 원본은 compat2022, 한컴 HWPX는
compat2024다. Native stage2256 binary와 fresh WASM stage2258 package를
해시로 고정했고 stage2262의 두 backend에서 각각 전쪽64쪽을 비교했다.

ignored output 접두어:
`output/pr-review/issue7642/failed-inputs-20261008/stage15-current-validation/`

- 전체 TSV: `stage2262-cairo-head-{native,wasm}/sample16-{hwp3,hwpx}/silhouette.tsv`.
- 원본/기준 PDF SHA·빌드·font·metric provenance: 같은 각 queue의 receipt.json.
- 최신6쪽 전체/확대 compare·review·standalone overlay: stage2315-sample16-current-review.
- 원본 제목·수식 제어/저장 줄 덤프: stage2317-sample16-source-ownership.

## 직접 판독과 남은 차이의 의미

6쪽에서 산식 제목·앞 설명·두 산식·후속 주의문을 같은 배율로 대조했다.
첫 식의 분자/분모, 둘째 식의 덧셈·대괄호·분자/분모와 내용 순서를 확인했다.
6쪽 전체의 score와 산식 영역만의 값을 구별한다. 산식 확대 영역 raw는
HWP3 WASM93.54%, HWPX Native93.55%이며 엄격 내용 픽셀 보조값은 각각
19.26%/16.78%다. 98%대 페이지 점수를 RGB 동일이나 모든 미세 획의 동일성으로
보고하지 않는다. 원본/변환본의 같은 의미를 지키며 잔여 래스터·미세 간격은
전체 수식 검토에서 계속 대조한다. 정확한 전쪽 점수는 TSV 원값을 따른다.

## 검사 교정의 독립 의미

HWP3의 Equation은 원본 수식 상자의 ref_pos=0을 공통 IR의 TAC로 옮기며,
수식 script만 별도 Equation으로 해석한다. 원본 두 산식과 한컴 변환 HWPX의
문단155/158이 모두 글줄 흐름에 참여한다. 기존 검사는 HWP3라면 TextLine
조상이 없어야 한다고 고정해 실패했으며, 이는 standalone 표시와 내부 구조를
혼동한 전제다. 같은6쪽 독립 Print는 두 형식의 수식과 문단 흐름을 보존한다.

새 이름 `verified_sample16_equation_reading_order.rs`는 원본 제어 소속과 TAC
속성, 같은 문단 줄의 세로 점유·본문 내 포함, 두 산식의 앞뒤 순서, script의
내용·길이·추출1회, 뒤 주의문을 함께 보호한다. 실제 소속을 삭제하거나 모든
수식을 무조건 임의의 문단으로 옮겨도 통과하는 검사로 바꾸지 않았다.
내부 노드 클래스 부재가 정상성의 근거라는 전제만 제거했다.

현재 HWP3→RHWP HWPX 왕복의 새 출력은 기존 Print 입력과 header.xml의
DOUBLE_SLIM 폭 표기가 달라 새 Print/전쪽 검증을 진행한다. 위 원본/한컴
변환본의 통과를 왕복본 시각 검증으로 대신하지 않는다.

## 정식 검사와 lint 실제 결과

source `2d83ac0a8973f0d82be45c545dde1112532baafb`에서 새 검사2개가 모두 통과했다.
release-test build39.05초, 실제 검사0.345초, exit0이다.
같은 source의 필수 lint8단계도 모두 exit0이며 source 해시 불변을 확인했다.
고정 base는 `b7a5388034ee66f58a7e1d5363974e83b6374f57`이고 build jobs8, target/pr-review를
공유해 순차 실행했다. 로그/receipt는 ignored output의 stage2327과 stage2330이다.
이는 RHWP 왕복 HWPX의 새 미달이나 나머지 전체 회귀를 면제하지 않는다.
