# PR #7671 리뷰 — 컨트롤 뒤 자동 번호의 저장·표시·소속

## 최종 판정

**메인터너 보정 후 수용 가능 — 검토 범위의 로컬 검증 완료.** 원 contributor head와 메인터너 보정을 포함한 통합 후보를 구분한다. 통합 code 후보는 `20d93fe1d4226acab640b66875964d09c7d6d59b`이며 원 PR을 그대로 merge하는 판정이 아니다.

통합 PR 생성·최신 정확한 head CI·mergeability 확인·merge·후속 처리는 작업지시자가 자동 승인했다. 원격 CI와 merge 완료는 별도 확인하며 로컬 통과만으로 완료 처리하지 않는다. 통합 PR에서 이미 archive review와 오늘할일을 함께 반영하고 별도 기록 PR을 만들지 않는다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR / author / base | [#7671](https://github.com/edwardkim/rhwp/pull/7671) / semanticist21 / devel |
| 원 head | `67764a121708efc6f71ed58f28993bae5718df44` |
| 통합 branch / code 후보 | `review/semanticist21-20261015` / `20d93fe1d4226acab640b66875964d09c7d6d59b` |
| 기준 upstream/devel | `2efefc6a1f9812a188e32c1cb283e9ef7962ea3e`; fetch 결과 동일하여 리베이스 생략 |
| 원 기여 체리픽 | `166641a6a,548b2c827`; 원 author와 -x 출처 유지 |
| reviewer / 접수 상태 | jangster77 assign 확인 / OPEN · non-draft |
| 이슈 종료 | 원 PR 본문에 closes/fixes/resolve 지시 없음. 관련 선행 번호는 참조로만 보존 |

## 변경과 메인터너 검토

구역 첫 문단 및 각주·미주·TAC 그림 뒤 번호를 원래 모델 자리표에 저장한다. 자동 번호 컨트롤/UTF-16 원문을 유지하면서 공개 HWP 저장 adapter 및 HWPX의 실제 표시를 검증했다.

원 serializer 보정만으로는 본문 PAGE/TotalPage paint가 비었다. 실제 페이지 소속에서 확정한 비직렬화 값을 composer·줄 수용·최종 분할 paint가 공유하게 보정했다. 한 em 수용과 자연 숫자 전진, 양쪽 정렬 탄성 간격은 독립 Print 반례로 정했다. 그림 뒤 char_start를 중복 증가시키는 결함도 보정했다.

원 diff의 파일 수는 2개다. 체리픽13건과 다른 원 PR을 함께 적용한 뒤 통합 상태를 검토했다. 원 head CI를 통합 code 후보의 CI로 재사용하지 않는다.

## 검증·입력·조판 원칙

[전체 통합 검증 결과](../assets/semanticist21_20261015/validation.md)에 실행 code SHA·명령·결과를 연결한다. 실제 실행한 HWP/HWPX/PDF31쌍을 code 후보의 Git blob SHA와 대조했고, 신규 font binary·raw JSON/TSV/log를 커밋하지 않았다.

[전쪽 시각 증거와 입력 SHA](../assets/semanticist21_20261015/README.md)의 해당 #7671 입력은 **15문서**이며 전쪽 Native/fresh WASM100%·독립 한컴 Print 쪽수 일치·직접 확대 판독을 확인했다. 문서마다 중요한 위치의±2px 검사와 소유·내용 순서·포함·겹침 관계를 함께 유지한다. 양쪽100% 페이지를 우선 선정했고, 미래에도 고정 쪽은 양쪽98% 이상이어야 한다.

- 조판 원칙: 적용. 원본/Print의 독립 기대값, 생산→측정→실제 배치 소비 경로와 원인 반례는 아래 구현 기록 및 시각 증거에 연결한다.
- 내용 보존: 쪽/문단/모델 문자 소속, 앞뒤 텍스트·필드·개체 순서, 각주/미주, 누락·중복을 실제 출력과 관계 검사로 확인한다.
- 저장 정보: 수동 XML 반례와 한컴 정상 재저장본을 구분한다. 무효가 된 생성/편집 줄 캐시를 원본 정상 저장 캐시와 같은 근거로 쓰지 않는다.
- base 대비: 같은 입력·PDF·폰트 조건의 페이지별 Native/WASM 값에0.01%p 이상 하락0. 평균 또는 다른 쪽 상승으로 상쇄하지 않았다.
- 확대 판독: 각 Native/WASM15개 시트 전체와 변경 위치를 실제 열어 대조했다. 번호·탭·그림·단·미세 간격에 이 PR 주장 범위의 미해결 결함은 관찰하지 않았다.
- 분할/이어받기: 본문 필드는 부분 문단3쪽과 다음 쪽 번호를 검사한다. 표 rowspan/continuation/clip 구현 변경은 비해당이다.

[메인터너 구현 기록](pr_7671_review_impl.md)을 연결한다.

## 원격 통합과 후속 처리

1. PR 직전 upstream/devel `2efefc6a1`이 그대로임을 확인해 리베이스를 생략했다.
2. 원본 저장소의 임시 head branch로 통합 PR #7935를 생성했고 archive review·오늘할일·PNG 및 본문의 실제 이미지·전쪽 TSV124개를 함께 반영했다.
3. 정확한 최신 head required checks 및 mergeability 통과 뒤 merge한다.
4. merge SHA에서 asset 존재를 확인하고 원 PR에 한국어 존댓말로 credit·메인터너 보정·실제 검증·merge SHA 고정 PNG를 게시, API로 본문을 재조회한다. 원 contributor PR은 통합 PR을 명시해 supersede close한다.
5. post_merge.md 순서대로 devel 동기화와 이번 작업의 branch/출력/임시 자료 정리를 수행한다. contributor fork branch와 공유 target은 보존한다.

## 통합 code candidate CI

통합 [PR #7935](https://github.com/edwardkim/rhwp/pull/7935)의 code candidate head `2ead4954547804215c9c4f4e2c145700467ddbb5`에서 [CI](https://github.com/edwardkim/rhwp/actions/runs/38799924267) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/38799924232) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/38799924058) · [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/38799924281) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/38799924242) 및 정책 status 성공을 확인했다. 실패·대기 없음. 검증한 제품 code는 `20d93fe1d4226acab640b66875964d09c7d6d59b`이며 그 뒤 제품/테스트/Studio 변경은 없다. 이 CI 기록만 추가하는 문서 후행 head의 required checks와 mergeability는 별도로 확인한 뒤 merge한다.
