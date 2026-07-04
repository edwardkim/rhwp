import type { CommandDef } from '../types';
import { OptionsDialog } from '../../ui/options-dialog';

export const toolCommands: CommandDef[] = [
  {
    id: 'tool:options',
    label: '환경 설정',
    execute(services) {
      const dlg = new OptionsDialog(services.eventBus);
      dlg.show();
    },
  },
];
