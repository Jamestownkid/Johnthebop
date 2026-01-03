// main.js - where svelte gets mounted
// nothing fancy here, just boilerplate

import './styles/global.css'
import App from './App.svelte'

const app = new App({
  target: document.getElementById('app'),
})

export default app
