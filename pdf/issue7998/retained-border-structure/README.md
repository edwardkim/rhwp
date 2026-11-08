# #7998 임의 굵기 실선의 행·열 상속

`generate.py`는 공개 `../compound-borders/compound-solid.pdf`의 부모·자식 표와 앞뒤 문장을
사용해 600×720pt의 독립 PDF를 재생성한다. `retained-pen.pdf`는 검정 0.50pt(50HU),
`odd-blue.pdf`는 #336699·Arial·0.51pt(51HU)다. `mixed-pen.pdf`는 부모의 세로 변만
0.70pt로 바꿔, 서로 다른 pen을 하나의 스타일로 추정하지 않는 반례다. 저장소 루트에서
`python3 pdf/issue7998/retained-border-structure/generate.py`로 바이트까지 재현할 수 있다.

새 셀은 실제 행·열의 빈 셀이다. 완전한 원본 잉크와 현재 소유를 검증한 뒤 셀 안/바깥
반쪽의 동일 폭 경계 띠를 생성하며, 최근접 표준 BorderFill 굵기로 바꾸지 않는다. 새 셀에
기존 문장이나 자식 표를 복제하지 않는다. 정수 crop 투영의 1HU 양자화 때문에 50HU의
관측 폭은 49HU이고 51HU는 그대로다. 반복 삽입·삭제로 폭이 누적 변하지 않아야 한다.

`print/`의 PDF 14개는 네 방향 삽입·원래 두 번째 행/열 삭제·파란 행 삽입의 HWP/HWPX
저장본을 메인테이너가 한컴 2020에서 직접 인쇄한 독립 기준이다. 첫 HWP의 도형 헤더
결함을 바로잡은 뒤 최종 7개 모두 정상 열림·인쇄를 확인했다. HWPX는 최종 입력 바이트가
이전 인쇄 입력과 같아 재사용했다. 입력/저장본/기준 PDF SHA와 Creator·Producer·실제
용지 크기를 `provenance.json`에 기록했다. 인쇄본은 A4 595×841pt이며 입력과 같은 물리
원점·배율에서 비교한다. 흰 용지만 보충하고 출력을 이동·확대해 맞추지 않는다.

Native/fresh WASM의 원본·양 저장·무변경 거절 원본 각 19쪽은 모두 90% 이상이다.
공유 가로선 농도·검정 Helvetica 대체 글꼴의 확대 비교를 포함해 2026-10-19 메인테이너가
**시각·편집 구조 통과**로 판정했다. 전체 원본 TSV·PNG·소스/빌드 SHA·실제 Studio 검증과
남은 범위는 [stage22](../../../mydocs/working/task_m100_7998_stage22.md)에 연결한다.
일반 점선·반투명·가림·혼합 pen·병합/지역 축의 상속이나 R2 문자 역삭제 원점은 이
대조군의 통과로 완료 처리하지 않는다.

stage23에서는 `mixed-pen.pdf`를 축별 일정 pen(세로 0.70pt·가로 0.50pt)의 양성 입력으로
확장한다. 위 stage22의 거절 기록은 당시 범위다. 새 `nonuniform-axis.pdf`는 왼쪽 세로선만
0.90pt이고 나머지 세로선은 0.70pt여서, 같은 축의 pen을 하나로 추정하지 않는 반례다.
같은 생성기로 재현하며 전쪽 Native/fresh WASM raw는 각각 99.98109% / 99.98086%다.

부모 선의 큰 PNG frame과 중첩 셀의 교차만으로 빈 Picture를 만드는 결함도 stage23에서
보완한다. 독립 PDF에서 실제 잉크가 없는 A/B/C/D 셀의 선택 가능한 투명 그림 8개를
발행하지 않으며 셀 문단·자체 테두리와 실제 내용 그림은 보존한다. 수정 전 FAIL과
수정 후 Native/실제 WASM 클릭·양 저장·전쪽 비교는
[stage23](../../../mydocs/working/task_m100_7998_stage23.md)에 연결한다.
2026-10-19 메인테이너가 수정본의 **시각·편집 구조 통과**와 **새 HWP/HWPX 14개 정상 열림**을
확인했다. 같은 source/runtime의 집중 32건·Native/fresh WASM 각 23쪽 증거를 재사용한다.

