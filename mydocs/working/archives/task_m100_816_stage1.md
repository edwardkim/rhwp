# Task #816 Stage 1: 현행 devel 재현 검증

## 1. 목적

#816의 보고 현상은 v0.7.11 기준이며, 현재 `devel`에서 이미 해소되었을 가능성이 높다.
따라서 소스 수정 없이 현행 렌더링/내보내기 결과를 검증했다.

## 2. 검증 환경

```text
branch: devel
rhwp: target/debug/rhwp v0.7.13
```

이슈 댓글은 없었다.

```text
gh api repos/edwardkim/rhwp/issues/816/comments
=> []
```

## 3. export-svg 검증

### exam_kor

```text
target/debug/rhwp export-svg samples/hwpx/exam_kor.hwpx -o output/poc/task816_current/exam_kor_hwpx_svg
```

결과:

```text
문서 로드 완료: samples/hwpx/exam_kor.hwpx (20페이지)
내보내기 완료: 20개 SVG 파일
```

비교 HWP:

```text
target/debug/rhwp export-svg samples/exam_kor.hwp -o output/poc/task816_current/exam_kor_hwp_svg
```

결과:

```text
문서 로드 완료: samples/exam_kor.hwp (20페이지)
내보내기 완료: 20개 SVG 파일
```

### aift

```text
target/debug/rhwp export-svg samples/hwpx/aift.hwpx -o output/poc/task816_current/aift_hwpx_svg
```

결과:

```text
문서 로드 완료: samples/hwpx/aift.hwpx (74페이지)
내보내기 완료: 74개 SVG 파일
```

비교 HWP:

```text
target/debug/rhwp export-svg samples/aift.hwp -o output/poc/task816_current/aift_hwp_svg
```

결과:

```text
문서 로드 완료: samples/aift.hwp (74페이지)
내보내기 완료: 74개 SVG 파일
```

## 4. 후반 페이지 텍스트 검증

`aift.hwpx`의 5~11페이지 SVG에는 `<text>` 노드가 존재한다.

```text
aift_005.svg: 263
aift_006.svg: 720
aift_007.svg: 906
aift_008.svg: 590
aift_009.svg: 463
aift_010.svg: 492
aift_011.svg: 831
```

이슈 본문에서 언급한 표 셀 텍스트 계열인 `소`, `속`, `직`, `급`, `성`, `명`도
후반 SVG에 실제 `<text>`로 출력된다.

## 5. export-pdf 단일 후반 페이지 검증

전체 PDF 변환은 시간이 길어 단일 후반 페이지로 텍스트 보존 여부를 확인했다.

```text
target/debug/rhwp export-pdf samples/hwpx/aift.hwpx -p 4 -o output/poc/task816_current/aift_hwpx_p005.pdf
target/debug/rhwp export-pdf samples/hwpx/aift.hwpx -p 10 -o output/poc/task816_current/aift_hwpx_p011.pdf
```

결과:

```text
aift_hwpx_p005.pdf: 1페이지, 28KB
aift_hwpx_p011.pdf: 1페이지, 45KB
```

`pdftotext` 확인 결과 두 PDF 모두 텍스트가 추출된다.
특히 11페이지 PDF는 표 내부 수치/항목 텍스트와 `<기술개발 핵심어>` 영역 텍스트가 추출된다.

## 6. 판정

현재 `devel`에서는 #816의 핵심 현상이 재현되지 않는다.

| 항목 | 판정 |
|---|---|
| HWPX export-svg가 4페이지만 생성 | 재현 안 됨 |
| 10페이지 이상 HWPX 전체 SVG 생성 | 성공 |
| HWPX 후반 페이지 표 셀 텍스트 누락 | 재현 안 됨 |
| HWP/HWPX 페이지 수 정합 | 성공 |
| export-pdf 후반 단일 페이지 텍스트 보존 | 성공 |

## 7. 잔여 관찰

`export-pdf samples/hwpx/exam_kor.hwpx` 전체 20페이지 변환은 장시간 완료되지 않아 중단했다.
이는 #816의 "후반 페이지 텍스트가 비어 있다" 현상과는 다른 성격의 성능/CLI PDF 변환 이슈로 보이며,
필요하면 별도 이슈로 분리하는 것이 적절하다.

## 8. 다음 단계

#816은 현행 `devel` 기준으로 이미 해소된 것으로 판정한다.
작업지시자 승인 후 GitHub 이슈에 검증 결과를 남기고 close 처리한다.
