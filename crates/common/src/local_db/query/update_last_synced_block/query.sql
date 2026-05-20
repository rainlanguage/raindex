INSERT INTO sync_status (
    chain_id,
    raindex_address,
    last_synced_block
) VALUES (
    ?1,
    ?2,
    ?3
)
ON CONFLICT(chain_id, raindex_address)
DO UPDATE SET
    last_synced_block = excluded.last_synced_block,
    updated_at = CURRENT_TIMESTAMP;
