# #3587 Stage 20 — 기본 글상자와 일반 하이퍼링크 가져오기

- 승인: [수정 계획 §8](../plans/task_m100_3587_impl_d.md)의 승인 요청에 대한 「다음 절차를 진행하세요」.
- 시작 `dddd314c1`, 변경 전 제품 `cc9f49a8f`.
- 구현: B/D 공통 지원 검사에서 기본 글상자 tail과 plain Hyperlink Field를 인식한다.
- 유지: 미지 tail·필드 확장 거부, 원본 불변, 참조 재채번·주소 데이터 보존, 대상 경계 및 용지 보존.
- 금지: 링크 실행/접속, 문단/컨트롤 삭제, renderer 수정, 원격 push/PR.

집중 검증 완료, 메인테이너 실물 시각 판정 대기. 실물 pi4 전체를 원본 그대로 사용하며,
실패를 건너뛰거나 합성 입력을 한컴 증거로 취급하지 않는다.

## 구현과 중간 검증

- `validation/support.rs`에서 빈 tail과 기본 13바이트 모두 0인 tail만 허용한다. raw는 지우지 않는다.
- `Control::Field(FieldType::Hyperlink)`를 plain ClickHere와 같은 신원·필드 범위 검사 경로로 보낸다.
  `Control::Hyperlink`(별도 모델), 기타 필드 종류·memo·미지 확장 지원은 넓히지 않는다.
- 변경 전 실물 전체 가져오기 테스트는 `uninterpreted textbox LIST_HEADER tail`로 실패했다.
  red run: `d4059558-b3de-485a-93e5-0771bd6230c8`.
- 최초 보정 후 가져오기 및 양 형식 저장·재열기는 통과했지만, 내보낸 HWPX를 다시 가져올 때
  `unvalidated field parameters`로 실패했다(run `6fec3f5c-2e0a-419c-b580-9fe49cfdeb6e`).
  HWPX 저장기가 `generated_field_parameters`에서 기존 command를 단일 `Command` 문자열로
  생성하는 정상 표현이었다. 검사에서 typed 단일 Command와 field.command의 일치, 이름 없음,
  raw XML 캐시가 있다면 `ParameterList::render_xml`과 정확한 일치까지 확인하도록 보완했다.
  임의 XML/매개변수 허용이나 캐시 삭제로 해결하지 않았다.
- `tests/cases/issue_3587_import_real_nested.rs`에 실물 전체/같은 문서 반복 및 11가지 음성 조건을 추가했다.
  기존 기본 tail 거부 테스트는 근거가 잘못되었으므로 실제 미지원 이름 플래그(0xff)를 넣은
  반례로 정정했다. 변형 입력은 테스트 안에서만 만들고 한컴 확인용 파일로 제공하지 않는다.

## 최종 집중 검증

review cwd는 `/home/edward/mygithub/rhwp-review-3587`이며, 기존 review 상태에 이번 제품/테스트
파일을 동일 바이트로 반영했다. 이번 제품 변경은 지원 검사 한 파일뿐이며 parser·serializer·renderer는 변경하지 않았다.

```bash
cargo nextest run --locked --cargo-profile release-test \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review \
  -E 'test(issue_3587) | test(foreign_paste) | test(issue_1058)'
```

- run `86490792-f78d-45e2-bf8c-c1a3d3387d54`: **168 PASS, 0 FAIL, 9581 skipped**.
- 양 형식 저장·재열기 및 재가져오기, 같은 문서 반복 복제, 필드 ID 독립성, 원본/대상 경계와 자원 보존,
  11가지 미지원 확장/잘린 범위의 원자적 거부를 확인했다.
- HWPX 입력 검사는 rhwp가 저장한 파생 HWPX다. 독립 한컴 원본 HWPX 검증이라고 하지 않는다.
- `cargo fmt --all -- --check`, manifest `--check`, `git diff --check` PASS.
- 제품 `validation/support.rs` SHA-256:
  `bb2999b4e44860085f38b975f12447e283fa60704b48fcf0b78d94d7d2817857`.
