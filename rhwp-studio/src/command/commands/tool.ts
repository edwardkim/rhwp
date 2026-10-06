import type { CommandDef } from '../types';
import { OptionsDialog } from '../../ui/options-dialog';

import { t } from '../../i18n/index.ts';
export const toolCommands: CommandDef[] = [
  {
    id: 'tool:options',
    opensDialog: true,
    label: t('command.tool.options.label'),
    execute(services) {
      const dlg = new OptionsDialog(services.eventBus);
      dlg.show();
    },
  },
];
