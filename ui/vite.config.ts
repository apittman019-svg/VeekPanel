import {writeFileSync} from 'node:fs';
import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
export default defineConfig({plugins:[svelte(),{name:'bundled-notices',generateBundle(_options,bundle){writeFileSync(new URL('./bundled-modules.json',import.meta.url),JSON.stringify(Object.values(bundle).flatMap(b=>b.type==='chunk'?Object.keys(b.modules).filter(id=>b.modules[id].renderedLength>0):[])));}}],clearScreen:false,server:{strictPort:true},build:{target:'es2022'}});
