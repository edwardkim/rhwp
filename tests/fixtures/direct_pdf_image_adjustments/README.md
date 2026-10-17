# 직접 PDF 이미지 밝기·대비 검증 자료

## 기여자가 제출한 PNG

이 폴더의 PNG6개는 원 PR #7494의 코드 후보
`488fa93257cd144489b673d1de8c7951c9d695a0`에서 제출한 과거 증적이다.
현재 통합 후보의 재출력이나 문서 전체 검증 결과로 간주하지 않는다.

- 원본: `samples/2025 행정업무운영 편람(최종).hwp`
- 원본 SHA-256: `40d6d05eac4d55bdc4b0c62c42d93af104d5123b447581246f36fd15de7bd46f`
- 원 기여자 PDF: `pdf/2025 행정업무운영 편람(최종)-hwp-2020.pdf`
- PDF SHA-256: `1ff2f5b3158c902700c40dde4afa258bc3fef3df61b7081e0d50027c94007cec`
- PDF 전체384쪽, 대표 물리3쪽(CLI `-p 2`). Creator는 Hwp2022이며 파일명과 구분한다.

`native_review_003.png`·`native_overlay_003.png`는 CLI SVG를 Chrome에서
래스터화한 자료다. **Native Skia PNG가 아니다.** `wasm_*`도 당시 WASM의
SVG 경로다. `direct_pdf_*`는 직접 PDF를 별도로 래스터화한 자료다.
후자는 SVG 이미지 의미와의 비교를 포함하며 독립 한컴 PDF 전체의 일치를 입증하지 않는다.

## 메인터너 보정의 독립 기대값

SVG의 `render_image`·`render_page_background_image`는 바깥 그림 효과 그룹 안에
밝기·대비 그룹을 둔다. 안쪽 밝기·대비와 채널별 clamp를 먼저 적용하고,
바깥 회색조를 나중에 적용해야 한다. 기본 색 공간은
[Filter Effects 1의 linearRGB](https://www.w3.org/TR/filter-effects-1/#propdef-color-interpolation-filters)다.

Chrome155에서 기존 SVG 중첩과 같은 사양 사례를 직접 실행했다.

| 입력·필터 | 실제 SVG RGB | 검출하는 오류 |
| --- | --- | --- |
| RGB(80,100,120), 대비50, 바깥 회색조 | (12,12,12) | 회색조를 먼저 적용하면 (0,0,0) |
| RGB(80,100,120), 밝기20·대비20 | (121,136,155) | 두 값이 모두 존재한다는 이유로 Native 보정을 생략 |
| RGB(170,170,170), 밝기10, 바깥 이산 흑백 | (255,255,255) | 고대비 행렬의 중간 회색을 흑백 성공으로 처리 |

색상·알파 계약은 `tests/render_p37_direct_pdf_export.rs`의 기존 Native 전용
통합 진입점에서 검사한다. 일반 그림·회색조는 선형 RGB와 올바른 합성 순서를
사용하고, 이미 bake된 payload는 다시 보정하지 않는다. 미지원 이산 흑백 조합은
직접 PDF에서 명시적으로 실패한다. Native의 기존 흑백·패턴 경로는 유지한다.
위 합성 색상 계약을 실제 한컴 문서 배치·쪽수 검증으로 승격하지 않는다.

```sh
node scripts/rust-test-suite-manifest.mjs --prepare
CARGO_BUILD_JOBS=8 node scripts/run-rust-test.mjs render_p37_direct_pdf_export -- \
  --cargo-profile release-test --features native-skia \
  --target-dir /home/tsjang/rhwp/target/pr-review
```

## 전체 문서와 독립 Print 기준

현재 devel의 편람은383쪽으로, 과거384쪽 PDF와 전체 쪽수가 다르다.
#7009·#7751의 기존 기록과 원본을 보존하며, 현재383쪽으로 과거 핀을 바꾸지 않는다.

같은 원본의 한컴2024+KoPub Print 기준은
`pdf/2025 행정업무운영 편람(최종)-hwp-kopub-2024.pdf`에 별도로 있다.
SHA-256 `3cdf329004cffad0ebacd122586e5e1702aff36b3015cbcb401d038857019593`,
383쪽이며 [변환 원장](../../../mydocs/tech/investigations/issue-7009/hancom_2024_kopub_oracle.json)에
engine2024·실제13.0.0.3901·Print1-up·입력/출력 해시가 기록돼 있다.
KoPub을 공급하고 compat2024로 비교한 결과는 기존384쪽 비교와 구분한다.

현재 실행의 전체 Native/fresh WASM TSV·로그·대표 PNG는 ignored
`output/pr-review/pr7494-20261016/`에 보존한다. 대표3쪽·일부 테스트의 성공만으로
문서 전체 시각 검증, PR 수용 또는 전체 직접 PDF 지원을 선언하지 않는다.
연결선·화살촉의 독립 미지원 경계는 #7501(물리63쪽)에서 계속 추적한다.
