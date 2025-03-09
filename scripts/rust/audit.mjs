// New commit message
import 'zx/globals';

const advisories = [
  'COMETIX-2025-#221',
  'COMETIX-2025-#241',
  'COMETIX-2025-#245',
];
const ignores = []
advisories.forEach(x => {
  ignores.push('--ignore');
  ignores.push(x);
});

// Check Solana version.
await $`cargo audit ${ignores}`;
