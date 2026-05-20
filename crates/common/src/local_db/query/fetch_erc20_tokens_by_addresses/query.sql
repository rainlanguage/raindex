SELECT chain_id, raindex_address, token_address, name, symbol, decimals
FROM erc20_tokens
WHERE chain_id = ?1 AND raindex_address = ?2
  /*ADDRESSES_CLAUSE*/;
