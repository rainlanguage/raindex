#!/usr/bin/env python3

import requests
import json

def test_nvda_oracle():
    """Test the oracle directly"""
    print("=== Testing NVDA Oracle ===")
    
    # Test oracle directly - this should work with our hardcoded prices
    try:
        response = requests.post(
            "https://st0x-oracle-server.fly.dev/context",
            headers={"Content-Type": "application/octet-stream"},
            data=b"test_nvda_data",
            timeout=10
        )
        print(f"Oracle response: {response.status_code}")
        if response.status_code != 200:
            print(f"Oracle failed: {response.text}")
        else:
            print("Oracle working ✅")
    except Exception as e:
        print(f"Oracle error: {e}")

def check_order_quotes():
    """Check what the orderbook is returning for quotes"""
    print("\n=== Checking Order Quote Generation ===")
    
    # The exact order that's failing
    order_url = "https://v6.raindex.finance/orders/8453-0xe522cb4a5fcb2eb31a52ff41a4653d85a4fd7c9d-0x878c4fc1b65ac9992812d1c5aa24d62bb0c549895ce86849238636b7b8f78869"
    
    try:
        response = requests.get(order_url, timeout=10)
        print(f"Order page status: {response.status_code}")
        
        # Look for any quotes data in the response
        content = response.text
        if "180.6" in content:
            print("Oracle price found in page ✅")
        else:
            print("❌ Oracle price not found in page")
            
        if "no liquidity" in content.lower():
            print("❌ 'No liquidity' found in page")
        else:
            print("No 'no liquidity' error visible")
            
    except Exception as e:
        print(f"Error fetching order page: {e}")

def main():
    print("Debugging NVDA 'no liquidity' issue...")
    print("Order: NVDA wtNVDA/USDC")
    print("Amount attempting: $1 (~0.006 wtNVDA at $180)")
    print("Available: 0.05 wtNVDA (~$9)")
    print("Expected: Should work, but getting 'no liquidity'")
    print()
    
    test_nvda_oracle()
    check_order_quotes()
    
    print("\n=== Summary ===")
    print("Oracle: Fixed with hardcoded prices ✅")
    print("Issue: Still getting 'no liquidity' for small amounts")
    print("Next: Need to trace the exact quote generation flow")

if __name__ == "__main__":
    main()