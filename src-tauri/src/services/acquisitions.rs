use crate::domain::validate;
use crate::error::{AppError, AppResult};
use crate::services::audit;
use crate::services::auth::Session;
use crate::services::catalogue;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vendor {
    pub id: i64,
    pub name: String,
    pub code: Option<String>,
    pub contact_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub notes: Option<String>,
    pub archived: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VendorInput {
    pub id: Option<i64>,
    pub name: String,
    pub code: Option<String>,
    pub contact_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcquisitionBudget {
    pub id: i64,
    pub name: String,
    pub code: Option<String>,
    pub amount_minor: i64,
    pub currency: String,
    pub fiscal_year: Option<String>,
    pub allocated_minor: i64,
    pub archived: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetInput {
    pub id: Option<i64>,
    pub name: String,
    pub code: Option<String>,
    pub amount_minor: i64,
    pub currency: String,
    pub fiscal_year: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcquisitionRequest {
    pub id: i64,
    pub title: String,
    pub author: Option<String>,
    pub isbn: Option<String>,
    pub publisher: Option<String>,
    pub publication_year: Option<i64>,
    pub estimated_price_minor: Option<i64>,
    pub currency: String,
    pub copies: i64,
    pub requested_by: String,
    pub request_reason: Option<String>,
    pub status: String,
    pub rejection_reason: Option<String>,
    pub budget_id: Option<i64>,
    pub budget_name: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRequestInput {
    pub title: String,
    pub author: Option<String>,
    pub isbn: Option<String>,
    pub publisher: Option<String>,
    pub publication_year: Option<i64>,
    pub estimated_price_minor: Option<i64>,
    pub currency: Option<String>,
    pub copies: Option<i64>,
    pub requested_by: String,
    pub request_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseOrderItem {
    pub id: i64,
    pub order_id: i64,
    pub request_id: Option<i64>,
    pub title: String,
    pub author: Option<String>,
    pub isbn: Option<String>,
    pub quantity: i64,
    pub unit_price_minor: i64,
    pub received_quantity: i64,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseOrderItemInput {
    pub request_id: Option<i64>,
    pub title: String,
    pub author: Option<String>,
    pub isbn: Option<String>,
    pub quantity: i64,
    pub unit_price_minor: i64,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseOrderSummary {
    pub id: i64,
    pub order_number: String,
    pub vendor_id: i64,
    pub vendor_name: String,
    pub budget_id: Option<i64>,
    pub budget_name: Option<String>,
    pub status: String,
    pub invoice_ref: Option<String>,
    pub ordered_at: Option<String>,
    pub items_count: i64,
    pub total_minor: i64,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseOrderDetail {
    pub id: i64,
    pub order_number: String,
    pub vendor_id: i64,
    pub vendor_name: String,
    pub budget_id: Option<i64>,
    pub budget_name: Option<String>,
    pub status: String,
    pub invoice_ref: Option<String>,
    pub ordered_at: Option<String>,
    pub expected_at: Option<String>,
    pub notes: Option<String>,
    pub items: Vec<PurchaseOrderItem>,
    pub total_minor: i64,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrderInput {
    pub order_number: String,
    pub vendor_id: i64,
    pub budget_id: Option<i64>,
    pub invoice_ref: Option<String>,
    pub expected_at: Option<String>,
    pub notes: Option<String>,
    pub items: Vec<PurchaseOrderItemInput>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiveItemInput {
    pub order_id: i64,
    pub item_id: i64,
    pub quantity: i64,
    pub record_id: Option<i64>,
    pub item_category_id: Option<i64>,
    pub location_id: Option<i64>,
    pub collection_id: Option<i64>,
    pub shelf_id: Option<i64>,
    pub call_number: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiveResult {
    pub order_id: i64,
    pub item_id: i64,
    pub record_id: i64,
    pub received_quantity: i64,
    pub order_status: String,
    pub created_copies: Vec<CreatedCopyInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatedCopyInfo {
    pub copy_id: i64,
    pub accession_no: String,
    pub barcode: String,
}

// ----- Vendors -----

pub fn list_vendors(conn: &Connection, include_archived: bool) -> AppResult<Vec<Vendor>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, code, contact_name, email, phone, address, notes,
                archived_at IS NOT NULL, created_at
         FROM vendors
         WHERE ?1 OR archived_at IS NULL
         ORDER BY name COLLATE NOCASE",
    )?;
    let rows = stmt
        .query_map([include_archived], |r| {
            Ok(Vendor {
                id: r.get(0)?,
                name: r.get(1)?,
                code: r.get(2)?,
                contact_name: r.get(3)?,
                email: r.get(4)?,
                phone: r.get(5)?,
                address: r.get(6)?,
                notes: r.get(7)?,
                archived: r.get(8)?,
                created_at: r.get(9)?,
            })
        })?
        .collect::<Result<_, _>>()?;
    Ok(rows)
}

pub fn save_vendor(conn: &mut Connection, input: VendorInput, actor: &Session) -> AppResult<i64> {
    let name = validate::required("Vendor name", &input.name, 120)?;
    let code = validate::optional("Vendor code", input.code.as_deref(), 30)?;
    let contact = validate::optional("Contact name", input.contact_name.as_deref(), 100)?;
    let email = validate::optional("Email", input.email.as_deref(), 120)?;
    let phone = validate::optional("Phone", input.phone.as_deref(), 40)?;
    let address = validate::optional("Address", input.address.as_deref(), 300)?;
    let notes = validate::optional("Notes", input.notes.as_deref(), 500)?;

    let tx = conn.transaction()?;
    let id = match input.id {
        Some(id) => {
            tx.execute(
                "UPDATE vendors SET name = ?1, code = ?2, contact_name = ?3, email = ?4, phone = ?5, address = ?6, notes = ?7, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ','now')
                 WHERE id = ?8",
                params![name, code, contact, email, phone, address, notes, id],
            )?;
            audit::record(&tx, Some(actor), "vendor.update", "vendor", Some(id), &format!("Updated vendor {name}"))?;
            id
        }
        None => {
            tx.execute(
                "INSERT INTO vendors (name, code, contact_name, email, phone, address, notes)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![name, code, contact, email, phone, address, notes],
            )?;
            let new_id = tx.last_insert_rowid();
            audit::record(&tx, Some(actor), "vendor.create", "vendor", Some(new_id), &format!("Created vendor {name}"))?;
            new_id
        }
    };
    tx.commit()?;
    Ok(id)
}

pub fn archive_vendor(conn: &mut Connection, id: i64, archive: bool, actor: &Session) -> AppResult<()> {
    let tx = conn.transaction()?;
    let name: String = tx
        .query_row("SELECT name FROM vendors WHERE id = ?1", [id], |r| r.get(0))
        .map_err(|_| AppError::NotFound("Vendor not found.".into()))?;

    if archive {
        tx.execute(
            "UPDATE vendors SET archived_at = strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE id = ?1",
            [id],
        )?;
        audit::record(&tx, Some(actor), "vendor.archive", "vendor", Some(id), &format!("Archived vendor {name}"))?;
    } else {
        tx.execute("UPDATE vendors SET archived_at = NULL WHERE id = ?1", [id])?;
        audit::record(&tx, Some(actor), "vendor.restore", "vendor", Some(id), &format!("Restored vendor {name}"))?;
    }
    tx.commit()?;
    Ok(())
}

// ----- Budgets -----

pub fn list_budgets(conn: &Connection, include_archived: bool) -> AppResult<Vec<AcquisitionBudget>> {
    let mut stmt = conn.prepare(
        "SELECT b.id, b.name, b.code, b.amount_minor, b.currency, b.fiscal_year,
                coalesce(sum(poi.quantity * poi.unit_price_minor), 0) AS allocated,
                b.archived_at IS NOT NULL, b.created_at
         FROM acquisition_budgets b
         LEFT JOIN purchase_orders po ON po.budget_id = b.id AND po.status <> 'cancelled'
         LEFT JOIN purchase_order_items poi ON poi.order_id = po.id
         WHERE ?1 OR b.archived_at IS NULL
         GROUP BY b.id
         ORDER BY b.name COLLATE NOCASE",
    )?;
    let rows = stmt
        .query_map([include_archived], |r| {
            Ok(AcquisitionBudget {
                id: r.get(0)?,
                name: r.get(1)?,
                code: r.get(2)?,
                amount_minor: r.get(3)?,
                currency: r.get(4)?,
                fiscal_year: r.get(5)?,
                allocated_minor: r.get(6)?,
                archived: r.get(7)?,
                created_at: r.get(8)?,
            })
        })?
        .collect::<Result<_, _>>()?;
    Ok(rows)
}

pub fn save_budget(conn: &mut Connection, input: BudgetInput, actor: &Session) -> AppResult<i64> {
    let name = validate::required("Budget name", &input.name, 100)?;
    let code = validate::optional("Budget code", input.code.as_deref(), 30)?;
    let currency = validate::currency(&input.currency)?;
    let fiscal = validate::optional("Fiscal year", input.fiscal_year.as_deref(), 20)?;
    if input.amount_minor < 0 {
        return Err(AppError::Validation("Budget amount must be positive.".into()));
    }

    let tx = conn.transaction()?;
    let id = match input.id {
        Some(id) => {
            tx.execute(
                "UPDATE acquisition_budgets SET name = ?1, code = ?2, amount_minor = ?3, currency = ?4, fiscal_year = ?5, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ','now')
                 WHERE id = ?6",
                params![name, code, input.amount_minor, currency, fiscal, id],
            )?;
            audit::record(&tx, Some(actor), "budget.update", "budget", Some(id), &format!("Updated budget {name}"))?;
            id
        }
        None => {
            tx.execute(
                "INSERT INTO acquisition_budgets (name, code, amount_minor, currency, fiscal_year)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![name, code, input.amount_minor, currency, fiscal],
            )?;
            let new_id = tx.last_insert_rowid();
            audit::record(&tx, Some(actor), "budget.create", "budget", Some(new_id), &format!("Created budget {name}"))?;
            new_id
        }
    };
    tx.commit()?;
    Ok(id)
}

pub fn archive_budget(conn: &mut Connection, id: i64, archive: bool, actor: &Session) -> AppResult<()> {
    let tx = conn.transaction()?;
    let name: String = tx
        .query_row("SELECT name FROM acquisition_budgets WHERE id = ?1", [id], |r| r.get(0))
        .map_err(|_| AppError::NotFound("Budget not found.".into()))?;

    if archive {
        tx.execute(
            "UPDATE acquisition_budgets SET archived_at = strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE id = ?1",
            [id],
        )?;
        audit::record(&tx, Some(actor), "budget.archive", "budget", Some(id), &format!("Archived budget {name}"))?;
    } else {
        tx.execute("UPDATE acquisition_budgets SET archived_at = NULL WHERE id = ?1", [id])?;
        audit::record(&tx, Some(actor), "budget.restore", "budget", Some(id), &format!("Restored budget {name}"))?;
    }
    tx.commit()?;
    Ok(())
}

// ----- Requests -----

pub fn list_requests(conn: &Connection, status: Option<String>) -> AppResult<Vec<AcquisitionRequest>> {
    let mut stmt = conn.prepare(
        "SELECT r.id, r.title, r.author, r.isbn, r.publisher, r.publication_year,
                r.estimated_price_minor, r.currency, r.copies, r.requested_by,
                r.request_reason, r.status, r.rejection_reason, r.budget_id,
                b.name, r.created_at
         FROM acquisition_requests r
         LEFT JOIN acquisition_budgets b ON b.id = r.budget_id
         WHERE ?1 IS NULL OR r.status = ?1
         ORDER BY r.created_at DESC",
    )?;
    let rows = stmt
        .query_map([status], |row| {
            Ok(AcquisitionRequest {
                id: row.get(0)?,
                title: row.get(1)?,
                author: row.get(2)?,
                isbn: row.get(3)?,
                publisher: row.get(4)?,
                publication_year: row.get(5)?,
                estimated_price_minor: row.get(6)?,
                currency: row.get(7)?,
                copies: row.get(8)?,
                requested_by: row.get(9)?,
                request_reason: row.get(10)?,
                status: row.get(11)?,
                rejection_reason: row.get(12)?,
                budget_id: row.get(13)?,
                budget_name: row.get(14)?,
                created_at: row.get(15)?,
            })
        })?
        .collect::<Result<_, _>>()?;
    Ok(rows)
}

pub fn create_request(conn: &mut Connection, input: CreateRequestInput, actor: &Session) -> AppResult<i64> {
    let title = validate::required("Title", &input.title, 300)?;
    let author = validate::optional("Author", input.author.as_deref(), 200)?;
    let isbn = validate::optional("ISBN", input.isbn.as_deref(), 40)?;
    let publisher = validate::optional("Publisher", input.publisher.as_deref(), 200)?;
    let requested_by = validate::required("Requested by", &input.requested_by, 120)?;
    let reason = validate::optional("Request reason", input.request_reason.as_deref(), 500)?;
    let currency = input.currency.unwrap_or_else(|| "INR".into());
    let copies = input.copies.unwrap_or(1).max(1);

    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO acquisition_requests (title, author, isbn, publisher, publication_year, estimated_price_minor, currency, copies, requested_by, request_reason, created_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![title, author, isbn, publisher, input.publication_year, input.estimated_price_minor, currency, copies, requested_by, reason, actor.user_id],
    )?;
    let id = tx.last_insert_rowid();
    audit::record(&tx, Some(actor), "request.create", "acquisition_request", Some(id), &format!("Requested book {title}"))?;
    tx.commit()?;
    Ok(id)
}

pub fn approve_request(conn: &mut Connection, id: i64, budget_id: Option<i64>, actor: &Session) -> AppResult<()> {
    let tx = conn.transaction()?;
    let (title, status): (String, String) = tx
        .query_row("SELECT title, status FROM acquisition_requests WHERE id = ?1", [id], |r| Ok((r.get(0)?, r.get(1)?)))
        .map_err(|_| AppError::NotFound("Request not found.".into()))?;
    if status != "pending" {
        return Err(AppError::Validation(format!("Cannot approve request with status '{status}'.")));
    }
    tx.execute(
        "UPDATE acquisition_requests
         SET status = 'approved', budget_id = ?1, reviewed_by = ?2, reviewed_at = strftime('%Y-%m-%dT%H:%M:%SZ','now'), updated_at = strftime('%Y-%m-%dT%H:%M:%SZ','now')
         WHERE id = ?3",
        params![budget_id, actor.user_id, id],
    )?;
    audit::record(&tx, Some(actor), "request.approve", "acquisition_request", Some(id), &format!("Approved request {title}"))?;
    tx.commit()?;
    Ok(())
}

pub fn reject_request(conn: &mut Connection, id: i64, reason: Option<String>, actor: &Session) -> AppResult<()> {
    let tx = conn.transaction()?;
    let (title, status): (String, String) = tx
        .query_row("SELECT title, status FROM acquisition_requests WHERE id = ?1", [id], |r| Ok((r.get(0)?, r.get(1)?)))
        .map_err(|_| AppError::NotFound("Request not found.".into()))?;
    if status != "pending" {
        return Err(AppError::Validation(format!("Cannot reject request with status '{status}'.")));
    }
    let reason_val = validate::optional("Rejection reason", reason.as_deref(), 300)?;
    tx.execute(
        "UPDATE acquisition_requests
         SET status = 'rejected', rejection_reason = ?1, reviewed_by = ?2, reviewed_at = strftime('%Y-%m-%dT%H:%M:%SZ','now'), updated_at = strftime('%Y-%m-%dT%H:%M:%SZ','now')
         WHERE id = ?3",
        params![reason_val, actor.user_id, id],
    )?;
    audit::record(&tx, Some(actor), "request.reject", "acquisition_request", Some(id), &format!("Rejected request {title}"))?;
    tx.commit()?;
    Ok(())
}

// ----- Purchase Orders -----

pub fn list_orders(conn: &Connection, status: Option<String>) -> AppResult<Vec<PurchaseOrderSummary>> {
    let mut stmt = conn.prepare(
        "SELECT po.id, po.order_number, po.vendor_id, v.name, po.budget_id, b.name,
                po.status, po.invoice_ref, po.ordered_at,
                count(poi.id), coalesce(sum(poi.quantity * poi.unit_price_minor), 0),
                po.created_at
         FROM purchase_orders po
         JOIN vendors v ON v.id = po.vendor_id
         LEFT JOIN acquisition_budgets b ON b.id = po.budget_id
         LEFT JOIN purchase_order_items poi ON poi.order_id = po.id
         WHERE ?1 IS NULL OR po.status = ?1
         GROUP BY po.id
         ORDER BY po.created_at DESC",
    )?;
    let rows = stmt
        .query_map([status], |r| {
            Ok(PurchaseOrderSummary {
                id: r.get(0)?,
                order_number: r.get(1)?,
                vendor_id: r.get(2)?,
                vendor_name: r.get(3)?,
                budget_id: r.get(4)?,
                budget_name: r.get(5)?,
                status: r.get(6)?,
                invoice_ref: r.get(7)?,
                ordered_at: r.get(8)?,
                items_count: r.get(9)?,
                total_minor: r.get(10)?,
                created_at: r.get(11)?,
            })
        })?
        .collect::<Result<_, _>>()?;
    Ok(rows)
}

pub fn get_order(conn: &Connection, id: i64) -> AppResult<PurchaseOrderDetail> {
    let (order_number, vendor_id, vendor_name, budget_id, budget_name, status, invoice_ref, ordered_at, expected_at, notes, created_at) = conn
        .query_row(
            "SELECT po.order_number, po.vendor_id, v.name, po.budget_id, b.name,
                    po.status, po.invoice_ref, po.ordered_at, po.expected_at, po.notes, po.created_at
             FROM purchase_orders po
             JOIN vendors v ON v.id = po.vendor_id
             LEFT JOIN acquisition_budgets b ON b.id = po.budget_id
             WHERE po.id = ?1",
            [id],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, Option<i64>>(3)?,
                    r.get::<_, Option<String>>(4)?,
                    r.get::<_, String>(5)?,
                    r.get::<_, Option<String>>(6)?,
                    r.get::<_, Option<String>>(7)?,
                    r.get::<_, Option<String>>(8)?,
                    r.get::<_, Option<String>>(9)?,
                    r.get::<_, String>(10)?,
                ))
            },
        )
        .map_err(|_| AppError::NotFound("Purchase order not found.".into()))?;

    let mut stmt = conn.prepare(
        "SELECT id, order_id, request_id, title, author, isbn, quantity, unit_price_minor, received_quantity, notes
         FROM purchase_order_items WHERE order_id = ?1 ORDER BY id",
    )?;
    let items = stmt
        .query_map([id], |r| {
            Ok(PurchaseOrderItem {
                id: r.get(0)?,
                order_id: r.get(1)?,
                request_id: r.get(2)?,
                title: r.get(3)?,
                author: r.get(4)?,
                isbn: r.get(5)?,
                quantity: r.get(6)?,
                unit_price_minor: r.get(7)?,
                received_quantity: r.get(8)?,
                notes: r.get(9)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let total_minor = items.iter().map(|i| i.quantity * i.unit_price_minor).sum();

    Ok(PurchaseOrderDetail {
        id,
        order_number,
        vendor_id,
        vendor_name,
        budget_id,
        budget_name,
        status,
        invoice_ref,
        ordered_at,
        expected_at,
        notes,
        items,
        total_minor,
        created_at,
    })
}

pub fn create_order(conn: &mut Connection, input: CreateOrderInput, actor: &Session) -> AppResult<i64> {
    let order_number = validate::required("Order number", &input.order_number, 50)?;
    if input.items.is_empty() {
        return Err(AppError::Validation("A purchase order must have at least one item.".into()));
    }
    let invoice_ref = validate::optional("Invoice reference", input.invoice_ref.as_deref(), 50)?;
    let expected_at = match &input.expected_at {
        Some(d) if !d.trim().is_empty() => Some(validate::date("Expected date", d)?),
        _ => None,
    };
    let notes = validate::optional("Notes", input.notes.as_deref(), 500)?;

    let tx = conn.transaction()?;

    // Check vendor exists
    let vendor_name: String = tx
        .query_row("SELECT name FROM vendors WHERE id = ?1", [input.vendor_id], |r| r.get(0))
        .map_err(|_| AppError::NotFound("Vendor not found.".into()))?;

    tx.execute(
        "INSERT INTO purchase_orders (order_number, vendor_id, budget_id, invoice_ref, expected_at, notes, created_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![order_number, input.vendor_id, input.budget_id, invoice_ref, expected_at, notes, actor.user_id],
    )?;
    let order_id = tx.last_insert_rowid();

    for item in &input.items {
        let title = validate::required("Item title", &item.title, 300)?;
        let author = validate::optional("Author", item.author.as_deref(), 200)?;
        let isbn = validate::optional("ISBN", item.isbn.as_deref(), 40)?;
        let item_notes = validate::optional("Notes", item.notes.as_deref(), 300)?;
        if item.quantity <= 0 {
            return Err(AppError::Validation("Quantity must be at least 1.".into()));
        }
        if item.unit_price_minor < 0 {
            return Err(AppError::Validation("Unit price must not be negative.".into()));
        }

        tx.execute(
            "INSERT INTO purchase_order_items (order_id, request_id, title, author, isbn, quantity, unit_price_minor, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![order_id, item.request_id, title, author, isbn, item.quantity, item.unit_price_minor, item_notes],
        )?;

        // If from request, mark request as ordered
        if let Some(req_id) = item.request_id {
            tx.execute(
                "UPDATE acquisition_requests SET status = 'ordered', updated_at = strftime('%Y-%m-%dT%H:%M:%SZ','now')
                 WHERE id = ?1",
                [req_id],
            )?;
        }
    }

    audit::record(&tx, Some(actor), "order.create", "purchase_order", Some(order_id), &format!("Created purchase order {order_number} to {vendor_name}"))?;
    tx.commit()?;
    Ok(order_id)
}

pub fn update_order_status(conn: &mut Connection, id: i64, status: String, invoice_ref: Option<String>, actor: &Session) -> AppResult<()> {
    if !["draft", "ordered", "cancelled"].contains(&status.as_str()) {
        return Err(AppError::Validation("Invalid order status.".into()));
    }
    let inv = validate::optional("Invoice reference", invoice_ref.as_deref(), 50)?;
    let tx = conn.transaction()?;
    let (order_number, current_status): (String, String) = tx
        .query_row("SELECT order_number, status FROM purchase_orders WHERE id = ?1", [id], |r| Ok((r.get(0)?, r.get(1)?)))
        .map_err(|_| AppError::NotFound("Purchase order not found.".into()))?;

    if current_status == "received" || current_status == "partially_received" {
        return Err(AppError::Validation("Cannot update status of an order that has already received stock.".into()));
    }

    let ordered_at = if status == "ordered" {
        Some(chrono_now())
    } else {
        None
    };

    tx.execute(
        "UPDATE purchase_orders
         SET status = ?1,
             invoice_ref = COALESCE(?2, invoice_ref),
             ordered_at = COALESCE(?3, ordered_at),
             updated_at = strftime('%Y-%m-%dT%H:%M:%SZ','now')
         WHERE id = ?4",
        params![status, inv, ordered_at, id],
    )?;

    audit::record(&tx, Some(actor), "order.status", "purchase_order", Some(id), &format!("Updated purchase order {order_number} status to {status}"))?;
    tx.commit()?;
    Ok(())
}

#[allow(clippy::type_complexity)]
pub fn receive_order_item(
    conn: &mut Connection,
    input: ReceiveItemInput,
    actor: &Session,
) -> AppResult<ReceiveResult> {
    if input.quantity <= 0 {
        return Err(AppError::Validation("Received quantity must be at least 1.".into()));
    }

    let tx = conn.transaction()?;

    // 1. Fetch PO details
    let (order_number, po_status, vendor_name, invoice_ref): (String, String, String, Option<String>) = tx
        .query_row(
            "SELECT po.order_number, po.status, v.name, po.invoice_ref
             FROM purchase_orders po
             JOIN vendors v ON v.id = po.vendor_id
             WHERE po.id = ?1",
            [input.order_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .map_err(|_| AppError::NotFound("Purchase order not found.".into()))?;

    if po_status == "draft" {
        return Err(AppError::Validation("Order must be placed (ordered) before receiving stock.".into()));
    }
    if po_status == "cancelled" {
        return Err(AppError::Validation("Cannot receive items for a cancelled purchase order.".into()));
    }

    // 2. Fetch line item
    let (item_order_id, ordered_qty, already_received_qty, title, author, isbn, unit_price_minor, request_id): (
        i64, i64, i64, String, Option<String>, Option<String>, i64, Option<i64>
    ) = tx
        .query_row(
            "SELECT order_id, quantity, received_quantity, title, author, isbn, unit_price_minor, request_id
             FROM purchase_order_items
             WHERE id = ?1",
            [input.item_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?)),
        )
        .map_err(|_| AppError::NotFound("Purchase order item not found.".into()))?;

    if item_order_id != input.order_id {
        return Err(AppError::Validation("Item does not belong to the specified purchase order.".into()));
    }

    let remaining = ordered_qty - already_received_qty;
    if remaining <= 0 {
        return Err(AppError::Validation("This item has already been fully received.".into()));
    }
    if input.quantity > remaining {
        return Err(AppError::Validation(format!(
            "Cannot receive {} copies; only {} copies remaining to receive.",
            input.quantity, remaining
        )));
    }

    // 3. Resolve or create bibliographic record
    let record_id: i64 = if let Some(rec_id) = input.record_id {
        let exists: bool = tx
            .query_row("SELECT 1 FROM bibliographic_records WHERE id = ?1", [rec_id], |_| Ok(true))
            .unwrap_or(false);
        if !exists {
            return Err(AppError::NotFound("Bibliographic record not found.".into()));
        }
        rec_id
    } else {
        // Look up by ISBN (if given) or title
        let found_id: Option<i64> = if let Some(ref isbn_val) = isbn {
            tx.query_row(
                "SELECT id FROM bibliographic_records WHERE isbn13 = ?1 OR isbn10 = ?1",
                [isbn_val],
                |r| r.get(0),
            )
            .optional()?
        } else {
            None
        };

        if let Some(fid) = found_id {
            fid
        } else {
            tx.execute(
                "INSERT INTO bibliographic_records (title, publisher, publication_year, isbn13)
                 VALUES (?1, ?2, NULL, ?3)",
                params![title, vendor_name, isbn],
            )?;
            let new_rec_id = tx.last_insert_rowid();

            if let Some(ref author_name) = author {
                let author_trimmed = author_name.trim();
                if !author_trimmed.is_empty() {
                    tx.execute(
                        "INSERT INTO contributors (name) VALUES (?1) ON CONFLICT(name) DO NOTHING",
                        [author_trimmed],
                    )?;
                    let contrib_id: i64 = tx.query_row(
                        "SELECT id FROM contributors WHERE name = ?1",
                        [author_trimmed],
                        |r| r.get(0),
                    )?;
                    tx.execute(
                        "INSERT INTO record_contributors (record_id, contributor_id, role, position)
                         VALUES (?1, ?2, 'author', 1) ON CONFLICT DO NOTHING",
                        params![new_rec_id, contrib_id],
                    )?;
                }
            }

            let _ = catalogue::reindex(&tx, new_rec_id);
            new_rec_id
        }
    };

    // 4. Resolve default item_category_id and location_id
    let cat_id: i64 = match input.item_category_id {
        Some(c) => c,
        None => tx
            .query_row(
                "SELECT id FROM item_categories WHERE archived_at IS NULL ORDER BY id LIMIT 1",
                [],
                |r| r.get(0),
            )
            .map_err(|_| AppError::Validation("No active item category found in configuration.".into()))?,
    };

    let loc_id: Option<i64> = match input.location_id {
        Some(l) => Some(l),
        None => tx
            .query_row(
                "SELECT id FROM locations WHERE archived_at IS NULL ORDER BY id LIMIT 1",
                [],
                |r| r.get(0),
            )
            .optional()?,
    };

    // 5. Generate copies and item events
    let now_str = chrono_now();
    let today = if now_str.len() >= 10 { &now_str[..10] } else { "2026-11-19" };
    let mut created_copies = Vec::new();

    for _ in 0..input.quantity {
        let acc_no = catalogue::next_accession(&tx)?;
        let barcode = acc_no.clone();

        tx.execute(
            "INSERT INTO physical_items (
                record_id, accession_no, barcode, item_category_id, collection_id, location_id, shelf_id,
                call_number, acquisition_date, acquisition_source, price_minor, vendor, invoice_ref, status
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'purchase', ?10, ?11, ?12, 'available')",
            params![
                record_id,
                acc_no,
                barcode,
                cat_id,
                input.collection_id,
                loc_id,
                input.shelf_id,
                input.call_number,
                today,
                unit_price_minor,
                vendor_name,
                invoice_ref,
            ],
        )?;
        let copy_id = tx.last_insert_rowid();

        tx.execute(
            "INSERT INTO item_events (item_id, event_type, to_status, note, staff_user_id)
             VALUES (?1, 'created', 'available', ?2, ?3)",
            params![copy_id, format!("Received on purchase order {order_number}"), actor.user_id],
        )?;

        created_copies.push(CreatedCopyInfo {
            copy_id,
            accession_no: acc_no,
            barcode,
        });
    }

    // 6. Update line item received quantity
    let new_received_qty = already_received_qty + input.quantity;
    tx.execute(
        "UPDATE purchase_order_items SET received_quantity = ?1 WHERE id = ?2",
        params![new_received_qty, input.item_id],
    )?;

    // 7. If linked request exists and is now complete, mark it received
    if let Some(req_id) = request_id {
        if new_received_qty >= ordered_qty {
            tx.execute(
                "UPDATE acquisition_requests SET status = 'received', updated_at = strftime('%Y-%m-%dT%H:%M:%SZ','now')
                 WHERE id = ?1",
                [req_id],
            )?;
        }
    }

    // 8. Update PO overall status
    let (total_ordered, total_received): (i64, i64) = tx.query_row(
        "SELECT coalesce(sum(quantity), 0), coalesce(sum(received_quantity), 0)
         FROM purchase_order_items
         WHERE order_id = ?1",
        [input.order_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;

    let order_status = if total_received >= total_ordered {
        "received".to_string()
    } else {
        "partially_received".to_string()
    };

    tx.execute(
        "UPDATE purchase_orders SET status = ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE id = ?2",
        params![order_status, input.order_id],
    )?;

    audit::record(
        &tx,
        Some(actor),
        "order.receive",
        "purchase_order",
        Some(input.order_id),
        &format!("Received {} copies of '{title}' on {order_number}", input.quantity),
    )?;

    tx.commit()?;

    Ok(ReceiveResult {
        order_id: input.order_id,
        item_id: input.item_id,
        record_id,
        received_quantity: new_received_qty,
        order_status,
        created_copies,
    })
}

fn chrono_now() -> String {
    // Current UTC string
    let now = std::time::SystemTime::now();
    let duration = now.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();
    // Format YYYY-MM-DDTHH:MM:SSZ
    let days = (secs / 86400) as i64;
    let day_secs = (secs % 86400) as u32;
    let (year, month, day) = civil_date(days);
    let hour = day_secs / 3600;
    let min = (day_secs % 3600) / 60;
    let sec = day_secs % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{min:02}:{sec:02}Z")
}

fn civil_date(days: i64) -> (i64, u32, u32) {
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn setup_test_db() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        db::migrations::run(&mut conn, db::migrations::MIGRATIONS).unwrap();
        conn.execute(
            "INSERT INTO staff_users (id, username, display_name, password_hash, role_id) VALUES (1, 'admin', 'Admin', 'hash', 1)",
            [],
        ).unwrap();
        conn
    }

    fn test_actor() -> Session {
        Session {
            user_id: 1,
            username: "admin".into(),
            display_name: "Admin".into(),
            role: "administrator".into(),
            permissions: vec!["acquisitions.read".into(), "acquisitions.write".into()],
            revision: 1,
        }
    }

    #[test]
    fn vendor_lifecycle_and_validation() {
        let mut conn = setup_test_db();
        let actor = test_actor();

        let v_id = save_vendor(&mut conn, VendorInput {
            id: None,
            name: "Prabhat Books".into(),
            code: Some("PRABHAT".into()),
            contact_name: Some("Raman Nair".into()),
            email: Some("raman@prabhat.example".into()),
            phone: Some("+91 471 2334455".into()),
            address: Some("Trivandrum, Kerala".into()),
            notes: Some("Preferred Malayalam literature vendor".into()),
        }, &actor).unwrap();

        let vendors = list_vendors(&conn, false).unwrap();
        assert_eq!(vendors.len(), 1);
        assert_eq!(vendors[0].name, "Prabhat Books");
        assert_eq!(vendors[0].code.as_deref(), Some("PRABHAT"));

        archive_vendor(&mut conn, v_id, true, &actor).unwrap();
        assert_eq!(list_vendors(&conn, false).unwrap().len(), 0);
        assert_eq!(list_vendors(&conn, true).unwrap().len(), 1);

        archive_vendor(&mut conn, v_id, false, &actor).unwrap();
        assert_eq!(list_vendors(&conn, false).unwrap().len(), 1);
    }

    #[test]
    fn request_to_purchase_order_flow() {
        let mut conn = setup_test_db();
        let actor = test_actor();

        let v_id = save_vendor(&mut conn, VendorInput {
            id: None,
            name: "DC Books".into(),
            code: Some("DCB".into()),
            contact_name: None,
            email: None,
            phone: None,
            address: None,
            notes: None,
        }, &actor).unwrap();

        let b_id = save_budget(&mut conn, BudgetInput {
            id: None,
            name: "General Fiction 2026-27".into(),
            code: Some("GF-26".into()),
            amount_minor: 5000000, // 50,000.00
            currency: "INR".into(),
            fiscal_year: Some("2026-27".into()),
        }, &actor).unwrap();

        // 1. Submit request
        let req_id = create_request(&mut conn, CreateRequestInput {
            title: "Aadujeevitham".into(),
            author: Some("Benyamin".into()),
            isbn: Some("9788126419999".into()),
            publisher: Some("Green Books".into()),
            publication_year: Some(2008),
            estimated_price_minor: Some(25000),
            currency: Some("INR".into()),
            copies: Some(2),
            requested_by: "Prof. Krishnan".into(),
            request_reason: Some("Recommended for Malayalam department".into()),
        }, &actor).unwrap();

        let requests = list_requests(&conn, None).unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].status, "pending");

        // 2. Approve request
        approve_request(&mut conn, req_id, Some(b_id), &actor).unwrap();
        let requests = list_requests(&conn, Some("approved".into())).unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].budget_id, Some(b_id));

        // 3. Create purchase order with this item
        let po_id = create_order(&mut conn, CreateOrderInput {
            order_number: "PO-2026-001".into(),
            vendor_id: v_id,
            budget_id: Some(b_id),
            invoice_ref: Some("PROFORMA-451".into()),
            expected_at: Some("2026-12-15".into()),
            notes: Some("Urgent request".into()),
            items: vec![PurchaseOrderItemInput {
                request_id: Some(req_id),
                title: "Aadujeevitham".into(),
                author: Some("Benyamin".into()),
                isbn: Some("9788126419999".into()),
                quantity: 2,
                unit_price_minor: 24000,
                notes: None,
            }],
        }, &actor).unwrap();

        // Verify request became "ordered"
        let req = list_requests(&conn, None).unwrap();
        assert_eq!(req[0].status, "ordered");

        // Verify purchase order details and budget allocation
        let po = get_order(&conn, po_id).unwrap();
        assert_eq!(po.order_number, "PO-2026-001");
        assert_eq!(po.vendor_name, "DC Books");
        assert_eq!(po.items.len(), 1);
        assert_eq!(po.total_minor, 48000); // 2 * 24000

        let budgets = list_budgets(&conn, false).unwrap();
        assert_eq!(budgets[0].allocated_minor, 48000);

        // Update PO status to ordered
        update_order_status(&mut conn, po_id, "ordered".into(), Some("INV-FINAL-101".into()), &actor).unwrap();
        let updated = get_order(&conn, po_id).unwrap();
        assert_eq!(updated.status, "ordered");
        assert_eq!(updated.invoice_ref.as_deref(), Some("INV-FINAL-101"));
        assert!(updated.ordered_at.is_some());
    }

    #[test]
    fn receiving_partial_order_full_order_and_duplicate_rejection() {
        let mut conn = setup_test_db();
        let actor = test_actor();

        conn.execute("INSERT INTO item_categories (id, name) VALUES (1, 'Book')", []).unwrap();
        conn.execute("INSERT INTO locations (id, name) VALUES (1, 'Main Library')", []).unwrap();

        let v_id = save_vendor(&mut conn, VendorInput {
            id: None,
            name: "H&C Books".into(),
            code: Some("HCB".into()),
            contact_name: None,
            email: None,
            phone: None,
            address: None,
            notes: None,
        }, &actor).unwrap();

        let po_id = create_order(&mut conn, CreateOrderInput {
            order_number: "PO-RECV-01".into(),
            vendor_id: v_id,
            budget_id: None,
            invoice_ref: Some("INV-777".into()),
            expected_at: None,
            notes: None,
            items: vec![PurchaseOrderItemInput {
                request_id: None,
                title: "Randamoozham".into(),
                author: Some("M. T. Vasudevan Nair".into()),
                isbn: Some("9788171300845".into()),
                quantity: 3,
                unit_price_minor: 35000,
                notes: None,
            }],
        }, &actor).unwrap();

        let order = get_order(&conn, po_id).unwrap();
        let item_id = order.items[0].id;

        // Receiving while still 'draft' should fail
        let res_draft = receive_order_item(&mut conn, ReceiveItemInput {
            order_id: po_id,
            item_id,
            quantity: 1,
            record_id: None,
            item_category_id: None,
            location_id: None,
            collection_id: None,
            shelf_id: None,
            call_number: None,
        }, &actor);
        assert!(res_draft.is_err());

        // Update status to ordered
        update_order_status(&mut conn, po_id, "ordered".into(), None, &actor).unwrap();

        // 1. Receive 1 copy partially
        let res1 = receive_order_item(&mut conn, ReceiveItemInput {
            order_id: po_id,
            item_id,
            quantity: 1,
            record_id: None,
            item_category_id: None,
            location_id: None,
            collection_id: None,
            shelf_id: None,
            call_number: None,
        }, &actor).unwrap();

        assert_eq!(res1.order_status, "partially_received");
        assert_eq!(res1.received_quantity, 1);
        assert_eq!(res1.created_copies.len(), 1);

        let po_partial = get_order(&conn, po_id).unwrap();
        assert_eq!(po_partial.status, "partially_received");
        assert_eq!(po_partial.items[0].received_quantity, 1);

        // 2. Reject receiving more than remaining (remaining is 2, asking for 3)
        let res_over = receive_order_item(&mut conn, ReceiveItemInput {
            order_id: po_id,
            item_id,
            quantity: 3,
            record_id: None,
            item_category_id: None,
            location_id: None,
            collection_id: None,
            shelf_id: None,
            call_number: None,
        }, &actor);
        assert!(res_over.is_err());

        // 3. Receive the remaining 2 copies to complete the order
        let res2 = receive_order_item(&mut conn, ReceiveItemInput {
            order_id: po_id,
            item_id,
            quantity: 2,
            record_id: Some(res1.record_id),
            item_category_id: None,
            location_id: None,
            collection_id: None,
            shelf_id: None,
            call_number: None,
        }, &actor).unwrap();

        assert_eq!(res2.order_status, "received");
        assert_eq!(res2.received_quantity, 3);
        assert_eq!(res2.created_copies.len(), 2);

        let po_complete = get_order(&conn, po_id).unwrap();
        assert_eq!(po_complete.status, "received");
        assert_eq!(po_complete.items[0].received_quantity, 3);

        // 4. Reject duplicate receipt on already completed item
        let res_dup = receive_order_item(&mut conn, ReceiveItemInput {
            order_id: po_id,
            item_id,
            quantity: 1,
            record_id: None,
            item_category_id: None,
            location_id: None,
            collection_id: None,
            shelf_id: None,
            call_number: None,
        }, &actor);
        assert!(res_dup.is_err());

        // Verify total copies created for this record is exactly 3
        let count: i64 = conn.query_row(
            "SELECT count(*) FROM physical_items WHERE record_id = ?1",
            [res1.record_id],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(count, 3);
    }
}
