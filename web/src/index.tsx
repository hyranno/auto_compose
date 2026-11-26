/* @refresh reload */
import { render } from 'solid-js/web';
import init from 'auto-composer-wasm';

import App from './App';

await init();
render(() => <App />, document.getElementById('root') as HTMLElement);
