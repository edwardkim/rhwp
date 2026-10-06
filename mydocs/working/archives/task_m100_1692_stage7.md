# Task #1692 Stage 7 - SO-SUEOP HWP3 외부 제목 이미지 복원

## 시작 상태

- 직전 커밋: `8d49615c3 task 1692: SO-SUEOP 머리말과 HMapsi OLE 보정`
- Stage 6에서 HWPX p1 HMapsi OLE placeholder는 preview clip fallback으로 제거됐다.
- Stage 6에서 HWP/HWPX p5 머리말 분산과 꼬리말 페이지 번호는 기준 PDF와 맞췄다.

## 남은 문제

- `samples/SO-SUEOP.hwp` p1 왼쪽 세로 제목은 여전히 placeholder로 보인다.
- 실측 결과 HWP3 p1 제목 그림은 내부 payload 없이 외부 OLE/link 이름만 가진다.
  - `type=1`
  - `name="00000000.OOO"`
  - `n_ext=0`
  - BinData: `Link (ID: 1, ext: OOO, loaded: 0 bytes)`

## 처리 계획

- HWP3 파서 종료 시에도 공통 Link BinData -> `Picture.external_path` 보강을 적용할 수 있는지 확인한다.
- `samples/00000000.OOO` 외부 샘플 이미지를 추가해 `populate_external_images_from_dir(samples)` 경로에서 p1 제목이 로드되는지 검증한다.
- 같은 커밋에는 이 stage 문서 하나만 포함한다.

## 처리 내용

- HWP3 파서 종료 시 `populate_link_image_paths()`를 호출해 Link BinData 기반 그림도 `external_path`를 보존하도록 했다.
- `samples/00000000.OOO`를 추가해 `samples/SO-SUEOP.hwp`의 p1 외부 제목 이미지를 로드할 수 있게 했다.
- SO-SUEOP HWP3 원본의 p1 `협성고등학교` 위치가 HWPX 기준보다 아래로 밀리는 문제를 보정했다.
  - 모든 HWP3에 기존 spacing_before 복원 경로를 강제하지 않고, HWP3 변환본 성격의 문서에만 유지한다.
- HWP3 `obj_type=1` 글상자를 `Shape`로 변환하지 않고 1x1 `Table` IR로 보존하도록 바꿨다.
  - SO-SUEOP p22 관계도 상자의 `TopAndBottom` 자리차지 흐름이 유지되어 본문 겹침이 사라졌다.
- 회귀 테스트를 추가했다.
  - p1 외부 제목 이미지 로드
  - p1 학교명 y 좌표 HWP/HWPX 정합
  - p22 관계도 상자 Table 흐름과 후속 문항 y 좌표 정합

## 검증

- `env CARGO_INCREMENTAL=0 cargo build -q --bin rhwp`
- `env CARGO_INCREMENTAL=0 cargo test -q --test issue_1692 -- --nocapture`
  - 10개 테스트 통과
- `git diff --check`
  - 통과
- SO-SUEOP 전체 시각 검증
  - 기준 PDF: `pdf/SO-SUEOP-2024.pdf`
  - 산출물: `tmp/stage7-visual-final/`
  - 페이지 수: PDF/HWP/HWPX 모두 46쪽
  - contact sheet: `tmp/stage7-visual-final/contact_1.png` ~ `contact_4.png`
  - 상세 확인: p5, p6, p22, p24, p29

## 확인 결과

- p1 HWP3 제목 이미지는 placeholder가 아니라 실제 이미지로 렌더된다.
- p5는 PDF/HWP/HWPX 모두 같은 페이지 흐름으로 맞는다.
- p22는 관계도 상자와 본문이 더 이상 겹치지 않고, 후속 문항 y 좌표가 HWPX와 정합한다.
- HWP 렌더 중 p6/p24/p29에서 몇 px 단위 overflow 로그가 남지만, 시각 확인상 PDF/HWPX와 같은 하단 밀집 흐름이며 눈에 띄는 잘림은 보이지 않는다.
- `issue_1116`의 `sample16_hwp3_page3_heading_positions_follow_hancom_grid`는 기존과 같은 실패를 재현한다.
  - 실패 위치: `2. 추진방향` y가 기대값 약 337.2px 대신 378.9px로 측정됨.
  - 이번 SO-SUEOP 보정 전에도 재현되던 HWP3 sample16 격자 정합 문제로 별도 후속 대상이다.

## 남은 사항

- SO-SUEOP p22 관계도 내부 연결선/번호 표현은 HWP3 원본 파싱에서 PDF/HWPX만큼 풍부하게 복원되지는 않는다.
  - 이번 stage에서는 페이지 흐름, 글색상, 외부 이미지, 자리차지 겹침을 우선 정리했다.
  - 도식 내부 선/번호 완전 복원은 HWP3 글상자/도식 세부 파싱 후속 과제로 분리하는 것이 안전하다.
