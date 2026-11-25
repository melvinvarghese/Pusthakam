(() => {
  const $ = selector => document.querySelector(selector);
  const classes = ['All classes', '000 General works', '100 Philosophy & psychology', '200 Religion', '300 Social sciences', '400 Language', '500 Science', '600 Technology', '700 Arts & recreation', '800 Literature', '900 History & geography'];
  let classification = '', page = 1, controller, request = 0, books = [];
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
  load();
})();