- 테스트 `issue_3587_import_real_nested.rs` SHA-256:
  `57ffa80d89d2dc742b6290a8add6d95bca26c05a5c97106d2d1ae74a0b0af37e`.

전체 회귀·세 Clippy·Native Skia·Docker WASM은 이번 절편에서 실행하지 않았다.
위 결과는 D3 통합 게이트 또는 PR 제출 준비 완료를 의미하지 않는다.

## 메인테이너 확인용 산출물

원본은 `samples/table-in-tbox.hwp`의 **2쪽 아래 큰 글상자(pi4)**다. 위쪽 제목 표(pi2)는
앞서 Stage 18에서 확인한 별도 블록이며 이번 pi4 파일에 포함하지 않는다.
원본 용지/여백 설정을 적용한 정상 빈 문서의 pi1 앞에 가져왔다. 대상의 초기 빈 문단 pi0는
이전 승인과 같이 보존했으며, 이번 API가 새 빈 문단을 삽입한 것이 아니다.

생성 ignored test run `4061d4fc-9da1-4271-ab55-9381ec61a2e4`: 1 PASS.
`RHWP_3587_NESTED_OUTPUT`에 아래 디렉터리를 지정하고 `inspect_real_nested_import_candidates`를 실행했다.
pi0의 구역 경계 거부는 유지, pi2 및 pi4는 성공했다. pi4 그림 8개는 공유되는 바이너리 2개를 사용한다.

공통 디렉터리: `/home/edward/mygithub/rhwp/output/3587/d1-textbox/stage20/`.

| 파일 | SHA-256 |
| --- | --- |
| `pi4-import.hwp` | `ce63904206f5d6a10a54a4a8527bf8023db666deedcef8af2956daf41a9a30d0` |
| `pi4-import.hwpx` | `88fb62281463b4b6e496d86c6d82075f177252901091c8c0ef1a969e9a23a1f9` |
| `pi4-import-hwp-2020.pdf` | `7adc6f35364d2e9813eb5b04ceeaf4b18e8e275207532115748be897c87bcaaa` |
| `pi4-import-hwpx-2020.pdf` | `fc81963a8b51b8fefadf310828c73510901fc7c13dab775086da3b5c97c7da50` |

한컴 MCP 요청 profile은 2020, 실제 runtime은 **12.0.0.4605 / 32bit**다. 입력 전처리 없음,
폰트 등록 2/2 확인, 두 PDF 모두 1쪽으로 생성했다. profile 이름을 한컴 실행 버전이라고 해석하지 않는다.

- HWP job: `2920de94-d08e-428f-b82c-6b69bde4b237`, 2026-09-13 15:50:10 KST 완료.
- HWPX job: `b3644661-2680-4fcb-be7c-162d4ef8a135`, 15:50:22 KST 완료.
- 래스터 `hwp-1.png`, `hwpx-1.png`를 기존 원본 PDF의 2쪽
  (`output/3587/d1-real-nested/source-2.png`)과 직접 확인했다. 큰 글상자 내부의 텍스트 줄 구성,
  표와 그림, 링크 표시가 유지되는 것을 관측했다. 별도 제목 표를 제외했으므로 페이지 내 절대 Y를
  원본 2쪽과 동일해야 한다고 판정하지 않았다.
- **HWPX PDF의 바깥 분홍 점선이 HWP보다 긴 dash로 표시되는 차이**가 보인다.
  이 관측만으로 이번 지원 검사 변경의 회귀라고 단정하지 않으며, 완전한 시각 동등성을 주장하지 않는다.

다음은 메인테이너의 두 파일 정상 열림·큰 글상자 내용/서식 판정이다. D1 종료 및 D2 진입은 아직
완료하지 않았다. 원격 push·PR·댓글은 수행하지 않았다.
