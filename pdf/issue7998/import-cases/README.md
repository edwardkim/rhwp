# #7998 추가 PDF 가져오기 검증 입력

사용자가 제공한 추가 입력이며, 표·원문 소유권/마스크/반복 XObject/collection/회전 및 글상자 편집의 재현에 사용했다. 입력 PDF 자체를 독립 렌더러로 해석한 출력과 대조하며 모든 출력 차이가 해결됐다는 뜻은 아니다. 원문·독립 기대값과 검증·수용 범위는 [stage37](../../../mydocs/working/task_m100_7998_stage37.md)과 [PR 통합 검증](../../../mydocs/working/task_m100_7998_stage38.md)에 연결한다.

| 파일 | 바이트 수 | SHA-256 |
| --- | ---: | --- |
| [tb_ownership_inf.pdf](tb_ownership_inf.pdf) | 1572698 | `378db14c4196dfea4e279ecf0adaf96d706256d40c4ac7fc72f26b910c2827f1` |
| [bounded_import_path-02.pdf](bounded_import_path-02.pdf) | 228901 | `0ba1f7d5b72b8bd92e19b1198017dc0efb4bbd68ec0bdde14dfb94ef3e0523b5` |
| [xo_visits.pdf](xo_visits.pdf) | 1307972 | `2c7cf86272ff1a8535e4a8c5db0a20b685268fa43199bef627593c25d49c7f97` |
| [extraction-002.pdf](extraction-002.pdf) | 102959 | `6e63f7295fecf18a013ad221ea43ec9c524f1ed249e37128b211851312a0f0f7` |
| [coll_entries.pdf](coll_entries.pdf) | 988629 | `917142de2704ae037b0132f5a344ce5331b1fd96cc19dd574666aaacfbf0268c` |
| [text_not_supported.pdf](text_not_supported.pdf) | 15285 | `edb9988a597d8fdc4323131bfd6b2cfeffc6f768ffc9e5e98b93b99ef8d250f8` |

162쪽 `reousrce_limit.pdf`는 기존 [resource-limit.pdf](../../../samples/issue7957/resource-limit.pdf)와 바이트가 같아 재사용한다.
개인이력서인 `font_metric.pdf` 원문과 식별 가능한 출력은 로컬 검증 자료로 보존한다. 공개 회귀에는 개인정보가 없는 합성 font-resource 입력을 사용하며 원본의 실물 확인과 구분한다. 비공개 견적서의 원본·이미지도 기존 지시대로 로컬에 보존한다.
