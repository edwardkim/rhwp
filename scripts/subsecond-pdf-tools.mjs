import { spawnSync } from 'node:child_process';

export function subsecondPdfToolSpecs(platform = process.platform) {
  return {
    ghostscript: {
      commands: platform === 'win32' ? ['gswin64c', 'gswin32c', 'gs'] : ['gs'],
      versionArgs: ['--version'],
    },
    pdfinfo: { commands: ['pdfinfo'], versionArgs: ['-v'] },
  };
}

export function findSubsecondPdfTool(
  tool,
  platform = process.platform,
  probe = (command, args) => spawnSync(command, args, { stdio: 'ignore' }),
) {
  const spec = subsecondPdfToolSpecs(platform)[tool];
  return spec.commands.find((command) => {
    const result = probe(command, spec.versionArgs);
    return !result.error && result.status === 0;
  }) ?? null;
}
