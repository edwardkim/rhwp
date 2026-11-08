# 초기 체리픽 원본 48개 현재 출력 검증

- production: `ddf17f86476293b114b6236bfa6cf1ebc9d9d474`; base: `552f5228d2dbfd6046c838f14241704f1bcb8a05`.
- 원본48개·전52쪽 Native/fresh WASM 최저99.62848%, 90% 미만·누락0, 각 문서 독립 한컴 Print PDF 쪽수 일치.
- 전체52쪽의 현재 Native/fresh RGBA와 이전 v396 Native RGBA가 동일하다. 현재24개 편집 입력과는 원본 SHA가 다르므로 별도로 확인했다.
- 현재 대표6입력의 review12개를 직접 읽었다. 머리말 서식·연속 표·원본 바깥 여백·빈 미주·6개 배경 무늬의 소유/경계/앞뒤 내용과 각주 번호를 확인했다.
- 같은 좌표 4배 확대3개로 머리말 이름/쪽번호/밑줄, 본문 위첨자 번호, 각주 구분선/번호를 대조했다. Native/fresh는 같으며 글자 획·래스터의 미세 차이는 남는다.
- 각 review/overlay의 파일명에 원본 SHA 앞12자와 backend를 보존했다. 실행 명령·입력/PDF SHA·전체 TSV는 ignored `output/pr-review/semanticist21-20261008/nested-frame-alignment-provenance-v453/initial-48-current-plan.json` 및 양쪽 결과/qualification에 보존한다.
- 표 바깥 여백 편집 입력2개는 [원본 입력 기록](../initial-outer-margin-inputs-v453/README.md)에 바이트 동일하게 보존했다.
- 전체 Rust10,267 통과와 별도로 남은 Skia/doc·가맹사업 fresh 전쪽·정식 관계 회귀 게이트 완료 후 승인한다.

## 원PR별 후속 댓글의 현재 대표 증적

원PR #7613/#7616/#7658/#7614/#7654에 사용할 현재 원본 review/overlay를 추가로 생성해 양쪽10개를 직접 열었다. 기존 빈 미주 대표와 중복되는2개를 포함하며 현재 저장된 대표 review는 총20개다.

- #7613 `original-03`: 앞뒤 연속 표의 셀·행과 분리·비겹침.
- #7616 `original-05`: 문단 자리표, 표의 줄 소속과 뒤 본문 순서.
- #7658 `original-08`: 탭과 문단 스타일의 글자 간격.
- #7614 `original-30`: 논리 캐럿 뒤 `XY`와 주석 마커/번호 보존.
- #7654 `original-45`: 두 미주 마커·빈 문단·구분선의 소속.

각 대표 양쪽2px 실루엣은100%이며 전체48개/52쪽의 최저값99.62848%를 바꾸지 않는다. 글자 획·래스터의 미세 차이와 실제 줄·개체 소속 판독을 구분한다. 아래 그림은 해당 원PR의 후속 댓글에서 merge SHA로 고정한다. 원문 후속댓글은 통합 PR 병합 전에는 게시하지 않는다.
