# 최신 통합 source의 전체 회귀·Native Skia 라이브러리 완료

검증 source `ba2ebfd34a4ebf402bce8e002254b672a09ef764`, 정책 비교 base `f529dc92d4d172bf32390d7749743a45d9e41bfc`.
검증 전후 Rust·Cargo·내장 자원 등 4,694개 파일의 고정 snapshot을 대조했다. 이후 문서 커밋은 제품 소스를 바꾸지 않는다.

| 완료 범위 | 실제 최종 결과 |
| --- | --- |
| 전체 integration nextest | **12,583 PASS·0 FAIL·45 SKIP·exit 0**. 실제 테스트 실행 919.391초 |
| Native Skia 라이브러리와 내부 crate | **4,040 PASS·0 FAIL·12 IGNORE·exit 0**. root 3,858개와 내부 crate 15·165·2개 |

회귀 환경은 시스템·사용자·상용 글꼴 경로를 숨기고 공개 자원 5개만 노출한 bwrap 환경이다.
`NotoSansKR-ExtraLight.ttf`, `NotoSansKR-Regular.ttf`, `NotoSansSymbols2-Regular.ttf`,
`RHWPOldHangul-Regular.otf`, `RHWPPublicBatang-Regular.ttf`를 실제 fc-list로 확인했다.
시각 비교의 시스템 글꼴 전체 공급과 구분하며, 설치된 비공개 OTF를 회귀 통과 조건으로 쓰지 않았다.

## 실행과 출처

Cargo는 같은 `/home/tsjang/rhwp/target/pr-review`를 순차 재사용했으며 build jobs 8, nextest threads 12다.
같은 target을 쓰는 다른 Cargo 작업은 겹치지 않았다. 각 명령의 실제 시작·종료는 컴파일 시간을 포함한다.

### full-nextest-minfont

실제 실행 head `dc703ef8180e0c410aa754011c34660e33e68480`. 시작 UTC `2026-11-05T01:47:35.341785+00:00` → 종료 UTC `2026-11-05T02:35:34.595430+00:00`.
로그 SHA-256 `d19e9baa93566191a436de9d0365fcdd5b267e2d71f824d8840b79f30152c459`.

```bash
bash /home/tsjang/rhwp/output/pr-review/planet6897-20261103/run_minfont_review.sh cargo nextest run --locked --cargo-profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --tests --test-threads 12 --no-fail-fast
```

### skia-lib

실제 실행 head `0e412ebe8dc6aa726c52dcf359241207d045930d`. 시작 UTC `2026-11-05T02:35:34.690855+00:00` → 종료 UTC `2026-11-05T02:46:41.806814+00:00`.
로그 SHA-256 `5f668d59d7539c3b173ac3f13cc9d60398560f1b96076069dbd096435d409063`.

```bash
bash /home/tsjang/rhwp/output/pr-review/planet6897-20261103/run_minfont_review.sh cargo test --locked --profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --features native-skia --lib -- --test-threads 12
```

## 남은 공통 검증

이 기록은 위 두 실행의 완료 증거다. CI Native 개별 모듈 21개와 doctest는 진행 중이다.
CI workflow에서 추출한 21개 모듈과 실행 계획의 21개 모듈이 정확히 일치하고 누락·추가가 없음을 확인했다.
새 base 실제 제품과의 기능 보존 대조·보호 대상 WASM 집계·최종 리뷰 갱신 전에는 PR 준비 완료나 최종 승인으로 판정하지 않는다.
원시 로그·JSON은 로컬 output에 보관하며 제출 자료에는 복사하지 않는다.
