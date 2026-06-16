import { RaindexClient } from './packages/orderbook/dist/esm/index.js';

async function testTakeNVDA() {
    console.log('Testing NVDA take order...');
    
    const client = new RaindexClient();
    
    // The failing NVDA order
    const orderHash = '0x878c4fc1b65ac9992812d1c5aa24d62bb0c549895ce86849238636b7b8f78869';
    const chainId = 8453; // Base
    const orderbookAddress = '0xe522cb4a5fcb2eb31a52ff41a4653d85a4fd7c9d';
    
    try {
        console.log('Getting order...');
        const orderResult = await client.getOrder(chainId, orderbookAddress, orderHash);
        if (orderResult.error) {
            console.error('Error getting order:', orderResult.error);
            return;
        }
        
        const order = orderResult.value;
        console.log('Order retrieved successfully');
        
        console.log('Getting quotes...');
        const quotesResult = await order.getQuotes();
        if (quotesResult.error) {
            console.error('Error getting quotes:', quotesResult.error);
            return;
        }
        
        const quotes = quotesResult.value;
        console.log('Quotes:', quotes.map(q => ({
            pair: q.pair.pairName,
            success: q.success,
            maxOutput: q.data?.formattedMaxOutput,
            ratio: q.data?.formattedRatio
        })));
        
        if (quotes.length === 0 || !quotes[0].success) {
            console.error('No valid quotes available');
            return;
        }
        
        const quote = quotes[0];
        console.log('Using quote:', quote.pair.pairName);
        
        // Try to take $1 worth (very small amount)
        const taker = '0x1234567890123456789012345678901234567890'; // dummy address
        const mode = 'buyUpTo';
        const amount = '0.001'; // About $1 worth of NVDA at ~$180
        const priceCap = '200'; // Well above current price
        
        console.log(`Attempting to take ${amount} wtNVDA with price cap ${priceCap}`);
        
        const calldataResult = await order.getTakeCalldata(
            quote.pair.inputIndex,
            quote.pair.outputIndex,
            taker,
            mode,
            amount,
            priceCap
        );
        
        if (calldataResult.error) {
            console.error('❌ TAKE CALLDATA FAILED:', calldataResult.error);
            console.error('Error details:', {
                msg: calldataResult.error.msg,
                readableMsg: calldataResult.error.readableMsg
            });
        } else {
            console.log('✅ Take calldata succeeded:', {
                isReady: calldataResult.value.isReady,
                isNeedsApproval: calldataResult.value.isNeedsApproval
            });
        }
        
    } catch (error) {
        console.error('Unexpected error:', error);
    }
}

testTakeNVDA().catch(console.error);
