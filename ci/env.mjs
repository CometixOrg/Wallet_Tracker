import { getSolanaVersion, getToolchain } from '../utils.mjs';

await $`echo "SOLANA_VERSION=${getSolanaVersion()}" >> $GITHUB_ENV`;
await $`echo "TOOLCHAIN_FORMAT=${getToolchain('format')}" >> $GITHUB_ENV`;
