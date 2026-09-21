export const Backups = { ofVolumes: () => ({ withPgDump: (pg?:unknown) => ({ withMysqlDump: (mysql?:unknown) => ({ addSync: (sync?:unknown) => ({pg,mysql,sync}) }) }) }) };
