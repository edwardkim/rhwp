# 전체 원본 그림 자르기 보정 검증

전체 원본 영역을 가리키는 그림에 불필요한 crop viewport가 만들어지는 문제를
검증했다. 독립 기준은 한컴 2020 Print PDF이며, 2026-11-01 KST에 96 DPI PNG와
색상 경계 보정 전 2px 내용 실루엣으로 Native와 fresh WASM의 전쪽을 비교했다.
제품 코드 기준 커밋은 `5d1294811b`이며 이후 측정 당시 `8c4eca265e`까지의 변경은
작업 기록이다. Native 진단 바이너리 SHA-256은
`26f6692bccaa9e8d392e38b161159afa9040451df47359299c490f28efbcb4a6`, WASM은
`bfce6526989df4aaef8b5fe1e8228237a53642e2c3a89879234af97567b823d2`다.

| 원본 | 한컴 Print | Native 전쪽 | WASM 전쪽 |
| --- | --- | --- | --- |
| [`exam-social-insert.hwp`](../../../../samples/pr8040/exam-social-insert.hwp) | [`exam-social-insert-2020.pdf`](../../../../pdf/pr8040/exam-social-insert-2020.pdf) | [TSV](issue8040-insert-native.tsv) | [TSV](issue8040-insert-wasm.tsv) |
| [`exam-social-insert-delete.hwp`](../../../../samples/pr8040/exam-social-insert-delete.hwp) | [`exam-social-insert-delete-2020.pdf`](../../../../pdf/pr8040/exam-social-insert-delete-2020.pdf) | [TSV](issue8040-insert-delete-native.tsv) | [TSV](issue8040-insert-delete-wasm.tsv) |

입력 SHA-256은 삽입본 `3c3ff50b430c3129a4ce06575b1a398d01fbd4ec17d9dd895f467a945f656e82`,
삭제본 `fe0953499311b5be82c85560a92a35c26ea6be3ee0b9f64460d111b3313dc8ee`다.
기준 PDF SHA-256은 각각 `d368c0ded6f0d9309784e9165aad7360b326a86c01d9938beb95c768ae0d4e74`,
`84704f8c2cccf955379c648ea7d9dbf04242e6c4fb2620e37d87ac004b7a330d`다.

각 원본은 4쪽이다. 모든 쪽이 90% 이상이지만 Native의 일부 쪽은 #8389·#8397에 기록된
정밀 기준보다 낮으므로 이 자료만으로 통합 PR의 무하락 조건을 통과한 것은 아니다.
TSV의 raw 값은 각 manifest의 미대응 픽셀 수와 합집합 픽셀 수에서 계산했다.

삽입 원본의 그림이 포함된 2쪽 직접 비교: Native [review](native-p002-review.png)·
[overlay](native-p002-overlay.png), WASM [review](wasm-p002-review.png)·
[overlay](wasm-p002-overlay.png). 이미지에 표시된 100.00%는 반올림된 값이며,
정밀 수치는 위 TSV를 따른다.

삭제 원본 2쪽: Native [review](delete-native-p002-review.png)·
[overlay](delete-native-p002-overlay.png), WASM [review](delete-wasm-p002-review.png)·
[overlay](delete-wasm-p002-overlay.png).
