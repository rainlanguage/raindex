#!/usr/bin/env python3
"""
Direct reproduction test - simulate the exact flow that fails
"""

import requests
import json

def test_oracle_directly():
    """Test the oracle with a valid request"""
    print("=== Testing Oracle Directly ===")
    
    # Test oracle endpoint
    oracle_url = "https://st0x-oracle-server.fly.dev/context"
    
    # Just test if oracle is responding
    try:
        response = requests.get("https://st0x-oracle-server.fly.dev/", timeout=5)
        print(f"Oracle root: {response.status_code}")
    except Exception as e:
        print(f"Oracle unreachable: {e}")
        
    # Test context endpoint with minimal data
    try:
        response = requests.post(
            oracle_url,
            headers={"Content-Type": "application/octet-stream"},
            data=b"minimal_test",
            timeout=5
        )
        print(f"Oracle context: {response.status_code}")
        if response.status_code != 200:
            print(f"  Error: {response.text[:100]}")
    except Exception as e:
        print(f"Oracle context failed: {e}")

def check_raindex_api():
    """Check if v6.raindex.finance has any API endpoints"""
    print("\n=== Testing Raindex API ===")
    
    base_url = "https://v6.raindex.finance"
    
    # Test various potential endpoints
    endpoints = [
        "/api/orders",
        "/api/quotes", 
        "/graphql",
        "/rpc",
        "/_api/orders",
        "/server/orders"
    ]
    
    for endpoint in endpoints:
        try:
            url = f"{base_url}{endpoint}"
            response = requests.get(url, timeout=3)
            if response.status_code != 404:
                print(f"  {endpoint}: {response.status_code}")
                if 'json' in response.headers.get('content-type', ''):
                    try:
                        data = response.json()
                        print(f"    JSON response: {type(data)}")
                    except:
                        pass
        except:
            pass

def analyze_nvda_situation():
    """Analyze the current NVDA situation"""
    print("\n=== NVDA Analysis ===")
    
    print("Order Details:")
    print("  - Hash: 0x878c4fc1b65ac9992812d1c5aa24d62bb0c549895ce86849238636b7b8f78869")
    print("  - Chain: Base (8453)")
    print("  - Available: ~0.05 wtNVDA (~$9 at $180/share)")
    print("  - Test amount: $1 (~0.0055 wtNVDA)")
    print("  - Oracle: Hardcoded 180.5/180.6 (should work)")
    print()
    
    print("Expected Flow:")
    print("  1. Get order details ✓")
    print("  2. Generate quotes")
    print("     - Call oracle for price ✓ (working)")
    print("     - Calculate max output from vault")
    print("     - Return quote with success=true")
    print("  3. User tries to take $1")
    print("  4. getTakeCalldata()")
    print("     - Check price cap vs actual ratio")
    print("     - Check amount vs availability") 
    print("     - Generate transaction data")
    print()
    
    print("❌ Issue: Getting 'NoLiquidity' even for small amounts")
    print("🤔 Theory: Quote generation failing, not take calculation")

def main():
    print("🧪 Direct NVDA Issue Reproduction")
    print("=" * 50)
    
    test_oracle_directly()
    check_raindex_api()
    analyze_nvda_situation()
    
    print("\n📋 Summary:")
    print("- Oracle hardcoded fix deployed ✅")
    print("- Need to identify where quote generation fails")
    print("- Likely RPC/contract call issue, not oracle issue")
    print("- Must test the actual quote generation flow")

if __name__ == "__main__":
    main()