### stage24 — 평행 경계별 굵기의 상속

`nonuniform-axis.pdf`의 세로선 0.90/0.70pt는 행 편집으로 길이를 이어갈 수 있다.
`nonuniform-horizontal.pdf`는 위 가로선 0.90pt·다른 가로선 0.50pt·세로선 0.70pt로,
열 편집의 같은 계약을 대조한다. 스타일이 다른 축 자체의 편집은 새 공유 변의 상속 충돌
계약이 필요하므로 원자 거절한다. `nonuniform-interval.pdf`는 왼쪽 세로선의 위/아래
interval이 0.70/0.90pt여서 한 경계선을 하나의 pen으로 추정하지 않는 반례다.

`parallel-provenance.json`과 `print/parallel-*.pdf`는 새 6종 편집의 HWP/HWPX 모두에
대한 형식별 독립 한컴 2020 Print다. 원본과 저장본 SHA·actual Print/1-up 실행·PDF SHA를
연결하며 이전 인쇄본과 구분한다. Native/fresh WASM 새 편집 12쪽 최저는
98.27716% / 97.20590%, 기존 대조군을 포함한 각 29쪽은 모두 90% 이상이다.
생성기의 기존 4개와 신규 2개 PDF를 별도 폴더에서 재현했다. 실제 source·수정 전후·
집중 34건·actual Studio·클릭·동일 backend 비교는
[stage24](../../../mydocs/working/task_m100_7998_stage24.md)를 따른다. 2026-10-19
메인테이너가 이번 절편을 **시각·편집 구조 통과**로 판정했다.

### stage26 — 셀 구간별 굵기의 상속

기존 `nonuniform-interval.pdf`는 왼쪽 외곽의 위/아래 셀 변이 0.70/0.90pt인 양성 입력이다.
새 `horizontal-interval.pdf`는 위 외곽의 왼쪽/오른쪽 셀 변이 0.90/0.50pt이며,
`shared-vertical-interval.pdf`와 `shared-horizontal-interval.pdf`는 공유 변의 한 셀 구간만
0.90pt다. 실제 셀 경계에서 변하는 pen을 선 전체의 한 값으로 축약하지 않는다.
`within-cell-vertical.pdf`·`within-cell-horizontal.pdf`는 단일 셀 변 안에서 pen이 변하므로
한 셀의 테두리 속성으로 확정하지 않고 구조 편집을 원자 거절하는 반례다.
같은 생성기로 기존 6개와 신규 5개 모두 바이트까지 재현했다.

`print/interval-provenance.json`과 `print/interval-*.pdf`는 16종 편집의 양 저장 32개 및
재열기 후 삽입한 선두 행/열 삭제 4개의 독립 한컴 2020 Print를 연결한다. HWP 경유
HWPX 재저장에서 실제 0 글상자 여백의 존재를 잃어 위 문구가 사라지던 결함도 보정했다.
첫 후보의 미수용 증거와 별도 진단은 [stage26](../../../mydocs/working/task_m100_7998_stage26.md)에
보존한다. HWPX의 absent/explicit-zero 구별은 유지한다.

Native/fresh WASM 각 100쪽의 최저는 97.45731% / 96.19767%, 새 편집 36쪽은
98.95186% / 96.69192%다. 기존 59쪽 PNG는 양 backend 각각 stage25 수용본과 동일하다.
2026-10-19 메인테이너가 **시각·편집 구조 통과**로 수용했다. 다른 색/스타일·dash/alpha·병합/지역 축·
미증명 원점과 R2/P5의 일반 상속을 이 결과로 완료 처리하지 않는다.
