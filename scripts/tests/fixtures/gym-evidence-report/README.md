# Gym evidence report test fixture

`complete/`는 #6669의 seal·HTML 변환 계약을 공개 환경에서 재현하기 위한 합성 증적이다. 실제
벤치마크 실행, 실제 Git OID, 제품 binary 또는 사설 코퍼스에서 가져온 값이 아니다.

고정 fixture는 두 개의 합성 task로 다음 정상 경계를 포함한다.

- positive 2/2 PASS
- discrimination 2/2 reject
- scorer가 오류를 반환해 음성 대조를 의도대로 거부한 항목 1건
- 다단계 trajectory load-bearing 1건
- 단일-step이어서 trajectory 대상이 아닌 N/A 1건
- 서로 다른 authority class 두 종류

FAIL, 미설명 score error, 입력 변조, `trajectory.ok=true`이지만 `trusted=false`인 경우는
`scripts/tests/test_gym_evidence_report.py`가 fixture를 임시 디렉터리에 복사한 뒤 변형한다. 원본
fixture와 저장된 PASS 샘플은 바꾸지 않는다.

저장소 루트에서 다음 명령으로 seal과 샘플을 재생성한다.

```bash
python3 gym/tools/evidence_report.py \
  --evidence-dir scripts/tests/fixtures/gym-evidence-report/complete \
  --seal
python3 gym/tools/evidence_report.py \
  --evidence-dir scripts/tests/fixtures/gym-evidence-report/complete \
  --out gym/examples/evidence-report.html
python3 -m unittest scripts.tests.test_gym_evidence_report
```

`evidence-manifest.json`과 `gym/examples/evidence-report.html`은 생성 산출물이지만 재현성 회귀를
byte-for-byte 검사하기 위해 함께 추적한다. 직접 수정하지 않는다.
