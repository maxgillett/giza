// Note that a dynamic `import` statement here is required due to
// webpack/webpack#6615, but in theory `import { greet } from './pkg';`
// will work here one day as well!
const rust = import('./pkg');

// Load a Cairo proof
import data from './output.bin';

rust
  .then(m => m.verify(data.data))
  .catch(console.error);


