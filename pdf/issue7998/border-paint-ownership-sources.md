# #7998 경계 조각 소유의 독립 입력

`periodic-borders/thin-fill-occluder.pdf`는 기존 `occluded-inner-dot.pdf`의 검정
stroke·내용·격자를 유지하고 마지막 흰 **fill** 사각형만 `(18,260,4,18)`에서
`(19.5,260,1,18)`pt로 바꾼 별도 입력이다. 원래 입력을 교체하지 않는다.
`generate_periodic_border_sources.py --output <폴더>`로 재현하며 기존 5개 PDF는
생성 변경 전과 바이트가 같다.

새 입력 SHA-256: `021b3b9c7b68b00f082313d60d569740b8b5bd585dd9d13a4ff88e732125b8e6`.

얇은 fill은 경계 후보에 기여할 수 있지만 source stroke와 같지 않다. 이 입력의 흰 면은
가림이며 경계 조각의 편집 소유로 승격하지 않는다. 색상 자체를 배제 조건으로 삼지 않는다.
source `paintKind`를 소비하지 않은 초기 코드의 흰 면 표지 1개가 현재 0개로 줄었고,
원래 검정 stroke 조각 288개의 소유는 실제 IR·양 저장 형식에서 보존된다.

입력 source 자체의 PDF paint가 독립 기준이다. 96 DPI·compat 2022·같은 Arial/Batang
공급으로 직접 열기/HWP/HWPX의 전쪽 Native 최저 99.40853%, fresh Docker WASM
최저 99.39166%를 먼저 확인하고 대표 compare/overlay를 직접 판독한 뒤 등록했다.
좌표/렌더링 golden은 추가하지 않는다. 한컴의 native 테두리 편집 완료를 뜻하지 않는다.

기존 `nested-dash-image.pdf`의 별도 image XObject와 `nested-periodic-background.pdf`의
넓은 색 채움은 경계 소유를 받지 않는 정상 대조다. 실제 검정 stroke는 이와 별도로 보존한다.
metadata 용량 검사는 자격 확인된 원본의 모델을 복제한 codec 검사이며 복제 문서의
한컴 쪽수·배치 일치 증거로 쓰지 않는다.

정식 검사: `tests/cases/issue_7998_pdf_border_paint_ownership.rs`.
source/빌드 신원·원시 TSV·실제 API·미완료 경계는
[stage15](../../mydocs/working/task_m100_7998_stage15.md)에 연결한다.
