# Stage86: WMF 비트맵 마스크의 원본 격자 합성

Issue: #7445. 부모 HEAD는 `3dada4386c000fbec6510aca204f660a1ae19ff8`,
base는 `d9fd85ed5bc3c4d9ce6b2981422e48bf9bb90867`이다.

## 원인과 공통 소비 경로

sample16 HWPX의 WMF에는 8×8 흑백 비트맵 패턴으로 바탕과 브러시를 선택하는
ROP 마스크가 있다. 비트 연산을 끝내기 전에 최종 장치 크기로 축소하면 선택 비트가
회색으로 보간되어 넓은 바탕 영역의 칠이 달라진다. 글꼴 교체로 해결되는 차이가 아니다.

`ubuntu-ted2`의 `task/8160-sample16-raw100`,
`0c3148ab54b3a4ca1bdc969e4d9283265625feec`에 있는
`src/paint/metafile_print/masks.rs`를 읽기 전용으로 가져왔다. 같은 바이트의 helper를
`src/paint/metafile_masks.rs`에 두고 기존 공통 `metafile_surface::payload`에 연결했다.
원격 작업 트리와 브랜치는 변경하지 않았다. 다른 원격 수정은 일괄 적용하지 않았다.

실제 소비 경로는 WMF 변환 SVG → 직접 자식의 비트맵 마스크를 원본 격자에서 합성 →
같은 사각형의 RGBA PNG → 기존 글꼴·벡터와 최종 장치 표본 생산 → 공통 Print payload다.
Native와 fresh WASM이 같은 생산 결과를 소비한다. 문서 ID·쪽 번호 분기,
원본 배치 이동이나 비교 문턱 변경은 없다.

helper는 글자와 foreignObject가 있는 마스크, 상위 transform·opacity·style의 해석이
필요한 루트에는 적용하지 않는다. 직접 마스크 개체만 바꾸고 벡터·문자 형제는 보존한다.
최대 67,108,864픽셀, SVG/누적 인코딩 32MiB, 512개 교체 제한을 유지한다.

## 전쪽 및 정상 대조군

같은 입력·독립 Print PDF·글꼴·canonical 도구를 사용했다. 96DPI,
2px 이웃 관용의 raw 실루엣, 90% 문턱을 유지했다. PDF 변환은 현재 정본의
PDFium 152.0.7947.0 / pypdfium2 5.12.1 / `pdfium_smooth_rgb_v1`이다.

| 기존 조건 | 쪽수 | Native 최저 | fresh WASM 최저 | 90% 미만 |
| --- | ---: | ---: | ---: | --- |
| sample16 HWPX | 64/64 | 96.75084% | 96.75000% | 없음 |
| 생물독소 | 205/205 | 98.42761% | 98.42748% | 없음 |
| 보도자료 | 12/12 | 87.89806% | 87.84792% | 양쪽 12쪽 |

6조건·562쪽 TSV를 새로 산출했으며, 모든 TSV의 hash·연속 쪽 번호·중복·누락과
실제 쪽수 일치를 확인했다. 고정 base 대비 2%p 이상 하락은 없다. 이 수치 확인만으로
문단·그림의 의미상 소속 검사를 대신하지 않는다.

집중 정상 대조군인 로드맵 153·295쪽, sample11 2·50쪽과 기존 합성
C352·C354·C356을 Native/fresh WASM에서 다시 확인했다. 집중 12조건은 모두 90% 이상이며
합성 세 조건은 양쪽 모두 100%다. 생물독소·보도자료에는 Stage85 보정도 포함돼 있다.

sample16 HWPX 18쪽은 Native 76.57875→96.75084%, fresh WASM
76.56403→96.75000%로, 35쪽은 82.56128→98.91162%와
82.54824→98.91082%로 개선됐다. 이 비교는 같은 부모 소스의 마스크 적용 전후다.
별도 PR로 제외한 sample16 HWP의 전쪽 완료라고 보고하지 않는다.

18·35쪽 Native/fresh WASM의 review와 overlay를 직접 열어 패턴 면의 복원을 확인했다.
WMF 글리프 굵기·일부 이미지 경계·본문 자간 차이는 남는다. 35쪽이 98% 이상이어도
이 차이가 있는 영역의 절대 px 답안을 새 기준으로 고정하지 않았다.

원본 TSV 6개, 대표 review/overlay 8개와 소스·빌드 출처는
[`wmf-source-grid-mask`](../pr/assets/issue7445/completed-20261009/rebase-d9fd85ed-20261023/wmf-source-grid-mask/)에
바이트 그대로 보존했다. 긴 실행 로그는 ignored output의
`rebase-d9fd85ed-20261023/stage86-mask-grid-preparation/`과
`stage86-affected-whole/`에만 보존한다.

## 검증 상태

Native와 Mac fresh WASM 빌드는 성공했다. WASM은 locked wrapper의
`--target web --out-dir pkg --no-opt` 로컬 대체 빌드이며 Docker 최적화 검증이 아니다.
루트 pkg와 Studio public의 JS/WASM hash는 일치한다. 새 Studio 초기화 응답의 실제
소비 검증과 최종 전체 회귀는 아직 완료하지 않았다.

기존 `issue_6865_wmf_monochrome_mask_rop` case에
`print_device_mask_reduction_preserves_source_bit_coverage`를 추가했다. 공통
LayerBuilder의 실제 Print WMF→장치 PNG 경로에서 16·20·32·100·200 크기의
50% 원본 비트 피복률과 브러시 색, 원본·배치 보존, 반복 조회를 검사한다.
24비트 패턴은 흑백 선택 마스크로 처리하지 않는 정상 대조군이다. 문서 좌표 핀은 아니다.

같은 최종 검사에서 마스크 격자 합성 연결만 제거하면 **17 PASS / 1 FAIL**(0.265초),
복원하면 **18 PASS / 0 FAIL**(0.173초)다. 실패는 16 크기의 원본 50% 비트 면적이
20.78431%로 줄어든 것으로 재현됐다. 두 실행 사이 기대값은 바꾸지 않았고 소스를 원복한
hash도 동결 빌드의 소스와 일치한다. 최초 작성 중의 v1·v2 색상 검사는 PNG의
비곱셈 RGB를 과도하게 제한했으므로 최종 증거로 쓰지 않는다. 최종 검사는 같은 색을
정수 premultiplied 채널로 되돌려 8비트 변환의 한 단계만 허용한다.

검사 본문의 크기 변경으로 suite 배정이 달라진 것을 `--check`에서 검출해
`--prepare`로 재생성했다. 최종 manifest 확인은 exit 0이며, 재생성한 실제 harness에서도 **18 PASS / 0 FAIL**(0.160초)을 확인했다. fmt와 diff 확인도 exit 0이다. 생성 suite·중간 JSON·로그는
커밋하지 않는다.
현재 KTX와 보도자료 12쪽의 미달이 남으므로 PR 제출을 계속 보류한다.
