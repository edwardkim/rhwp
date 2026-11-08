# 정확한 입력 크기 경계 PDF

`generate.py`가 만든 정상 1쪽 PDF다. 화면 내용은 `SIZE_BOUNDARY`이며, 사용하지 않는
stream의 길이로 전체 파일 크기를 맞춘다. 무효 PDF의 거절과 정상 큰 파일의 수용을 구분한다.

```bash
python3 samples/issue7962/generate.py --out samples/issue7962
# 64 MiB 경계는 저장소 밖에서만 생성한다.
python3 samples/issue7962/generate.py --out /local/pdf-boundaries --large
```

16 MiB의 직전·정확한 경계·직후 파일만 저장소에 포함한다. 64 MiB의 세 파일은 모두
50 MiB 이상이므로 로컬에만 보관한다. 생성기는 기존 파일이 다른 바이트이면 중단하며,
실제로 저장한 파일의 SHA-256과 qpdf 결과를 출력한다. 과거의 임시 파일을 검사 없이 재사용하지 않는다.
앞 단계의 임시 16 MiB 파일 3개는 기록 해시와 달라 보존하고, 원래 생성식의 기록 해시와
일치하는 파일을 재생성하여 최종 검증 입력으로 삼았다.

| 바이트 | SHA-256 |
| ---: | --- |
| 16777215 | `992b31ffa7cbb9280f3dc4f37b1ad6ccf7f96c91ce70e88c6993535555816968` |
| 16777216 | `0aeb2e75f7dbee75cabe11694cb69d61e0ab02970eee253379a6c1d6db66069a` |
| 16777217 | `e6f6835520f3434ac5ed2ccde5bd7f88143e1b15eb5c28af4c8d3fb33c30d90a` |

로컬 전용 64 MiB 파일의 해시와 실행 결과는 통합 보고의 입력 manifest를 참조한다.
