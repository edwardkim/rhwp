# PR #7900 리뷰 — MCP 세션 stderr 교착 방지

## 최종 판정

**승인 — 검토 범위의 로컬 검증 완료.** 원 contributor head와 메인터너 보정을 포함한 통합 후보를 구분한다. 통합 code 후보는 `20d93fe1d4226acab640b66875964d09c7d6d59b`이며 원 PR을 그대로 merge하는 판정이 아니다.

통합 PR 생성·최신 정확한 head CI·mergeability 확인·merge·후속 처리는 작업지시자가 자동 승인했다. 원격 CI와 merge 완료는 별도 확인하며 로컬 통과만으로 완료 처리하지 않는다. 통합 PR에서 이미 archive review와 오늘할일을 함께 반영하고 별도 기록 PR을 만들지 않는다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| 원 PR / author / base | [#7900](https://github.com/edwardkim/rhwp/pull/7900) / semanticist21 / devel |
| 원 head | `7ed9c3d1a2cdfbc839c04d98447c193e7783e717` |
| 통합 branch / code 후보 | `review/semanticist21-20261015` / `20d93fe1d4226acab640b66875964d09c7d6d59b` |
| 기준 upstream/devel | `2efefc6a1f9812a188e32c1cb283e9ef7962ea3e`; fetch 결과 동일하여 리베이스 생략 |
| 원 기여 체리픽 | `30066d906`; 원 author와 -x 출처 유지 |
| reviewer / 접수 상태 | jangster77 assign 확인 / OPEN · non-draft |
| 이슈 종료 | 원 PR 본문에 closes/fixes/resolve 지시 없음. 관련 선행 번호는 참조로만 보존 |

## 변경과 메인터너 검토

MCP 자식 stderr를 읽지 않는 pipe로 열지 않고 실행기로 전달한다. stdout JSON-RPC·검색·치환·저장·페이지 주소6개 검사 및 자식 정리는 유지한다.

추가 생산 코드 보정 없음. 통합 전체 nextest에서 실제 MCP 세션 계약을 실행한다.

원 diff의 파일 수는 1개다. 체리픽13건과 다른 원 PR을 함께 적용한 뒤 통합 상태를 검토했다. 원 head CI를 통합 code 후보의 CI로 재사용하지 않는다.

## 검증·입력·조판 원칙

[전체 통합 검증 결과](../assets/semanticist21_20261015/validation.md)에 실행 code SHA·명령·결과를 연결한다. 실제 실행한 HWP/HWPX/PDF31쌍을 code 후보의 Git blob SHA와 대조했고, 신규 font binary·raw JSON/TSV/log를 커밋하지 않았다.

조판 원칙은 원 변경에 비해당이다. MCP/provenance 테스트 helper 또는 Studio clipboard 상태/삭제 순서를 바꾸며, 줄 메트릭·pagination·paint 좌표의 생산자를 바꾸지 않는다. 이 PR에 별도 렌더링 개선 점수나 fixture를 배정하지 않는다. 통합 branch의 렌더링 변경은 해당 원 PR review가 별도로 책임진다.

## 원격 통합과 후속 처리

1. PR 직전 upstream/devel `2efefc6a1`이 그대로임을 확인해 리베이스를 생략했다.
2. 원본 저장소의 임시 head branch로 통합 PR #7935를 생성했고 archive review·오늘할일·PNG 및 본문의 실제 이미지·전쪽 TSV124개를 함께 반영했다.
3. 정확한 최신 head required checks 및 mergeability 통과 뒤 merge한다.
4. merge SHA에서 asset 존재를 확인하고 원 PR에 한국어 존댓말로 credit·메인터너 보정·실제 검증·merge SHA 고정 PNG를 게시, API로 본문을 재조회한다. 원 contributor PR은 통합 PR을 명시해 supersede close한다.
5. post_merge.md 순서대로 devel 동기화와 이번 작업의 branch/출력/임시 자료 정리를 수행한다. contributor fork branch와 공유 target은 보존한다.

## 통합 code candidate CI

통합 [PR #7935](https://github.com/edwardkim/rhwp/pull/7935)의 code candidate head `2ead4954547804215c9c4f4e2c145700467ddbb5`에서 [CI](https://github.com/edwardkim/rhwp/actions/runs/38799924267) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/38799924232) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/38799924058) · [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/38799924281) · [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/38799924242) 및 정책 status 성공을 확인했다. 실패·대기 없음. 검증한 제품 code는 `20d93fe1d4226acab640b66875964d09c7d6d59b`이며 그 뒤 제품/테스트/Studio 변경은 없다. 이 CI 기록만 추가하는 문서 후행 head의 required checks와 mergeability는 별도로 확인한 뒤 merge한다.
