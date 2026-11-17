CREATE TABLE stock_transfers (
    id INTEGER PRIMARY KEY,
    item_id INTEGER NOT NULL REFERENCES physical_items(id) ON DELETE RESTRICT,
    barcode TEXT NOT NULL,
    title TEXT NOT NULL,
    source_location_id INTEGER NOT NULL REFERENCES locations(id) ON DELETE RESTRICT,
    source_shelf_id INTEGER REFERENCES shelves(id) ON DELETE RESTRICT,
    destination_location_id INTEGER NOT NULL REFERENCES locations(id) ON DELETE RESTRICT,
    destination_shelf_id INTEGER REFERENCES shelves(id) ON DELETE RESTRICT,
    source_label TEXT NOT NULL,
    destination_label TEXT NOT NULL,
    sent_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    sent_by INTEGER NOT NULL REFERENCES staff_users(id) ON DELETE RESTRICT,
    finished_at TEXT,
    finished_by INTEGER REFERENCES staff_users(id) ON DELETE RESTRICT,
    status TEXT NOT NULL DEFAULT 'in_transit' CHECK (status IN ('in_transit', 'received', 'cancelled')),
    CHECK ((status = 'in_transit' AND finished_at IS NULL AND finished_by IS NULL)
        OR (status <> 'in_transit' AND finished_at IS NOT NULL AND finished_by IS NOT NULL))
);

CREATE UNIQUE INDEX stock_transfers_one_open_per_copy ON stock_transfers (item_id) WHERE status = 'in_transit';

CREATE TRIGGER stock_transfer_guard_copy BEFORE UPDATE OF location_id, shelf_id, status ON physical_items
WHEN (OLD.location_id IS NOT NEW.location_id OR OLD.shelf_id IS NOT NEW.shelf_id OR OLD.status IS NOT NEW.status)
    AND EXISTS(SELECT 1 FROM stock_transfers WHERE item_id = OLD.id AND status = 'in_transit')
BEGIN SELECT RAISE(ABORT, 'close the stock transfer before moving the copy'); END;
