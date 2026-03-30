#!/usr/bin/env python3
"""
search.py - Search tool using DuckDuckGo for research queries.
Usage: python3 search.py "your search query" [--limit 10]
"""

import sys
import argparse

def search(query: str, limit: int = 10):
    try:
        from duckduckgo_search import DDGS
        results = []
        with DDGS() as ddgs:
            for r in ddgs.text(query, max_results=limit):
                results.append(r)
        return results
    except ImportError:
        print("Installing duckduckgo_search...", file=sys.stderr)
        import subprocess
        subprocess.check_call([sys.executable, "-m", "pip", "install", "duckduckgo_search", "-q"])
        from duckduckgo_search import DDGS
        results = []
        with DDGS() as ddgs:
            for r in ddgs.text(query, max_results=limit):
                results.append(r)
        return results


def main():
    parser = argparse.ArgumentParser(description="DuckDuckGo search tool")
    parser.add_argument("query", help="Search query")
    parser.add_argument("--limit", type=int, default=10, help="Number of results (default: 10)")
    args = parser.parse_args()

    results = search(args.query, args.limit)
    for i, r in enumerate(results, 1):
        print(f"\n[{i}] {r.get('title', 'No title')}")
        print(f"    URL: {r.get('href', 'No URL')}")
        print(f"    {r.get('body', '')[:200]}")


if __name__ == "__main__":
    main()
