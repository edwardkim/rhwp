# #5169 변경 추적의 저장 보기 대조군

공개 `samples/issue5169_viewtext_changetracking.hwp`를 한컴 2020으로 변환한 HWPX다.
메인터너 제공본과 독립 변환본의 header.xml·section0.xml이 동일함을 확인했다.
입력 SHA-256: `1c822a1c9962eadf9ae4a68206051c7d1474f811d5401074fbb05de309f36b26`.

- 기준 출력: `pdf/issue7674/issue5169_viewtext_changetracking-hwpx-2020.pdf`.
- Print: Hancom 2020, Hwp 11.0.0.9136, method 0, 6쪽, job `9d83c530-a806-405a-8a10-216a591b8d26`.
- 독립 기대: 가시 표 4개, 삭제된 옛 제목·문구 제외, 본문 첫 쪽에서 빈 줄 두 개 뒤에 제목.
- Native/fresh Docker WASM 전체 6쪽 최저 90.87964%; 메인터너 Studio 시각 판정 통과.
- XML 대조군은 문단 형제 요소 사이 개행만 추가하며 문단 안 텍스트·빈 줄은 바꾸지 않는다.

검증 source·전쪽 TSV·대표 이미지·한계는 `mydocs/pr/pr_7698_review_impl.md`에 기록한다.
HWPX 보기 전환 UI·변경 수락/거부 및 이력 전체 저장을 검증한 입력은 아니다.
