# OPAC browser prototypes

Run from the repository root:

```powershell
node design/opac/server.mjs
```

Open http://127.0.0.1:5193/design/opac/ and choose The Shelf, The Index or Reading Table. Stop with Ctrl+C. Uses synthetic holdings; does not connect to the library database. Approved logo and fonts load locally.

Actual Chrome screenshots and responsive/interaction evidence are in previews/. Review harness requires a fresh isolated Chrome debugging instance on port 9361, then `node design/opac/review.mjs`; it owns its preview server and closes its browser at completion. Owner selection is pending; production OPAC is unchanged.
