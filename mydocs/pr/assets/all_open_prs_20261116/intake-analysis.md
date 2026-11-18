# 열린 PR 7건 통합 검토의 선행 분석

2026-11-16 기준 브랜치: `review/integration-20261116-open-prs`.
검증 base: `53774cc1508591b6ac794950f0063452fbaf6e87`.
원본 fork 브랜치는 변경하지 않고 기능·검사·증적 커밋 29개를 누적 체리픽했다.
이 기록은 접수·충돌 보정 분석이며 검증 완료 또는 수용 판정이 아니다.

| 원 PR | 고정한 원 head | 검토 범위 |
| --- | --- | --- |
| [#9022](https://github.com/edwardkim/rhwp/pull/9022) | `4cf97fe5f40bde52b6e80dfc461b5111b51139b6` | 칸 안 어울림 그림의 텍스트 바닥과 사진 행 높이 |
| [#9105](https://github.com/edwardkim/rhwp/pull/9105) | `559be7a737a19a29cc091d0724a015c357cd6fd6` | 블록 가져오기의 번호·글머리표 원시 글자모양 참조 |
| [#9107](https://github.com/edwardkim/rhwp/pull/9107) | `ac181689d3799dcbe549d1a943613724be126fe2` | HWPX 1.4 일반 줄 간격의 단위 변환 |
| [#9109](https://github.com/edwardkim/rhwp/pull/9109) | `6c73bc8c5b400429aee7f7a6328a3ae5c36d8da1` | 저장 줄 없는 글상자의 위아래 배치 표 흐름 |
| [#9127](https://github.com/edwardkim/rhwp/pull/9127) | `7e5bbc0a195a667bbdb467a769aeb599905b13e9` | 고딕 대체 글꼴의 Noto Sans CJK KR 선택 |
| [#9137](https://github.com/edwardkim/rhwp/pull/9137) | `125f8c80c2eec043f604b1fc6d9815e3e3e7dcaf` | 저장 줄이 있는 글상자의 비인라인 표 높이·정렬 |
| [#9141](https://github.com/edwardkim/rhwp/pull/9141) | `aea7b6a1fadac99257e7b302bf15b5afb3c38225` | FIELD_END 뒤 제어문자의 보조 슬롯 소속 |

적용 순서는 #9105 → #9107 → #9109 → #9127 → #9137 → #9141 → #9022이다.
#9137의 history에 포함된 #9109의 공통 커밋 6개는 중복 적용하지 않았다.
각 원 PR의 reviewer `jangster77` 지정과 milestone `v1.0.0`을 API로 확인했다.
통합 PR에 저장소 owner를 자동 reviewer로 지정하지 않는다.

## 충돌 해소와 기존 기능 보존

개별 PR과 21개 쌍의 사전 시뮬레이션에서 #9022의 두 파일이 기존 devel과 충돌했다.
다른 여섯 PR의 개별 적용은 충돌하지 않았다. 실제 누적 체리픽의 보정 내용은 다음과 같다.

1. `ece28e0c76`의 `src/renderer/layout/table_layout.rs` 충돌: 새 그림 바닥 계산과
   기존 TAC 표의 float 회피를 함께 유지했다. 바닥을 먼저 적용한 뒤 기존 float 회피가
   최종 문단 원점을 소비하게 했다. 기존 계약은 `f98132897334b487b33deb3c15beea9573856211`에서 추적했다.
2. `270e40ce54`의 `src/renderer/height_measurer.rs` 충돌: 기존
   `stored_flow_fits_declared_cell` 판정을 유지하고 새 `wrap_bottom` 호출에 문단별 흐름 높이를
   전달했다. 기존 #8939 계약은 `45fe234d0e5f5f1514f08b5f0fc3b3407b06f472`에서 추적했다.

체리픽 직후 `5d7999054bd26cb0f93c3a24a8c11cb9edc2d864`의 Native 빌드는
32.076초 뒤 E0061로 실패했다. #9022의 새 재조판 호출이 현재 devel의 `line_wrap_squeeze`
인자를 누락했다. 기존 #8947의 `a531e0f9ceaae9df89ab6acf9db2ac8152536224`와 주변 호출을
대조해 `cell.line_wrap == CELL_LINE_WRAP_SQUEEZE`를 동일하게 전달한다.
이는 빌드 호환 보정이며 시각·기능 검증 통과를 의미하지 않는다.

## 독립 기준과 필요한 후속 검증

- base Native는 정확한 base에서 새로 빌드했고 exit 0, 206.484초를 확인했다.
  실행 파일 SHA-256은 `a8f9b5c0f8f0ee7c14926a743aab2e4784ebfe57ac2e2506f3c08e5237cbce19`이다.
- 원 PR의 웹한글기안기 PDF는 데스크톱 한컴 Print 검증의 기준으로 수용하지 않는다.
  입력 61개를 커밋 바이트와 대조했으며, 저장 제품에 맞는 한컴 2020·2024 Print를 확보한다.
- 첫 독립 기준은 `pdf/pr-review-all-open-20261116/cell-caption-below-single-picture-hwpx-2024.pdf`다.
  한컴 2024 `13.0.0.3901`, managed direct DLL host,
  `frame_print_to_pdf_ex_one_up`, Print method 0, preprocess none, 1쪽이다.
  job `8b019668-afae-489f-b5ec-bd30d867b41d`, PDF SHA-256
  `5e7953a8c86da70574c7c1b511b069e17a8f7a1c8f313b7751238445a5bbd870`.
  대응 HWPX SHA-256은 `ba479c93996f696ee8a5540c37d3aa449e326c922b6d6b25e10b9c754627ed48`이다.
- #9022의 기존 reviewer가 제시한 두 그림 host 반례를 반드시 재현한다. 새 작은 그림 host가
  앞의 큰 그림 장애물을 덮어쓰는지 base·후보·독립 Print로 확인하고 필요하면 공통 경로를 보정한다.
- #9105는 실제 가져오기와 반복 저장, #9141은 필드 끝·후행 제어문자의 슬롯 소속을 확인한다.
  나머지는 각 PR 주장과 그 경계의 실제 Native/fresh WASM 출력으로 확인한다.
- 회귀 검사는 최소 글꼴 환경, Visual Sweep은 원본 글꼴 환경으로 구분한다.
  보호 TSV와 편람 gate, 필수 lint·전체 회귀·실제 소비 경로는 아직 완료하지 않았다.

실행 로그와 진행 상태는 임시 `output/pr-review/all-open-pr-integration-20261116/`에 기록한다.
최종 검증과 원 PR별 review·이미지를 정식 증적으로 반영한 뒤 PR 제출 여부를 판정한다.
