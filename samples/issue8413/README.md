# #8413–#8416 PDF 가져오기 재현 자료

2026-11-01 메인테이너가 사용자 제공 원본 PDF 4개의 공개 커밋을 명시적으로 승인했다.
파일 이름과 원래 바이트를 보존했으며 [manifest.json](manifest.json)에 크기·SHA-256을 기록했다.
기존 `/mnt/e/tmp/`의 실제 검증 파일과 해시가 같다. 각 PDF는 50 MiB 미만이다.
추출한 글꼴 프로그램·실제 사용자 암호는 포함하지 않는다.

| 파일 | 쪽수 | 재현 범위 |
| --- | ---: | --- |
| img_clipping_01.pdf | 32 | 곡선/복합 clip, 회전 이미지, Type3 paint와 색 보존 |
| img_bug_01.pdf | 37 | 큰 이미지·반복 배치·디코딩 공유 |
| PDF_ENCRYPTION_UNSUPPORTED.pdf | 57 | 빈 암호 인증·회전 텍스트·이미지/배경 보존 |
| f87pro.pdf | 1 | 많은 content operator, soft mask와 곡선 clip |

[`curve-mask-controls.hwpx`와 `.hwp`](../../tests/fixtures/issue8413/README.md)는 [공개 합성 13쪽 PDF](../../pdf/issue8413/README.md)를
가져와 저장·재열기한 결과이며 별도 한컴 원문으로 주장하지 않는다. 생성 원본 PDF와 코드는
기존 `pdf/issue8413/`에 있다. 암호 필수 입력의 R2–R6 대조군은
[기존 인증 fixture](../../pdf/issue8415/authentication/README.md)를 재사용한다.

```bash
rhwp import-pdf samples/issue8413/f87pro.pdf output/f87pro.hwpx --json
```

Studio에서도 같은 파일을 열 수 있다. 큰 입력의 실제 검증은 표준 프로필·제한 시간 300초로
수행했으며 초기 표시가 빠르다는 뜻은 아니다. 완료 범위·전쪽 Native/fresh WASM 수치·환경은
[작업 기록](../../mydocs/working/task_m100_8413_implementation.md), 남은 지연·글꼴·세선 차이는
[후속 백로그](../../mydocs/plans/task_m100_8413_backlog.md)에 있다.
