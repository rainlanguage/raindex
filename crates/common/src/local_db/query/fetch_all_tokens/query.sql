SELECT DISTINCT chain_id, raindex_address, token_address, name, symbol, decimals
FROM erc20_tokens
WHERE 1=1
  /*CHAIN_IDS_CLAUSE*/
  /*RAINDEXES_CLAUSE*/
ORDER BY chain_id, raindex_address, token_address;

