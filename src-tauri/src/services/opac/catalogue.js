(() => {
  const $ = selector => document.querySelector(selector);
  const classes = ['All classes', '000 General works', '100 Philosophy & psychology', '200 Religion', '300 Social sciences', '400 Language', '500 Science', '600 Technology', '700 Arts & recreation', '800 Literature', '900 History & geography'];
  let classification = '', page = 1, controller, request = 0, books = [];
  let patron = { signedIn: false, csrfToken: null, member: null, unpaidFineMinor: 0, loans: [], reservations: [] };
  const formatMoney = (minor, currency) => {
    try { return new Intl.NumberFormat(undefined, { style: 'currency', currency: currency || 'INR' }).format(minor / 100); }
    catch { return `${currency || 'INR'} ${(minor / 100).toFixed(2)}`; }
  };
  const element = (tag, text, className) => {
    const node = document.createElement(tag);
    if (text !== undefined) node.textContent = text;
    if (className) node.className = className;
    return node;
  };
  classes.forEach((label, index) => {
    const button = element('button', label);
    button.type = 'button';
    button.dataset.class = index ? String(index - 1) : '';
    button.setAttribute('aria-pressed', String(!index));
    button.addEventListener('click', () => {
      classification = button.dataset.class;
      document.querySelectorAll('[data-class]').forEach(item => item.setAttribute('aria-pressed', String(item === button)));
      page = 1;
      load();
    });
    $('#classes').append(button);
  });
  function availability(book) {
    if (!book.totalCopies) return 'No circulating copies';
    return book.availableCopies > 0 ? `Available (${book.availableCopies} of ${book.totalCopies})` : `No copies available (${book.totalCopies} total)`;
  }
  function bookCover(book) {
    const frame = element('div', undefined, 'book-cover');
    const fallback = element('span', 'No cover', 'cover-fallback');
    fallback.setAttribute('aria-hidden', 'true');
    frame.append(fallback);
    const isbn = String(book.isbn || '').replace(/[-\s]/g, '');
    const validIsbn = /^(?:\d{13}|\d{9}[\dXx])$/.test(isbn);
    if (!Number.isSafeInteger(book.id) || book.id < 1) return frame;
    const image = element('img');
    image.alt = `Cover of ${book.title}`;
    image.width = 64; image.height = 96;
    image.style.opacity = '0'; image.loading = 'lazy'; image.decoding = 'async'; image.referrerPolicy = 'no-referrer';
    let remote = false;
    image.addEventListener('load', () => { fallback.hidden = true; image.hidden = false; image.style.opacity = '1'; });
    image.addEventListener('error', () => {
      if (!remote && validIsbn) {
        remote = true;
        image.src = `https://covers.openlibrary.org/b/isbn/${isbn}-M.jpg?default=false`;
      } else { image.hidden = true; fallback.hidden = false; }
    });
    image.src = `/api/opac/cover/${book.id}`;
    frame.append(image);
    return frame;
  }
  function details(book) {
    const content = $('#detail-content');
    content.replaceChildren();
    const title = element('h2', book.title); title.id = 'detail-title';
    content.append(bookCover(book), title);
    if (book.subtitle) content.append(element('p', book.subtitle));
    const list = element('dl');
    const fields = [
      ['Authors', book.authors.join(', ') || 'Not recorded'], ['Publisher', book.publisher],
      ['Year', book.year], ['ISBN', book.isbn], ['Classification', book.classification],
      ['Subjects', book.subjects.join(', ')], ['Locations', book.locations.join(', ') || 'Ask library staff'],
      ['Availability', availability(book)],
    ];
    fields.forEach(([label, value]) => {
      if (value !== null && value !== undefined && value !== '') list.append(element('dt', label), element('dd', String(value)));
    });
    content.append(list);
    const holdBar = element('div', undefined, 'hold-bar');
    if (!patron.signedIn) {
      const signInHint = element('button', 'Sign in to request a hold', 'hold-btn');
      signInHint.type = 'button';
      signInHint.addEventListener('click', () => { $('#details').close(); openAccount(); });
      holdBar.append(signInHint);
    } else {
      const holdBtn = element('button', 'Request hold', 'hold-btn');
      holdBtn.type = 'button';
      const holdMsg = element('p', undefined, 'ok-note'); holdMsg.hidden = true;
      const holdErr = element('p', undefined, 'danger-note'); holdErr.hidden = true;
      holdBtn.addEventListener('click', async () => {
        holdBtn.disabled = true; holdMsg.hidden = true; holdErr.hidden = true;
        try {
          const res = await fetch('/api/opac/patron/hold', {
            method: 'POST', credentials: 'same-origin',
            headers: { 'Content-Type': 'application/json', 'X-CSRF-Token': patron.csrfToken || '' },
            body: JSON.stringify({ recordId: book.id })
          });
          const data = await res.json();
          if (!res.ok) throw new Error(data.error || 'Could not place hold.');
          patron = data; updateAccountButton();
          holdMsg.textContent = 'Hold placed. View status in My account.'; holdMsg.hidden = false;
          load();
        } catch (err) {
          holdErr.textContent = err.message || 'Could not place hold.'; holdErr.hidden = false;
          holdBtn.disabled = false;
        }
      });
      holdBar.append(holdBtn, holdMsg, holdErr);
    }
    content.append(holdBar);
    $('#details').showModal();
  }
  function render(items) {
    const container = $('#results');
    container.replaceChildren();
    if (!items.length) {
      container.append(element('p', 'No titles found. Try another title, author, or ISBN, or clear the filters.', 'empty'));
      return;
    }
    items.forEach(book => {
      const row = element('article', undefined, 'record');
      row.append(element('div', book.classification || 'Unclassified', 'classification'));
      const bibliography = element('div', undefined, 'bibliography');
      const metadata = element('div');
      const heading = element('h2');
      const button = element('button', book.title, 'title-button');
      button.type = 'button'; button.addEventListener('click', () => details(book));
      heading.append(button); metadata.append(heading);
      if (book.subtitle) metadata.append(element('p', book.subtitle, 'subtitle'));
      metadata.append(element('p', book.authors.join(', ') || 'Author not recorded', 'meta'));
      metadata.append(element('p', [book.year, book.publisher].filter(Boolean).join(' · '), 'meta'));
      bibliography.append(bookCover(book), metadata);
      const location = element('div', undefined, 'location');
      location.append(element('span', 'Location', 'location-label'), element('span', book.locations.join(', ') || 'Ask library staff'));
      row.append(bibliography, location, element('div', availability(book), `availability${book.availableCopies ? '' : ' unavailable'}`));
      container.append(row);
    });
  }
  async function load() {
    controller?.abort(); controller = new AbortController();
    const sequence = ++request;
    const query = new URLSearchParams({ q: $('#q').value.trim(), page: String(page), class: classification, available: String($('#available').checked) });
    $('#results').setAttribute('aria-busy', 'true');
    $('#results').replaceChildren(); books = [];
    $('#summary').textContent = 'Loading catalogue…';
    $('#error').hidden = true; $('#retry').hidden = true; $('.pagination').hidden = true;
    try {
      const response = await fetch(`/api/opac/search?${query}`, { signal: controller.signal, cache: 'no-store' });
      const data = await response.json();
      if (!response.ok) throw new Error(response.status === 403 ? 'The public catalogue is currently disabled. Please ask library staff.' : 'Could not load the catalogue. Try again.');
      if (sequence !== request) return;
      if (!Array.isArray(data.items) || !Number.isFinite(data.total) || !(data.pageSize > 0)) throw new Error('Could not read the catalogue response. Try again.');
      books = data.items;
      if (data.total && (page - 1) * data.pageSize >= data.total) { page = 1; load(); return; }
      render(books);
      $('#summary').textContent = `${data.total} ${data.total === 1 ? 'title' : 'titles'} found`;
      const pages = Math.max(1, Math.ceil(data.total / data.pageSize));
      $('#page-label').textContent = `Page ${page} of ${pages}`;
      $('#previous').disabled = page <= 1; $('#next').disabled = page >= pages;
      $('.pagination').hidden = pages <= 1;
    } catch (error) {
      if (sequence !== request || error.name === 'AbortError') return;
      $('#summary').textContent = 'Catalogue unavailable';
      $('#error').textContent = error instanceof TypeError ? 'Cannot connect to the library. Check your connection and try again.' : error.message;
      $('#error').hidden = false; $('#retry').hidden = false;
    } finally {
      if (sequence === request) $('#results').setAttribute('aria-busy', 'false');
    }
  }
  $('#search-form').addEventListener('submit', event => { event.preventDefault(); page = 1; load(); });
  $('#available').addEventListener('change', () => { page = 1; load(); });
  $('#reset').addEventListener('click', () => { $('#q').value = ''; $('#available').checked = false; $('[data-class=""]').click(); });
  $('#retry').addEventListener('click', load);
  $('#previous').addEventListener('click', () => { page--; load(); });
  $('#next').addEventListener('click', () => { page++; load(); });
  $('#close-details').addEventListener('click', () => $('#details').close());
  document.addEventListener('keydown', event => {
    if (event.key === '/' && !$('#details').open && !['INPUT', 'TEXTAREA', 'SELECT'].includes(document.activeElement.tagName)) { event.preventDefault(); $('#q').focus(); }
  });
  function updateAccountButton() {
    const btn = $('#account-btn');
    if (!btn) return;
    btn.textContent = patron.signedIn && patron.member
      ? `${patron.member.name} (${patron.loans.length} loans)`
      : 'My account';
  }
  function renderAccount() {
    const box = $('#account-content');
    if (!box) return;
    box.replaceChildren();
    if (!patron.signedIn || !patron.member) {
      const h = element('h2', 'Member sign in'); h.id = 'account-title';
      const hint = element('p', 'Enter your member number or OPAC username, plus your password, PIN, or registered phone/email.', 'account-meta');
      const form = element('form', undefined, 'account-form');
      const l1 = element('label', 'Member number or username');
      const numIn = document.createElement('input'); numIn.required = true; numIn.autocomplete = 'username'; numIn.placeholder = 'e.g. M-101 or ananya.nair';
      l1.append(numIn);
      const l2 = element('label', 'Password, PIN, or phone/email');
      const verIn = document.createElement('input'); verIn.type = 'password'; verIn.required = true; verIn.autocomplete = 'current-password'; verIn.placeholder = 'Password, PIN, or phone/email';
      l2.append(verIn);
      const err = element('p', undefined, 'danger-note'); err.setAttribute('role', 'alert'); err.hidden = true;
      const submit = element('button', 'Sign in'); submit.type = 'submit';
      form.append(l1, l2, err, submit);
      form.addEventListener('submit', async e => {
        e.preventDefault(); err.hidden = true; submit.disabled = true;
        try {
          const res = await fetch('/api/opac/patron/login', {
            method: 'POST', credentials: 'same-origin',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ memberNumber: numIn.value.trim(), verification: verIn.value.trim() })
          });
          const data = await res.json();
          if (!res.ok) throw new Error(data.error || 'Could not sign in.');
          patron = data; updateAccountButton(); renderAccount();
        } catch (ex) {
          err.textContent = ex.message || 'Could not sign in.'; err.hidden = false; submit.disabled = false;
        }
      });
      box.append(h, hint, form);
      return;
    }
    const m = patron.member;
    const head = element('div', undefined, 'account-head');
    const info = element('div');
    const h = element('h2', m.name); h.id = 'account-title';
    const sub = element('p', `${m.memberNumber} · ${m.categoryName} · Expires ${m.expiresOn}`, 'account-meta');
    info.append(h, sub);
    if (patron.unpaidFineMinor > 0) {
      info.append(element('p', `Unpaid balance: ${formatMoney(patron.unpaidFineMinor, m.currency)}`, 'danger-note'));
    }
    const outBtn = element('button', 'Sign out'); outBtn.type = 'button';
    outBtn.addEventListener('click', async () => {
      await fetch('/api/opac/patron/logout', {
        method: 'POST', credentials: 'same-origin',
        headers: { 'X-CSRF-Token': patron.csrfToken || '' }
      });
      patron = { signedIn: false, csrfToken: null, member: null, unpaidFineMinor: 0, loans: [], reservations: [] };
      updateAccountButton(); renderAccount();
    });
    head.append(info, outBtn);
    box.append(head);

    const loanSec = element('section', undefined, 'account-section');
    loanSec.append(element('h3', `Active loans (${patron.loans.length})`));
    if (!patron.loans.length) {
      loanSec.append(element('p', 'No books currently checked out.', 'account-meta'));
    } else {
      const ul = element('ul', undefined, 'account-list');
      patron.loans.forEach(l => {
        const li = element('li', undefined, 'account-item');
        const left = element('div');
        left.append(element('strong', l.title), element('div', `${l.barcode} · Due ${l.dueOn}`, 'account-meta'));
        const badge = element('span', l.overdue ? 'Overdue' : 'On loan', `status-pill${l.overdue ? ' overdue' : ''}`);
        li.append(left, badge);
        ul.append(li);
      });
      loanSec.append(ul);
    }
    box.append(loanSec);

    const holdSec = element('section', undefined, 'account-section');
    holdSec.append(element('h3', `Holds (${patron.reservations.length})`));
    if (!patron.reservations.length) {
      holdSec.append(element('p', 'No active holds.', 'account-meta'));
    } else {
      const ul = element('ul', undefined, 'account-list');
      patron.reservations.forEach(r => {
        const li = element('li', undefined, 'account-item');
        const left = element('div');
        left.append(element('strong', r.title), element('div', `${r.barcode} · ${r.status === 'ready' ? 'Ready for pickup' : `Pending in queue${r.queuePosition ? ` (#${r.queuePosition})` : ''}`}`, 'account-meta'));
        const cancelBtn = element('button', 'Cancel hold'); cancelBtn.type = 'button';
        cancelBtn.addEventListener('click', async () => {
          cancelBtn.disabled = true;
          const res = await fetch('/api/opac/patron/cancel-hold', {
            method: 'POST', credentials: 'same-origin',
            headers: { 'Content-Type': 'application/json', 'X-CSRF-Token': patron.csrfToken || '' },
            body: JSON.stringify({ reservationId: r.id })
          });
          if (res.ok) { patron = await res.json(); updateAccountButton(); renderAccount(); load(); }
          else { cancelBtn.disabled = false; }
        });
        li.append(left, cancelBtn);
        ul.append(li);
      });
      holdSec.append(ul);
    }
    box.append(holdSec);
  }
  function openAccount() { renderAccount(); $('#account-dialog')?.showModal(); }
  async function loadAccount() {
    try {
      const res = await fetch('/api/opac/patron/session', { credentials: 'same-origin', cache: 'no-store' });
      if (res.ok) { patron = await res.json(); updateAccountButton(); }
    } catch { /* OPAC search remains usable if session check fails */ }
  }
  $('#account-btn')?.addEventListener('click', openAccount);
  $('#close-account')?.addEventListener('click', () => $('#account-dialog')?.close());
  const copyrightYearEl = $('#copyright-year');
  if (copyrightYearEl) {
    const year = new Date().getFullYear();
    copyrightYearEl.textContent = year > 2026 ? `2026–${year}` : '2026';
  }
  loadAccount();
  load();
})();
