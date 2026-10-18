# #7998 내용 근거의 지역 축·닫힌 빈 셀 대조군

`generate_sources.rs`는 공개 native-borders scaffold에서 정상 create/insert/merge/border API로
9개 HWPX를 생성한다(`print-source-inputs/`). 저장 LineSeg를 수동 조작하지 않는다.
engine 2020·11.0.0.9136·PrintMethod 0·hancom2020_pdf_driver_one_up의 독립 인쇄 PDF다.
`generate_controls.py --print-dir <독립 Print 폴더> --output <별도 출력 폴더>`는 pikepdf로
활성 clip 누락/Q 복원/부분 범위·열린 외곽·Form 이동·명시적인 공백 text-show를 대조한다.
PDF container ID가 달라질 수 있으므로 아래 hash는 검증한 실제 입력 바이트다.

셀 수는 원본 생성 API의 독립 계약으로 정한다. local-columns/outer-only/l-shape/form은
별도 4셀, linear-columns/rows는 1행/1열의 2셀, merged-blank/space-cell은 위 두 칸 병합과
아래 두 칸의 3셀이다. 마지막 입력은 공백 U+0020 하나가 실제 Paragraph에 있어야 한다.
빈 병합 행·숨긴 변의 빈 셀·죽거나 누락/불완전한 clip·열린 외곽은 근거 부족으로 보류한다.
출력 선이 없다는 사실만으로 셀을 합치거나 빈 셀을 발명하지 않는다.

`independent-roots.pdf`는 원문 글자·그림/Form·주석·metadata·marked content를 제거하고
geometry에 C/L+1/2로 만든 label 37개를 넣은 별도 입력이다. 폰트 자원은 위 정상 Print의
Arial이다. `independent-roots-origin.json`에 생성 절차·입력 hash·독립 주소를 기록한다.
정상 상단 18셀은 실패한 다른 paint family와 독립적으로 편집 IR에 남아야 하며, 아래쪽을
포함한 label 전부가 정확히 한 번 보존되어야 한다. 아래쪽 미지원 후보의 보류를 영구 기대값으로
고정하지 않는다. 원래 자료나 별도 폰트 바이너리는 배포하지 않는다.

## 등록 전 시각 검증

각 입력의 PDF 직접 열기와 HWP/HWPX 저장 재열기를 전쪽 Native/fresh Docker WASM으로
대조했다(각 backend 48쪽). 같은 배율 확대 판독에서 경계의 존재/부재·문자 간격·셀 소유를
확인했다. 아래는 세 형식 중 최저 raw 2px 실루엣 일치율이다. 절대 px·SVG hash 대신 실제
주소/span, 내용 순서·유일 소유·셀 포함과 문자/행열 편집·양 저장·재열기를 검사한다.

| 입력 | bytes | SHA-256 | Native 최저 | fresh WASM 최저 |
|---|---:|---|---:|---:|
| `empty-merged-row.pdf` | 15791 | `6c420fe82e9e9df1180419395ec1f209f49c0a01be07a27e6f702d738178649e` | 100.00000% | 100.00000% |
| `hidden-blank.pdf` | 15274 | `73160ae3f2a3a1825386ecaa2f844d3d49c4578b7f2abfb2a861c8b9850052e6` | 100.00000% | 100.00000% |
| `independent-roots.pdf` | 62646 | `998c2e20d074eca08f0596200bcdde70b28d42871a07e90eaaedd3d7fce0a1d2` | 100.00000% | 100.00000% |
| `linear-columns.pdf` | 15201 | `6062602f816a7c944002dd12951d2eb14c2c214fbea53ebecf51078ec8f1083a` | 100.00000% | 100.00000% |
| `linear-rows.pdf` | 15184 | `49c2c41ac89c0054a973f8650fc332d3ebf1988cd4e58ccc9235443dbd7626e3` | 100.00000% | 100.00000% |
| `local-blank.pdf` | 15267 | `3558a5f0480e6aaf59bc327772efa0e8b4062c989aa4482896cc253e2e182d6d` | 100.00000% | 100.00000% |
| `local-columns.pdf` | 15290 | `c33bf0d9d08718bb1048bcda6354fb699debfcde3bbe70d47f8df7b9280de429` | 100.00000% | 100.00000% |
| `local-form.pdf` | 15027 | `3b9692061e204424ece7ec3d308d02702c7a6c4a39a4e8549cc66c0d70f57332` | 100.00000% | 100.00000% |
| `local-l-shape.pdf` | 15299 | `edf5efee16bd5491737ea434c8e27df8179a8ee29805489ae7cff562014a2ada` | 100.00000% | 100.00000% |
| `local-missing-clip.pdf` | 14806 | `0813a44d760a3349cf5d11fe60f8993f8adb81b7233528239dc2669548993186` | 100.00000% | 100.00000% |
| `local-open-exterior.pdf` | 14808 | `80f8854ac39da3cd30eb17662f0e37a2ad0acb0b85ef8d6411d64e61500954e7` | 100.00000% | 100.00000% |
| `local-outer-only.pdf` | 15281 | `d57ab53c860ddff97cdea10103d8cddb3803251fea3995ded6997a0450fa113a` | 100.00000% | 100.00000% |
| `local-partial-clip.pdf` | 14808 | `6bc08457447a6f27644f594b272bb0f9a769acd8c485fcd0de68f177c5644862` | 100.00000% | 100.00000% |
| `local-restored-clip.pdf` | 14806 | `3388ee39b8b365e267ca367928d8cb7dd2cf5066117015935886c9bd09365c6e` | 100.00000% | 100.00000% |
| `merged-blank.pdf` | 17924 | `edcc46ed240203264c3960de0bafbdd45b2f903627ee0a2f31b27585be34dd6b` | 100.00000% | 100.00000% |
| `space-cell.pdf` | 17496 | `5fda6d7c17c88e772b7fb46a142d86ccb2a2589f2d18d8659412112e5a23e057` | 100.00000% | 100.00000% |

기존 B18 `unwitnessed-cell`은 사방의 실제 경계가 확인되는 빈 단일 tile다. 새 자격에서는
정상 3셀로 복원하므로 기존 보류 기대를 수정했다. 선 부재로 빈 병합 영역을 추측하는 반례는
그대로 보류한다. 기대값 완화나 golden 교체로 회귀를 숨긴 변경이 아니다.

제품 manifest·실제 producer·명령·TSV·PNG·수정 전 FAIL/후 PASS·직접 판정 범위는
[stage9](../../mydocs/working/task_m100_7998_stage9.md)에 연결한다. Native 검증 파일 생성 시
HWPX 재열기 뒤 저수준 HWP writer를 사용한 실수와 실제 어댑터 경로의 재생성은 구분한다.
PDF→HWP/HWPX의 일반 지역 격자 전체, source paint 일반 편집 및 P5의 최종 제출 검증까지
완료했다고 주장하지 않는다.

2026-10-19 메인테이너가 같은 제품과 비교 화면에 **시각 판정 통과**를 내렸다.
실제 Native/Studio API의 편집·양 저장 구조 검증과 함께 적용하며 P4 전체 완료를 뜻하지 않는다.


B20 후속 절편은 같은 `independent-roots.pdf`를 재사용하여 하단 19셀도 실제 편집 IR로
연결했다. 이전의 보류를 영구 기대값으로 고정하지 않았으며, 새 경계 소유·독립 출력·전후
검사 범위는 [paint 소유 증거](paint-ownership.md)와 [stage10](../../mydocs/working/task_m100_7998_stage10.md)을 따른다.
