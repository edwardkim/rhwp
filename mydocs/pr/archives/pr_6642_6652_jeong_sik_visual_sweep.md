# jeong-sik #6642, #6650, #6652 통합 시각 검토

## 범위와 기준

- 통합 브랜치: `review/jeong-sik-open-3pr-20260903`
- 기준 `upstream/devel`: `8d4c25d014dc42992aad6fa92c8eb761254c6bfc`
- 통합 commit: `76c27710a`, `018b64322`, `e0e07783d`
- native Skia CLI: `CARGO_TARGET_DIR=target/pr-review-jeong-sik-open-batch-20260903 cargo build --locked --profile release-test --features native-skia --bin rhwp`
- 이미지 출력: `target/pr-review-jeong-sik-open-batch-20260903/release-test/rhwp export-png`

## 증적

| 원 PR | HWP 원본과 쪽 | PDF 기준과 쪽 | 비교 첨부 | 관찰 범위 |
| --- | --- | --- | --- | --- |
| [#6642](https://github.com/edwardkim/rhwp/pull/6642) | `samples/exam_kor.hwp` 5쪽 | `pdf/exam_kor-2022.pdf` 5쪽 | [PNG](../assets/pr_6642_6652_jeong_sik_integration_20260903/review_6642_rhwp_oracle.png) | 셀, 표, 글자처럼 도형의 페이지 내 구조와 배치 |
| [#6650](https://github.com/edwardkim/rhwp/pull/6650) | `samples/k-water-rfp.hwp` 17쪽 | `pdf/k-water-rfp-2022.pdf` 17쪽 | [PNG](../assets/pr_6642_6652_jeong_sik_integration_20260903/review_6650_rhwp_oracle.png) | 중첩 표와 도식의 물리 배치, 여백 계약 |
| [#6652](https://github.com/edwardkim/rhwp/pull/6652) | `samples/table-in-tbox.hwp` 2쪽 | `pdf/table-in-tbox-2022.pdf` 2쪽 | [PNG](../assets/pr_6642_6652_jeong_sik_integration_20260903/review_6652_rhwp_oracle.png) | 글상자, inline 그림, 표의 물리 배치 |

## 판정

- #6642: 셀 안 줄 높이를 보존하는 대상 구조가 기준 PDF와 같은 페이지 위치에 있다.
- #6650: 안쪽 표 및 도식의 물리 배치를 확인했고, 새 회귀 테스트가 바깥 여백을 포함한 좌표 계약을 단언한다.
- #6652: 글상자, inline 그림, 표의 상대 위치를 확인했고, 새 회귀 테스트가 그림 뒤 텍스트 시작 좌표를 단언한다.

## 한계와 재현성

현 Mac의 native Skia 글꼴 집합에는 두 한국어 원본의 모든 글꼴이 없어 #6650과 #6652의 RHWP 이미지 일부 글리프가 대체 문자로 보인다. 따라서 이 검토는 물리 배치와 회귀 테스트의 좌표 계약을 증명하며, 문자 글리프의 픽셀 동일성을 주장하지 않는다. 이는 원본 PDF의 래스터 출력과 RHWP 출력이 서로 다른 글꼴 렌더링 환경에서 생성됐기 때문이다.

기준 PDF는 저장소의 실제 정본 파일을 사용했다. 별도의 임의 PDF나 새 golden은 만들지 않았다.

## 제외

[#6647](https://github.com/edwardkim/rhwp/pull/6647)은 기여자가 `2026-09-02T15:14:20Z`에 CLOSED 처리했다. 해당 figure-space 변경은 이미 `upstream/devel`의 기존 #6597 반영에 동등한 구현과 회귀 증적이 있으므로, 이번 3건 통합과 시각 검토의 대상이 아니다.
