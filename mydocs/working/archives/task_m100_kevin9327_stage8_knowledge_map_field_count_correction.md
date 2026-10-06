# Stage 8 - 지식 지도 전수 사전 수치 보정

## 발견

전체 integration 회귀의 마지막 결과에서 `knowledge_map_field_dictionary_contract::dictionary_heading_count_matches_rows`만 실패했다. 머리말·꼬리말 생성 커밋을 병합하면서 기존 누적 수치와 새 수치를 충돌 해결로 합쳤지만, §2-2 표의 실제 유니크 필드 수와 일치하지 않았다.

## 실측

release-test 바이너리의 `rhwp capabilities`를 기준으로 `recordFields` 유니크 수는 298개다. §2-2 사전에는 선언 밖 실측 필드 `applyTo`, `assertions`, `docId`, `isHeader`, `preview` 다섯 개가 더 있어 유니크 수가 303개다.

## 보정

§2-2 헤딩과 설명을 303개/298개+5개로 맞췄다. 이 수치는 사전 행을 파싱하는 계약 테스트와 동일한 산정 방식이다.

## 검증 계획

실패했던 문서 계약을 먼저 확인하고, 이어 전체 integration 회귀를 다시 수행한다. generated suite와 manifest는 검증 산출물로만 남기고 커밋하지 않는다.
