/**
 * 설치할 `dioxus-cli` 버전을 저장소가 실제로 컴파일하는 `subsecond` 크레이트에서 유도한다.
 *
 * [#4580] 두 버전은 같아야 한다 — `dx` 가 만든 점프 테이블을 `subsecond::apply_patch` 가
 * 역직렬화하므로, 어긋나면 패치가 조용히 안 먹는다(#4578). 그래서 버전이 저장소 안에 여러 벌
 * 있었고 실제로 갈라졌다: dependabot `79461bf36` 이 `Cargo.toml` 만 올렸고 나머지는
 * `de2c2c226` 에서 사람이 손으로 맞췄다.
 *
 * 사본을 `Cargo.toml` 의 정확 핀 하나로 줄이는 대신, 이 스크립트가 그 핀과 잠금 파일이 해결한
 * 버전을 맞대 보고 갈라져 있으면 **설치하지 않고 멈춘다.** 이 층의 실패는 조용하므로, 어느
 * 쪽과도 맞지 않는 CLI 를 까느니 서는 편이 낫다.
 *
 * 사용: `node scripts/dioxus-cli-version.mjs` — 버전 한 줄을 stdout 으로 낸다.
 */
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(fileURLToPath(new URL('..', import.meta.url)));
const CRATE = 'subsecond';

/** `Cargo.toml` 이 요구하는 정확 핀(`=x.y.z`). 정확 핀이 아니면 유도 자체가 성립하지 않는다. */
function pinnedRequirement(manifest) {
  const match = new RegExp(`^\\s*${CRATE}\\s*=\\s*\\{[^}]*version\\s*=\\s*"([^"]+)"`, 'm')
    .exec(manifest);
  if (!match) {
    throw new Error(`Cargo.toml 에서 \`${CRATE}\` 의존성을 찾지 못했다`);
  }
  const requirement = match[1];
  if (!requirement.startsWith('=')) {
    throw new Error(
      `\`${CRATE}\` 가 정확 핀이 아니다(${requirement}). dx 와 버전을 맞출 수 없으므로 `
      + '`=x.y.z` 로 고정해야 한다.',
    );
  }
  return requirement.slice(1);
}

/** `Cargo.lock` 이 해결한 버전. 실제로 컴파일되는 버전이다. */
function resolvedVersion(lockfile) {
  const versions = [...lockfile.matchAll(/^\[\[package\]\]\nname = "([^"]+)"\nversion = "([^"]+)"/gm)]
    .filter(([, name]) => name === CRATE)
    .map(([, , version]) => version);
  if (versions.length !== 1) {
    throw new Error(
      `Cargo.lock 에서 \`${CRATE}\` 를 정확히 한 번 찾지 못했다(${versions.length}건)`,
    );
  }
  return versions[0];
}

/**
 * 설치할 dioxus-cli 버전.
 *
 * @throws 매니페스트의 핀과 잠금 파일이 갈라져 있으면 — 그대로 설치하면 어느 쪽과도 맞지 않는
 *   CLI 가 깔린다.
 */
export function dioxusCliVersion(root = ROOT) {
  const pinned = pinnedRequirement(readFileSync(path.join(root, 'Cargo.toml'), 'utf8'));
  const resolved = resolvedVersion(readFileSync(path.join(root, 'Cargo.lock'), 'utf8'));
  if (pinned !== resolved) {
    throw new Error(
      `\`${CRATE}\` 의 정확 핀(Cargo.toml: ${pinned})과 해결된 버전(Cargo.lock: ${resolved})이 `
      + '다르다. `cargo update -p subsecond` 로 잠금 파일을 맞춘 뒤 다시 실행한다.',
    );
  }
  return resolved;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  process.stdout.write(`${dioxusCliVersion()}\n`);
}
