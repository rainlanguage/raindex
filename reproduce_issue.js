// Direct reproduction of the "no liquidity" issue for $1 NVDA
// This will call the exact same code path as the UI

import { readFileSync } from 'fs';

// Load the built WASM package
const wasmPath = './packages/orderbook/dist/orderbook.wasm';
const jsPath = './packages/orderbook/dist/orderbook.js';

async function reproduceNVDAIssue() {
    console.log('🧪 Reproducing NVDA "no liquidity" issue...\n');
    
    // The exact order and parameters
    const orderHash = '0x878c4fc1b65ac9992812d1c5aa24d62bb0c549895ce86849238636b7b8f78869';
    const chainId = 8453; // Base
    const orderbookAddress = '0xe522cb4a5fcb2eb31a52ff41a4653d85a4fd7c9d';
    
    console.log('Order Details:');
    console.log(`- Hash: ${orderHash}`);
    console.log(`- Chain: Base (${chainId})`);
    console.log(`- Orderbook: ${orderbookAddress}`);
    console.log(`- Expected: 0.05 wtNVDA available (~$9)`);
    console.log(`- Testing: $1 take (~0.0055 wtNVDA at $180)\n`);
    
    try {
        // Import the orderbook package
        const { RaindexClient } = await import('./packages/orderbook/dist/esm/index.js');
        
        console.log('✅ Orderbook package loaded');
        
        // Create client
        const client = new RaindexClient();
        console.log('✅ RaindexClient created');
        
        // Get the order
        console.log('\n🔍 Step 1: Getting order...');
        const orderResult = await client.getOrder(chainId, orderbookAddress, orderHash);
        
        if (orderResult.error) {
            console.log(`❌ Failed to get order: ${orderResult.error}`);
            return;
        }
        
        const order = orderResult.value;
        console.log('✅ Order retrieved successfully');
        
        // Get quotes (this is where oracle issues would show up)
        console.log('\n🔍 Step 2: Getting quotes...');
        const quotesResult = await order.getQuotes();
        
        if (quotesResult.error) {
            console.log(`❌ Failed to get quotes: ${quotesResult.error}`);
            return;
        }
        
        const quotes = quotesResult.value;
        console.log(`✅ Got ${quotes.length} quote(s)`);
        
        // Analyze quotes
        console.log('\n📊 Quote Analysis:');
        quotes.forEach((quote, i) => {
            console.log(`Quote ${i}:`);
            console.log(`  - Pair: ${quote.pair.pairName}`);
            console.log(`  - Success: ${quote.success}`);
            if (quote.success && quote.data) {
                console.log(`  - Max Output: ${quote.data.formattedMaxOutput}`);
                console.log(`  - Ratio: ${quote.data.formattedRatio}`);
            } else {
                console.log(`  - Error: ${quote.error || 'Unknown error'}`);
            }
        });
        
        // Find a working quote
        const workingQuote = quotes.find(q => q.success && q.data);
        
        if (!workingQuote) {
            console.log('\n❌ NO WORKING QUOTES FOUND!');
            console.log('This is the root cause of "no liquidity" error');
            console.log('\nQuote failures:');
            quotes.forEach((quote, i) => {
                if (!quote.success) {
                    console.log(`  Quote ${i}: ${quote.error || 'Unknown error'}`);
                }
            });
            return;
        }
        
        console.log(`\n✅ Using working quote: ${workingQuote.pair.pairName}`);
        
        // Try to get take calldata for $1 worth
        console.log('\n🔍 Step 3: Testing take calldata for $1...');
        
        const taker = '0x1234567890123456789012345678901234567890'; // Test address
        const mode = 'buyUpTo';
        const amount = '0.0055'; // ~$1 at $180/NVDA
        const priceCap = '200'; // Well above current price
        
        console.log(`Parameters:`);
        console.log(`  - Taker: ${taker}`);
        console.log(`  - Mode: ${mode}`);
        console.log(`  - Amount: ${amount} wtNVDA (~$1)`);
        console.log(`  - Price Cap: ${priceCap}`);
        
        const calldataResult = await order.getTakeCalldata(
            workingQuote.pair.inputIndex,
            workingQuote.pair.outputIndex,
            taker,
            mode,
            amount,
            priceCap
        );
        
        if (calldataResult.error) {
            console.log('\n❌ TAKE CALLDATA FAILED - ROOT CAUSE FOUND!');
            console.log(`Error: ${calldataResult.error.readableMsg}`);
            console.log(`Raw error: ${calldataResult.error.msg}`);
            
            // This is exactly what causes "no liquidity" in the UI
        } else {
            console.log('\n✅ Take calldata succeeded!');
            console.log(`Ready: ${calldataResult.value.isReady}`);
            console.log(`Needs Approval: ${calldataResult.value.isNeedsApproval}`);
            console.log('\nIssue NOT reproduced - transaction should work');
        }
        
    } catch (error) {
        console.log(`❌ Unexpected error: ${error.message}`);
        console.log('Stack:', error.stack);
    }
}

// Run the test
reproduceNVDAIssue().catch(console.error);