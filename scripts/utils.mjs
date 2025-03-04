import 'zx/globals';
import { parse as parseToml } from '@iarna/toml';

$.verbose = true;
process.env.FORCE_COLOR = 3;
process.env.CARGO_TERM_COLOR = 'always';

export const workingDirectory = (await $`pwd`.quiet()).toString().trim();

export function cliArguments() {
  return process.argv.slice(3);
}

export function getAllProgramIdls() {
  return getAllProgramFolders().map((folder) =>
    path.join(workingDirectory, folder, 'idl.json')
  );
}

export function getExternalProgramOutputDir() {
  const config = getCargoMetadata()?.solana?.['external-programs-output'];
  return path.join(workingDirectory, config ?? 'target/deploy');
}

export function getExternalProgramAddresses() {
  const addresses = getProgramFolders().flatMap(
    (folder) => getCargoMetadata(folder)?.solana?.['program-dependencies'] ?? []
  );
  return addresses;
}
