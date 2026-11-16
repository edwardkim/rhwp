# Canvas 그림 가장자리 보정 결과

> 이 문서는 당시 단계의 분석 기록이다. 최종 결과와 대표 증적은 [검증 기록](../verification-record.md)과 [최종 대상 증적](../final/main/README.md)을 따른다. 원시 로그·중간 분석 JSON·중간 PNG는 제출에서 제외했다.

[선행 분석과 계획](../canvas-image-coverage-plan.md)의 실제 제품 보정 단계다.
최적화 fresh WASM은 부모 `5ea4c47c4bcf339d64376adbbdf259a5f339c9db` 위의 보정 소스로
빌드했다. 실제 Rust 파일·JS/WASM·원본·PDF 해시는 [검증 기록](../verification-record.md)에 있다.
Native는 `5fdd749763` 단계의 고정 제품 binary다. 이번 변경은 WASM Canvas에만 적용된다.

| 같은 원본·독립 PDF | 전체 쪽수 | Native | 수정 전 WASM | 보정 제품 fresh WASM |
| --- | ---: | ---: | ---: | ---: |
| `samples/pic-crop-01.hwp` | 1 | 100.00% | 94.65% | 100.00% |

96 DPI·Print·compat 2022·색상 경계 보정 전 2px raw다. 기존
`pdf/pic-crop-01-2022.pdf`를 재사용했으며 입력·PDF·좌표·source crop·픽셀·문턱을 변경하지 않았다.
전체 [Native TSV의 검증 요약](../verification-record.md)와 [fresh WASM TSV의 검증 요약](../verification-record.md)를 제출한다.
사설 설치 글꼴을 사용하지 않는 그림 입력이다.

## 실제 수정과 회귀

Native가 사용하는 그림 가장자리 anti-alias에 대응하도록, Print의 축에 평행한 일반
Canvas bitmap 그리기에 실제 목적 사각형의 부분 픽셀 덮임을 공급했다. 기존 장치 프레임·
축소 재구성이 직접 그리는 경로와 Screen·회전/기울임은 유지한다. 캐시 canvas와 image,
전체 그림과 crop이 같은 scope를 사용하며 `Path2D`로 기존 context path를 변경하지 않는다.

원본의 두 그림 위 테두리가 연속 잉크로 남는지 검사하는 관계형 회귀는 수정 전 두 번째
테두리에서 연속 잉크 `0/570`으로 [실패](../verification-record.md)했다. 기존 JPEG
장치 프레임 검사 2건과 새 검사는 보정 fresh WASM에서 [3건 모두 통과](../verification-record.md)했다.
기존 CI의 동일 browser aggregate 경로도 [20 PASS / 0 FAIL](../verification-record.md)이다.
이 aggregate는 최종 test summary를 보존했으며, 별도 process의 exit code는 수집하지 못해
그 값을 추정하지 않는다. 코드 실행 이후 검사 첫머리의 측정 근거 주석만 갱신했다.

## 같은 페이지 직접 판독

[review](../verification-record.md)와 [overlay](../verification-record.md)는 진단 교체 PNG가 아닌 새 제품의
실제 캡처다. 두 그림의 crop·내용·위 테두리 보존을 확인했다. raw의 100.00%는 2px 실루엣의
공식 표시값이며 색·획 굵기까지 픽셀 단위로 동일하다는 뜻은 아니다. PDF의 더 진한 테두리와
그림 내부 재표본 색 차이는 보인다. Native에서도 기존에 보이는 차이를 이 보정으로 해결했다고
주장하지 않는다. 이번 결함은 Canvas가 위 테두리 전체를 지우던 것이며 그 소실을 복구했다.

![실제 제품과 독립 Print 검토](../verification-record.md)

![같은 쪽 겹침](../verification-record.md)

관련 보호 입력 `issue3307_outline_number.hwpx`의 새 fresh WASM도 전 9쪽 100.00%다.
[전쪽 TSV의 검증 요약](../verification-record.md)를 보존한다. 원인 분리가 끝나지 않은 다른 문서의
하락은 이 결과로 면제하지 않는다. 중첩 표·sample10·전체 보호·편람 gate·전체 로컬 검증과
최종 리뷰·원격 PR·CI·병합·후속 처리가 남아 있어 PR 전체 검증 완료를 뜻하지 않는다.
