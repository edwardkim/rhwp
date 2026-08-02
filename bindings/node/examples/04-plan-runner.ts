/*
 * 계획 실행 — 검사하고, 통과하면 원자적으로 적용한다.
 *
 * 여러 편집 중 하나라도 불가능하면 아무것도 저장하지 않는다. 편집을 한 줄씩
 * 흘려보내면 "3번째까지만 적용된 문서"가 남는데, 그건 원본도 결과물도 아니다.
 *
 *   npx tsx examples/04-plan-runner.ts 서식.hwp 제출본.hwp
 */

import process from 'node:process';

import * as rhwp from '../src/index.js';

async function main(source: string, target: string): Promise<number> {
  const available = (await rhwp.fields(source))
    .children('fields')
    .map((f) => f.get<string>('name'));
  if (available.length === 0) {
    console.log('누름틀이 없는 문서입니다.');
    return 1;
  }

  const data: Record<string, string> = {};
  for (const name of [...available].sort()) {
    data[name] = '계획으로 입력';
  }

  const plan = new rhwp.Plan(source, target)
    .fillFields(data)
    // 이름이 하나라도 문서에 없으면 실패로 친다. 기본값은 "있는 것만 채우기"인데,
    // 서식 제출처럼 빠진 칸이 곧 반려인 경우엔 조용한 성공이 더 위험하다.
    .requireAllFieldsFound(true)
    .verify(true);

  // 1) 검사 — 디스크를 건드리지 않는다. 위반은 **예외가 아니라 결과**다.
  //    빌더는 문법만 본다(좌표가 0 이상 정수인지 등). 문서와 대조하는 것은 여기서 일어난다.
  //
  //    다만 `check()` **자체**는 던질 수 있다. rhwp 가 계획 `--dry-run` 을 모르는
  //    버전이면(#3759 이전) 바인딩이 예외로 끊는다 — 계획서의 `dryRun` 을 rhwp 가
  //    조용히 무시하면 "검사"인 줄 알고 부른 호출이 문서를 **편집·저장**해 버리기
  //    때문이다. 그래서 이 catch 는 예외를 삼키고 `run()` 으로 내려가지 않는다.
  //    검사할 수 없었다는 사실이 곧 "실행하면 안 된다"는 결론이다.
  let checked: rhwp.PlanResult;
  try {
    checked = await plan.check();
  } catch (error) {
    if (!(error instanceof rhwp.RhwpError)) throw error;
    console.error('계획을 검사하지 못했습니다 — 실행으로 대체하지 않습니다.');
    console.error(`  ${error.message}`);
    console.error('  계획서를 눈으로 검토하려면: plan.toJSON({ dryRun: true })');
    // 도구도 문서도 정상이다. 이 rhwp 로는 검사 자체가 불가능한 것이므로 런타임 실패(1)다.
    return 1;
  }

  if (!checked.ok) {
    console.log('계획에 문제가 있습니다:');
    console.log(checked.describeViolations());
    // 계획 선검증 위반은 호출을 조립한 쪽의 버그다 — 도구 사전에서 exit 2.
    return 2;
  }

  // preview 는 봉투 배열이다 — 표시용이라 필드를 좁히지 않고 원문 그대로 찍는다.
  console.log(`검사 통과 (디스크 무변경: ${checked.isDryRun}). 실행 예정:`);
  for (const step of checked.preview) {
    console.log(`  ${JSON.stringify(step.raw)}`);
  }

  // 2) 실행 — 검사에서 본 것과 같은 계획을 그대로 적용한다.
  const applied = await plan.run();
  console.log(`\n적용한 step: ${applied.steps.length}`);

  if (!applied.ok) {
    // 여기서 ok 가 거짓이라는 것은 "검사는 통과했는데 적용 결과가 단언을 어겼다"는 뜻이다.
    // 도구가 고장난 게 아니라 문서에 대한 단언이 틀린 것이므로 exit 3.
    console.log('검증 실패:');
    console.log(applied.describeViolations());
    return 3;
  }

  console.log('검증 통과 — 저장본이 계획과 같습니다.');
  return 0;
}

const argv = process.argv.slice(2);
const [source, target] = argv;
if (argv.length !== 2 || source === undefined || target === undefined) {
  console.error('사용법: npx tsx examples/04-plan-runner.ts 서식.hwp 제출본.hwp');
  process.exit(2);
}

process.exit(await main(source, target));
