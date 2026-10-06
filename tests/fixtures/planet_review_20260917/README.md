# planet6897 PR 통합 검토 입력 (2026-09-17)

#7211·#7214의 실제 브라우저 동작과 #7225의 3→4쪽 회귀를 검토하는 원문이다.
기존 Git 원문과 크기·SHA-256을 대조해 동일 사본이 없는 3개만 원래 파일명으로 추가했다.

| 식별자 | 용도 | 저장 제품 | 원문 SHA-256 | 기준 PDF / SHA-256 |
| --- | --- | --- | --- | --- |
| 3026219 | 중첩 표 hover·drag·Undo | hancom-office-2020 / 11.0.0.2129 | `d9ff6d9670750bae8db9a522d0d8a93c44d7b77e65e31d85e82624809662be7e` | `pdf/planet-review-20260917/3026219-2020.pdf` / `6a10c83440715cd10664981626e603296958d0d640dffa96834f59aa70a738fd` |
| 3147199 | 병합 셀 경계 가이드 구간 | hancom-office-2020 / 11.0.0.2129 | `bc4239d50de6b2663ee6774ca3231988fab4b686c887fdec10d2a26e4b875d35` | `pdf/planet-review-20260917/3147199-2020.pdf` / `d008724a9e8eb1a573558319555a7b3197d81889a5337596e2f919598070ddf1` |
| 156676190 | 문단 트림 수정의 쪽수 반례 | hancom-office-2018 / 10.0.0.14241 | `e91571ea1dc745d01d3653d6b21361231dc8a6eb3137fa1dfe59d9a9983827fb` | `pdf/planet-review-20260917/156676190-2020.pdf` / `619a55175c88c7866b490e06ba09ec15a291c163b797eb513f7412de0b79ba89` |

기준 PDF는 동일 원문을 hwp2024 MCP client의 `start → status → download`로 변환했다.
저장 제품에 따라 `engine=2020`, 실제 Hancom `11.0.0.9136`, `input_preprocess=none`이었다.
3026219는 2쪽, 3147199는 1쪽, 156676190는 3쪽이며 EOF·SHA 검증 후 저장했다.
변환본 자체의 존재는 PR 시각 검증 통과를 뜻하지 않는다. 실제 판정은 개별 PR review에 기록한다.

## 메인터너 보정의 추가 대조 기준

원문은 기존 `samples/issue6145/worklife_balance_index_156607916.hwpx`를 그대로 재사용한다
(SHA-256 `7778e5373baf7371a7c643cf4cca73defb065a92ecc941a3e55fff34fcdd2d17`).
`rhwp info --json`은 저장 제품 2018, 버전 `10.0.0.11529`를 보고했다.
2026-09-17 같은 MCP의 `start → status → download`, engine 2020 / Hancom `11.0.0.9136`,
`input_preprocess=none`으로 6쪽 PDF를 받았다(job `3acc4c0a-f310-4287-9a6e-e1a5ebe3c9b6`).
추가 PDF는 `pdf/planet-review-20260917/worklife_balance_index_156607916-2020.pdf`,
SHA-256 `0cc7229c1d8da5ef425ef76f3ba143db17171a71449bc4e341579a18a07a47ec`다.
원문을 새 이름으로 복제하지 않았다.
