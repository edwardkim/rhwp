# 원본 프로그램과 준비된 그림의 공통 Print 프레임 복원

서로 다른 원인으로 잃었던 기존 물리 종이 좌표 계약을 복원했다. 입력·독립 Print PDF·글꼴·
DPI·실루엣 계산과 보호 TSV는 바꾸지 않았다. 특정 파일명·쪽 번호·좌표로 제품을 분기하지 않는다.

## 실제 준비된 WMF의 프레임

일반 WMF는 공통 paint 단계에서 PNG로 준비되지만, source MIME이 WMF여서 일반 PNG의
Print 장치 좌표 투영을 건너뛰었다. 같은 배치의 공개 합성 WMF와 PNG를 대조한 검사는
보정 전 실제 1 FAIL이었다. `metafile-aspect-explicit-pen` 및 `metafile-equal-axis-controls`의
독립 한컴 Print도 대조했다. 단순한 내부 PNG 크기 ceil 변경은 기존 반올림 반례와 모순되어
사용하지 않았다.

실제 준비가 끝난 출력 장치 PNG만 기존 raster 프레임을 소비하게 했다. 중간 WMFC·별도
색/마스크 전달·회전·안여백·자르기와 채움 모드는 기존 소유 경로를 유지한다. 원본 bbox·
준비 전 자원 해상도는 보존하며 그림 테두리의 소유 프레임도 같은 최종 프레임을 참조한다.
보정 후 공개 관계 검사 1 PASS, 기존 WMFC/PNG 계약 5 PASS를 확인했다.

## 원본 HFT 프로그램의 존재 확인과 종이 프레임

명조를 Myeongjo의 같은 원본 프로그램으로 확인했어도 초기 fallback에서 잃은 HFT 종이
프레임이 복원되지 않았다. 원본 프로그램 조회가 인증됐고 원본 메트릭을 보존하는 경우에만
그 프레임을 복원한다. 자연 전진을 선택한 요청·다른 대체 조회·내장 face는 포함하지 않는다.
공개 합성 CFF/HWPML의 원본 조회·자연 전진·대체 조회 검사에서 보정 전 1 FAIL을 확인하고,
수정 뒤 1 PASS를 확인했다. 기존 HFT 윤곽 계약 19건도 모두 PASS다.

이 원인 분석은 정본 보호 TSV의 원 해결·후속 검증 PR #8389/#8397과 같은 입력·PDF를
대조했다. 합성 굵기 원 구현은 이슈 #8430을 해결하는 PR #8605의 실제 head
`5d37286267136c9978d7ac2f804913730ab6becf`와 통합 커밋 `373ec6fa58`을 별도로 확인했다.
원 이슈 번호와 PR 번호를 혼동하지 않는다.

## 실제 출력과 검증 범위

| 입력·쪽 | Native | fresh WASM | 보호값 Native / WASM |
| --- | ---: | ---: | ---: |
| sample10 509쪽 | 99.91% | 99.91% | 99.89% / 99.90% |
| sample10 757쪽 | 99.55% | 99.55% | 99.41% / 99.40% |
| sample16 HWP2024 35쪽 | 100.00% | 100.00% | 99.99% / 100.00% |
| sample16 HWPX 35쪽 | 100.00% | 100.00% | 99.99% / 100.00% |

[집중 비교 TSV](focus-comparison.tsv)는 backend별 7조건·16쪽, 총 32행이다.
그중 정본 보호값이 있는 24행 모두 하락 0이며, 독립 WMF 대조군 4종도 양쪽 모두 100%다.
최소 글꼴 Native 집중 검사 26 PASS와 로컬 no-opt fresh WASM 실제 Chrome 6 PASS를 확인했다.
이 빌드는 Docker 최적화 빌드가 아니다. 원본/저장 정보·원문 내용·줄 소속·쪽 번호를 보존한다.

다음 review·overlay를 함께 직접 대조했다. raw는 같은 실제 출력 PNG다.

- sample10 509쪽: [review](sample10-wasm-p509-review.png) · [overlay](sample10-wasm-p509-overlay.png) · [raw](sample10-wasm-p509-raw.png) · [PDF raw](sample10-wasm-p509-pdf-raw.png)
- sample10 757쪽: [review](sample10-wasm-p757-review.png) · [overlay](sample10-wasm-p757-overlay.png) · [raw](sample10-wasm-p757-raw.png) · [PDF raw](sample10-wasm-p757-pdf-raw.png)
- sample16 35쪽: [review](sample16-hwp2024-native-p035-review.png) · [overlay](sample16-hwp2024-native-p035-overlay.png) · [raw](sample16-hwp2024-native-p035-raw.png) · [PDF raw](sample16-hwp2024-native-p035-pdf-raw.png)

raw 100.00%는 정본의 2px 내용 실루엣 지표이며 RGB 모든 픽셀의 동일함을 뜻하지 않는다.
미세 래스터·색상 차이도 overlay에 그대로 보존했다. 이 집중 결과를 문서 전쪽·정본 전체·
전체 회귀 완료로 확대하지 않는다. 최종 안정 소스의 전쪽·전체 회귀·소비 경로 검증이 남는다.
[실행·producer·hash](validation.json)에 실제 결과와 이전 전체 검사의 단일 분류 실패 및
그 후속 보정을 구분했다. 비공개 글꼴·로그·임시 진단은 ignored output에만 보관한다.
