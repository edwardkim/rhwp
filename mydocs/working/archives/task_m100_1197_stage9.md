# Stage 9 완료 보고서 - Task M100-1197

- 이슈: #1197
- 제목: HWPX 용지 기준 BehindText 그림/표 z-order 보존
- 브랜치: `local/task1197`
- 작성일: 2026-06-02
- 상태: 완료

## 1. 배경

작업지시자가 #1197 재현 샘플의 저작권 문제가 해결되었음을 확인했고,
PR #1252에 원본 샘플과 참조 PDF를 함께 포함하도록 요청했다.

## 2. 추가 파일

기존 저장소 관례에 맞춰 다음 위치에 배치했다.

- HWPX 원본: `samples/hwpx/[2027] 온새미로 1 본교재.hwpx`
- HWP 대응 파일: `samples/hwpx/hancom-hwp/[2027] 온새미로 1 본교재.hwp`
- 참조 PDF: `pdf-large/hwpx/[2027] 온새미로 1 본교재.pdf`

## 3. 검증

통과한 확인:

```sh
cargo run --bin rhwp -- info 'samples/hwpx/[2027] 온새미로 1 본교재.hwpx'
cargo run --bin rhwp -- info 'samples/hwpx/hancom-hwp/[2027] 온새미로 1 본교재.hwp'
python3 -c "import fitz; d=fitz.open('pdf-large/hwpx/[2027] 온새미로 1 본교재.pdf'); print(d.page_count)"
```

확인 결과:

- HWPX: 정상 로드, rhwp 기준 47쪽
- HWP: 정상 로드, rhwp 기준 50쪽
- PDF: 정상 로드, 46쪽

## 4. 비고

PDF 46쪽, HWPX 47쪽, HWP 50쪽의 page count 차이는 이번 z-order/layer 합성 수정과 별도인
pagination/partial table 후속 이슈로 분리한다.
