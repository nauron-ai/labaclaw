use super::super::normalize::normalize_phone_number;
use super::super::WhatsAppIdentity;
use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};

#[derive(Debug, Clone)]
pub(crate) struct StoredIdentity {
    pub(crate) row_id: i64,
    pub(crate) identity: WhatsAppIdentity,
}

pub(crate) fn init_schema(conn: &Connection) -> anyhow::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS whatsapp_contact_identities (
            id INTEGER PRIMARY KEY,
            lid TEXT,
            phone_number TEXT,
            full_name TEXT,
            first_name TEXT,
            push_name TEXT,
            username TEXT,
            about TEXT,
            verified_name TEXT,
            last_seen_jid TEXT,
            updated_at INTEGER NOT NULL,
            device_id INTEGER NOT NULL,
            UNIQUE(lid, device_id),
            UNIQUE(phone_number, device_id)
        );
        CREATE INDEX IF NOT EXISTS idx_whatsapp_contact_identities_updated_at
        ON whatsapp_contact_identities(device_id, updated_at DESC);",
    )?;
    Ok(())
}

pub(crate) fn primary_row_id(
    lid_match: Option<&StoredIdentity>,
    phone_match: Option<&StoredIdentity>,
) -> Option<i64> {
    [
        lid_match.map(|row| row.row_id),
        phone_match.map(|row| row.row_id),
    ]
    .into_iter()
    .flatten()
    .min()
}

pub(crate) fn fetch_by_lid(
    conn: &Connection,
    device_id: i32,
    lid: Option<&str>,
) -> Result<Option<StoredIdentity>> {
    fetch_identity(conn, device_id, lid, "WHERE lid = ?1 AND device_id = ?2")
}

pub(crate) fn fetch_by_phone_number(
    conn: &Connection,
    device_id: i32,
    phone_number: Option<&str>,
) -> Result<Option<StoredIdentity>> {
    fetch_identity(
        conn,
        device_id,
        phone_number,
        "WHERE phone_number = ?1 AND device_id = ?2",
    )
}

pub(crate) fn insert_identity(
    conn: &Connection,
    device_id: i32,
    identity: &WhatsAppIdentity,
) -> Result<()> {
    conn.execute(
        "INSERT INTO whatsapp_contact_identities (
            lid, phone_number, full_name, first_name, push_name, username, about,
            verified_name, last_seen_jid, updated_at, device_id
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            identity.lid.as_deref(),
            identity.phone_number.as_deref(),
            identity.full_name.as_deref(),
            identity.first_name.as_deref(),
            identity.push_name.as_deref(),
            identity.username.as_deref(),
            identity.about.as_deref(),
            identity.verified_name.as_deref(),
            identity.last_seen_jid.as_deref(),
            identity.updated_at,
            device_id,
        ],
    )?;
    Ok(())
}

pub(crate) fn update_identity(
    conn: &Connection,
    device_id: i32,
    row_id: i64,
    identity: &WhatsAppIdentity,
) -> Result<()> {
    conn.execute(
        "UPDATE whatsapp_contact_identities
         SET lid = ?1, phone_number = ?2, full_name = ?3, first_name = ?4, push_name = ?5,
             username = ?6, about = ?7, verified_name = ?8, last_seen_jid = ?9, updated_at = ?10
         WHERE id = ?11 AND device_id = ?12",
        params![
            identity.lid.as_deref(),
            identity.phone_number.as_deref(),
            identity.full_name.as_deref(),
            identity.first_name.as_deref(),
            identity.push_name.as_deref(),
            identity.username.as_deref(),
            identity.about.as_deref(),
            identity.verified_name.as_deref(),
            identity.last_seen_jid.as_deref(),
            identity.updated_at,
            row_id,
            device_id,
        ],
    )?;
    Ok(())
}

pub(crate) fn delete_identity(conn: &Connection, device_id: i32, row_id: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM whatsapp_contact_identities WHERE id = ?1 AND device_id = ?2",
        params![row_id, device_id],
    )?;
    Ok(())
}

pub(crate) fn lookup_phone_number(
    conn: &Connection,
    device_id: i32,
    lid: Option<&str>,
) -> Result<Option<String>> {
    let Some(lid) = lid else {
        return Ok(None);
    };

    conn.query_row(
        "SELECT phone_number
         FROM lid_pn_mapping
         WHERE lid = ?1 AND device_id = ?2
         ORDER BY updated_at DESC
         LIMIT 1",
        params![lid, device_id],
        |row| row.get::<_, String>(0),
    )
    .optional()
    .map(|phone_number| phone_number.as_deref().and_then(normalize_phone_number))
    .map_err(Into::into)
}

pub(crate) fn lookup_lid(
    conn: &Connection,
    device_id: i32,
    phone_number: Option<&str>,
) -> Result<Option<String>> {
    let Some(phone_number) = phone_number else {
        return Ok(None);
    };

    conn.query_row(
        "SELECT lid
         FROM lid_pn_mapping
         WHERE phone_number = ?1 AND device_id = ?2
         ORDER BY updated_at DESC
         LIMIT 1",
        params![phone_number.trim_start_matches('+'), device_id],
        |row| row.get(0),
    )
    .optional()
    .map_err(Into::into)
}

fn fetch_identity(
    conn: &Connection,
    device_id: i32,
    value: Option<&str>,
    predicate: &str,
) -> Result<Option<StoredIdentity>> {
    let Some(value) = value else {
        return Ok(None);
    };

    let sql = format!(
        "SELECT id, lid, phone_number, full_name, first_name, push_name, username, about,
                verified_name, last_seen_jid, updated_at
         FROM whatsapp_contact_identities
         {predicate}"
    );

    conn.query_row(&sql, params![value, device_id], map_row)
        .optional()
        .map_err(Into::into)
}

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<StoredIdentity> {
    Ok(StoredIdentity {
        row_id: row.get(0)?,
        identity: WhatsAppIdentity {
            lid: row.get(1)?,
            phone_number: row.get(2)?,
            full_name: row.get(3)?,
            first_name: row.get(4)?,
            push_name: row.get(5)?,
            username: row.get(6)?,
            about: row.get(7)?,
            verified_name: row.get(8)?,
            last_seen_jid: row.get(9)?,
            updated_at: row.get(10)?,
        },
    })
}
