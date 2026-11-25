// Actual compiled React app and real API; called by the isolated-server review.
import { mkdir, writeFile } from 'node:fs/promises';

export async function staffMobileChecks({ send, evaluate, waitFor, assert, pause, checks }) {
  const output = new URL('../design/mobile/previews/', import.meta.url);
  const geometryErrors = [];
  await mkdir(output, { recursive: true });
  await assert('!!document.querySelector(".staff-menu-toggle")', 'Rebuild the server: this binary has older staff assets.');
  const command = (command, args = {}) => evaluate(`(async()=>{
    const response=await fetch('/api/staff/commands',{method:'POST',headers:{'Content-Type':'application/json','X-CSRF-Token':document.querySelector('meta[name=pusthakam-staff-csrf]').content},body:JSON.stringify(${JSON.stringify({ command, args })})});
    const result=await response.json();if(!response.ok)throw Error(JSON.stringify(result));return result.result;
  })()`);
  const config = await command('config_overview');
  await command('catalogue_save_record', { input: { title: 'Mobile verification: a long book title for phone readers', publisher: 'Library Press', contributors: [], subjects: [] } });
  await command('members_save', { input: { name: 'Mobile Test Reader', memberNumber: 'MOBILE-001', categoryId: config.memberCategories[0].id } });
  const tap = async expression => {
    const point = await evaluate(`(()=>{const target=${expression};if(!target)throw Error('Missing touch target');target.scrollIntoView({block:'nearest'});const r=target.getBoundingClientRect();return {x:r.left+r.width/2,y:r.top+r.height/2};})()`);
    await send('Input.dispatchTouchEvent', { type: 'touchStart', touchPoints: [{ ...point, radiusX: 1, radiusY: 1, force: 1 }] });
    await send('Input.dispatchTouchEvent', { type: 'touchEnd', touchPoints: [] });
  };
  const menu = 'document.querySelector(".staff-menu-toggle")';
  const settleDialog = () => evaluate('Promise.all(document.querySelector("[role=dialog]").getAnimations().map(a=>a.finished.catch(()=>undefined))).then(()=>true)');
  const escape = async () => {
    await send('Input.dispatchKeyEvent', { type: 'keyDown', key: 'Escape', code: 'Escape', windowsVirtualKeyCode: 27 });
    await send('Input.dispatchKeyEvent', { type: 'keyUp', key: 'Escape', code: 'Escape', windowsVirtualKeyCode: 27 });
  };
  const openMenu = async () => {
    await tap(menu);
    await waitFor('!!document.querySelector(".staff-mobile-rail")');
    await settleDialog();
    await assert('document.querySelector("[role=dialog]").contains(document.activeElement)', 'menu takes focus');
    await assert('document.querySelector(".staff-workspace").inert', 'menu background is inert');
  };
  const routes = [
    ['catalogue', 'Catalogue'], ['members', 'Members'], ['circulation', 'Circulation'],
    ['accession', 'Accession Register'], ['classification', 'Classification'], ['inventory', 'Inventory'],
    ['acquisitions', 'Acquisitions'], ['periodicals', 'Periodicals'], ['reports', 'Reports'],
    ['printing', 'Barcode & Printing'], ['settings', 'Library Settings'], ['backup', 'Database Backups'],
    ['audit', 'Audit Log'], ['staff', 'Staff & Permissions'],
  ];
  await send('Emulation.setTouchEmulationEnabled', { enabled: true });
  for (const [width, height] of [[320, 740], [390, 844], [768, 1024]]) {
    await send('Emulation.setDeviceMetricsOverride', { width, height, deviceScaleFactor: 1, mobile: true });
    for (const [route, label] of routes) {
      await openMenu();
      const targets = await evaluate('Array.from(document.querySelectorAll(".staff-mobile-rail nav a")).map(a=>({name:a.textContent,height:a.getBoundingClientRect().height,minHeight:getComputedStyle(a).minHeight}))');
      if (targets.some(target => target.height < 44)) throw Error(`navigation touch targets: ${JSON.stringify(targets)}`);
      await tap(`Array.from(document.querySelectorAll('.staff-mobile-rail nav a')).find(a=>a.textContent.trim()===${JSON.stringify(label)})`);
      await waitFor(`location.hash===${JSON.stringify(`#/${route}`)} && !document.querySelector('.staff-mobile-rail') && !!document.querySelector('main h1')`);
      await pause(150);
      await waitFor('!document.querySelector("main").innerText.includes("Loading ")');
      await assert('document.documentElement.scrollWidth<=innerWidth', `page overflow ${route} ${width}`);
      await assert('Math.abs(document.querySelector("main").getBoundingClientRect().width-innerWidth)<2', `full workspace ${route} ${width}`);
      await assert('document.querySelector("main").getBoundingClientRect().height>=innerHeight-100', `usable workspace height ${route} ${width}`);
      const overflow = await evaluate('(()=>{const m=document.querySelector("main");return m.scrollWidth<=m.clientWidth+2?null:{width:m.clientWidth,scroll:m.scrollWidth,elements:Array.from(m.querySelectorAll("*")).filter(e=>e.getBoundingClientRect().right>innerWidth+2 && !e.closest("table")).slice(0,12).map(e=>({tag:e.tagName,classes:e.className,width:e.getBoundingClientRect().width,right:e.getBoundingClientRect().right,text:e.textContent.slice(0,80)}))};})()');
      if (overflow) geometryErrors.push(`workspace overflow ${route} ${width}: ${JSON.stringify(overflow)}`);
      const focus = await evaluate('({menu:document.activeElement===document.querySelector(".staff-menu-toggle"),element:document.activeElement.outerHTML.slice(0,400)})');
      if (!focus.menu) throw Error(`menu focus ${route} ${width}: ${JSON.stringify(focus)}`);
      const errors = await evaluate('window.apiErrors');
      if (errors.length) throw Error(`${route} mobile API errors: ${JSON.stringify(errors)}`);
      checks.push(`actual touch navigation and layout ${route} ${width}`);
      const tabs = await evaluate('Array.from(document.querySelectorAll("main [role=tab]")).map(t=>t.textContent.trim())');
      for (const label of tabs) {
        await tap(`Array.from(document.querySelectorAll('main [role=tab]')).find(t=>t.textContent.trim()===${JSON.stringify(label)})`);
        await pause(200);
        await waitFor('!document.querySelector("main").innerText.includes("Loading ")');
        const tabOverflow = await evaluate('(()=>{const m=document.querySelector("main");return m.scrollWidth<=m.clientWidth+2?null:Array.from(m.querySelectorAll("*")).filter(e=>e.getBoundingClientRect().right>innerWidth+2 && !e.closest("table,[role=tablist]")).slice(0,8).map(e=>({tag:e.tagName,classes:e.className,right:e.getBoundingClientRect().right,text:e.textContent.slice(0,60)}));})()');
        if (tabOverflow) geometryErrors.push(`${route}/${label} overflow ${width}: ${JSON.stringify(tabOverflow)}`);
        const errors = await evaluate('window.apiErrors');
        if (errors.length) throw Error(`${route}/${label} API errors: ${JSON.stringify(errors)}`);
        checks.push(`actual touch tab ${route}/${label} ${width}`);
      }
      if (width === 390) {
        const page = await send('Page.captureScreenshot', { format: 'png' });
        await writeFile(new URL(`actual-staff-${route}-390.png`, output), Buffer.from(page.data, 'base64'));
      }
    }
    await openMenu();
    if (width === 390) {
      const shot = await send('Page.captureScreenshot', { format: 'png' });
      await writeFile(new URL('actual-staff-navigation-390.png', output), Buffer.from(shot.data, 'base64'));
    }
    await escape();
    await waitFor('!document.querySelector(".staff-mobile-rail")');
    await assert('document.activeElement===document.querySelector(".staff-menu-toggle")', 'Escape restores menu focus');
    checks.push(`actual menu Escape and focus ${width}`);
    await openMenu();
    await send('Input.dispatchTouchEvent', { type: 'touchStart', touchPoints: [{ x: 1, y: 1 }] });
    await send('Input.dispatchTouchEvent', { type: 'touchEnd', touchPoints: [] });
    await waitFor('!document.querySelector(".staff-mobile-rail")');
    await assert('document.activeElement===document.querySelector(".staff-menu-toggle")', 'backdrop restores menu focus');
    checks.push(`actual touch backdrop and focus ${width}`);
    await evaluate("location.hash='/catalogue'");
    await waitFor('document.querySelector("main").innerText.includes("Mobile verification:")');
    await assert('document.querySelector(".staff-card-table td:nth-child(2)").getBoundingClientRect().width>=150', 'populated book title has readable width');
    await tap('Array.from(document.querySelectorAll("main button")).find(b=>b.textContent==="Add record")');
    await waitFor('!!document.querySelector("[role=dialog]")');
    await settleDialog();
    await assert('Array.from(document.querySelectorAll("[role=dialog] input")).filter(i=>i.getBoundingClientRect().width>0 && !["checkbox","radio"].includes(i.type)).every(i=>parseFloat(getComputedStyle(i).fontSize)>=16 && i.getBoundingClientRect().height>=44)', 'record form text and touch size');
    await assert('document.querySelector("[role=dialog]").getBoundingClientRect().right<=innerWidth && document.querySelector("[role=dialog]").getBoundingClientRect().height<=innerHeight', 'record dialog fits');
    if (width === 390) {
      const shot = await send('Page.captureScreenshot', { format: 'png' });
      await writeFile(new URL('actual-staff-book-dialog-390.png', output), Buffer.from(shot.data, 'base64'));
    }
    await escape(); await waitFor('!document.querySelector("[role=dialog]")');
    await evaluate("location.hash='/members'");
    await waitFor('document.querySelector("main").innerText.includes("Mobile Test Reader")');
    await assert('Array.from(document.querySelectorAll(".staff-member-table button")).every(b=>b.getBoundingClientRect().right<=innerWidth && b.getBoundingClientRect().height>=44)', 'member actions reachable');
    checks.push(`actual populated records, member actions and book dialog ${width}`);
    if (width === 390) {
      const shot = await send('Page.captureScreenshot', { format: 'png' });
      await writeFile(new URL('actual-staff-members-390.png', output), Buffer.from(shot.data, 'base64'));
    }
  }
  await send('Emulation.setDeviceMetricsOverride', { width: 740, height: 360, deviceScaleFactor: 1, mobile: true });
  await openMenu();
  await assert('document.querySelector("[role=dialog]").getBoundingClientRect().height<=innerHeight', 'landscape navigation fits');
  await escape();
  checks.push('actual landscape navigation');
  await send('Emulation.setDeviceMetricsOverride', { width: 390, height: 844, deviceScaleFactor: 1, mobile: true });
  await tap('Array.from(document.querySelectorAll(".staff-account button")).find(b=>b.textContent.trim()==="Sign out")');
  await waitFor('!!document.querySelector("input[autocomplete=username]")');
  await assert('Array.from(document.querySelectorAll("form input")).every(i=>parseFloat(getComputedStyle(i).fontSize)>=16 && i.getBoundingClientRect().height>=44)', 'mobile sign-in fields');
  await tap('document.querySelector("input[autocomplete=username]")');
  await send('Input.insertText', { text: 'owner' });
  await tap('document.querySelector("input[type=password]")');
  await send('Input.insertText', { text: 'test-password' });
  await tap('document.querySelector("button[type=submit]")');
  await waitFor('!!document.querySelector(".staff-menu-toggle")');
  checks.push('actual mobile sign out and keyboard sign in');
  await send('Emulation.setDeviceMetricsOverride', { width: 1366, height: 950, deviceScaleFactor: 1, mobile: false });
  await send('Emulation.setTouchEmulationEnabled', { enabled: false });
  await evaluate("location.hash='/catalogue'");
  await waitFor('!!document.querySelector("main input[type=search]")');
  await assert('document.activeElement===document.querySelector("main input[type=search]")', 'desktop search autofocus retained');
  checks.push('desktop search autofocus retained');
  if (geometryErrors.length) throw Error(geometryErrors.join('\n'));
}

export async function publicMobileChecks({ send, evaluate, waitFor, assert, checks }) {
  const output = new URL('../design/mobile/previews/', import.meta.url);
  const origin = await evaluate('location.origin');
  await send('Page.navigate', { url: `${origin}/opac` });
  await waitFor('document.querySelectorAll(".record").length===1');
  for (const [width, height] of [[320, 740], [390, 844], [768, 1024]]) {
    await send('Emulation.setDeviceMetricsOverride', { width, height, deviceScaleFactor: 1, mobile: true });
    await assert('document.documentElement.scrollWidth<=innerWidth', `real OPAC overflow ${width}`);
    await assert('document.querySelector(".title-button").getBoundingClientRect().height>=44', `OPAC title target ${width}`);
    await assert('!document.body.innerText.includes("Mobile Test Reader") && !document.body.innerText.includes("MOBILE-001")', 'public OPAC excludes member data');
    if (width === 390) {
      const shot = await send('Page.captureScreenshot', { format: 'png' });
      await writeFile(new URL('actual-opac-390.png', output), Buffer.from(shot.data, 'base64'));
    }
    await evaluate('document.querySelector(".title-button").click()');
    await waitFor('document.querySelector("#details").open');
    await assert('document.querySelector("#details").getBoundingClientRect().right<=innerWidth && document.querySelector("#close-details").getBoundingClientRect().height>=44', 'real OPAC phone details');
    await evaluate('document.querySelector("#close-details").click()');
    checks.push(`actual embedded OPAC results/details/privacy ${width}`);
  }
  await evaluate('document.querySelector("#q").value="not-a-real-title-000";document.querySelector("#search-form").requestSubmit()');
  await waitFor('!!document.querySelector(".empty")');
  await evaluate('document.querySelector("#reset").click()');
  await waitFor('document.querySelectorAll(".record").length===1');
  checks.push('actual embedded OPAC search, empty state and reset');
}
