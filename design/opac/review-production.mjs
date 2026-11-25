import { createServer } from 'node:http';
import { readFile, mkdir, writeFile } from 'node:fs/promises';
import { browserReview } from './browser.mjs';
const root = new URL('../../', import.meta.url);
const source = path => readFile(new URL(path, root), 'utf8');
const binary = async path => (await readFile(new URL(path, root))).toString('base64');
const css = (await source('src-tauri/src/services/opac/styles.css'))
  .replace('{{SANS_FONT}}', await binary('node_modules/@fontsource-variable/hanken-grotesk/files/hanken-grotesk-latin-wght-normal.woff2'))
  .replace('{{MONO_FONT}}', await binary('node_modules/@fontsource/courier-prime/files/courier-prime-latin-400-normal.woff2'));
const html = (await source('src-tauri/src/services/opac/index.html'))
  .replace('{{STYLES}}', css).replace('{{SCRIPT}}', await source('src-tauri/src/services/opac/catalogue.js'))
  .replaceAll('{{LOGO}}', await binary('src/assets/pusthakam-logo.png')).replaceAll('{{LIBRARY_NAME}}', 'Town Public Library');
// Contract fixtures only. Rust HTTP and database tests independently verify the real API.
const fixtures = Array.from({length:25}, (_, i) => ({id:i+1,title:i===19?'<img src=x onerror="window.injected=true"> A book':(i===0?"Balyakalasakhi":`Catalogue title ${i+1}`),subtitle:i===1?'A'.repeat(160):null,authors:[i===0?'Basheer':'Library Author'],year:2000+i,publisher:'Library Press',isbn:String(9780000000000n+BigInt(i)),classification:i<22?'823.9':'005.1',subjects:['Literature'],availableCopies:i%2?0:1,totalCopies:i===1?0:2,locations:['Main Library — '+ 'Shelf'.repeat(i===1?30:1)]}));
const coverPng = await readFile(new URL('src/assets/pusthakam-logo.png', root));
fixtures[2].isbn='invalid';
let mode='ok'; let lastQuery;
const server = createServer(async (req,res) => {
  const url = new URL(req.url,'http://127.0.0.1');
  if (url.pathname==='/opac') {res.setHeader('Content-Type','text/html; charset=utf-8');res.end(html);return;}
  if(url.pathname.startsWith('/api/opac/cover/')) {
    const id=Number(url.pathname.split('/').at(-1));
    if(id===2||id===3){res.writeHead(404);res.end();return;}
    res.setHeader('Content-Type','image/png');res.end(coverPng);return;
  }
  if(url.pathname==='/api/opac/search') {
    lastQuery=url.searchParams;
    if(mode==='error'){res.writeHead(503,{'Content-Type':'application/json'});res.end('{}');return;}
    if(mode==='disabled'){res.writeHead(403,{'Content-Type':'application/json'});res.end('{}');return;}
    const q=url.searchParams.get('q')||'';
    const items=fixtures.filter(b=>(!q||`${b.title} ${b.authors.join(' ')} ${b.isbn}`.toLowerCase().includes(q.toLowerCase()))&&(!url.searchParams.get('class')||b.classification.startsWith(url.searchParams.get('class')))&&(url.searchParams.get('available')!=='true'||b.availableCopies>0));
    const page=Number(url.searchParams.get('page')||1);const data={items:items.slice((page-1)*20,page*20),total:items.length,page,pageSize:20};
    const delay=q==='slow'?300:0;
    setTimeout(()=>{res.setHeader('Content-Type','application/json');res.end(JSON.stringify(data));},delay);return;
  }
  res.writeHead(404);res.end();
});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const output=new URL('./previews/',import.meta.url);await mkdir(output,{recursive:true});
const checks=[];
try {
  await browserReview(async ({send,evaluate,waitFor,assert,pause,exceptions,requests})=>{
    await send('Network.setBlockedURLs',{urls:['https://covers.openlibrary.org/*']});
    const url=`http://127.0.0.1:${server.address().port}/opac`;
    await send('Emulation.setDeviceMetricsOverride',{width:1366,height:950,deviceScaleFactor:1,mobile:false});
    await send('Page.navigate',{url});await waitFor('document.querySelectorAll(".record").length===20');
    await evaluate('document.fonts.ready.then(()=>true)');
    for(const [width,height] of [[1366,950],[1024,768],[768,1024],[640,900],[390,844],[320,800]]) {
      await send('Emulation.setDeviceMetricsOverride',{width,height,deviceScaleFactor:1,mobile:width<=640});await pause(70);
      await assert('document.documentElement.scrollWidth<=innerWidth',`production overflow at ${width}`);
      await assert('document.querySelector("#q").getBoundingClientRect().width>100',`search at ${width}`);
      checks.push(`layout ${width}`);
      if(width===1366||width===390){const shot=await send('Page.captureScreenshot',{format:'png'});await writeFile(new URL(`swiss-production-${width}.png`,output),Buffer.from(shot.data,'base64'));}
    }
    await assert('!window.injected && !Array.from(document.querySelectorAll("#results img")).find(img=>img.getAttribute("src")==="x") && Array.from(document.querySelectorAll(".title-button")).some(b=>b.textContent.includes("<img"))','catalogue markup treated as text');checks.push('HTML injection');
    await waitFor('document.querySelector(".book-cover img").complete && document.querySelector(".book-cover img").naturalWidth>0');
    await assert('!document.querySelector(".book-cover img").hidden','local saved cover visible');checks.push('saved cover');
    await evaluate('document.querySelectorAll(".record")[1].scrollIntoView()');
    await waitFor('document.querySelectorAll(".record")[1].querySelector("img").src.startsWith("https://covers.openlibrary.org/")');
    await waitFor('document.querySelectorAll(".record")[1].querySelector("img").hidden');
    checks.push('automatic ISBN fallback and unavailable-provider placeholder');
    await evaluate('document.querySelectorAll(".record")[2].scrollIntoView()');
    await waitFor('document.querySelectorAll(".record")[2].querySelector("img").hidden');
    await assert('document.querySelectorAll(".record")[2].querySelector("img").src.includes("/api/opac/cover/")','invalid ISBN never sent to provider');checks.push('invalid ISBN placeholder');
    await evaluate('document.querySelector(".title-button").focus()');
    await send('Input.dispatchKeyEvent',{type:'keyDown',key:'Enter',code:'Enter',windowsVirtualKeyCode:13,text:'\r',unmodifiedText:'\r'});
    await send('Input.dispatchKeyEvent',{type:'keyUp',key:'Enter',code:'Enter',windowsVirtualKeyCode:13});
    await assert('document.querySelector("#details").open && document.querySelector("#detail-title").textContent==="Balyakalasakhi"','keyboard detail dialog');
    await send('Input.dispatchKeyEvent',{type:'keyDown',key:'Escape',code:'Escape',windowsVirtualKeyCode:27});
    await send('Input.dispatchKeyEvent',{type:'keyUp',key:'Escape',code:'Escape',windowsVirtualKeyCode:27});
    await assert('!document.querySelector("#details").open && document.activeElement.className==="title-button"','Escape and focus restoration');checks.push('keyboard dialog and focus');
    await evaluate('document.querySelector("#next").click()');await waitFor('document.querySelectorAll(".record").length===5');
    await assert('document.querySelector("#page-label").textContent==="Page 2 of 2" && document.querySelector("#next").disabled','pagination');checks.push('pagination');
    await evaluate('document.querySelector("[data-class="+CSS.escape("0")+"]").click()');await waitFor('document.querySelectorAll(".record").length===3');
    await assert('document.querySelector("#summary").textContent==="3 titles found"','classification resets page');checks.push('classification');
    await evaluate('document.querySelector("#available").click()');await waitFor('document.querySelectorAll(".record").length===2');
    if(lastQuery.get('available')!=='true'||lastQuery.get('class')!=='0')throw Error('Filters not sent to API');checks.push('availability');
    const search=async term=>{await evaluate(`document.querySelector('#q').value=${JSON.stringify(term)};document.querySelector('#search-form').requestSubmit()`);};
    await evaluate('document.querySelector("#reset").click()');await waitFor('document.querySelectorAll(".record").length===20');
    await search('Basheer');await waitFor('document.querySelectorAll(".record").length===1');checks.push('author search');
    await search('absent');await waitFor('!!document.querySelector(".empty")');checks.push('empty state');
    mode='error';await search('');await waitFor('!document.querySelector("#error").hidden');
    await assert('document.querySelectorAll(".record").length===0 && !document.querySelector("#retry").hidden','error clears stale results');
    mode='ok';await evaluate('document.querySelector("#retry").click()');await waitFor('document.querySelectorAll(".record").length===20');checks.push('error and retry');
    mode='disabled';await search('');await waitFor('document.querySelector("#error").textContent.includes("disabled")');mode='ok';checks.push('disabled OPAC');
    await search('slow');await search('Basheer');await waitFor('document.querySelectorAll(".record").length===1');await pause(400);
    await assert('document.querySelectorAll(".record").length===1 && document.querySelector(".record").textContent.includes("Basheer")','late response isolation');checks.push('stale response');
    await send('Emulation.setEmulatedMedia',{features:[{name:'prefers-reduced-motion',value:'reduce'}]});
    await assert('getComputedStyle(document.querySelector(".record")).animationName==="none"','reduced motion');checks.push('reduced motion');
    const external=requests.filter(value=>/^https?:/.test(value)&&new URL(value).hostname!=='127.0.0.1');
    const unexpected=external.filter(value=>!/^https:\/\/covers\.openlibrary\.org\/b\/isbn\/(?:\d{13}|\d{9}[\dXx])-M\.jpg\?default=false$/.test(value));
    if(exceptions.length||unexpected.length)throw Error(JSON.stringify({exceptions,unexpected}));
    await writeFile(new URL('production-review-results.json',output),JSON.stringify({at:new Date().toISOString(),data:'public DTO contract fixtures; real database/HTTP covered by Rust tests',checks,exceptions,external,coverNetwork:'Open Library requests blocked in browser; no real provider access during tests'},null,2)+'\n');
    console.log(JSON.stringify({checks:checks.length,exceptions:exceptions.length,externalRequests:external.length}));
  });
} finally {server.closeAllConnections();server.close();}
