# Task #86 단계2 완료 보고서: CLI thumbnail 명령

## 수행 내용

`rhwp thumbnail` CLI 명령을 추가하여 HWP 파일에서 썸네일을 추출한다.

## 변경 파일

| 파일 | 변경 내용 |
|------|----------|
| `src/main.rs` | `thumbnail` 서브커맨드 + help 추가 |
| `mydocs/tech/hwp_spec_errata.md` | #27 PrvImage PNG 포맷 미기재 정오표 |

## CLI 사용법

```bash
rhwp thumbnail sample.hwp                    # output/sample_thumb.png 저장
rhwp thumbnail sample.hwp -o my_thumb.png    # 지정 경로에 저장
rhwp thumbnail sample.hwp --base64           # base64 stdout 출력
rhwp thumbnail sample.hwp --data-uri         # data:image/png;base64,... 출력
```

## 검증 결과

| 파일 | 포맷 | 크기 | 용량 |
|------|------|------|------|
| biz_plan.hwp | PNG | 724x1024 | 12,097 bytes |
| shift-return.hwp | GIF | 177x250 | 1,264 bytes |

- 최신 HWP: PNG 사용, 구 버전 HWP: GIF 사용 확인
- PrvImage 없는 파일: 에러 메시지 + exit code 1
- `--base64`, `--data-uri` 모드 정상 동작
- 785 테스트 통과

## 커밋

`087f953` Task #86 단계2: CLI thumbnail 명령 + PrvImage PNG 정오표
