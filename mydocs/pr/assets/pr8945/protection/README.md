# 정본 보호 전쪽 대조

기준/후보 각 Native 1,771쪽·WASM 1,771쪽. 비교 3,542쌍의 PNG가 모두 바이트 동일하다.
과거 정본 공식 두 자리 값의 미재현은 Native 862쪽·WASM 49쪽, 합계 911행이다.
현재 base 대비 하락은 0쪽, 90% 미만과 누락은 없다. 보호 TSV 자체는 변경하지 않았다.

| 문서 | backend | 쪽 | 후보 최저 raw | 과거 보호값 미달 쪽 | base/후보 PNG |
| --- | --- | ---: | ---: | ---: | --- |
| yeongwol | native | 4 | 99.93% | 1 | 전쪽 동일 |
| yeongwol | wasm | 4 | 99.99% | 0 | 전쪽 동일 |
| seo-body-list-typed | native | 1 | 100.00% | 0 | 전쪽 동일 |
| seo-body-list-typed | wasm | 1 | 100.00% | 0 | 전쪽 동일 |
| handbook-hwp | native | 383 | 99.96% | 7 | 전쪽 동일 |
| handbook-hwp | wasm | 383 | 99.95% | 4 | 전쪽 동일 |
| seo-helvetica-styles | native | 1 | 99.63% | 0 | 전쪽 동일 |
| seo-helvetica-styles | wasm | 1 | 100.00% | 0 | 전쪽 동일 |
| sample16-hwpx | native | 64 | 99.65% | 12 | 전쪽 동일 |
| sample16-hwpx | wasm | 64 | 99.86% | 2 | 전쪽 동일 |
| sample16-hwp2022 | native | 64 | 99.54% | 56 | 전쪽 동일 |
| sample16-hwp2022 | wasm | 64 | 99.48% | 37 | 전쪽 동일 |
| seo-re-03-latin | native | 1 | 100.00% | 0 | 전쪽 동일 |
| seo-re-03-latin | wasm | 1 | 100.00% | 0 | 전쪽 동일 |
| angle-narrow | native | 1 | 100.00% | 0 | 전쪽 동일 |
| angle-narrow | wasm | 1 | 100.00% | 0 | 전쪽 동일 |
| outline | native | 9 | 99.97% | 4 | 전쪽 동일 |
| outline | wasm | 9 | 99.99% | 0 | 전쪽 동일 |
| issue8040-insert | native | 4 | 98.69% | 4 | 전쪽 동일 |
| issue8040-insert | wasm | 4 | 99.99% | 0 | 전쪽 동일 |
| nested | native | 17 | 97.29% | 5 | 전쪽 동일 |
| nested | wasm | 17 | 98.06% | 0 | 전쪽 동일 |
| sample16-hwp2024 | native | 64 | 99.65% | 12 | 전쪽 동일 |
| sample16-hwp2024 | wasm | 64 | 99.86% | 2 | 전쪽 동일 |
| handbook-hwpx | native | 383 | 99.96% | 0 | 전쪽 동일 |
| handbook-hwpx | wasm | 383 | 99.95% | 4 | 전쪽 동일 |
| issue7905 | native | 6 | 98.82% | 2 | 전쪽 동일 |
| issue7905 | wasm | 6 | 98.86% | 0 | 전쪽 동일 |
| sample10 | native | 763 | 98.20% | 755 | 전쪽 동일 |
| sample10 | wasm | 763 | 99.56% | 0 | 전쪽 동일 |
| issue8040-insert-delete | native | 4 | 98.69% | 4 | 전쪽 동일 |
| issue8040-insert-delete | wasm | 4 | 99.99% | 0 | 전쪽 동일 |
| seo-pic-crop-01 | native | 1 | 100.00% | 0 | 전쪽 동일 |
| seo-pic-crop-01 | wasm | 1 | 94.65% | 0 | 전쪽 동일 |
| seo-shape-group-02 | native | 1 | 100.00% | 0 | 전쪽 동일 |
| seo-shape-group-02 | wasm | 1 | 100.00% | 0 | 전쪽 동일 |

전체 수치·쪽·원 PNG SHA-256은 [comparison.tsv](comparison.tsv)에 있다.
현재 원본/PDF/producer/helper/글꼴·PNG·부분 산출물 해시와 TSV 일치를 검사한 72 dataset·7,084쪽의 결과는
[saved-results-verification.json](saved-results-verification.json)에 있다.
여러 단독 명령이 같은 출력 루트를 사용해 완료 문서의 target-plan 재개가 거부된 기록은
실행 성공으로 바꾸지 않았다. 이미 완료한 문서는 per-target manifest와 모든 산출물 해시를 독립 검증했다.
글꼴 cache의 변환 중 수집된 0바이트 임시 파일 1개는 유효한 글꼴 프로그램이 아니므로 별도 명시했다.
실제 글꼴 프로그램과 입력·출력 해시는 모두 확인했다. 미완료 구간은 정본 재개 기능으로 완료했다.

sample10은 연결 그림 누락을 확인한 뒤 세 그림을 공급하고 두 producer 모두 새 출력 경로에서 전 763쪽을 다시 실행했다.
누락된 실행의 첫 쪽을 수용하거나 비교 영역에서 가리지 않았다. [직접 비교](review/sample10-linked-first-page-wasm.png).
글꼴의 선택은 [outline](font-contexts/outline-reference-fonts.json)과 [법령](font-contexts/issue7905-reference-fonts.json)에 보존했다.
그 외 글꼴 설정은 각 run manifest와 원 저장소의 기존 설정 파일을 따른다.

반복되는 `font_supply.files` 메타데이터는 `font-inventories/`의 동일 해시 목록을 참조하도록
중복 제거했다. 펼친 객체가 원본과 완전히 같음을 검증했으며, 원본/보존 파일 SHA-256과
목록 참조는 [provenance-normalization.json](provenance-normalization.json)에 있다.
실행 TSV와 검증 당시 원본 manifest의 SHA-256은 변경하지 않았다.

기존 차이의 대표 직접 판독: [nested 16쪽](review/nested-native-p016.png),
[sample10 2쪽](review/sample10-native-p002.png), [sample16 51쪽](review/sample16-hwp2022-native-p051.png).
본문 글꼴·글자 간격과 제목 차례 괘선 등의 기존 차이가 남는다. 그 차이를 이번 PR이 해결했다고 주장하지 않는다.
해당 출력의 base/후보 PNG는 모두 같다. 이 사실은 과거 보호값의 통과를 대신하지 않는다.
