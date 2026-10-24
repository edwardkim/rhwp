# 단계 402: 규제분석·KTX·low-slack 전쪽 검증

## 범위와 사전 확인

- Issue: #7445. 브랜치 `fix/7445-completed-20261009`의 첫 세 완료 대상부터 마무리한다.
- 검증 코드 head는 `c0ce4f717a3b69e2d59ae66d0df1e5a5a6d5e604`, 기준 `upstream/devel`은 `21ed64071f81f35f37618717693bd30f727392c9`다. 원격 devel도 같은 SHA임을 조회했다.
- 이전 후보의 완료 기록을 최신 코드의 수용으로 대신하지 않는다. 최신 Native Skia와 Mac no-opt fresh WASM 빌드 산출물을 고정하고 같은 원본·독립 Print PDF·DPI·글꼴 환경으로 각각 전쪽 TSV를 산출했다. 캡처는 최대 8개씩 병렬 수행했다.
- 원본과 기준 PDF 6개는 실제 실행 파일과 HEAD의 Git blob이 바이트 동일하다. PDF 재산출이나 추가 테스트 없이 기존 입력을 사용했다.

## 결과

| 입력 | 독립 PDF / Native / fresh WASM 쪽수 | Native 최저 | fresh WASM 최저 | 90% 미만 |
| --- | --- | ---: | ---: | ---: |
| `samples/issue7445/80250_regulatory_analysis.hwp` | 17 / 17 / 17 | 95.29505% | 95.29505% | 0 |
| `samples/basic/KTX-003.hwp` | 1 / 1 / 1 | 95.86468% | 95.86468% | 0 |
| `samples/issue6535/36339092_low_slack_absorb_block.hwpx` | 1 / 1 / 1 | 95.86412% | 95.86412% | 0 |

19쪽의 Native/fresh PNG는 각각 바이트 동일하다. TSV의 자동 gate는
`not_evaluated`이며, 아래 실제 판독 및 관계 검사와 별개로 기록한다.

- 규제분석 2·10·11·15쪽: 규제 개요 표, 경쟁영향평가 문단 순서, 예비분석 결과표의 제목-표 순서, 마지막 글줄 뒤 다음 문단의 소속을 대조했다.
- KTX 1쪽: 지도·제주 삽입 도형·범례·요금표·시간표의 소속과 표시를 대조했다. 제목 굵기 등 잔존 잉크 차이를 그대로 보존하며 픽셀 동일성을 주장하지 않는다.
- low-slack 1쪽: 본문과 서명·연락처 틀이 같은 쪽에 남는다. 연락처는 소속 하단 표 안에 한 번씩 표시된다.
- 기존 관계형 회귀 4건은 4 PASS / 0 FAIL / 671 skip, 0.040초, exit 0이다. 규제분석의 종료 글자모양·빈 캡션 줄 소유 2건과 low-slack의 쪽 소속·하단 연락처 소유 2건이다. KTX의 전쪽 시각 비교를 이 4건의 실행 범위로 오인하지 않는다.
- 이번 단계에서는 생산 코드·기존 테스트·기대값을 변경하지 않았고 새 테스트를 추가하지 않았다.

## 증적과 남은 범위

입력·빌드 해시와 결과 요약（로컬 중간 자료 `final-first-three-validation.json`）,
[규제분석 10쪽](../pr/assets/issue7445/completed-20261009/final-first-three-regulatory17-p010-review.png),
[11쪽](../pr/assets/issue7445/completed-20261009/final-first-three-regulatory17-p011-review.png),
[15쪽](../pr/assets/issue7445/completed-20261009/final-first-three-regulatory17-p015-review.png),
[KTX 1쪽](../pr/assets/issue7445/completed-20261009/final-first-three-ktx003-p001-review.png),
[low-slack 1쪽](../pr/assets/issue7445/completed-20261009/final-first-three-low-slack6535-p001-review.png).

전체 TSV·실행 로그·중간 JSON은 ignored
`output/pr-review/7445-completed-20261009/stage402-first-three-final/`에 보존한다.
`.log`는 커밋에 포함하지 않는다.

27건·504쪽 중 최신 후보에서 마무리한 범위는 **3건·19쪽**이다. 나머지
24건·485쪽의 전쪽 재검증, 잔존 전체 회귀 실패 판정, 최종 전체 검증은 남아 있다.
이 단계의 완료는 전체 PR 준비 완료를 뜻하지 않는다.

## 다음 실패 처리 기준

- 실패한 원 회귀가 고정한 `upstream/devel`에서도 90% 미만인 문서에 등록된 부적합한 렌더링 검사인지 같은 입력·독립 PDF로 먼저 판정한다.
- 사용자 지시에 따라 해당 검사·증거를 #7445로 이관한다. 같은 원본·같은 미해결 증상의 이관 기록이 이미 있으면 중복 등록하지 않고 남은 활성 검사·수집기 참조를 제거한다. 원본과 PDF는 재현 자산으로 보존한다.
- 현재 수정으로 새로 발생한 회귀는 구현을 고친다. 이미 보정·전쪽 90% 이상·확대 판독 후 복원한 정상 검사는 과거 이관 기록만으로 다시 제거하지 않는다.
- 90% 이상인 정상 문서의 px 고정 기대값이 부적합하면 독립 출력과 원본 소유 근거로 쪽·문단·개체의 소속·순서·포함·누락·중복 관계 검사로 변경한다.

첫 세 대상의 전쪽 수치 검증·대표 영역 판독·기존 관계형 회귀 검증을 완료했다.
