BEGIN TRANSACTION;

-- Global DB metadata (singleton)
CREATE TABLE IF NOT EXISTS db_metadata (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    db_schema_version INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Per-target watermarks keyed by (chain_id, raindex_address)
CREATE TABLE IF NOT EXISTS target_watermarks (
    chain_id INTEGER NOT NULL,
    raindex_address TEXT NOT NULL,
    last_block INTEGER NOT NULL DEFAULT 0,
    last_hash TEXT,
    updated_at INTEGER NOT NULL DEFAULT (CAST(strftime('%s', 'now') AS INTEGER) * 1000),
    PRIMARY KEY (chain_id, raindex_address)
);

CREATE TABLE IF NOT EXISTS sync_status (
    chain_id INTEGER NOT NULL,
    raindex_address TEXT NOT NULL,
    last_synced_block INTEGER NOT NULL DEFAULT 0,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (chain_id, raindex_address)
);

CREATE TABLE raw_events (
    chain_id INTEGER NOT NULL,
    raindex_address TEXT NOT NULL,
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    block_number INTEGER NOT NULL,
    block_timestamp INTEGER,
    address TEXT NOT NULL,
    topics TEXT NOT NULL,
    data TEXT NOT NULL,
    raw_json TEXT NOT NULL,
    PRIMARY KEY (chain_id, raindex_address, transaction_hash, log_index)
);
CREATE INDEX idx_raw_events_block ON raw_events(chain_id, raindex_address, block_number, log_index);
CREATE INDEX idx_raw_events_address ON raw_events(chain_id, raindex_address, address);

CREATE TABLE deposits (
    chain_id INTEGER NOT NULL,
    raindex_address TEXT NOT NULL,
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    block_number INTEGER NOT NULL,
    block_timestamp INTEGER NOT NULL,
    sender TEXT NOT NULL,
    token TEXT NOT NULL,
    vault_id TEXT NOT NULL,
    deposit_amount TEXT NOT NULL,
    deposit_amount_uint256 TEXT NOT NULL,
    PRIMARY KEY (chain_id, raindex_address, transaction_hash, log_index)
);

CREATE TABLE withdrawals (
    chain_id INTEGER NOT NULL,
    raindex_address TEXT NOT NULL,
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    block_number INTEGER NOT NULL,
    block_timestamp INTEGER NOT NULL,
    sender TEXT NOT NULL,
    token TEXT NOT NULL,
    vault_id TEXT NOT NULL,
    target_amount TEXT NOT NULL,
    withdraw_amount TEXT NOT NULL,
    withdraw_amount_uint256 TEXT NOT NULL,
    PRIMARY KEY (chain_id, raindex_address, transaction_hash, log_index)
);

CREATE TABLE order_events (
    chain_id INTEGER NOT NULL,
    raindex_address TEXT NOT NULL,
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    block_number INTEGER NOT NULL,
    block_timestamp INTEGER NOT NULL,
    sender TEXT NOT NULL,
    interpreter_address TEXT NOT NULL,
    store_address TEXT NOT NULL,
    order_hash TEXT NOT NULL,
    event_type TEXT NOT NULL,
    order_owner TEXT NOT NULL,
    order_nonce TEXT NOT NULL,
    order_bytes TEXT NOT NULL,
    PRIMARY KEY (chain_id, raindex_address, transaction_hash, log_index)
);

CREATE TABLE order_ios (
    chain_id INTEGER NOT NULL,
    raindex_address TEXT NOT NULL,
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    io_index INTEGER NOT NULL,
    io_type TEXT NOT NULL,
    token TEXT NOT NULL,
    vault_id TEXT NOT NULL,
    PRIMARY KEY (chain_id, raindex_address, transaction_hash, log_index, io_index, io_type),
    FOREIGN KEY (
        chain_id,
        raindex_address,
        transaction_hash,
        log_index
    ) REFERENCES order_events (
        chain_id,
        raindex_address,
        transaction_hash,
        log_index
    )
);

CREATE TABLE take_orders (
    chain_id INTEGER NOT NULL,
    raindex_address TEXT NOT NULL,
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    block_number INTEGER NOT NULL,
    block_timestamp INTEGER NOT NULL,
    sender TEXT NOT NULL,
    order_owner TEXT NOT NULL,
    order_nonce TEXT NOT NULL,
    input_io_index INTEGER NOT NULL,
    output_io_index INTEGER NOT NULL,
    taker_input TEXT NOT NULL,
    taker_output TEXT NOT NULL,
    PRIMARY KEY (chain_id, raindex_address, transaction_hash, log_index)
);

CREATE TABLE take_order_contexts (
    chain_id INTEGER NOT NULL,
    raindex_address TEXT NOT NULL,
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    context_index INTEGER NOT NULL,
    context_value TEXT NOT NULL,
    PRIMARY KEY (
        chain_id,
        raindex_address,
        transaction_hash,
        log_index,
        context_index
    ),
    FOREIGN KEY (
        chain_id,
        raindex_address,
        transaction_hash,
        log_index
    ) REFERENCES take_orders (
        chain_id,
        raindex_address,
        transaction_hash,
        log_index
    )
);

CREATE TABLE context_values (
    chain_id INTEGER NOT NULL,
    raindex_address TEXT NOT NULL,
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    context_index INTEGER NOT NULL,
    value_index INTEGER NOT NULL,
    value TEXT NOT NULL,
    PRIMARY KEY (
        chain_id,
        raindex_address,
        transaction_hash,
        log_index,
        context_index,
        value_index
    ),
    FOREIGN KEY (
        chain_id,
        raindex_address,
        transaction_hash,
        log_index,
        context_index
    ) REFERENCES take_order_contexts (
        chain_id,
        raindex_address,
        transaction_hash,
        log_index,
        context_index
    )
);

CREATE TABLE clear_v3_events (
    chain_id INTEGER NOT NULL,
    raindex_address TEXT NOT NULL,
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    block_number INTEGER NOT NULL,
    block_timestamp INTEGER NOT NULL,
    sender TEXT NOT NULL,
    alice_order_hash TEXT NOT NULL,
    alice_order_owner TEXT NOT NULL,
    alice_input_io_index INTEGER NOT NULL,
    alice_output_io_index INTEGER NOT NULL,
    alice_bounty_vault_id TEXT NOT NULL,
    alice_input_vault_id TEXT NOT NULL,
    alice_output_vault_id TEXT NOT NULL,
    bob_order_hash TEXT NOT NULL,
    bob_order_owner TEXT NOT NULL,
    bob_input_io_index INTEGER NOT NULL,
    bob_output_io_index INTEGER NOT NULL,
    bob_bounty_vault_id TEXT NOT NULL,
    bob_input_vault_id TEXT NOT NULL,
    bob_output_vault_id TEXT NOT NULL,
    PRIMARY KEY (chain_id, raindex_address, transaction_hash, log_index)
);

CREATE TABLE after_clear_v2_events (
    chain_id INTEGER NOT NULL,
    raindex_address TEXT NOT NULL,
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    block_number INTEGER NOT NULL,
    block_timestamp INTEGER NOT NULL,
    sender TEXT NOT NULL,
    alice_output TEXT NOT NULL,
    bob_output TEXT NOT NULL,
    alice_input TEXT NOT NULL,
    bob_input TEXT NOT NULL,
    PRIMARY KEY (chain_id, raindex_address, transaction_hash, log_index)
);

CREATE TABLE meta_events (
    chain_id INTEGER NOT NULL,
    raindex_address TEXT NOT NULL,
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    block_number INTEGER NOT NULL,
    block_timestamp INTEGER NOT NULL,
    sender TEXT NOT NULL,
    subject TEXT NOT NULL,
    meta TEXT NOT NULL,
    PRIMARY KEY (chain_id, raindex_address, transaction_hash, log_index)
);
CREATE INDEX idx_deposits_vault ON deposits(chain_id, raindex_address, sender, token, vault_id);
CREATE INDEX idx_deposits_block ON deposits(chain_id, raindex_address, block_number);
CREATE INDEX idx_deposits_token ON deposits(chain_id, raindex_address, token);

CREATE INDEX idx_withdrawals_vault ON withdrawals(chain_id, raindex_address, sender, token, vault_id);
CREATE INDEX idx_withdrawals_block ON withdrawals(chain_id, raindex_address, block_number);
CREATE INDEX idx_withdrawals_token ON withdrawals(chain_id, raindex_address, token);

CREATE INDEX idx_order_events_hash ON order_events(chain_id, raindex_address, order_hash);
CREATE INDEX idx_order_events_owner ON order_events(chain_id, raindex_address, order_owner);
CREATE INDEX idx_order_events_block ON order_events(chain_id, raindex_address, block_number);
CREATE INDEX idx_order_events_store ON order_events(chain_id, raindex_address, store_address);

CREATE INDEX idx_order_ios_token ON order_ios(chain_id, raindex_address, token);

CREATE INDEX idx_take_orders_owner ON take_orders(chain_id, raindex_address, order_owner);
CREATE INDEX idx_take_orders_block ON take_orders(chain_id, raindex_address, block_number);

CREATE INDEX idx_clear_events_alice_bob ON clear_v3_events(chain_id, raindex_address, alice_order_hash, bob_order_hash);
CREATE INDEX idx_clear_events_block ON clear_v3_events(chain_id, raindex_address, block_number);
CREATE INDEX idx_clear_alice_vaults ON clear_v3_events(chain_id, raindex_address, alice_input_vault_id, alice_output_vault_id);
CREATE INDEX idx_clear_bob_vaults ON clear_v3_events(chain_id, raindex_address, bob_input_vault_id, bob_output_vault_id);

CREATE INDEX idx_after_clear_block ON after_clear_v2_events(chain_id, raindex_address, block_number);

CREATE INDEX idx_meta_subject ON meta_events(chain_id, raindex_address, subject);
CREATE INDEX idx_meta_block ON meta_events(chain_id, raindex_address, block_number);

CREATE TABLE erc20_tokens (
    chain_id INTEGER NOT NULL,
    raindex_address TEXT NOT NULL,
    token_address TEXT NOT NULL,
    name     TEXT    NOT NULL,
    symbol   TEXT    NOT NULL,
    decimals INTEGER NOT NULL,
    PRIMARY KEY (chain_id, raindex_address, token_address)
);
CREATE INDEX idx_erc20_tokens_token
    ON erc20_tokens(chain_id, raindex_address, token_address);

CREATE TABLE interpreter_store_sets (
    chain_id INTEGER NOT NULL,
    raindex_address TEXT NOT NULL,
    store_address TEXT NOT NULL,
    transaction_hash TEXT NOT NULL,
    log_index INTEGER NOT NULL,
    block_number INTEGER NOT NULL,
    block_timestamp INTEGER NOT NULL,
    namespace TEXT NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    PRIMARY KEY (chain_id, raindex_address, transaction_hash, log_index)
);
CREATE INDEX idx_store_sets_store ON interpreter_store_sets(chain_id, raindex_address, store_address);
CREATE INDEX idx_store_sets_block ON interpreter_store_sets(chain_id, raindex_address, block_number);
CREATE INDEX idx_store_sets_namespace ON interpreter_store_sets(chain_id, raindex_address, namespace);

CREATE TABLE IF NOT EXISTS vault_balance_changes (
    chain_id INTEGER NOT NULL,
    raindex_address TEXT NOT NULL,
    transaction_hash TEXT NOT NULL,
    owner TEXT NOT NULL,
    token TEXT NOT NULL,
    vault_id TEXT NOT NULL,
    block_number INTEGER NOT NULL,
    block_timestamp INTEGER NOT NULL,
    log_index INTEGER NOT NULL,
    change_type TEXT NOT NULL,
    delta TEXT NOT NULL,
    running_balance TEXT NOT NULL,
    PRIMARY KEY (
        chain_id,
        raindex_address,
        owner,
        token,
        vault_id,
        block_number,
        log_index
    )
);
CREATE INDEX idx_vbc_timestamp ON vault_balance_changes(
    chain_id,
    raindex_address,
    owner,
    token,
    vault_id,
    block_timestamp DESC,
    block_number DESC,
    log_index DESC
);

CREATE TABLE IF NOT EXISTS running_vault_balances (
    chain_id INTEGER NOT NULL,
    raindex_address TEXT NOT NULL,
    owner TEXT NOT NULL,
    token TEXT NOT NULL,
    vault_id TEXT NOT NULL,
    balance TEXT NOT NULL,
    last_block INTEGER NOT NULL,
    last_log_index INTEGER NOT NULL,
    updated_at INTEGER NOT NULL DEFAULT (CAST(strftime('%s','now') AS INTEGER) * 1000),
    PRIMARY KEY (chain_id, raindex_address, owner, token, vault_id)
);
CREATE INDEX idx_rvb_owner ON running_vault_balances(chain_id, raindex_address, owner);
CREATE INDEX idx_rvb_token ON running_vault_balances(chain_id, raindex_address, token, vault_id);

COMMIT;
