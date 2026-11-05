# PR #8622 반영 후보의 확대 판독

2026-11-05 KST. 실제 출력 source `d53115fabc92bd392044d9b906c599baddb4368d`, 비교 base `6e1de406209ba9a58032f6ca6de45b18aa687d1c`. [전 124쪽 원본 TSV](summary.json)와 [각 이미지의 입력·쪽·crop·SHA-256](image-index.tsv)을 먼저 확인했다. 새 base와 통합 후보의 Native·최적화 fresh WASM을 새로 출력했다. 이전 source의 그림을 최신 재출력으로 표시하지 않았다.

| 직접 대조한 그림 | 확인 결과 |
| --- | --- |
| [테두리 전체](../../images/post8622/borders-review.png)·[원형 점선 확대](../../images/post8622/borders-circle-detail.png) | 원형 점선과 단일·이중 물결선이 복원됐다. 글자·문단 테두리의 다른 공통 소비 경로와 기존 3D 비표시 규칙을 보존했다. 원형 점의 96 DPI 가장자리 차이는 별도 2020/2024 16굵기 물리 측정과 구분한다. |
| [문자 외곽선](../../images/post8622/outline-review.png) | 여섯 종류의 투명한 내부·획·반복 형태를 확인했다. 같은 글꼴의 600 DPI 검증과 물리 pen 계약은 [이전 고해상도 판독](../pre8622/direct-reading.md)을 연결한다. renderer·글꼴 관련 source와 실제 96 DPI 결과의 보존을 별도로 확인했다. |
| [원 각주](../../images/post8622/footnote-review.png)·[저장본](../../images/post8622/footnote-saved-review.png)·[설정 변경](../../images/post8622/footnote-configured-review.png) | 로마자 II와 참조의 분리, 참조 위첨자, 실제 본문 아래 각주, 저장한 DASH와 붉은 하단 구분선을 확인했다. 본문 내용과 각주 번호의 순서·줄 소속을 보존했다. |
| [바탕쪽](../../images/post8622/master-review.png) | 본문 프레임 원점으로 복원됐다. 독립 Print도 첫 본문 위에 바탕쪽 텍스트를 겹치므로 그 편집 의도를 유지한다. 머리말·참조·본문의 대응을 확인했다. |
| [셀 블록 높이 적용](../../images/post8622/equal-height-edited-review.png) | 동일 입력의 적용 전과 적용 후를 구분했다. 적용 후 독립 Print와 셀 안의 내용·새 격자·표 하단·후속 설명문을 대조했다. |
| [계산식](../../images/post8622/formula-review.png) | 저장 결과의 숫자·소수 자리·쉼표·셀 안 포함·뒤 본문을 대조했다. 실제 평가와 37개 기대값·필드 생성/삭제/순환 거부는 별도 API 검사다. |
| [개요 ID](../../images/post8622/outline-id-review.png) | 가) 머리와 본문 사이 간격을 확인했다. 잘못된 ID·타입의 원자적 거부는 실제 API 검사로 구분한다. |
| [문단 머리](../../images/post8622/paragraph-head-review.png) | 글머리표·번호·개요·그림 머리, 이어지는 줄, 일반/해제 문단의 순서와 내용을 확인했다. 첫 줄의 저장 bit는 별도 binary/serializer 검사다. |
| [첫 글자 장식](../../images/post8622/drop-cap-review.png) | 기존 大와 뒤 본문 표시를 보존했다. 속성 저장 보존 범위를 새 장식 생성·자동 재배치 UI로 확대하지 않는다. |

## 새 base의 406쪽 보존

[80개 입력/Print/compat·전쪽 TSV](../inherited8622/summary.json)를 대조했다. Native·WASM 각각 403/406쪽 PNG가 base와 byte 단위로 같으며, 달라진 3쪽도 점수 하락이 없다.

- [#8508의 4쪽](../../images/post8622/inherited-8508-p4.png): 표 경계의 원형 점선 소비가 달라졌다. 셀 내용·순서·색·표 하단·뒤 본문 위치는 유지된다. 독립 Print의 실선과 다른 기존 테두리 표현을 이번 변경으로 해결했다고 설명하지 않는다.
- [#8512의 4쪽](../../images/post8622/inherited-8512-p4.png): 안내 상자의 점선 모양이 공통 경로를 소비한다. 본문·상자 크기·다음 제목은 유지된다. 독립 Print의 실선은 base에도 있던 차이다.
- [#8512의 5쪽](../../images/post8622/inherited-8512-p5.png): 체크 사각형 가장자리 차이를 확대 대조했다. 텍스트와 뒤 표의 흐름은 유지된다.

[이미지의 실제 crop과 지표](../inherited8622/direct-image-index.json)를 연결한다. 90% 통과와 직접 판독을 구분해 완료했으며, 위 기존 차이에 대한 새로운 해결이나 100% 일치를 주장하지 않는다.

## 마지막 계산 API 보정의 source 구분

이후 `584c9503ba`는 일반 계산 API에서 NaN/무한대를 반환·셀 기록 전에 거부한다. 저장 FORMULA에 이미 있던 동일 유한수 검사를 공통화했다. [변경 경로와 동일 subtree](finite-fix-scope.json)로 renderer·paint·parser·model·serializer·Studio·입력/PDF 불변을 확인했다. 위 530쪽과 편람 383쪽의 producer는 d531로 보존한다. 최종 source의 새 Native/WASM에서 계산식 7상태·7쪽을 추가 실행했다. 양쪽 모두 최저 100.00%이며 기존 producer와 PNG가 byte 단위로 같다. [전쪽 TSV·실제 API·대화창 결과](../finite-final/summary.json)를 구분해 보존한다. 실제 일반 API의 잘못된 정의역 5개×write 2모드도 저장 bytes 불변을 확인했다. 대화창의 LOG(100)=2·LN(EXP(1))=1·undo/redo·LN(-1) 정상 안내와 취소를 확인했다.
